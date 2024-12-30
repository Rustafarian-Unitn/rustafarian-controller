use std::collections::HashMap;

use crossbeam_channel::unbounded;
use rustafarian_client::{chat_client::ChatClient, client::Client};
use rustafarian_drone::RustafarianDrone;
use rustafarian_shared::messages::{commander_messages::{
    SimControllerCommand, SimControllerResponseWrapper,
}, general_messages::ServerType};
use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    packet::Packet,
};

use rustafarian_content_server::content_server::ContentServer;
use crate::simulation_controller::DroneChannels;
use crate::simulation_controller::NodeChannels;
use crate::simulation_controller::SimulationController;

pub fn setup() -> (ChatClient, ContentServer, RustafarianDrone, SimulationController) {
    let mut drone_neighbors = HashMap::new();
    let mut client_neighbors = HashMap::new();
    let mut server_neighbors = HashMap::new();

    // Drone channels
    let drone_packet_channels = unbounded::<Packet>();
    let drone_event_channels = unbounded::<DroneEvent>();
    let drone_command_channels = unbounded::<DroneCommand>();

    // Client channels
    let client_packet_channels = unbounded::<Packet>();
    let client_command_channels = unbounded::<SimControllerCommand>();
    let client_response_channels = unbounded::<SimControllerResponseWrapper>();

    // Server channels
    let server_packet_channels = unbounded::<Packet>();
    let server_command_channels = unbounded::<SimControllerCommand>();
    let server_response_channels = unbounded::<SimControllerResponseWrapper>();

    drone_neighbors.insert(1, client_packet_channels.0.clone());
    drone_neighbors.insert(3, server_packet_channels.0.clone());

    // Simulation controller
    let client_channels = NodeChannels {
        send_packet_channel: client_packet_channels.0.clone(),
        receive_packet_channel: client_packet_channels.1.clone(),
        send_command_channel: client_command_channels.0.clone(),
        receive_response_channel: client_response_channels.1.clone(),
        send_response_channel: client_response_channels.0.clone(),
    };

    let server_channels = NodeChannels {
        send_packet_channel: server_packet_channels.0.clone(),
        receive_packet_channel: server_packet_channels.1.clone(),
        send_command_channel: server_command_channels.0.clone(),
        receive_response_channel: server_response_channels.1.clone(),
        send_response_channel: server_response_channels.0.clone(),
    };

    let drone_channels = DroneChannels {
        send_command_channel: drone_command_channels.0.clone(),
        receive_command_channel: drone_command_channels.1.clone(),
        send_packet_channel: drone_packet_channels.0.clone(),
        receive_packet_channel: drone_packet_channels.1.clone(),
        send_event_channel: drone_event_channels.0.clone(),
        receive_event_channel: drone_event_channels.1.clone(),
    };

    client_neighbors.insert(2, drone_packet_channels.0.clone());
    server_neighbors.insert(2, drone_packet_channels.0.clone());
    
    let drone = RustafarianDrone::new(
        2,
        drone_event_channels.0,
        drone_command_channels.1,
        drone_packet_channels.1,
        drone_neighbors,
        0.0,
    );
    
    let server = ContentServer::new(3, server_neighbors,server_packet_channels.1,  server_command_channels.1, server_response_channels.0, "files", "media", ServerType::Media);

    let mut client = ChatClient::new(
        1,
        client_neighbors,
        client_packet_channels.1.clone(),
        client_command_channels.1.clone(),
        client_response_channels.0,
    );

    client.topology().add_node(1);
    client.topology().add_node(2);
    client.topology().add_node(3);
    client.topology().add_edge(1, 2);
    client.topology().add_edge(2, 3);


    let mut drones_channels = HashMap::new();
    drones_channels.insert(2, drone_channels);

    let mut nodes_channels = HashMap::new();
    nodes_channels.insert(1, client_channels);
    nodes_channels.insert(3, server_channels);

    let simulation_controller = SimulationController::new(
        nodes_channels,
        drones_channels,
        Vec::new(),
        client.topology().clone(),
    );

    (client, server, drone, simulation_controller)
}
