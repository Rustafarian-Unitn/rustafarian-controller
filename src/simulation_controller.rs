use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::Error;
use rustafarian_drone::RustafarianDrone;
// use rustafarian_shared::assembler::{assembler::Assembler, disassembler::Disassembler};
use crate::drone_functions::rustafarian_drone;
use crate::runnable::Runnable;
use crate::server::Server;
use rustafarian_client::chat_client::ChatClient;
use rustafarian_client::client::Client;
use rustafarian_shared::messages::commander_messages::{
    SimControllerCommand, SimControllerEvent, SimControllerResponseWrapper
};
use rustafarian_shared::messages::commander_messages::SimControllerEvent::PacketForwarded;
use rustafarian_shared::topology::Topology;
use std::collections::HashMap;
use std::thread::JoinHandle;
use std::{fs, thread};
use wg_2024::config::{
    Client as ClientConfig, Config, Drone as DroneConfig, Server as ServerConfig,
};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

pub struct NodeChannels {
    pub send_packet_channel: Sender<Packet>,
    pub send_command_channel: Sender<SimControllerCommand>,
    pub receive_response_channel: Receiver<SimControllerResponseWrapper>,
}

pub struct DroneChannels {
    pub send_command_channel: Sender<DroneCommand>,
    pub receive_command_channel: Receiver<DroneCommand>,
    pub send_packet_channel: Sender<Packet>,
    pub receive_packet_channel: Receiver<Packet>,
    pub receive_event_channel: Receiver<DroneEvent>,
    pub send_event_channel: Sender<DroneEvent>,
}

type DroneFactory = fn(
    id: NodeId,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
) -> Box<dyn Runnable>;

pub struct SimulationController {
    pub topology: Topology,
    pub nodes_channels: HashMap<NodeId, NodeChannels>,
    pub drone_channels: HashMap<NodeId, DroneChannels>,
    pub handles: Vec<JoinHandle<()>>,
}

impl SimulationController {
    fn new(
        nodes_channels: HashMap<NodeId, NodeChannels>,
        drone_channels: HashMap<NodeId, DroneChannels>,
        handles: Vec<JoinHandle<()>>,
        topology: Topology,
    ) -> Self {
        SimulationController {
            topology,
            nodes_channels,
            drone_channels,
            handles,
        }
    }

    pub fn build(config: &str) -> Self {
        let config = SimulationController::parse_config(config);
        let clients: Vec<ChatClient> = Vec::new();
        // let server: Vec<Server> = Vec::new();

        // Create a factory function for the implementations
        let drone_factories: Vec<DroneFactory> = vec![rustafarian_drone];

        let mut drone_factories = drone_factories.into_iter().cycle();

        let mut drone_channels: HashMap<NodeId, DroneChannels> = HashMap::new();
        let mut node_channels: HashMap<NodeId, NodeChannels> = HashMap::new();

        let mut handles = Vec::new();

        let mut topology = Topology::new();

        Self::init_drones(
            &mut handles,
            config.drone,
            &mut drone_factories,
            &mut drone_channels,
            &mut topology,
        );
        Self::init_clients(
            &mut handles,
            config.client,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
        );

        Self::init_servers(
            &mut handles,
            config.server,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
        );

        SimulationController::new(node_channels, drone_channels, handles, topology)
    }

