use crate::simulation_controller::SimulationController;
use rustafarian_shared::messages::commander_messages::{
    SimControllerCommand, SimControllerMessage, SimControllerResponseWrapper,
};
use std::{
    collections::{HashMap, HashSet},
    thread,
};
use wg_2024::network::NodeId;

#[test]
fn test_simulation_controller_build_complex_topology() {
    let config_str = "src/tests/configurations/topology_10_nodes.toml";
    assert!(
        std::path::Path::new(config_str).exists(),
        "Config file does not exist at the specified path"
    );
    let controller = SimulationController::build(
        config_str,
        "resources/files".to_string(),
        "resources/media".to_string(),
        false,
    );
    thread::sleep(std::time::Duration::from_millis(500));

    assert_eq!(controller.drones_channels.len(), 3);
    assert_eq!(controller.nodes_channels.len(), 7);
    assert_eq!(controller.handles.len(), 10);
    assert_eq!(controller.topology.nodes().len(), 10);

    // Check the topology
    let edges = controller.topology.edges();
    println!("{edges:?}");
    assert_topology(edges);
}

#[test]
fn test_client_topology() {
    let config_str = "src/tests/configurations/topology_10_nodes.toml";
    assert!(
        std::path::Path::new(config_str).exists(),
        "Config file does not exist at the specified path"
    );
    let controller = SimulationController::build(
        config_str,
        "resources/files".to_string(),
        "resources/media".to_string(),
        false,
    );

    thread::sleep(std::time::Duration::from_secs(5));
    let client_id = 4;
    let client_channels = controller.nodes_channels.get(&client_id).unwrap();
    let client_send_command_channel = &client_channels.send_command_channel;

    // Make the client discover the topology
    let command = client_send_command_channel.send(SimControllerCommand::Topology);
    assert!(command.is_ok());

    // Wait for the response
    for response in client_channels.receive_response_channel.iter() {
        if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
            topology,
        )) = response
        {
            let edges = topology.edges();
            assert_topology(edges);
            break;
        }
    }
}

fn assert_topology(edges: &HashMap<NodeId, HashSet<NodeId>>) {
    assert_eq!(edges.len(), 10);
    assert!(edges.get(&1).unwrap().contains(&2));
    assert!(edges.get(&1).unwrap().contains(&3));
    assert!(edges.get(&1).unwrap().contains(&4));
    assert!(edges.get(&1).unwrap().contains(&5));
    assert!(edges.get(&1).unwrap().contains(&7));
    assert!(edges.get(&1).unwrap().contains(&9));
    assert_eq!(edges.get(&1).unwrap().len(), 6);

    assert!(edges.get(&2).unwrap().contains(&1));
    assert!(edges.get(&2).unwrap().contains(&3));
    assert!(edges.get(&2).unwrap().contains(&4));
    assert!(edges.get(&2).unwrap().contains(&6));
    assert!(edges.get(&2).unwrap().contains(&7));
    assert!(edges.get(&2).unwrap().contains(&8));
    assert!(edges.get(&2).unwrap().contains(&10));
    assert_eq!(edges.get(&2).unwrap().len(), 7);

    assert!(edges.get(&3).unwrap().contains(&1));
    assert!(edges.get(&3).unwrap().contains(&2));
    assert!(edges.get(&3).unwrap().contains(&5));
    assert!(edges.get(&3).unwrap().contains(&6));
    assert!(edges.get(&3).unwrap().contains(&8));
    assert!(edges.get(&3).unwrap().contains(&9));
    assert!(edges.get(&3).unwrap().contains(&10));
    assert_eq!(edges.get(&3).unwrap().len(), 7);

    assert!(edges.get(&4).unwrap().contains(&1));
    assert!(edges.get(&4).unwrap().contains(&2));
    assert_eq!(edges.get(&4).unwrap().len(), 2);

    assert!(edges.get(&5).unwrap().contains(&1));
    assert!(edges.get(&5).unwrap().contains(&3));
    assert_eq!(edges.get(&5).unwrap().len(), 2);

    assert!(edges.get(&6).unwrap().contains(&2));
    assert!(edges.get(&6).unwrap().contains(&3));
    assert_eq!(edges.get(&6).unwrap().len(), 2);

    assert!(edges.get(&7).unwrap().contains(&1));
    assert!(edges.get(&7).unwrap().contains(&2));
    assert_eq!(edges.get(&7).unwrap().len(), 2);

    assert!(edges.get(&8).unwrap().contains(&2));
    assert!(edges.get(&8).unwrap().contains(&3));
    assert_eq!(edges.get(&8).unwrap().len(), 2);

    assert!(edges.get(&9).unwrap().contains(&1));
    assert!(edges.get(&9).unwrap().contains(&3));
    assert_eq!(edges.get(&9).unwrap().len(), 2);

    assert!(edges.get(&10).unwrap().contains(&2));
    assert!(edges.get(&10).unwrap().contains(&3));
    assert_eq!(edges.get(&10).unwrap().len(), 2);
}
