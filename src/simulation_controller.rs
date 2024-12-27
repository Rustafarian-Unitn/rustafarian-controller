use crate::config_parser;
use crate::drone_functions::rustafarian_drone;
use crate::runnable::Runnable;
use crate::server::Server;
use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::Error;
use rustafarian_client::browser_client::BrowserClient;
use rustafarian_client::chat_client::ChatClient;
use rustafarian_client::client::Client;
use rustafarian_shared::messages::commander_messages::SimControllerEvent::PacketForwarded;
use rustafarian_shared::messages::commander_messages::{
    SimControllerCommand, SimControllerEvent, SimControllerResponseWrapper,
};
use rustafarian_shared::topology::Topology;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use wg_2024::config::{Client as ClientConfig, Drone as DroneConfig, Server as ServerConfig};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

pub const TICKS: u64 = 100;

pub struct NodeChannels {
    pub send_packet_channel: Sender<Packet>,
    pub receive_packet_channel: Receiver<Packet>,
    pub send_command_channel: Sender<SimControllerCommand>,
    pub receive_response_channel: Receiver<SimControllerResponseWrapper>,
    pub send_response_channel: Sender<SimControllerResponseWrapper>,
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
) -> (Box<dyn Runnable>, String);

pub struct SimulationController {
    pub topology: Topology,
    pub nodes_channels: HashMap<NodeId, NodeChannels>,
    pub drone_channels: HashMap<NodeId, DroneChannels>,
    pub handles: Vec<JoinHandle<()>>,
}