    fn init_drones(
        handles: &mut Vec<JoinHandle<()>>,
        drones_config: Vec<DroneConfig>,
        drone_factories: &mut dyn Iterator<Item = DroneFactory>,
        drone_channels: &mut HashMap<NodeId, DroneChannels>,
        topology: &mut Topology,
    ) {
        let mut drones = Vec::<Box<dyn Runnable>>::new();

        for drone_config in drones_config.iter() {
            // Generate the channels for drone communication with sc
            let (send_command_channel, receive_command_channel) = unbounded::<DroneCommand>();
            let (send_event_channel, receive_event_channel) = unbounded::<DroneEvent>();

            // Generate the channels for inter-drone communication
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

            // Save the drone's channel counterparts for later use
            drone_channels.insert(
                drone_config.id,
                DroneChannels {
                    send_command_channel,
                    receive_command_channel,
                    send_packet_channel,
                    receive_packet_channel,
                    receive_event_channel,
                    send_event_channel,
                },
            );

            topology.add_node(drone_config.id);
            drone_config.connected_node_ids.iter().for_each(|node_id| {
                topology.add_edge(drone_config.id, *node_id);
            });
        }

        // For each drone config pick the next factory in a circular fashion to generate a drone instance
        for drone_config in drones_config.iter() {
            // Get the next drone in line
            let factory = drone_factories.next().unwrap();

            // Neighbouring nodes
            let neighbor_channels: HashMap<NodeId, Sender<Packet>> = drone_channels
                .iter()
                .filter(|(k, v)| drone_config.connected_node_ids.contains(k))
                .map(|(k, v)| (*k, v.send_packet_channel.clone()))
                .collect();

            let drone_channels = drone_channels.get(&drone_config.id).unwrap();

            let mut drone: Box<dyn Runnable> = factory(
                drone_config.id,
                drone_channels.send_event_channel.clone(),
                drone_channels.receive_command_channel.clone(),
                drone_channels.receive_packet_channel.clone(),
                neighbor_channels,
                drone_config.pdr,
            );

            handles.push(thread::spawn(move || drone.run()));
        }
    }

    fn init_clients(
        handles: &mut Vec<JoinHandle<()>>,
        clients_config: Vec<ClientConfig>,
        node_channels: &mut HashMap<NodeId, NodeChannels>,
        drone_channels: &mut HashMap<NodeId, DroneChannels>,
        topology: &mut Topology,
    ) {
        for client_config in clients_config {
            // Generate the channels for client communication with sc

            // Simulation controller keeps send command channel, client keeps receive command channel
            let (send_command_channel, receive_command_channel) =
                unbounded::<SimControllerCommand>();

            // Simulation controller keeps receive Response channel, client keeps send Response channel
            let (send_response_channel, receive_response_channel) =
                unbounded::<SimControllerResponseWrapper>();

            // Generate the channels for inter-drone communication
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

            // Save the client's channel counterparts for later use
            node_channels.insert(
                client_config.id,
                NodeChannels {
                    send_packet_channel,      // network comm.
                    send_command_channel,     // sc sends commands to client
                    receive_response_channel, // sc receives responses the client got from the servers
                },
            );

            // Register assigned neighbouring drones
            let neighbour_drones = drone_channels
                .iter()
                .filter(|(k, v)| client_config.connected_drone_ids.contains(k))
                .map(|(k, v)| (*k, v.send_packet_channel.clone()))
                .collect();

            // Register the client's node in the topology
            topology.add_node(client_config.id);
            client_config
                .connected_drone_ids
                .iter()
                .for_each(|node_id| {
                    topology.add_edge(client_config.id, *node_id);
                });

            // Start off the client
            handles.push(thread::spawn(move || {
                let mut client = ChatClient::new(
                    client_config.id,
                    neighbour_drones,
                    receive_packet_channel,
                    receive_command_channel,
                    send_response_channel,
                );
                client.run()
            }));
        }
    }

