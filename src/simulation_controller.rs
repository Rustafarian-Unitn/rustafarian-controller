use crate::config_parser;
use crate::drone_functions::rustafarian_drone;
use crate::runnable::Runnable;
use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::Error;
use rustafarian_chat_server::chat_server::ChatServer;
use rustafarian_client::browser_client::BrowserClient;
use rustafarian_client::chat_client::ChatClient;
use rustafarian_client::client::Client;
use rustafarian_content_server::content_server::ContentServer;
use rustafarian_shared::messages::commander_messages::SimControllerEvent::PacketForwarded;
use rustafarian_shared::messages::commander_messages::{
    SimControllerCommand, SimControllerEvent, SimControllerResponseWrapper,
};
use rustafarian_shared::messages::general_messages::ServerType;
use rustafarian_shared::topology::Topology;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use wg_2024::config::{Client as ClientConfig, Drone as DroneConfig, Server as ServerConfig};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

pub const TICKS: u64 = u64::MAX;
pub const FILE_FOLDER: &str = "resources/files";
pub const MEDIA_FOLDER: &str = "resources/media";
pub const DEBUG: bool = true;
///Internal channel management structures to distribute the channels among the instances of the topology
#[derive(Debug)]
pub struct NodeChannels {
    pub send_packet_channel: Sender<Packet>,
    pub receive_packet_channel: Receiver<Packet>,
    pub send_command_channel: Sender<SimControllerCommand>,
    pub receive_command_channel: Receiver<SimControllerCommand>,
    pub receive_response_channel: Receiver<SimControllerResponseWrapper>,
    pub send_response_channel: Sender<SimControllerResponseWrapper>,
}
///Internal channel management structures to distribute the channels among the instances of the topology
#[derive(Debug)]
pub struct DroneChannels {
    pub send_command_channel: Sender<DroneCommand>,
    pub receive_command_channel: Receiver<DroneCommand>,
    pub send_packet_channel: Sender<Packet>,
    pub receive_packet_channel: Receiver<Packet>,
    pub receive_event_channel: Receiver<DroneEvent>,
    pub send_event_channel: Sender<DroneEvent>,
}

/// A factory function that creates drone instances for every drone configuration and drone acquired
type DroneFactory = fn(
    id: NodeId,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<NodeId, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String);

/// A controller that manages the simulation of a network composed of drones, clients, and servers.
///
/// The `SimulationController` is responsible for:
/// - Initializing and managing communication channels between different network components
/// - Setting up the network topology
/// - Handling packet routing between nodes
///
/// # Methods
///
/// - `new`: Creates a new instance with pre-configured channels and topology
/// - `build`: Constructs a controller from a configuration string
/// - `init_drones`: Initializes drone nodes in the network
/// - `init_clients`: Sets up client nodes (chat and browser clients)
/// - `init_servers`: Configures server nodes (chat, media, and text servers)
/// - `handle_controller_shortcut`: Processes direct packet routing between nodes
///
/// # Components
///
/// The controller manages three types of nodes:
/// - Drones: Network routing nodes
/// - Clients: End-user applications (chat and browser clients)
/// - Servers: Service providers (chat, media, and text servers)
///
/// Each node type has its own communication channels and is registered in the network topology.
///
/// # Communication
///
/// The controller uses crossbeam channels for:
/// - Command distribution
/// - Event handling
/// - Packet routing
/// - Response processing
///
/// # Topology
///
/// Maintains a graph-like structure representing network connections between:
/// - Drones to drones
/// - Clients to drones
/// - Servers to drones
#[derive(Debug)]
pub struct SimulationController {
    pub topology: Topology,
    pub nodes_channels: HashMap<NodeId, NodeChannels>,
    pub drone_channels: HashMap<NodeId, DroneChannels>,
    pub handles: Vec<Option<JoinHandle<()>>>,
}

impl SimulationController {
    pub fn new(
        nodes_channels: HashMap<NodeId, NodeChannels>,
        drone_channels: HashMap<NodeId, DroneChannels>,
        handles: Vec<Option<JoinHandle<()>>>,
        topology: Topology,
    ) -> Self {
        SimulationController {
            topology,
            nodes_channels,
            drone_channels,
            handles,
        }
    }

    /// Builds a simulation controller from a configuration string.
    /// # Arguments
    /// * `config` - A string containing the path to the configuration for the simulation
    /// # Returns
    /// A `SimulationController` instance with the network topology and channels set up.
    pub fn build(config: &str, debug_mode: bool) -> Self {
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
        Self::init_channels(
            &config,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
        );

        Self::init_drones(
            &mut handles,
            config.drone,
            &mut drone_factories,
            &mut drone_channels,
            &mut node_channels,
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
            debug_mode,
        );

        SimulationController::new(node_channels, drone_channels, handles, topology)
    }

