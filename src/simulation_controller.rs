use crate::config_parser;
use crate::drone_functions::{
    cpp_enjoyers_drone, d_r_o_n_e_drone, dr_one_drone, get_droned_drone, lockheed_rustin_drone,
    rust_busters_drone, rust_do_it_drone, rustastic_drone, rusteze_drone, rusty_drone,
};
use crate::runnable::Runnable;
use crossbeam_channel::{unbounded, Receiver, RecvTimeoutError, Sender};
use rand::Error;
use rustafarian_chat_server::chat_server::ChatServer;
use rustafarian_client::browser_client::BrowserClient;
use rustafarian_client::chat_client::ChatClient;
use rustafarian_client::client::Client;
use rustafarian_content_server::content_server::ContentServer;
use rustafarian_shared::logger::{LogLevel, Logger};
use rustafarian_shared::messages::commander_messages::SimControllerEvent::PacketForwarded;
use rustafarian_shared::messages::commander_messages::{
    SimControllerCommand, SimControllerEvent, SimControllerResponseWrapper,
};
use rustafarian_shared::messages::general_messages::ServerType;
use rustafarian_shared::topology::Topology;
use std::collections::HashMap;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use wg_2024::config::{Client as ClientConfig, Drone as DroneConfig, Server as ServerConfig};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::network::NodeId;
use wg_2024::packet::{Packet, PacketType};

pub const TICKS: u64 = u64::MAX;
pub const FILE_FOLDER: &str = "resources/files";
pub const MEDIA_FOLDER: &str = "resources/media";

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

pub struct ControllerConfig {
    pub nodes_channels: HashMap<NodeId, NodeChannels>,
    pub drones_channels: HashMap<NodeId, DroneChannels>,
    pub shutdown_channel: (Sender<()>, Receiver<()>),
    pub handles: Vec<Option<JoinHandle<()>>>,
    pub topology: Topology,
    pub logger: Logger,
    pub debug_mode: bool,
}

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
    pub drones_channels: HashMap<NodeId, DroneChannels>,
    shutdown_channel: (Sender<()>, Receiver<()>),
    pub handles: Vec<Option<JoinHandle<()>>>,
    pub logger: Logger,
}

impl SimulationController {
    const SHUTDOWN_TIMEOUT: Duration = Duration::from_millis(200);
    const THREAD_SLEEP: Duration = Duration::from_millis(100);

    pub fn new(config: ControllerConfig) -> Self {
        SimulationController {
            topology: config.topology,
            nodes_channels: config.nodes_channels,
            drones_channels: config.drones_channels,
            shutdown_channel: config.shutdown_channel,
            handles: config.handles,
            logger: config.logger,
        }
    }

    pub fn destroy(&mut self) {
        self.logger
            .log("Destroying simulation controller...", LogLevel::INFO);

        // Send shutdown signal
        if let Err(e) = self.shutdown_channel.0.send(()) {
            self.logger.log(
                &format!("Failed to send shutdown signal: {}", e),
                LogLevel::ERROR,
            );
        }

        // Join threads with timeout
        for handle in self.handles.iter_mut() {
            if let Some(h) = handle.take() {
                match Self::join_with_timeout(h, Self::SHUTDOWN_TIMEOUT) {
                    Ok(_) => self
                        .logger
                        .log("Thread joined successfully", LogLevel::INFO),
                    Err(e) => {
                        self.logger
                            .log(&format!("Thread failed to join: {}", e), LogLevel::ERROR);
                    }
                }
            }
        }

        // Clear topology and channels
        self.topology = Topology::new();
        self.nodes_channels.clear();
        self.drones_channels.clear();
        self.handles.clear();

        self.logger
            .log("Simulation controller destroyed", LogLevel::INFO);
    }

    pub fn rebuild(
        &mut self,
        config: &str,
        file_folder: String,
        media_folder: String,
        debug_mode: bool,
    ) {
        self.destroy();

        let config = config_parser::parse_config(config);

        let logger = Logger::new("Controller".to_string(), 0, debug_mode);

        // Create a factory function for the implementations
        let drone_factories = SimulationController::get_active_drone_factories();
        let mut drone_factories = drone_factories.into_iter().cycle();
        let mut controller_config = ControllerConfig {
            nodes_channels: HashMap::new(),
            drones_channels: HashMap::new(),
            shutdown_channel: self.shutdown_channel.clone(),
            handles: Vec::new(),
            topology: Topology::new(),
            logger,
            debug_mode,
        };

        Self::init_channels(&config, &mut controller_config);

        Self::init_drones(config.drone, &mut drone_factories, &mut controller_config);

        Self::init_clients(config.client, &mut controller_config);

        Self::init_servers(
            config.server,
            &mut controller_config,
            file_folder,
            media_folder,
        );

        self.topology = controller_config.topology;
        self.nodes_channels = controller_config.nodes_channels;
        self.drones_channels = controller_config.drones_channels;
        self.handles = controller_config.handles;
        self.logger = controller_config.logger;
    }