    fn init_servers(
        handles: &mut Vec<JoinHandle<()>>,
        servers_config: Vec<ServerConfig>,
        node_channels: &mut HashMap<NodeId, NodeChannels>,
        drone_channels: &mut HashMap<NodeId, DroneChannels>,
        topology: &mut Topology,
    ) {
        // For each drone config pick the next factory in a circular fashion to generate a drone instance
        for server_config in servers_config {
            let (send_command_channel, receive_command_channel) =
                unbounded::<SimControllerCommand>();

            let (send_response_channel, receive_response_channel) =
                unbounded::<SimControllerResponseWrapper>();

            // Generate the channels for inter-drone communication
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

            // Save the client's channel counterparts for later use
            node_channels.insert(
                server_config.id,
                NodeChannels {
                    send_packet_channel,
                    send_command_channel,
                    receive_response_channel,
                },
            );

            let drones = drone_channels
                .iter()
                .filter(|(k, v)| server_config.connected_drone_ids.contains(k))
                .map(|(k, v)| (*k, v.send_packet_channel.clone()))
                .collect();

            // Register the server's node in the topology
            topology.add_node(server_config.id);
            server_config
                .connected_drone_ids
                .iter()
                .for_each(|node_id| {
                    topology.add_edge(server_config.id, *node_id);
                });

            // Start off the server
            handles.push(thread::spawn(move || {
                let mut server = Server::new(server_config.id, receive_packet_channel, drones);
                server.run()
            }));
        }
    }

    pub fn parse_config(file: &str) -> Config {
        // Let it panic if file not found
        let file_str = fs::read_to_string(file).unwrap();

        // Let it panic if toml is misconfigured
        let parsed_config: Config = toml::from_str(&file_str).unwrap();
        parsed_config
    }

    pub fn handle_controller_shortcut(&self, packet: Packet) -> Result<SimControllerEvent, Error> {
        let packet_type = packet.pack_type.clone();
        let session_id = packet.session_id;
        let source = packet.routing_header.hops[0];
        let destination = packet.routing_header.hops[packet.routing_header.hops.len() - 1];
        if self
            .nodes_channels
            .get(&destination)
            .unwrap()
            .send_packet_channel
            .send(packet)
            == Ok(())
        {
            match packet_type {
                PacketType::MsgFragment(fragment) => {
                    Ok(PacketForwarded {
                        session_id,
                        packet_type: fragment.to_string(),
                        source,
                        destination,
                    })
                }
                PacketType::Ack(ack) => Ok(PacketForwarded {
                    session_id,
                    packet_type: ack.to_string(),
                    source,
                    destination,
                }),
                PacketType::Nack(nack) => Ok(PacketForwarded {
                    session_id,
                    packet_type: nack.to_string(),
                    source,
                    destination,
                }),
                PacketType::FloodRequest(flood_request) => Ok(PacketForwarded {
                    session_id,
                    packet_type: flood_request.to_string(),
                    source,
                    destination,
                }),
                PacketType::FloodResponse(flood_response) => Ok(PacketForwarded {
                    session_id,
                    packet_type: flood_response.to_string(),
                    source,
                    destination,
                }),
            }
        } else {
            Err(Error::new("Failed to send packet"))
        }
    }
}









#[cfg(test)]
mod tests {
    use super::*;
    use crate::drone_functions::rustafarian_drone;
    use rustafarian_shared::topology::Topology;
    use wg_2024::{drone::Drone, network::SourceRoutingHeader, packet::{Fragment, Packet, PacketType}};
    use std::collections::HashMap;
    use crossbeam_channel::unbounded;
    use rustafarian_drone::RustafarianDrone;

    #[test]
    fn test_simulation_controller_new() {
        let nodes_channels = HashMap::new();
        let drone_channels = HashMap::new();
        let handles = Vec::new();

        let controller =
            SimulationController::new(nodes_channels, drone_channels, handles, Topology::new());

        assert!(controller.topology.nodes().len() == 0);
        assert!(controller.nodes_channels.is_empty());
        assert!(controller.drone_channels.is_empty());
        assert!(controller.handles.is_empty());
    }

    #[test]
    fn test_simulation_controller_build() {
        let config_str = "tests/configurations/test_config.toml";

        let controller = SimulationController::build(config_str);

        assert_eq!(controller.drone_channels.len(), 1);
        assert_eq!(controller.nodes_channels.len(), 2);
        assert_eq!(controller.handles.len(), 3);
        assert_eq!(controller.topology.nodes().len(), 3);
    }