    fn init_channels(
        config: &wg_2024::config::Config,
        node_channels: &mut HashMap<NodeId, NodeChannels>,
        drone_channels: &mut HashMap<NodeId, DroneChannels>,
        topology: &mut Topology,
    ) {
        for drone_config in config.drone.iter() {
            let (send_command_channel, receive_command_channel) = unbounded::<DroneCommand>();
            let (send_event_channel, receive_event_channel) = unbounded::<DroneEvent>();
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

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
            // Build the topology to represent the network in the frontend
            topology.add_node(drone_config.id);
            drone_config.connected_node_ids.iter().for_each(|node_id| {
                topology.add_edge(drone_config.id, *node_id);
            });

            topology.set_node_type(drone_config.id, "drone".to_string());
        }

        for client_config in config.client.iter() {
            let (send_command_channel, receive_command_channel) =
                unbounded::<SimControllerCommand>();
            let (send_response_channel, receive_response_channel) =
                unbounded::<SimControllerResponseWrapper>();
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

            node_channels.insert(
                client_config.id,
                NodeChannels {
                    send_packet_channel,
                    receive_packet_channel,
                    send_command_channel,
                    receive_command_channel,
                    receive_response_channel,
                    send_response_channel,
                },
            );
        }

        for server_config in config.server.iter() {
            let (send_command_channel, receive_command_channel) =
                unbounded::<SimControllerCommand>();
            let (send_response_channel, receive_response_channel) =
                unbounded::<SimControllerResponseWrapper>();
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

            node_channels.insert(
                server_config.id,
                NodeChannels {
                    send_packet_channel,
                    receive_packet_channel,
                    send_command_channel,
                    receive_command_channel,
                    receive_response_channel,
                    send_response_channel,
                },
            );
        }
    }

    /// Initializes the drone nodes in the network.
    /// # Arguments
    /// * `handles` - A mutable reference to the vector of thread handles
    /// * `drones_config` - A vector of drone configurations parsed from the configuration file
    /// * `drone_factories` - A mutable reference to the drone factory iterator which returns in a round robin fashion a drone factory for each drone acquired
    /// * `drone_channels` - A mutable reference to the drone channels hashmap for communication
    /// * `topology` - A mutable reference to the network topology. This is used to represent the network in the frontend
    /// # Returns
    /// A vector of thread handles for the drone instances
    fn init_drones(
        handles: &mut Vec<Option<JoinHandle<()>>>,
        drones_config: Vec<DroneConfig>,
        drone_factories: &mut dyn Iterator<Item = DroneFactory>,
        drone_channels: &mut HashMap<NodeId, DroneChannels>,
        node_channels: &mut HashMap<NodeId, NodeChannels>,
        topology: &mut Topology,
    ) {
        // For each drone config pick the next factory in a circular fashion to generate a drone instance
        for drone_config in drones_config.iter() {
            // Get the next drone in line
            let factory = drone_factories.next().unwrap();

            // Neighbouring drones
            let neighbor_channels: HashMap<NodeId, Sender<Packet>> = drone_channels
                .iter()
                .filter(|(k, _)| drone_config.connected_node_ids.contains(k))
                .map(|(k, v)| (*k, v.send_packet_channel.clone()))
                .chain(
                    node_channels
                        .iter()
                        .filter(|(k, _)| drone_config.connected_node_ids.contains(k))
                        .map(|(k, v)| (*k, v.send_packet_channel.clone())),
                )
                .collect();

            let drone_channels = drone_channels.get(&drone_config.id).unwrap();

            println!(
                "Drone {} has neighbours {:?}",
                drone_config.id,
                neighbor_channels.keys()
            );

            let (mut drone, name) = factory(
                drone_config.id,
                drone_channels.send_event_channel.clone(),
                drone_channels.receive_command_channel.clone(),
                drone_channels.receive_packet_channel.clone(),
                neighbor_channels,
                drone_config.pdr,
            );
            topology.set_label(drone_config.id, name);

            handles.push(Some(thread::spawn(move || drone.run())));
        }
    }

    /// Initializes the client nodes in the network.
    /// # Arguments
    /// * `handles` - A mutable reference to the vector of thread handles
    /// * `clients_config` - A vector of client configurations parsed from the configuration file
    /// * `node_channels` - A mutable reference to the node channels hashmap for communication
    /// * `drone_channels` - A mutable reference to the drone channels hashmap for communication
    /// * `topology` - A mutable reference to the network topology. This is used to represent the network in the frontend
    /// # Returns
    /// A vector of thread handles for the client instances