impl SimulationController {
    pub fn new(
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
        let config = config_parser::parse_config(config);
        // let clients: Vec<ChatClient> = Vec::new();
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
                topology.add_edge(drone_config.id, *node_id); });

            topology.set_node_type(drone_config.id, "Drone".to_string());
        }

        // For each drone config pick the next factory in a circular fashion to generate a drone instance
        for drone_config in drones_config.iter() {
            // Get the next drone in line
            let factory = drone_factories.next().unwrap();

            // Neighbouring nodes
            let neighbor_channels: HashMap<NodeId, Sender<Packet>> = drone_channels
                .iter()
                .filter(|(k, _)| drone_config.connected_node_ids.contains(k))
                .map(|(k, v)| (*k, v.send_packet_channel.clone()))
                .collect();

            let drone_channels = drone_channels.get(&drone_config.id).unwrap();

            let (mut drone, name) = factory(
                drone_config.id,
                drone_channels.send_event_channel.clone(),
                drone_channels.receive_command_channel.clone(),
                drone_channels.receive_packet_channel.clone(),
                neighbor_channels,
                drone_config.pdr,
            );
            topology.set_label(drone_config.id, name);

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
        let mut counter = 0;
        for client_config in clients_config {
            counter += 1;

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
                    send_packet_channel,                                    // network comm.
                    receive_packet_channel: receive_packet_channel.clone(), // network comm.
                    send_command_channel,     // sc sends commands to client
                    receive_response_channel, // sc receives responses the client got from the servers
                    send_response_channel: send_response_channel.clone(), // sc sends responses to the client
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
            if counter % 2 == 0 {
                topology.set_label(client_config.id, "Chat client".to_string());
                topology.set_node_type(client_config.id, "chat_client".to_string());
            } else {
                topology.set_label(client_config.id, "Browser client".to_string());
                topology.set_node_type(client_config.id, "browser_client".to_string());
            }

            client_config
                .connected_drone_ids
                .iter()
                .for_each(|node_id| {
                    topology.add_edge(client_config.id, *node_id);
                });

            // Start off the client
            handles.push(thread::spawn(move || {
                if counter % 2 == 0 {
                    let mut client = ChatClient::new(
                        client_config.id,
                        neighbour_drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                    );
                    client.run(TICKS)
                } else {
                    let mut client = BrowserClient::new(
                        client_config.id,
                        neighbour_drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                    );
                    client.run(TICKS)
                }
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
        let mut counter = 0;
        // For each drone config pick the next factory in a circular fashion to generate a drone instance
        for server_config in servers_config {
            counter += 1;
            let (send_command_channel, _) = unbounded::<SimControllerCommand>();

            let (send_response_channel, receive_response_channel) =
                unbounded::<SimControllerResponseWrapper>();

            // Generate the channels for inter-drone communication
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

            // Save the client's channel counterparts for later use
            node_channels.insert(
                server_config.id,
                NodeChannels {
                    send_packet_channel,
                    receive_packet_channel: receive_packet_channel.clone(),
                    send_command_channel,
                    receive_response_channel,
                    send_response_channel: send_response_channel.clone(),
                },
            );

            let drones = drone_channels
                .iter()
                .filter(|(k, _)| server_config.connected_drone_ids.contains(k))
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

            if counter % 3 == 0 {
                topology.set_label(server_config.id, "Chat server".to_string());
                topology.set_node_type(server_config.id, "chat_server".to_string());
            } else if counter % 3 == 1 {
                topology.set_label(server_config.id, "Media server".to_string());
                topology.set_node_type(server_config.id, "media_server".to_string());
            } else {
                topology.set_label(server_config.id, "Text server".to_string());
                topology.set_node_type(server_config.id, "text_server".to_string());
            }

            // Start off the server
            handles.push(thread::spawn(move || {
                if counter % 3 == 0 {
                    let mut server = Server::new(server_config.id, receive_packet_channel, drones);
                    server.run()
                } else if counter % 3 == 1 {
                    let mut server = Server::new(server_config.id, receive_packet_channel, drones);
                    server.run()
                } else {
                    let mut server = Server::new(server_config.id, receive_packet_channel, drones);
                    server.run()
                }
            }));
        }
    }

    pub fn handle_controller_shortcut(&self, packet: Packet) -> Result<SimControllerEvent, Error> {
        let packet_type = packet.pack_type.clone();
        let session_id = packet.session_id;
        let source = packet.routing_header.hops[0];
        let destination = packet.routing_header.hops[packet.routing_header.hops.len() - 1];

        // Send the packet to the destination node and return the event
        if let Some(node_channels) = self.nodes_channels.get(&destination) {
            if node_channels.send_packet_channel.send(packet) == Ok(()) {
                {
                    match packet_type {
                        PacketType::MsgFragment(fragment) => Ok(PacketForwarded {
                            session_id,
                            packet_type: fragment.to_string(),
                            source,
                            destination,
                        }),
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
                }
            } else {
                Err(Error::new("Failed to send packet"))
            }
        } else {
            Err(Error::new("Failed to send packet"))
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////// --TESTS -- ///////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drone_functions::rustafarian_drone;
    use crate::tests::setup;
    use rustafarian_shared::topology::Topology;
    use std::collections::HashMap;
    use wg_2024::{
        network::SourceRoutingHeader,
        packet::{Fragment, Packet, PacketType},
    };

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
        let config_str = "src/tests/configurations/test_config.toml";

        let controller = SimulationController::build(config_str);

        assert_eq!(controller.drone_channels.len(), 1);
        assert_eq!(controller.nodes_channels.len(), 2);
        assert_eq!(controller.handles.len(), 3);
        assert_eq!(controller.topology.nodes().len(), 3);
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
    fn test_handle_controller_shortcut_success() {
        let (_, _, controller) = setup::setup();

        let server_receive_packet_channel = controller
            .nodes_channels
            .get(&3)
            .unwrap()
            .receive_packet_channel
            .clone();

        let packet = Packet {
            pack_type: PacketType::MsgFragment(Fragment {
                fragment_index: 0,
                total_n_fragments: 2,
                length: 3,
                data: [0; 128],
            }),
            session_id: 1,
            routing_header: SourceRoutingHeader {
                hops: vec![1, 2, 3],
                hop_index: 1,
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        let message = server_receive_packet_channel.recv().unwrap();

        assert!(matches!(message.pack_type, PacketType::MsgFragment(_)));
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), PacketForwarded { .. }));
    }

    #[test]
    fn test_handle_controller_shortcut_failure() {
        let (_, _, controller) = setup::setup();

        let packet = Packet {
            pack_type: PacketType::MsgFragment(Fragment {
                fragment_index: 0,
                total_n_fragments: 2,
                length: 3,
                data: [0; 128],
            }),
            session_id: 1,
            routing_header: SourceRoutingHeader {
                hops: vec![1, 2, 4],
                hop_index: 1,
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        assert!(result.is_err());
    }
}
