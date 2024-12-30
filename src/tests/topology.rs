use std::thread;

#[cfg(test)]
use crate::simulation_controller::SimulationController;
use rustafarian_client::client::Client;
use rustafarian_shared::messages::commander_messages::{
    SimControllerCommand, SimControllerMessage, SimControllerResponseWrapper
};

#[test]
fn test_simulation_controller_build_complex_topology() {
    let config_str = "src/tests/configurations/test_complex_config.toml";
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
fn test_client_topology() {
    use super::setup;

    let (mut chat_client, _,_, controller) = setup::setup();
    let client_id = chat_client.client_id();
    let client_channels = controller.nodes_channels.get(&client_id).unwrap();
    let client_send_command_channel = &client_channels.send_command_channel;

    // Make the client discover the topology
    let command = client_send_command_channel.send(SimControllerCommand::Topology);
    assert!(command.is_ok());

    thread::spawn(move || {
        chat_client.run(500);
    });
    // Wait for the response
    let response = client_channels.receive_response_channel.recv().unwrap();
    assert!(matches!(
        response.clone(),
        SimControllerResponseWrapper::Message(msg) if matches!(msg, SimControllerMessage::TopologyResponse(_))
    ));

    // Check the client topology: 3 nodes (1,2,3) and 2 edges (1-2, 2-3)
    if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(topology)) = response {
        assert_eq!(topology.nodes().len(), 3);
        assert_eq!(topology.edges().len(), 3);
    }

    
    
}