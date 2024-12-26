use std::fs;

use wg_2024::config::Config;

pub fn parse_config(file: &str) -> Config {
    // Let it panic if file not found
    let file_str = fs::read_to_string(file).unwrap();

    // Let it panic if toml is misconfigured
    let parsed_config: Config = toml::from_str(&file_str).unwrap();
    parsed_config
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
