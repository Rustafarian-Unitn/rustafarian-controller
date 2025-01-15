use std::{collections::HashSet, fs};

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

fn validate_config(config: &Config) {
    // Validate drone
    for drone in &config.drone {
        assert!(
            !drone.connected_node_ids.is_empty(),
            "Drone must be connected to at least 1 other node"
        );
        assert!(
            !drone.connected_node_ids.contains(&drone.id),
            "Drone {} cannot be connected to itself",
            drone.id
        );
        let unique_node_ids: HashSet<_> = drone.connected_node_ids.iter().collect();
        assert_eq!(
            unique_node_ids.len(),
            drone.connected_node_ids.len(),
            "Drone's {} connected nodes must be unique",
            drone.id
        );
    }

    // Validate client
    for client in &config.client {
        assert!(
            client.connected_drone_ids.len() < 3 && !client.connected_drone_ids.is_empty(),
            "Client must be connected to 1 or 2 drones"
        );
    }

    // Validate server
    for server in &config.server {
        assert!(
            server.connected_drone_ids.len() >= 2,
            "Server {} must be connected to at least 2 drones",
            server.id
        );
    }

    // Validate reciprocity between nodes
    for drone in &config.drone {
        for connected_node in &drone.connected_node_ids {
            let mut found = false;
            for client in &config.client {
                if &client.id == connected_node {
                    assert!(
                        client.connected_drone_ids.contains(&drone.id),
                        "Client {} must be connected to drone {}",
                        client.id,
                        drone.id
                    );
                    found = true;
                    break;
                }
            }

            for server in &config.server {
                if &server.id == connected_node {
                    assert!(
                        server.connected_drone_ids.contains(&drone.id),
                        "Server {} must be connected to drone {}",
                        server.id,
                        drone.id
                    );
                    found = true;
                    break;
                }
            }

            for other_drone in &config.drone {
                if &other_drone.id == connected_node {
                    assert!(
                        other_drone.connected_node_ids.contains(&drone.id),
                        "Drone {} must be connected to drone {}",
                        other_drone.id,
                        drone.id
                    );
                    found = true;
                    break;
                }
            }

            if !found {
                panic!(
                    "Drone {} is connected to an inexistent client {}",
                    drone.id, connected_node
                );
            }
        }
    }

    for client in &config.client {
        for connected_node in &client.connected_drone_ids {
            let mut found = false;
            for drone in &config.drone {
                if &drone.id == connected_node {
                    assert!(
                        drone.connected_node_ids.contains(&client.id),
                        "Drone {} must be connected to client {}",
                        drone.id,
                        client.id
                    );
                    found = true;
                    break;
                }
            }

            for server in &config.server {
                assert!(
                    server.id != *connected_node,
                    "Client {} cannot be connected to server {}",
                    client.id,
                    server.id
                );
            }

            for other_client in &config.client {
                assert!(
                    other_client.id != *connected_node,
                    "Client {} cannot be connected to client {}",
                    client.id,
                    other_client.id
                );
            }

            if !found {
                panic!(
                    "Client {} is connected to an inexistent drone {}",
                    client.id, connected_node
                );
            }
        }
    }

    for server in &config.server {
        for connected_node in &server.connected_drone_ids {
            let mut found = false;
            for drone in &config.drone {
                if &drone.id == connected_node {
                    assert!(
                        drone.connected_node_ids.contains(&server.id),
                        "Drone {} must be connected to server {}",
                        drone.id,
                        server.id
                    );
                    found = true;
                    break;
                }
            }

            for client in &config.client {
                assert!(
                    client.id != *connected_node,
                    "Server {} cannot be connected to client {}",
                    server.id,
                    client.id
                );
            }

            for other_server in &config.server {
                assert!(
                    other_server.id != *connected_node,
                    "Server {} cannot be connected to server {}",
                    server.id,
                    other_server.id
                );
            }

            if !found {
                panic!(
                    "Server {} is connected to an inexistent drone {}",
                    server.id, connected_node
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let config_str = "src/tests/configurations/topology_1.toml";

        let config = parse_config(config_str);
        println!("{:?}", config);
        assert_eq!(config.drone.len(), 5);
        assert_eq!(config.client.len(), 1);
        assert_eq!(config.server.len(), 1);
    }

    #[test]
    fn test_simulation_controller_parse_config_invalid_path() {
        let config_str = "invalid/path/to/config.toml";

        let result = std::panic::catch_unwind(|| parse_config(config_str));

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config() {
        let config_str = "src/tests/configurations/topology_1.toml";

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

    #[test]
    fn test_validate_reciprocal_client() {
        let config_str = "src/tests/configurations/invalid_client_no_reciprocal.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_client_inexistent_drone() {
        let config_str = "src/tests/configurations/invalid_client_inexistent_connection.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_client_to_client() {
        let config_str = "src/tests/configurations/invalid_client_to_client.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_client_to_server() {
        let config_str = "src/tests/configurations/invalid_client_to_server.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_client_too_few_connections() {
        let config_str = "src/tests/configurations/invalid_client_too_few_conn.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_client_too_many_connections() {
        let config_str = "src/tests/configurations/invalid_client_too_many_conn.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_isolated_drone() {
        let config_str = "src/tests/configurations/invalid_isolated_drone.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_reciprocal_drone() {
        let config_str = "src/tests/configurations/invalid_drone_no_reciprocal.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_reciprocal_server() {
        let config_str = "src/tests/configurations/invalid_server_no_reciprocal.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_server_to_client() {
        let config_str = "src/tests/configurations/invalid_server_to_client.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_server_to_server() {
        let config_str = "src/tests/configurations/invalid_server_to_server.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }

    #[test]
    fn test_server_too_few_connections() {
        let config_str = "src/tests/configurations/invalid_server_too_few_conn.toml";

        // First catch panic from parse_config
        let result = std::panic::catch_unwind(|| parse_config(config_str));
        assert!(result.is_err());
    }
}