    fn init_clients(
        handles: &mut Vec<Option<JoinHandle<()>>>,
        clients_config: Vec<ClientConfig>,
        node_channels: &mut HashMap<NodeId, NodeChannels>,
        drone_channels: &mut HashMap<NodeId, DroneChannels>,
        topology: &mut Topology,
    ) {
        for client_config in clients_config {
            // Register assigned neighbouring drones
            let neighbour_drones = drone_channels
                .iter()
                .filter(|(k, _)| client_config.connected_drone_ids.contains(k))
                .map(|(k, v)| (*k, v.send_packet_channel.clone()))
                .collect();

            // Register the client's node in the topology
            topology.add_node(client_config.id);
            if client_config.id % 2 == 0 {
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

            let receive_packet_channel = node_channels
                .get(&client_config.id)
                .unwrap()
                .receive_packet_channel
                .clone();
            let receive_command_channel = node_channels
                .get_mut(&client_config.id)
                .unwrap()
                .receive_command_channel
                .clone();
            let send_response_channel = node_channels
                .get(&client_config.id)
                .unwrap()
                .send_response_channel
                .clone();

            // Start off the client
            handles.push(Some(thread::spawn(move || {
                if client_config.id % 2 == 0 {
                    let mut client = ChatClient::new(
                        client_config.id,
                        neighbour_drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                        true
                    );
                    client.run(TICKS)
                } else {
                    let mut client = BrowserClient::new(
                        client_config.id,
                        neighbour_drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                        true
                    );
                    client.run(TICKS)
                }
            })));
        }
    }

    /// Initializes the server nodes in the network.
    /// # Arguments
    /// * `handles` - A mutable reference to the vector of thread handles
    /// * `servers_config` - A vector of server configurations parsed from the configuration file
    /// * `node_channels` - A mutable reference to the node channels hashmap for communication
    /// * `drone_channels` - A mutable reference to the drone channels hashmap for communication
    /// * `topology` - A mutable reference to the network topology. This is used to represent the network in the frontend
    /// # Returns
    /// A vector of thread handles for the server instances
    ///
    fn init_servers(
        handles: &mut Vec<Option<JoinHandle<()>>>,
        servers_config: Vec<ServerConfig>,
        node_channels: &mut HashMap<NodeId, NodeChannels>,
        drone_channels: &mut HashMap<NodeId, DroneChannels>,
        topology: &mut Topology,
        debug_mode: bool,
    ) {
        // Generate the file and media folders
        let _ = std::fs::create_dir_all(FILE_FOLDER);
        let _ = std::fs::create_dir_all(MEDIA_FOLDER);

        // For each drone config pick the next factory in a circular fashion to generate a drone instance
        for server_config in servers_config {
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

            if server_config.id % 3 == 0 {
                topology.set_label(server_config.id, "Chat server".to_string());
                topology.set_node_type(server_config.id, "Chat".to_string());
            } else if server_config.id % 3 == 1 {
                topology.set_label(server_config.id, "Media server".to_string());
                topology.set_node_type(server_config.id, "Media".to_string());
            } else {
                topology.set_label(server_config.id, "Text server".to_string());
                topology.set_node_type(server_config.id, "Text".to_string());
            }

            let receive_packet_channel = node_channels
                .get(&server_config.id)
                .unwrap()
                .receive_packet_channel
                .clone();
            let receive_command_channel = node_channels
                .get_mut(&server_config.id)
                .unwrap()
                .receive_command_channel
                .clone();
            let send_response_channel = node_channels
                .get(&server_config.id)
                .unwrap()
                .send_response_channel
                .clone();

            // Start off the server
            handles.push(Some(thread::spawn(move || {
                if server_config.id % 3 == 0 {
                    let mut server = ChatServer::new(
                        server_config.id,
                        receive_command_channel,
                        send_response_channel,
                        receive_packet_channel,
                        drones,
                        debug_mode,
                    );
                    println!("Server {} is running", server_config.id);
                    server.run()
                } else if server_config.id % 3 == 1 {
                    let mut server = ContentServer::new(
                        server_config.id,
                        drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                        FILE_FOLDER,
                        MEDIA_FOLDER,
                        ServerType::Media,
                        debug_mode
                    );
                    println!(
                        "Server {} of type {:?}is running",
                        server_config.id,
                        ServerType::Media
                    );

                    server.run()
                } else {
                    let mut server = ContentServer::new(
                        server_config.id,
                        drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                        FILE_FOLDER,
                        MEDIA_FOLDER,
                        ServerType::Text,
                        debug_mode
                    );
                    println!(
                        "Server {} of type {:?}is running",
                        server_config.id,
                        ServerType::Text
                    );

                    server.run()
                }
            })));
        }
    }

    /// Handles direct packet routing between nodes.
    /// # Arguments
    /// * `packet` - A packet to be routed between nodes
    /// # Returns
    /// A `SimControllerEvent` representing the event of the packet being forwarded
    /// # Errors
    /// Returns an error if the packet could not be sent to the destination node
    ///
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

        assert!(controller.topology.nodes().is_empty());
        assert!(controller.nodes_channels.is_empty());
        assert!(controller.drone_channels.is_empty());
        assert!(controller.handles.is_empty());
    }

