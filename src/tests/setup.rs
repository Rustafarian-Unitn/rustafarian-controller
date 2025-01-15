use std::collections::HashMap;

use ::rustafarian_chat_server::chat_server::ChatServer;
use ::rustafarian_client::browser_client;
use crossbeam_channel::unbounded;
use rustafarian_client::{browser_client::BrowserClient, chat_client::ChatClient, client::Client};
use rustafarian_drone::RustafarianDrone;
use rustafarian_shared::{
    logger::Logger,
    messages::{
        commander_messages::{SimControllerCommand, SimControllerResponseWrapper},
        general_messages::ServerType,
    },
};
use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    packet::{NodeType, Packet},
};

use crate::simulation_controller::DroneChannels;
use crate::simulation_controller::NodeChannels;
use crate::simulation_controller::SimulationController;
use rustafarian_content_server::content_server::ContentServer;
const DEBUG: bool = true;

pub fn setup() -> (
    (ChatClient, ChatClient, BrowserClient),
    ContentServer,
    ChatServer,
    Vec<RustafarianDrone>,
    SimulationController,
) {
    let mut drone_neighbors = HashMap::new();
    let mut drone_2_neighbors = HashMap::new();
    let mut drone_3_neighbors = HashMap::new();
    let mut client_neighbors = HashMap::new();
    let mut client_2_neighbors = HashMap::new();
    let mut browser_client_neighbors = HashMap::new();
    let mut content_server_neighbors = HashMap::new();
    let mut chat_server_neighbors = HashMap::new();

    // Drone 1 channels
    let drone_packet_channels = unbounded::<Packet>();
    let drone_event_channels = unbounded::<DroneEvent>();
    let drone_command_channels = unbounded::<DroneCommand>();

    // Drone 2 channels
    let drone_2_packet_channels = unbounded::<Packet>();
    let drone_2_event_channels = unbounded::<DroneEvent>();
    let drone_2_command_channels = unbounded::<DroneCommand>();

    // Drone 3 channels
    let drone_3_packet_channels = unbounded::<Packet>();
    let drone_3_event_channels = unbounded::<DroneEvent>();
    let drone_3_command_channels = unbounded::<DroneCommand>();

    // Client 1 channels
    let client_packet_channels = unbounded::<Packet>();
    let client_command_channels = unbounded::<SimControllerCommand>();
    let client_response_channels = unbounded::<SimControllerResponseWrapper>();

    // Client 2 channels
    let client_2_packet_channels = unbounded::<Packet>();
    let client_2_command_channels = unbounded::<SimControllerCommand>();
    let client_2_response_channels = unbounded::<SimControllerResponseWrapper>();

    // Content Server channels
    let content_server_packet_channels = unbounded::<Packet>();
    let content_server_command_channels = unbounded::<SimControllerCommand>();
    let content_server_response_channels = unbounded::<SimControllerResponseWrapper>();

    // Chat Server channels
    let chat_server_packet_channels = unbounded::<Packet>();
    let chat_server_command_channels = unbounded::<SimControllerCommand>();
    let chat_server_response_channels = unbounded::<SimControllerResponseWrapper>();

    // Browser Client channels
    let browser_client_packet_channels = unbounded::<Packet>();
    let browser_client_command_channels = unbounded::<SimControllerCommand>();
    let browser_client_response_channels = unbounded::<SimControllerResponseWrapper>();

    drone_neighbors.insert(1, client_packet_channels.0.clone());
    drone_neighbors.insert(3, content_server_packet_channels.0.clone());
    drone_neighbors.insert(4, chat_server_packet_channels.0.clone());
    drone_neighbors.insert(5, client_2_packet_channels.0.clone());
    drone_neighbors.insert(8, browser_client_packet_channels.0.clone());

    drone_2_neighbors.insert(1, client_packet_channels.0.clone());
    drone_2_neighbors.insert(3, content_server_packet_channels.0.clone());
    drone_2_neighbors.insert(4, chat_server_packet_channels.0.clone());
    drone_2_neighbors.insert(5, client_2_packet_channels.0.clone());
    drone_2_neighbors.insert(8, browser_client_packet_channels.0.clone());

    drone_3_neighbors.insert(1, client_packet_channels.0.clone());
    drone_3_neighbors.insert(3, content_server_packet_channels.0.clone());
    drone_3_neighbors.insert(4, chat_server_packet_channels.0.clone());
    drone_3_neighbors.insert(5, client_2_packet_channels.0.clone());
    drone_3_neighbors.insert(8, browser_client_packet_channels.0.clone());

    // Simulation controller
    let client_channels = NodeChannels {
        send_packet_channel: client_packet_channels.0.clone(),
        receive_packet_channel: client_packet_channels.1.clone(),
        send_command_channel: client_command_channels.0.clone(),
        receive_command_channel: client_command_channels.1.clone(),
        receive_response_channel: client_response_channels.1.clone(),
        send_response_channel: client_response_channels.0.clone(),
    };

    let client_2_channels = NodeChannels {
        send_packet_channel: client_2_packet_channels.0.clone(),
        receive_packet_channel: client_2_packet_channels.1.clone(),
        send_command_channel: client_2_command_channels.0.clone(),
        receive_command_channel: client_2_command_channels.1.clone(),
        receive_response_channel: client_2_response_channels.1.clone(),
        send_response_channel: client_2_response_channels.0.clone(),
    };

    let browser_client_channels = NodeChannels {
        send_packet_channel: browser_client_packet_channels.0.clone(),
        receive_packet_channel: browser_client_packet_channels.1.clone(),
        send_command_channel: browser_client_command_channels.0.clone(),
        receive_command_channel: browser_client_command_channels.1.clone(),
        receive_response_channel: browser_client_response_channels.1.clone(),
        send_response_channel: browser_client_response_channels.0.clone(),
    };

    let content_server_channels = NodeChannels {
        send_packet_channel: content_server_packet_channels.0.clone(),
        receive_packet_channel: content_server_packet_channels.1.clone(),

        send_command_channel: content_server_command_channels.0.clone(),
        receive_command_channel: content_server_command_channels.1.clone(),
        receive_response_channel: content_server_response_channels.1.clone(),
        send_response_channel: content_server_response_channels.0.clone(),
    };

    let chat_server_channels = NodeChannels {
        send_packet_channel: chat_server_packet_channels.0.clone(),
        receive_packet_channel: chat_server_packet_channels.1.clone(),
        send_command_channel: chat_server_command_channels.0.clone(),
        receive_command_channel: chat_server_command_channels.1.clone(),
        receive_response_channel: chat_server_response_channels.1.clone(),
        send_response_channel: chat_server_response_channels.0.clone(),
    };

    let drone_channels = DroneChannels {
        send_command_channel: drone_command_channels.0.clone(),
        receive_command_channel: drone_command_channels.1.clone(),
        send_packet_channel: drone_packet_channels.0.clone(),
        receive_packet_channel: drone_packet_channels.1.clone(),
        send_event_channel: drone_event_channels.0.clone(),
        receive_event_channel: drone_event_channels.1.clone(),
    };

    let drone_2_channels = DroneChannels {
        send_command_channel: drone_2_command_channels.0.clone(),
        receive_command_channel: drone_2_command_channels.1.clone(),
        send_packet_channel: drone_2_packet_channels.0.clone(),
        receive_packet_channel: drone_2_packet_channels.1.clone(),
        send_event_channel: drone_2_event_channels.0.clone(),
        receive_event_channel: drone_2_event_channels.1.clone(),
    };

    let drone_3_channels = DroneChannels {
        send_command_channel: drone_3_command_channels.0.clone(),
        receive_command_channel: drone_3_command_channels.1.clone(),
        send_packet_channel: drone_3_packet_channels.0.clone(),
        receive_packet_channel: drone_3_packet_channels.1.clone(),
        send_event_channel: drone_3_event_channels.0.clone(),
        receive_event_channel: drone_3_event_channels.1.clone(),
    };

    client_neighbors.insert(2, drone_packet_channels.0.clone());
    client_neighbors.insert(6, drone_2_packet_channels.0.clone());
    client_neighbors.insert(7, drone_3_packet_channels.0.clone());

    client_2_neighbors.insert(2, drone_packet_channels.0.clone());
    client_2_neighbors.insert(6, drone_2_packet_channels.0.clone());
    client_2_neighbors.insert(7, drone_3_packet_channels.0.clone());

    browser_client_neighbors.insert(2, drone_packet_channels.0.clone());
    browser_client_neighbors.insert(6, drone_2_packet_channels.0.clone());
    browser_client_neighbors.insert(7, drone_3_packet_channels.0.clone());

    content_server_neighbors.insert(2, drone_packet_channels.0.clone());
    content_server_neighbors.insert(6, drone_2_packet_channels.0.clone());
    content_server_neighbors.insert(7, drone_3_packet_channels.0.clone());

    chat_server_neighbors.insert(2, drone_packet_channels.0.clone());
    chat_server_neighbors.insert(6, drone_2_packet_channels.0.clone());
    chat_server_neighbors.insert(7, drone_3_packet_channels.0.clone());

    let drone = RustafarianDrone::new(
        2,
        drone_event_channels.0,
        drone_command_channels.1,
        drone_packet_channels.1,
        drone_neighbors,
        0.0,
    );

    let drone_2 = RustafarianDrone::new(
        6,
        drone_2_event_channels.0,
        drone_2_command_channels.1,
        drone_2_packet_channels.1,
        drone_2_neighbors,
        0.0,
    );

    let drone_3 = RustafarianDrone::new(
        7,
        drone_3_event_channels.0,
        drone_3_command_channels.1,
        drone_3_packet_channels.1,
        drone_3_neighbors,
        0.0,
    );

    let mut content_server = ContentServer::new(
        3,
        content_server_neighbors,
        content_server_packet_channels.1,
        content_server_command_channels.1,
        content_server_response_channels.0,
        "resources/files",
        "resources/media",
        ServerType::Text,
        DEBUG,
    );

    let mut chat_server = ChatServer::new(
        4,
        chat_server_command_channels.1,
        chat_server_response_channels.0,
        chat_server_packet_channels.1,
        chat_server_neighbors,
        DEBUG,
    );

    let mut client = ChatClient::new(
        1,
        client_neighbors,
        client_packet_channels.1.clone(),
        client_command_channels.1.clone(),
        client_response_channels.0,
        DEBUG,
    );

    let mut client_2 = ChatClient::new(
        5,
        client_2_neighbors,
        client_2_packet_channels.1.clone(),
        client_2_command_channels.1.clone(),
        client_2_response_channels.0,
        DEBUG,
    );

    let mut browser_client = browser_client::BrowserClient::new(
        8,
        browser_client_neighbors,
        browser_client_packet_channels.1.clone(),
        browser_client_command_channels.1.clone(),
        browser_client_response_channels.0,
        DEBUG,
    );

    client.topology().add_node(1);
    client.topology().add_node(2);
    client.topology().add_node(3);
    client.topology().add_node(4);
    client.topology().add_node(5);
    client.topology().add_node(6);
    client.topology().add_node(7);
    client.topology().add_edge(1, 2);
    client.topology().add_edge(1, 6);
    client.topology().add_edge(1, 7);
    client.topology().add_edge(2, 3);
    client.topology().add_edge(2, 4);
    client.topology().add_edge(2, 5);
    client.topology().add_edge(2, 6);
    client.topology().add_edge(2, 7);

    client_2.topology().add_node(1);
    client_2.topology().add_node(2);
    client_2.topology().add_node(3);
    client_2.topology().add_node(4);
    client_2.topology().add_node(5);
    client_2.topology().add_node(6);
    client_2.topology().add_node(7);
    client_2.topology().add_edge(5, 2);
    client_2.topology().add_edge(2, 3);
    client_2.topology().add_edge(2, 4);
    client_2.topology().add_edge(2, 1);
    client_2.topology().add_edge(2, 6);
    client_2.topology().add_edge(2, 7);

    content_server.topology.add_node(1);
    content_server.topology.add_node(2);
    content_server.topology.add_node(3);
    content_server.topology.add_node(4);
    content_server.topology.add_node(5);
    content_server.topology.add_node(6);
    content_server.topology.add_node(7);
    content_server.topology.add_edge(3, 2);
    content_server.topology.add_edge(2, 1);
    content_server.topology.add_edge(2, 4);
    content_server.topology.add_edge(2, 5);
    content_server.topology.add_edge(2, 6);
    content_server.topology.add_edge(2, 7);

    browser_client.topology().add_node(1);
    browser_client.topology().add_node(2);
    browser_client.topology().add_node(3);
    browser_client.topology().add_node(4);
    browser_client.topology().add_node(5);
    browser_client.topology().add_node(6);
    browser_client.topology().add_node(7);
    browser_client.topology().add_node(8);
    browser_client.topology().add_edge(8, 2);
    browser_client.topology().add_edge(8, 6);
    browser_client.topology().add_edge(8, 7);
    browser_client.topology().add_edge(2, 1);
    browser_client.topology().add_edge(2, 3);
    browser_client.topology().add_edge(2, 4);
    browser_client.topology().add_edge(2, 5);
    browser_client.topology().add_edge(2, 6);
    browser_client.topology().add_edge(2, 7);

    chat_server.update_topology(
        vec![
            (1, NodeType::Client),
            (2, NodeType::Client),
            (3, NodeType::Server),
            (4, NodeType::Server),
            (5, NodeType::Client),
            (6, NodeType::Drone),
            (7, NodeType::Drone),
            (8, NodeType::Client),
        ],
        vec![(1, 2), (2, 3), (2, 4), (2, 5), (2, 6), (2, 7), (2, 8)],
    );

    let mut drones_channels = HashMap::new();
    drones_channels.insert(2, drone_channels);
    drones_channels.insert(6, drone_2_channels);
    drones_channels.insert(7, drone_3_channels);

    let mut nodes_channels = HashMap::new();
    nodes_channels.insert(1, client_channels);
    nodes_channels.insert(3, content_server_channels);
    nodes_channels.insert(4, chat_server_channels);
    nodes_channels.insert(5, client_2_channels);
    nodes_channels.insert(8, browser_client_channels);

    let shutdown_channel = unbounded::<()>();

    let simulation_controller = SimulationController::new(
        nodes_channels,
        drones_channels,
        shutdown_channel,
        Vec::new(),
        client.topology().clone(),
        Logger::new("SimulationController".to_string(), 0, true),
    );

    (
        (client, client_2, browser_client),
        content_server,
        chat_server,
        vec![drone, drone_2, drone_3],
        simulation_controller,
    )
}