    #[test]
    fn test_parse_config() {
        let config_str = "tests/configurations/test_config.toml";

        let config = SimulationController::parse_config(config_str);
        println!("{:?}", config);
        assert_eq!(config.drone.len(), 1);
        assert_eq!(config.client.len(), 1);
        assert_eq!(config.server.len(), 1);
    }

    #[test]
    fn test_init_drones() {
        let mut handles = Vec::new();
        let drones_config = vec![DroneConfig {
            id: 1,
            pdr: 0.9,
            connected_node_ids: vec![2],
        }];
        let drone_factories: Vec<DroneFactory> = vec![rustafarian_drone];
        let mut drone_factories = drone_factories.into_iter().cycle();
        let mut drone_channels = HashMap::new();
        let mut topology = Topology::new();
        SimulationController::init_drones(
            &mut handles,
            drones_config,
            &mut drone_factories,
            &mut drone_channels,
            &mut topology,
        );

        assert_eq!(drone_channels.len(), 1);
        assert_eq!(handles.len(), 1);
    }

    #[test]
    fn test_init_clients() {
        let mut handles = Vec::new();
        let clients_config = vec![ClientConfig {
            id: 2,
            connected_drone_ids: vec![1],
        }];
        let mut node_channels = HashMap::new();
        let mut drone_channels = HashMap::new();
        let mut topology = Topology::new();
        SimulationController::init_clients(
            &mut handles,
            clients_config,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
        );

        assert_eq!(node_channels.len(), 1);
        assert_eq!(handles.len(), 1);
    }

    #[test]
    fn test_init_servers() {
        let mut handles = Vec::new();
        let servers_config = vec![ServerConfig {
            id: 3,
            connected_drone_ids: vec![1],
        }];
        let mut node_channels = HashMap::new();
        let mut drone_channels = HashMap::new();
        let mut topology = Topology::new();
        SimulationController::init_servers(
            &mut handles,
            servers_config,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
        );

        assert_eq!(node_channels.len(), 1);
        assert_eq!(handles.len(), 1);
    }

    #[test]
    fn test_message_from_client_to_server() {
        let mut drone_neighbors = HashMap::new();
        let mut client_neighbors = HashMap::new();
        let mut server_neighbors = HashMap::new();

        let drone_packet_channels = unbounded::<Packet>();
        let drone_event_channels = unbounded::<DroneEvent>();
        let drone_command_channels = unbounded::<DroneCommand>();

        let client_packet_channels = unbounded::<Packet>();
        let client_command_channels = unbounded::<SimControllerCommand>();
        let client_response_channels = unbounded::<SimControllerResponseWrapper>();
        
        client_neighbors.insert(2, drone_packet_channels.0);
        let client = ChatClient::new(1, client_neighbors, client_packet_channels.1, client_command_channels.1, client_response_channels.0);

        let server_packet_channels = unbounded::<Packet>();
        server_neighbors.insert(2, drone_packet_channels.0);
        let server = Server::new(2, server_packet_channels.1, neighbors);

        let drone = RustafarianDrone::new(1, drone_event_channels.0, drone_command_channels.1, drone_packet_channels.0, drone_packet_channels.1, 0);

        let controller = SimulationController::new(nodes_channels, drone_channels, handles, topology);

        // Create a packet to send from client to server
        let packet = Packet {
            pack_type: PacketType::MsgFragment(Fragment { fragment_index: 0, total_n_fragments: 1, length: 3, data: [0; 128] }),
            session_id: 1,
            routing_header: SourceRoutingHeader {
                hops: vec![1, 2, 3],
                hop_index: 1, // Client -> Controller -> Server
            },
        };

        // Simulate sending the packet from client to server
        let result = controller.handle_controller_shortcut(packet);

        // Check if the packet was forwarded successfully
        assert!(result.is_ok());
        let packet_forwarded = result.unwrap();
        assert_eq!(packet_forwarded.session_id, 1);
        assert_eq!(packet_forwarded.packet_type, "MsgFragment");
        assert_eq!(packet_forwarded.source, 1);
        assert_eq!(packet_forwarded.destination, 3);
    }


