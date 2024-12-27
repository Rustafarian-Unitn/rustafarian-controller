
#[cfg(test)]
use crate::simulation_controller::SimulationController;

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
