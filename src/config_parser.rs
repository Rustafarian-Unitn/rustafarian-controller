use std::fs;

use wg_2024::config::Config;

pub fn parse_config(file: &str) -> Config {
    // Let it panic if file not found
    let file_str = fs::read_to_string(file).unwrap();

    // Let it panic if toml is misconfigured
    let parsed_config: Config = toml::from_str(&file_str).unwrap();
    // Validate the config
    validate_config(&parsed_config);
    
    parsed_config
} 
fn validate_config(config: &Config) -> () {
    // Validate drone
    for drone in &config.drone {
        assert!(!drone.connected_node_ids.is_empty(), "Drone id is empty");
        assert!(
            drone.connected_node_ids.len() >= 1,
            "Drone must be connected to at least 1 other drone"
        );
    }

    // Validate client
    for client in &config.client {
        assert!(!client.connected_drone_ids.is_empty(), "Client id is empty");
        assert!(
            client.connected_drone_ids.len() <= 2 && client.connected_drone_ids.len() >= 1,
            "Client must be connected to 1 or 2 drones"
        );
    }

    // Validate server
    for server in &config.server {
        assert!(!server.connected_drone_ids.is_empty(), "Server id is empty");
        assert!(
            server.connected_drone_ids.len() <= 2 && server.connected_drone_ids.len() >= 1,
            "Server must be connected to 1 or 2 drones"
        );
    }

    // Validate reciprocity between nodes
    for drone in &config.drone {
        for connected_node in &drone.connected_node_ids {
            let mut found = false;
            for client in &config.client {
                if &client.id == connected_node {
                    assert!(client.connected_drone_ids.contains(&drone.id), "Client {} must be connected to drone {}", client.id, drone.id);
                    found = true;
                    break;
                }
            }

            for server in &config.server {
                if &server.id == connected_node {
                    assert!(server.connected_drone_ids.contains(&drone.id), "Server {} must be connected to drone {}", server.id, drone.id);
                    found = true;
                    break;
                }
            }

            for other_drone in &config.drone {
                if &other_drone.id == connected_node {
                    assert!(other_drone.connected_node_ids.contains(&drone.id), "Drone {} must be connected to drone {}", other_drone.id, drone.id);
                    found = true;
                    break;
                }
            }
            
            if !found {
                panic!("Drone {} is connected to an inexistent client {}", drone.id, connected_node);
            }
        }
    }
}

#[test]
fn test_parse_config() {
    let config_str = "src/tests/configurations/test_config.toml";

    let config = parse_config(config_str);
    println!("{:?}", config);
    assert_eq!(config.drone.len(), 1);
    assert_eq!(config.client.len(), 1);
    assert_eq!(config.server.len(), 1);
}

#[test]
fn test_simulation_controller_parse_config_invalid_file() {
    let config_str = "invalid/path/to/config.toml";

    let result = std::panic::catch_unwind(|| parse_config(config_str));

    assert!(result.is_err());
}

#[test]
fn test_simulation_controller_parse_config_invalid_toml() {
    let config_str = "src/tests/configurations/invalid_config.toml";

    let result = std::panic::catch_unwind(|| parse_config(config_str));

    assert!(result.is_err());
}

#[test]
fn test_validate_config() {
    let config_str = "src/tests/configurations/test_config.toml";

    let config = parse_config(config_str);
    let result = std::panic::catch_unwind(|| validate_config(&config));

    assert!(result.is_ok());
}

#[test]
fn test_validate_large_config() {
    let config_str = "src/tests/configurations/topology_20_drones.toml";

    let config = parse_config(config_str);
    let result = std::panic::catch_unwind(|| validate_config(&config));

    assert!(result.is_ok());
}