    #[test]
    fn test_simulation_controller_build_complex_topology() {
        let config_str = "tests/configurations/test_complex_config.toml";
        assert!(
            std::path::Path::new(config_str).exists(),
            "Config file does not exist at the specified path"
        );
        let controller = SimulationController::build(config_str);

        assert_eq!(controller.drone_channels.len(), 3);
        assert_eq!(controller.nodes_channels.len(), 3);
        assert_eq!(controller.handles.len(), 6);
        assert_eq!(controller.topology.nodes().len(), 6);

        // Check the topology
        let edges = controller.topology.edges();
        assert!(edges.len() == 6);
        assert!(edges.get(&1).unwrap().contains(&2));
        assert!(edges.get(&1).unwrap().contains(&3));
        assert!(edges.get(&1).unwrap().contains(&4));
        assert!(edges.get(&1).unwrap().contains(&6));

        assert!(edges.get(&2).unwrap().contains(&1));
        assert!(edges.get(&2).unwrap().contains(&3));
        assert!(edges.get(&2).unwrap().contains(&4));
        assert!(edges.get(&2).unwrap().contains(&5));

        assert!(edges.get(&3).unwrap().contains(&1));
        assert!(edges.get(&3).unwrap().contains(&2));
        assert!(edges.get(&3).unwrap().contains(&5));
        assert!(edges.get(&3).unwrap().contains(&6));

        assert!(edges.get(&4).unwrap().contains(&1));
        assert!(edges.get(&4).unwrap().contains(&2));

        assert!(edges.get(&5).unwrap().contains(&2));
        assert!(edges.get(&5).unwrap().contains(&3));

        assert!(edges.get(&6).unwrap().contains(&1));
        assert!(edges.get(&6).unwrap().contains(&3));
    }

    #[test]
    fn test_handle_controller_shortcut_success() {
        let nodes_channels = HashMap::new();
        let drone_channels = HashMap::new();
        let handles = Vec::new();
        let topology = Topology::new();

        let controller = SimulationController::new(nodes_channels, drone_channels, handles, topology);

        let packet = Packet {
            pack_type: PacketType::MsgFragment(Fragment { fragment_index: 0, total_n_fragments: 2, length: 3, data: [0; 128] }),
            session_id: 1,
            routing_header: RoutingHeader {
                hops: vec![1, 2],
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        assert!(result.is_ok());
        let packet_forwarded = result.unwrap();
        assert_eq!(packet_forwarded.session_id, 1);
        assert_eq!(packet_forwarded.packet_type, "MsgFragment");
        assert_eq!(packet_forwarded.source, 1);
        assert_eq!(packet_forwarded.destination, 2);
    }

    #[test]
    fn test_handle_controller_shortcut_failure() {
        let nodes_channels = HashMap::new();
        let drone_channels = HashMap::new();
        let handles = Vec::new();
        let topology = Topology::new();

        let controller = SimulationController::new(nodes_channels, drone_channels, handles, topology);

        let packet = Packet {
            pack_type: PacketType::MsgFragment(Fragment  { fragment_index: 0, total_n_fragments: 2, length: 3, data: [0; 128] }),
            session_id: 1,
            routing_header: RoutingHeader {
                hops: vec![1, 3],
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        assert!(result.is_err());
    }

    #[test]
    fn test_simulation_controller_parse_config_invalid_file() {
        let config_str = "invalid/path/to/config.toml";

        let result = std::panic::catch_unwind(|| SimulationController::parse_config(config_str));

        assert!(result.is_err());
    }

    #[test]
    fn test_simulation_controller_parse_config_invalid_toml() {
        let config_str = "tests/configurations/invalid_config.toml";

        let result = std::panic::catch_unwind(|| SimulationController::parse_config(config_str));

        assert!(result.is_err());
    }

    
}