    #[test]
    fn test_simulation_controller_build() {
        let config_str = "src/tests/configurations/test_config.toml";

        let controller = SimulationController::build(config_str, false);

        assert_eq!(controller.drone_channels.len(), 1);
        assert_eq!(controller.nodes_channels.len(), 2);
        assert_eq!(controller.handles.len(), 3);
        assert_eq!(controller.topology.nodes().len(), 3);
    }

    #[test]
    fn test_init_drones() {
        let mut handles = Vec::new();
        let drone_factories: Vec<DroneFactory> = vec![rustafarian_drone];
        let mut drone_factories = drone_factories.into_iter().cycle();
        let mut drone_channels = HashMap::new();
        let mut node_channels = HashMap::new();
        let mut topology = Topology::new();
        let config = config_parser::parse_config("src/tests/configurations/test_config.toml");
        SimulationController::init_channels(
            &config,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
        );

        let drones_config = config.drone;
        SimulationController::init_drones(
            &mut handles,
            drones_config,
            &mut drone_factories,
            &mut drone_channels,
            &mut node_channels,
            &mut topology,
        );

        assert_eq!(drone_channels.len(), 1);

        assert_eq!(handles.len(), 1);
    }

    #[test]
    fn test_init_clients() {
        let mut handles = Vec::new();
        let config = config_parser::parse_config("src/tests/configurations/test_config.toml");
        let mut node_channels = HashMap::new();
        let mut drone_channels = HashMap::new();
        let mut topology = Topology::new();

        SimulationController::init_channels(
            &config,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
        );

        let clients_config = config.client;
        SimulationController::init_clients(
            &mut handles,
            clients_config,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
        );

        assert_eq!(node_channels.len(), 2);
        assert_eq!(handles.len(), 1);
    }

    #[test]
    fn test_init_servers() {
        let mut handles = Vec::new();
        let config = config_parser::parse_config("src/tests/configurations/test_config.toml");
        let mut node_channels = HashMap::new();
        let mut drone_channels = HashMap::new();
        let mut topology = Topology::new();

        SimulationController::init_channels(
            &config,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
        );
        let servers_config = config.server;

        SimulationController::init_servers(
            &mut handles,
            servers_config,
            &mut node_channels,
            &mut drone_channels,
            &mut topology,
            false,
        );

        assert_eq!(node_channels.len(), 2);
        assert_eq!(handles.len(), 1);
    }

    #[test]
    fn test_handle_controller_shortcut_success() {
        let (_, _, _, _, controller) = setup::setup();

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
        let (_, _, _, _, controller) = setup::setup();

        let packet = Packet {
            pack_type: PacketType::MsgFragment(Fragment {
                fragment_index: 0,
                total_n_fragments: 2,
                length: 3,
                data: [0; 128],
            }),
            session_id: 1,
            routing_header: SourceRoutingHeader {
                hops: vec![1, 2, 14],
                hop_index: 1,
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        assert!(result.is_err());
    }

    // test circular drone factory
    #[test]
    fn test_drone_factories_cycle() {
        let drone_factories: Vec<DroneFactory> = vec![rustafarian_drone];
        let mut drone_factories = drone_factories.into_iter().cycle();

        let drone1 = drone_factories.next().unwrap();
        let drone2 = drone_factories.next().unwrap();
        let drone3 = drone_factories.next().unwrap();
        let drone4 = drone_factories.next().unwrap();

        assert_eq!(
            drone1
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            rustafarian_drone
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String)
        );
        assert_eq!(
            drone2
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            rustafarian_drone
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String)
        );
        assert_eq!(
            drone3
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            rustafarian_drone
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String)
        );
        assert_eq!(
            drone4
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            rustafarian_drone
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String)
        );
    }
}