    /// Builds a simulation controller from a configuration string.
    /// # Arguments
    /// * `config` - A string containing the path to the configuration for the simulation
    /// # Returns
    /// A `SimulationController` instance with the network topology and channels set up.
    pub fn build(
        config: &str,
        file_folder: String,
        media_folder: String,
        debug_mode: bool,
    ) -> Self {
        let config = config_parser::parse_config(config);
        let logger = Logger::new("Controller".to_string(), 0, debug_mode);

        logger.log("Building the simulation controller", LogLevel::INFO);

        let (shutdown_sx, shutdown_rx) = unbounded::<()>();

        // Create a factory function for the implementations
        let drone_factories = SimulationController::get_active_drone_factories();

        let mut drone_factories = drone_factories.into_iter().cycle();

        let drones_channels: HashMap<NodeId, DroneChannels> = HashMap::new();
        let node_channels: HashMap<NodeId, NodeChannels> = HashMap::new();

        let handles = Vec::new();

        let topology = Topology::new();

        let mut controller_config = ControllerConfig {
            nodes_channels: node_channels,
            drones_channels,
            shutdown_channel: (shutdown_sx, shutdown_rx),
            handles,
            topology,
            logger,
            debug_mode,
        };

        Self::init_channels(&config, &mut controller_config);

        Self::init_drones(config.drone, &mut drone_factories, &mut controller_config);
        Self::init_clients(config.client, &mut controller_config);

        Self::init_servers(
            config.server,
            &mut controller_config,
            file_folder,
            media_folder,
        );

        SimulationController::new(controller_config)
    }

    fn init_channels(config: &wg_2024::config::Config, controller_config: &mut ControllerConfig) {
        controller_config
            .logger
            .log("Initializing the channels", LogLevel::INFO);

        controller_config
            .logger
            .log("Drone channels", LogLevel::DEBUG);
        for drone_config in config.drone.iter() {
            controller_config.logger.log(
                format!("Creating drone channels for node {}", drone_config.id).as_str(),
                LogLevel::DEBUG,
            );

            let (send_command_channel, receive_command_channel) = unbounded::<DroneCommand>();
            let (send_event_channel, receive_event_channel) = unbounded::<DroneEvent>();
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

            controller_config.drones_channels.insert(
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

            controller_config
                .logger
                .log("Adding drone to topology", LogLevel::DEBUG);
            // Build the topology to represent the network in the frontend
            controller_config.topology.add_node(drone_config.id);

            controller_config
                .logger
                .log("Adding edges to topology", LogLevel::DEBUG);
            drone_config.connected_node_ids.iter().for_each(|node_id| {
                controller_config
                    .topology
                    .add_edge(drone_config.id, *node_id);
            });

            controller_config
                .logger
                .log("Setting node type in topology", LogLevel::DEBUG);
            controller_config
                .topology
                .set_node_type(drone_config.id, "drone".to_string());
        }

        for client_config in config.client.iter() {
            controller_config.logger.log(
                format!("Creating client channels for node {}", client_config.id).as_str(),
                LogLevel::DEBUG,
            );
            let (send_command_channel, receive_command_channel) =
                unbounded::<SimControllerCommand>();
            let (send_response_channel, receive_response_channel) =
                unbounded::<SimControllerResponseWrapper>();
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

            controller_config.nodes_channels.insert(
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
            controller_config.logger.log(
                format!("Creating server channels for node {}", server_config.id).as_str(),
                LogLevel::DEBUG,
            );
            let (send_command_channel, receive_command_channel) =
                unbounded::<SimControllerCommand>();
            let (send_response_channel, receive_response_channel) =
                unbounded::<SimControllerResponseWrapper>();
            let (send_packet_channel, receive_packet_channel) = unbounded::<Packet>();

            controller_config.nodes_channels.insert(
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
    /// * `drones_channels` - A mutable reference to the drone channels hashmap for communication
    /// * `topology` - A mutable reference to the network topology. This is used to represent the network in the frontend
    /// # Returns
    /// A vector of thread handles for the drone instances
    fn init_drones(
        drones_config: Vec<DroneConfig>,
        drone_factories: &mut dyn Iterator<Item = DroneFactory>,
        controller_config: &mut ControllerConfig,
    ) {
        controller_config
            .logger
            .log("Initializing drones", LogLevel::INFO);
        // For each drone config pick the next factory in a circular fashion to generate a drone instance

        for drone_config in drones_config.iter() {
            controller_config.logger.log(
                format!("Creating drone {}", drone_config.id).as_str(),
                LogLevel::DEBUG,
            );
            // Get the next drone in line
            let factory = drone_factories.next().unwrap();

            // Neighbouring drones
            let neighbor_channels: HashMap<NodeId, Sender<Packet>> = controller_config
                .drones_channels
                .iter()
                .filter(|(k, _)| drone_config.connected_node_ids.contains(k))
                .map(|(k, v)| (*k, v.send_packet_channel.clone()))
                .chain(
                    controller_config
                        .nodes_channels
                        .iter()
                        .filter(|(k, _)| drone_config.connected_node_ids.contains(k))
                        .map(|(k, v)| (*k, v.send_packet_channel.clone())),
                )
                .collect();

            let drones_channels = controller_config
                .drones_channels
                .get(&drone_config.id)
                .unwrap();
            let drone_id = drone_config.id;

            let (drone, name) = factory(
                drone_id,
                drones_channels.send_event_channel.clone(),
                drones_channels.receive_command_channel.clone(),
                drones_channels.receive_packet_channel.clone(),
                neighbor_channels,
                drone_config.pdr,
            );

            controller_config.logger.log(
                format!("Creating drone {} with {} factory", drone_id, name.clone()).as_str(),
                LogLevel::DEBUG,
            );

            controller_config.topology.set_label(drone_config.id, name);

            let shutdown_rx = controller_config.shutdown_channel.1.clone();

            controller_config.handles.push(Some(thread::spawn(move || {
                let mut drone = drone;
                loop {
                    match shutdown_rx.recv_timeout(Self::THREAD_SLEEP) {
                        Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                            println!("Drone {} shutting down", drone_id);
                            break;
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            drone.run();
                        }
                    }
                }
            })));
            controller_config.logger.log(
                format!("Drone {} started successfully", drone_id).as_str(),
                LogLevel::DEBUG,
            );
        }
    }

    /// Initializes the client nodes in the network.
    /// # Arguments
    /// * `handles` - A mutable reference to the vector of thread handles
    /// * `clients_config` - A vector of client configurations parsed from the configuration file
    /// * `node_channels` - A mutable reference to the node channels hashmap for communication
    /// * `drones_channels` - A mutable reference to the drone channels hashmap for communication
    /// * `topology` - A mutable reference to the network topology. This is used to represent the network in the frontend
    /// # Returns
    /// A vector of thread handles for the client instances
    fn init_clients(clients_config: Vec<ClientConfig>, controller_config: &mut ControllerConfig) {
        controller_config
            .logger
            .log("Initializing clients", LogLevel::INFO);

        for client_config in clients_config {
            controller_config.logger.log(
                format!("Creating client {}", client_config.id).as_str(),
                LogLevel::DEBUG,
            );
            // Register assigned neighbouring drones
            let neighbour_drones = controller_config
                .drones_channels
                .iter()
                .filter(|(k, _)| client_config.connected_drone_ids.contains(k))
                .map(|(k, v)| (*k, v.send_packet_channel.clone()))
                .collect();

            // Register the client's node in the topology
            controller_config.topology.add_node(client_config.id);
            if client_config.id % 2 == 0 {
                controller_config.logger.log(
                    format!("Node {} is a Chat client", client_config.id).as_str(),
                    LogLevel::DEBUG,
                );
                controller_config
                    .topology
                    .set_label(client_config.id, "Chat client".to_string());
                controller_config
                    .topology
                    .set_node_type(client_config.id, "chat_client".to_string());
            } else {
                controller_config.logger.log(
                    format!("Node {} is a Browser client", client_config.id).as_str(),
                    LogLevel::DEBUG,
                );
                controller_config
                    .topology
                    .set_label(client_config.id, "Browser client".to_string());
                controller_config
                    .topology
                    .set_node_type(client_config.id, "browser_client".to_string());
            }

            client_config
                .connected_drone_ids
                .iter()
                .for_each(|node_id| {
                    controller_config
                        .topology
                        .add_edge(client_config.id, *node_id);
                });

            let receive_packet_channel = controller_config
                .nodes_channels
                .get(&client_config.id)
                .unwrap()
                .receive_packet_channel
                .clone();
            let receive_command_channel = controller_config
                .nodes_channels
                .get_mut(&client_config.id)
                .unwrap()
                .receive_command_channel
                .clone();
            let send_response_channel = controller_config
                .nodes_channels
                .get(&client_config.id)
                .unwrap()
                .send_response_channel
                .clone();

            controller_config.logger.log(
                format!("Starting client {}", client_config.id).as_str(),
                LogLevel::DEBUG,
            );

            let shutdown_rx = controller_config.shutdown_channel.1.clone();

            let client_id = client_config.id;

            let debug_mode = controller_config.debug_mode;
            // Start off the client
            controller_config.handles.push(Some(thread::spawn(move || {
                if client_id % 2 == 0 {
                    let mut client = ChatClient::new(
                        client_config.id,
                        neighbour_drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                        debug_mode,
                    );
                    loop {
                        match shutdown_rx.recv_timeout(Self::THREAD_SLEEP) {
                            Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                                println!("Chat client {} shutting down", client_id);
                                break;
                            }
                            Err(RecvTimeoutError::Timeout) => {
                                client.run(TICKS);
                            }
                        }
                    }
                } else {
                    let mut client = BrowserClient::new(
                        client_config.id,
                        neighbour_drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                        debug_mode,
                    );
                    loop {
                        match shutdown_rx.recv_timeout(Self::THREAD_SLEEP) {
                            Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                                println!("Browser client {} shutting down", client_id);
                                break;
                            }
                            Err(RecvTimeoutError::Timeout) => {
                                client.run(TICKS);
                                thread::sleep(Self::THREAD_SLEEP);
                            }
                        }
                    }
                }
            })));
            controller_config.logger.log(
                format!("Client {} started successfully", client_id).as_str(),
                LogLevel::DEBUG,
            );
        }
    }

    /// Initializes the server nodes in the network.
    /// # Arguments
    /// * `handles` - A mutable reference to the vector of thread handles
    /// * `servers_config` - A vector of server configurations parsed from the configuration file
    /// * `node_channels` - A mutable reference to the node channels hashmap for communication
    /// * `drones_channels` - A mutable reference to the drone channels hashmap for communication
    /// * `topology` - A mutable reference to the network topology. This is used to represent the network in the frontend
    /// # Returns
    /// A vector of thread handles for the server instances
    ///
    fn init_servers(
        servers_config: Vec<ServerConfig>,
        controller_config: &mut ControllerConfig,
        file_folder: String,
        media_folder: String,
    ) {
        controller_config
            .logger
            .log("Initializing servers", LogLevel::INFO);
        // Generate the file and media folders
        let _ = std::fs::create_dir_all(FILE_FOLDER);
        let _ = std::fs::create_dir_all(MEDIA_FOLDER);

        // For each drone config pick the next factory in a circular fashion to generate a drone instance
        for server_config in servers_config {
            let media_folder = media_folder.clone();
            let file_folder = file_folder.clone();

            controller_config.logger.log(
                format!("Creating server {}", server_config.id).as_str(),
                LogLevel::DEBUG,
            );

            let drones = controller_config
                .drones_channels
                .iter()
                .filter(|(k, _)| server_config.connected_drone_ids.contains(k))
                .map(|(k, v)| (*k, v.send_packet_channel.clone()))
                .collect();

            // Register the server's node in the topology
            controller_config.topology.add_node(server_config.id);
            server_config
                .connected_drone_ids
                .iter()
                .for_each(|node_id| {
                    controller_config
                        .topology
                        .add_edge(server_config.id, *node_id);
                });

            if server_config.id % 3 == 0 {
                controller_config.logger.log(
                    format!("Node {} is a Chat server", server_config.id).as_str(),
                    LogLevel::DEBUG,
                );
                controller_config
                    .topology
                    .set_label(server_config.id, "Chat server".to_string());
                controller_config
                    .topology
                    .set_node_type(server_config.id, "Chat".to_string());
            } else if server_config.id % 3 == 1 {
                controller_config.logger.log(
                    format!("Node {} is a Media server", server_config.id).as_str(),
                    LogLevel::DEBUG,
                );
                controller_config
                    .topology
                    .set_label(server_config.id, "Media server".to_string());
                controller_config
                    .topology
                    .set_node_type(server_config.id, "Media".to_string());
            } else {
                controller_config.logger.log(
                    format!("Node {} is a Text server", server_config.id).as_str(),
                    LogLevel::DEBUG,
                );
                controller_config
                    .topology
                    .set_label(server_config.id, "Text server".to_string());
                controller_config
                    .topology
                    .set_node_type(server_config.id, "Text".to_string());
            }

            let receive_packet_channel = controller_config
                .nodes_channels
                .get(&server_config.id)
                .unwrap()
                .receive_packet_channel
                .clone();
            let receive_command_channel = controller_config
                .nodes_channels
                .get_mut(&server_config.id)
                .unwrap()
                .receive_command_channel
                .clone();
            let send_response_channel = controller_config
                .nodes_channels
                .get(&server_config.id)
                .unwrap()
                .send_response_channel
                .clone();

            controller_config.logger.log(
                format!("Starting server {}", server_config.id).as_str(),
                LogLevel::DEBUG,
            );

            let shutdown_rx = controller_config.shutdown_channel.1.clone();
            let server_id = server_config.id;
            let debug_mode = controller_config.debug_mode;
            // Start off the server
            controller_config.handles.push(Some(thread::spawn(move || {
                if server_id % 3 == 0 {
                    let mut server = ChatServer::new(
                        server_config.id,
                        receive_command_channel,
                        send_response_channel,
                        receive_packet_channel,
                        drones,
                        debug_mode,
                    );
                    loop {
                        match shutdown_rx.recv_timeout(Self::THREAD_SLEEP) {
                            Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                                println!("Chat server {} shutting down", server_id);
                                break;
                            }
                            Err(RecvTimeoutError::Timeout) => {
                                server.run();
                            }
                        }
                    }
                } else if server_id % 3 == 1 {
                    let mut server = ContentServer::new(
                        server_config.id,
                        drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                        file_folder.as_str(),
                        media_folder.as_str(),
                        ServerType::Media,
                        debug_mode,
                    );
                    println!(
                        "Server {} of type {:?}is running",
                        server_config.id,
                        ServerType::Media
                    );
                    loop {
                        match shutdown_rx.recv_timeout(Self::THREAD_SLEEP) {
                            Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                                println!("Media server {} shutting down", server_id);
                                break;
                            }
                            Err(RecvTimeoutError::Timeout) => {
                                server.run();
                            }
                        }
                    }
                } else {
                    let mut server = ContentServer::new(
                        server_config.id,
                        drones,
                        receive_packet_channel,
                        receive_command_channel,
                        send_response_channel,
                        file_folder.as_str(),
                        media_folder.as_str(),
                        ServerType::Text,
                        debug_mode,
                    );
                    println!(
                        "Server {} of type {:?}is running",
                        server_config.id,
                        ServerType::Text
                    );
                    loop {
                        match shutdown_rx.recv_timeout(Self::THREAD_SLEEP) {
                            Ok(_) | Err(RecvTimeoutError::Disconnected) => {
                                println!("Text server {} shutting down", server_id);
                                break;
                            }
                            Err(RecvTimeoutError::Timeout) => {
                                server.run();
                            }
                        }
                    }
                }
            })));

            controller_config.logger.log(
                format!("Server {} started successfully", server_config.id).as_str(),
                LogLevel::DEBUG,
            );
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
                    self.logger.log(
                        "Packet forwarded successfully to destination",
                        LogLevel::INFO,
                    );
                    match packet_type {
                        PacketType::MsgFragment(fragment) => {
                            self.logger.log("Packet type: MsgFragment", LogLevel::DEBUG);
                            Ok(PacketForwarded {
                                session_id,
                                packet_type: fragment.to_string(),
                                source,
                                destination,
                            })
                        }
                        PacketType::Ack(ack) => {
                            self.logger.log("Packet type: Ack", LogLevel::DEBUG);
                            Ok(PacketForwarded {
                                session_id,
                                packet_type: ack.to_string(),
                                source,
                                destination,
                            })
                        }
                        PacketType::Nack(nack) => {
                            self.logger.log("Packet type: Nack", LogLevel::DEBUG);
                            Ok(PacketForwarded {
                                session_id,
                                packet_type: nack.to_string(),
                                source,
                                destination,
                            })
                        }
                        PacketType::FloodRequest(flood_request) => {
                            self.logger
                                .log("Packet type: FloodRequest", LogLevel::DEBUG);
                            Ok(PacketForwarded {
                                session_id,
                                packet_type: flood_request.to_string(),
                                source,
                                destination,
                            })
                        }
                        PacketType::FloodResponse(flood_response) => {
                            self.logger
                                .log("Packet type: FloodResponse", LogLevel::DEBUG);
                            Ok(PacketForwarded {
                                session_id,
                                packet_type: flood_response.to_string(),
                                source,
                                destination,
                            })
                        }
                    }
                }
            } else {
                self.logger.log("Failed to send packet", LogLevel::ERROR);
                Err(Error::new("Failed to send packet"))
            }
        } else {
            self.logger
                .log("Destination is not a node", LogLevel::ERROR);
            Err(Error::new("Destination is not a node"))
        }
    }

    fn get_active_drone_factories() -> Vec<DroneFactory> {
        vec![
            cpp_enjoyers_drone,
            get_droned_drone,
            rusteze_drone,
            dr_one_drone,
            rust_do_it_drone,
            lockheed_rustin_drone,
            rustastic_drone,
            rust_busters_drone,
            rusty_drone,
            d_r_o_n_e_drone,
        ]
    }

    fn join_with_timeout(handle: JoinHandle<()>, timeout: Duration) -> Result<(), String> {
        let (tx, rx) = crossbeam_channel::bounded(1);

        thread::spawn(move || {
            let result = handle.join();
            let _ = tx.send(result);
        });

        match rx.recv_timeout(timeout) {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(_)) => Err("Thread panicked".to_string()),
            Err(_) => Err("Thread join timed out".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::setup;
    use rustafarian_shared::topology::Topology;
    use std::{collections::HashMap, time::Duration};
    use wg_2024::{
        network::SourceRoutingHeader,
        packet::{
            Ack, FloodRequest, FloodResponse, Fragment, Nack, NackType, NodeType, Packet,
            PacketType,
        },
    };

    #[test]
    fn test_simulation_controller_new() {
        let nodes_channels = HashMap::new();
        let drones_channels = HashMap::new();
        let handles = Vec::new();
        let logger = Logger::new("Controller".to_string(), 0, false);
        let shutdoun_channel = unbounded::<()>();
        let controller = SimulationController::new(ControllerConfig {
            nodes_channels,
            drones_channels,
            shutdown_channel: shutdoun_channel,
            handles,
            topology: Topology::new(),
            logger,
            debug_mode: false,
        });

        assert!(controller.topology.nodes().is_empty());
        assert!(controller.nodes_channels.is_empty());
        assert!(controller.drones_channels.is_empty());
        assert!(controller.handles.is_empty());
    }

    #[test]
    fn test_simulation_controller_build() {
        let config_str = "src/tests/configurations/topology_1.toml";

        let controller = SimulationController::build(
            config_str,
            MEDIA_FOLDER.to_string(),
            FILE_FOLDER.to_string(),
            false,
        );

        assert_eq!(controller.drones_channels.len(), 5);
        assert_eq!(controller.nodes_channels.len(), 2);
        assert_eq!(controller.handles.len(), 7);
        assert_eq!(controller.topology.nodes().len(), 7);
    }

    #[test]
    fn test_init_drones() {
        let drone_factories: Vec<DroneFactory> = vec![
            cpp_enjoyers_drone,
            get_droned_drone,
            rusteze_drone,
            dr_one_drone,
            rust_do_it_drone,
            rust_busters_drone,
            rusty_drone,
            d_r_o_n_e_drone,
        ];
        let mut drone_factories = drone_factories.into_iter().cycle();
        let drones_channels = HashMap::new();
        let nodes_channels = HashMap::new();
        let topology = Topology::new();
        let logger = Logger::new("Controller".to_string(), 0, false);

        let config =
            config_parser::parse_config("src/tests/configurations/topology_20_drones.toml");

        let mut controller_config = ControllerConfig {
            nodes_channels,
            drones_channels,
            shutdown_channel: (unbounded::<()>()),
            handles: Vec::new(),
            topology,
            logger,
            debug_mode: false,
        };

        SimulationController::init_channels(&config, &mut controller_config);
        let drones_config = config.drone;

        SimulationController::init_drones(
            drones_config,
            &mut drone_factories,
            &mut controller_config,
        );

        assert_eq!(controller_config.drones_channels.len(), 20);

        assert_eq!(controller_config.handles.len(), 20);
    }

    #[test]
    fn test_init_clients() {
        let config = config_parser::parse_config("src/tests/configurations/topology_1.toml");
        let nodes_channels = HashMap::new();
        let drones_channels = HashMap::new();
        let topology = Topology::new();
        let logger = Logger::new("Controller".to_string(), 0, false);
        let shutdown_channel = unbounded::<()>();

        let mut controller_config = ControllerConfig {
            nodes_channels,
            drones_channels,
            shutdown_channel,
            handles: Vec::new(),
            topology,
            logger,
            debug_mode: false,
        };
        SimulationController::init_channels(&config, &mut controller_config);

        let clients_config = config.client;
        SimulationController::init_clients(clients_config, &mut controller_config);

        assert_eq!(controller_config.nodes_channels.len(), 2);
        assert_eq!(controller_config.handles.len(), 1);
    }

    #[test]
    fn test_init_servers() {
        let config = config_parser::parse_config("src/tests/configurations/topology_1.toml");
        let nodes_channels = HashMap::new();
        let drones_channels = HashMap::new();
        let topology = Topology::new();
        let logger = Logger::new("Controller".to_string(), 0, false);
        let shutdown_channel = unbounded::<()>();

        let mut controller_config = ControllerConfig {
            nodes_channels,
            drones_channels,
            shutdown_channel,
            handles: Vec::new(),
            topology,
            logger,
            debug_mode: false,
        };

        SimulationController::init_channels(&config, &mut controller_config);
        let servers_config = config.server;
        SimulationController::init_servers(
            servers_config,
            &mut controller_config,
            MEDIA_FOLDER.to_string(),
            FILE_FOLDER.to_string(),
        );

        assert_eq!(controller_config.nodes_channels.len(), 2);
        assert_eq!(controller_config.handles.len(), 1);
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
    fn test_handle_controller_shortcuts() {
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
                hops: vec![1, 2, 3],
                hop_index: 1,
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(matches!(event, PacketForwarded { .. }));

        let packet = Packet {
            pack_type: PacketType::Ack(Ack { fragment_index: 1 }),
            session_id: 1,
            routing_header: SourceRoutingHeader {
                hops: vec![1, 2, 3],
                hop_index: 1,
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(matches!(event, PacketForwarded { .. }));

        let packet = Packet {
            pack_type: PacketType::Nack(Nack {
                fragment_index: 1,
                nack_type: NackType::ErrorInRouting(1),
            }),
            session_id: 1,
            routing_header: SourceRoutingHeader {
                hops: vec![1, 2, 3],
                hop_index: 1,
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(matches!(event, PacketForwarded { .. }));

        let packet = Packet {
            pack_type: PacketType::FloodRequest(FloodRequest {
                flood_id: 1,
                initiator_id: 1,
                path_trace: vec![(1 as NodeId, NodeType::Drone)],
            }),
            session_id: 1,
            routing_header: SourceRoutingHeader {
                hops: vec![1, 2, 3],
                hop_index: 1,
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(matches!(event, PacketForwarded { .. }));

        let packet = Packet {
            pack_type: PacketType::FloodResponse(FloodResponse {
                flood_id: 1,
                path_trace: vec![(1 as NodeId, NodeType::Drone)],
            }),
            session_id: 1,
            routing_header: SourceRoutingHeader {
                hops: vec![1, 2, 3],
                hop_index: 1,
            },
        };

        let result = controller.handle_controller_shortcut(packet);

        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(matches!(event, PacketForwarded { .. }));
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
        let drone_factories: Vec<DroneFactory> = vec![
            cpp_enjoyers_drone,
            get_droned_drone,
            rusteze_drone,
            dr_one_drone,
            rust_do_it_drone,
            rust_busters_drone,
            rusty_drone,
            rustastic_drone,
            lockheed_rustin_drone,
            d_r_o_n_e_drone,
        ];
        let mut drone_factories = drone_factories.into_iter().cycle();

        let drone1 = drone_factories.next().unwrap();
        let drone2 = drone_factories.next().unwrap();
        let drone3 = drone_factories.next().unwrap();
        let drone4 = drone_factories.next().unwrap();
        let drone5 = drone_factories.next().unwrap();
        let drone6 = drone_factories.next().unwrap();
        let drone7 = drone_factories.next().unwrap();
        let drone8 = drone_factories.next().unwrap();
        let drone9 = drone_factories.next().unwrap();
        let drone10 = drone_factories.next().unwrap();
        let drone11 = drone_factories.next().unwrap();
        let drone12 = drone_factories.next().unwrap();

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
            cpp_enjoyers_drone
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
            get_droned_drone
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
            rusteze_drone
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
            dr_one_drone
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
            drone5
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            rust_do_it_drone
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
            drone6
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            rust_busters_drone
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
            drone7
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            rusty_drone
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
            drone8
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            rustastic_drone
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
            drone9
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            lockheed_rustin_drone
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
            drone10
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            d_r_o_n_e_drone
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
            drone11
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            cpp_enjoyers_drone
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
            drone12
                as *const fn(
                    NodeId,
                    Sender<DroneEvent>,
                    Receiver<DroneCommand>,
                    Receiver<Packet>,
                    HashMap<NodeId, Sender<Packet>>,
                    f32,
                ) -> (Box<dyn Runnable>, String),
            get_droned_drone
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

    #[test]
    fn test_shut_down_controller() {
        let mut controller = SimulationController::build(
            "src/tests/configurations/topology_1.toml",
            FILE_FOLDER.to_string(),
            MEDIA_FOLDER.to_string(),
            false,
        );
        thread::sleep(Duration::from_secs(1));

        controller.destroy();

        for handle in controller.handles.iter() {
            assert!(handle.is_none());
        }
    }

    #[test]
    fn test_rebuild_controller() {
        let mut controller = SimulationController::build(
            "src/tests/configurations/topology_1.toml",
            FILE_FOLDER.to_string(),
            MEDIA_FOLDER.to_string(),
            false,
        );

        let initial_handles = controller.handles.len();
        let initial_nodes = controller.nodes_channels.len();
        let initial_drones = controller.drones_channels.len();

        controller.rebuild(
            "src/tests/configurations/topology_1.toml",
            FILE_FOLDER.to_string(),
            MEDIA_FOLDER.to_string(),
            false,
        );

        assert_eq!(initial_handles, controller.handles.len());
        println!("Handles {}\n", controller.handles.len());

        assert_eq!(initial_nodes, controller.nodes_channels.len());
        println!("Nodes {}\n", controller.handles.len());
        assert_eq!(initial_drones, controller.drones_channels.len());
        println!("Drones {}\n", controller.handles.len());

        for handle in controller.handles.iter() {
            assert!(handle.is_some());
        }
    }

    #[test]
    fn test_topology_after_build() {
        let controller = SimulationController::build(
            "src/tests/configurations/topology_1.toml",
            FILE_FOLDER.to_string(),
            MEDIA_FOLDER.to_string(),
            false,
        );
        let drone_1_id = 1;
        let drone_2_id = 2;
        let drone_3_id = 3;
        let drone_4_id = 4;
        let drone_5_id = 5;
        let client_7_id = 7;
        let server_11_id = 11;
        // Check nodes exist
        assert!(controller.topology.nodes().contains(&drone_1_id));
        assert!(controller.topology.nodes().contains(&drone_2_id));
        assert!(controller.topology.nodes().contains(&drone_3_id));
        assert!(controller.topology.nodes().contains(&drone_4_id));
        assert!(controller.topology.nodes().contains(&drone_5_id));
        assert!(controller.topology.nodes().contains(&client_7_id));
        assert!(controller.topology.nodes().contains(&server_11_id));

        // Check node types
        assert_eq!(
            controller.topology.get_node_type(drone_1_id).unwrap(),
            "drone"
        );
        assert_eq!(
            controller.topology.get_node_type(drone_2_id).unwrap(),
            "drone"
        );
        assert_eq!(
            controller.topology.get_node_type(drone_3_id).unwrap(),
            "drone"
        );
        assert_eq!(
            controller.topology.get_node_type(drone_4_id).unwrap(),
            "drone"
        );
        assert_eq!(
            controller.topology.get_node_type(drone_5_id).unwrap(),
            "drone"
        );
        assert_eq!(
            controller.topology.get_node_type(client_7_id).unwrap(),
            "browser_client"
        );
        assert_eq!(
            controller.topology.get_node_type(server_11_id).unwrap(),
            "Text"
        );

        // Check edges exist
        assert!(controller
            .topology
            .edges()
            .get(&drone_1_id)
            .unwrap()
            .contains(&client_7_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_1_id)
            .unwrap()
            .contains(&drone_3_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_2_id)
            .unwrap()
            .contains(&client_7_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_2_id)
            .unwrap()
            .contains(&drone_3_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_3_id)
            .unwrap()
            .contains(&drone_1_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_3_id)
            .unwrap()
            .contains(&drone_2_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_3_id)
            .unwrap()
            .contains(&drone_4_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_3_id)
            .unwrap()
            .contains(&drone_5_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_4_id)
            .unwrap()
            .contains(&drone_3_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_4_id)
            .unwrap()
            .contains(&server_11_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_5_id)
            .unwrap()
            .contains(&drone_3_id));
        assert!(controller
            .topology
            .edges()
            .get(&drone_5_id)
            .unwrap()
            .contains(&server_11_id));
        assert!(controller
            .topology
            .edges()
            .get(&server_11_id)
            .unwrap()
            .contains(&drone_4_id));
        assert!(controller
            .topology
            .edges()
            .get(&server_11_id)
            .unwrap()
            .contains(&drone_5_id));
    }

    #[test]
    fn test_channels_after_build() {
        let controller = SimulationController::build(
            "src/tests/configurations/topology_1.toml",
            FILE_FOLDER.to_string(),
            MEDIA_FOLDER.to_string(),
            false,
        );

        // Check node channels exist
        let drone_1_id = 1;
        let drone_2_id = 2;
        let drone_3_id = 3;
        let drone_4_id = 4;
        let drone_5_id = 5;
        let client_7_id = 7;
        let server_11_id = 11;

        // Check drone channels exist
        assert!(controller.drones_channels.contains_key(&drone_1_id));
        assert!(controller.drones_channels.contains_key(&drone_2_id));
        assert!(controller.drones_channels.contains_key(&drone_3_id));
        assert!(controller.drones_channels.contains_key(&drone_4_id));
        assert!(controller.drones_channels.contains_key(&drone_5_id));

        // Check node channels exist
        assert!(controller.nodes_channels.contains_key(&client_7_id));
        assert!(controller.nodes_channels.contains_key(&server_11_id));

        // Check channel properties
        let node_channel = controller.nodes_channels.get(&client_7_id).unwrap();
        assert!(node_channel.send_packet_channel.is_empty());
        assert!(node_channel.receive_packet_channel.is_empty());

        let drone_channel = controller.drones_channels.get(&drone_1_id).unwrap();
        assert!(drone_channel.send_packet_channel.is_empty());
        assert!(drone_channel.receive_packet_channel.is_empty());
    }
}
