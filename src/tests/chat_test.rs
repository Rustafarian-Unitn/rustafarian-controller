mod tests {

    use std::{collections::HashSet, thread};

    use rustafarian_shared::messages::commander_messages::{
        SimControllerCommand, SimControllerEvent, SimControllerMessage,
        SimControllerResponseWrapper,
    };

    use crate::simulation_controller::SimulationController;

    #[test]
    fn known_servers() {
        let chat_client_id = 4;

        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&chat_client_id)
            .unwrap()
            .send_command_channel
            .clone();

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&chat_client_id)
            .unwrap()
            .receive_response_channel
            .clone();

        // Wait for the flood to finish
        std::thread::sleep(std::time::Duration::from_secs(1));
        //send known servers request
        let res = client_command_channel.send(SimControllerCommand::KnownServers);
        assert!(res.is_ok());

        // Listen for response
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::KnownServers(
                known_servers,
            )) = response
            {
                if known_servers.len() == 1 {
                    assert!(known_servers.contains_key(&9));
                }
                break;
            }
        }
    }

    #[test]
    fn test_message_from_client_to_server() {
        let controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );
        let client_id: u8 = 4;
        let client_2_id: u8 = 6;
        let server_id: u8 = 9;

        let client_command_channel = controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();
        let client_2_command_channel = controller
            .nodes_channels
            .get(&client_2_id)
            .unwrap()
            .send_command_channel
            .clone();

        let client_2_response_channel = controller
            .nodes_channels
            .get(&client_2_id)
            .unwrap()
            .receive_response_channel
            .clone();

        // Wait for the flood to finish
        std::thread::sleep(std::time::Duration::from_secs(5));

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(server_id));
        assert!(res.is_ok());

        // Instruct client 2 to register to server
        let res = client_2_command_channel.send(SimControllerCommand::Register(server_id));
        assert!(res.is_ok());

        // Wait for registration to happen
        std::thread::sleep(std::time::Duration::from_secs(2));
        // Instruct client to send message to server
        let res = client_command_channel.send(SimControllerCommand::SendMessage(
            "Hello".to_string(),
            server_id,
            client_2_id,
        ));
        assert!(res.is_ok());

        // ignore messages until message is received
        for response in client_2_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::MessageReceived(
                _,
                _,
                _,
            )) = response
            {
                println!("TEST - Message received {:?}", response);
                let _expected_response = "Hello".to_string();
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::MessageReceived(
                                _server_id,
                                _client_id,
                                _expected_response2
                            )
                        )
                    ),
                    "Expected message received"
                );
                break;
            }
        }
    }

    #[test]
    fn test_client_list() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            true,
        );
        let client_id: u8 = 4;
        let client_2_id: u8 = 6;
        let server_id = 9;

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();

        let client_2_command_channel = simulation_controller
            .nodes_channels
            .get(&client_2_id)
            .unwrap()
            .send_command_channel
            .clone();

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .receive_response_channel
            .clone();

        // Wait for flood request
        thread::sleep(std::time::Duration::from_secs(5));

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(server_id));
        assert!(res.is_ok());

        // Instruction client 2 to register to server
        let res = client_2_command_channel.send(SimControllerCommand::Register(server_id));
        assert!(res.is_ok());

        // Instruct client to request client list
        let res = client_command_channel.send(SimControllerCommand::ClientList(server_id));
        assert!(res.is_ok());

        thread::sleep(std::time::Duration::from_secs(2));

        // Ignore messages until ClientListResponse is received
        for response in client_response_channel.iter() {
            println!("TEST - Message received {:?}", response);
            if let SimControllerResponseWrapper::Message(
                SimControllerMessage::ClientListResponse(_, _),
            ) = response
            {
                println!("TEST - Client list response {:?}", response);
                let _expected_list = [6];
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::ClientListResponse(9, _expected_list)
                        )
                    ),
                    "Expected client list"
                );
                break;
            }
        }
    }

    //Test flood request
    #[test]
    fn test_flood_request() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );

        let client_id: u8 = 4;

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();

        let controller_response_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .receive_response_channel
            .clone();

        // Instruct client to request flood
        let res = client_command_channel.send(SimControllerCommand::FloodRequest);
        assert!(res.is_ok());

        // ignore messages until flood response is received
        for response in controller_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::FloodResponse(_)) =
                response
            {
                println!("TEST - Flood response {:?}", response);
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(SimControllerMessage::FloodResponse(
                            _
                        ))
                    ),
                    "Expected flood response"
                );
                break;
            }
        }
    }

    // Test known servers
    #[test]
    fn test_known_servers() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );
        let client_id: u8 = 4;

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();

        let controller_response_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .receive_response_channel
            .clone();

        // Instruct client to request known servers
        let res = client_command_channel.send(SimControllerCommand::KnownServers);
        assert!(res.is_ok());
        // Listen for packets from client
        // Ignore messages until KnownServers Response is received
        for response in controller_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::KnownServers(_)) =
                response
            {
                println!("TEST - Known servers response {:?}", response);
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(SimControllerMessage::KnownServers(
                            _
                        ))
                    ),
                    "Expected known servers response"
                );
                break;
            }
        }
    }

    // Test registered servers
    #[test]
    fn test_registered_servers() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );
        let client_id: u8 = 4;
        let server_id = 9;
        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .receive_response_channel
            .clone();

        // Wait for flood to finish
        thread::sleep(std::time::Duration::from_secs(3));
        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(server_id));
        assert!(res.is_ok());

        // Wait for registration to happen
        thread::sleep(std::time::Duration::from_millis(500));

        // Instruct client to request registered servers
        let res = client_command_channel.send(SimControllerCommand::RegisteredServers);
        assert!(res.is_ok());

        // Ignore messages until RegisteredServersResponse is received
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(
                SimControllerMessage::RegisteredServersResponse(_),
            ) = response
            {
                let _expected_response = [4];
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::RegisteredServersResponse(_expected_response)
                        )
                    ),
                    "Expected registered servers response"
                );
                break;
            }
        }
    }

    // Test remove sender from server
    #[test]
    fn test_client_remove_senders() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );
        let drone_1_id: u8 = 1;
        let drone_2_id: u8 = 2;
        let drone_3_id: u8 = 3;
        let client_id: u8 = 4;

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .receive_response_channel
            .clone();

        // Leave time for flood request
        thread::sleep(std::time::Duration::from_secs(1));

        // Instruct client to remove sender 1
        let res = client_command_channel.send(SimControllerCommand::RemoveSender(drone_1_id));
        assert!(res.is_ok());

        // Instruct client to remove sender 2
        let res = client_command_channel.send(SimControllerCommand::RemoveSender(drone_2_id));
        assert!(res.is_ok());

        // Instruct client to remove sender 3
        let res = client_command_channel.send(SimControllerCommand::RemoveSender(drone_3_id));
        assert!(res.is_ok());

        // Test client topology
        let res = client_command_channel.send(SimControllerCommand::Topology);
        assert!(res.is_ok());

        // Listen for topology response
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
                topology,
            )) = response
            {
                println!(
                    "TEST - Topology response for client {} -  {:?}",
                    client_id,
                    topology.edges().get(&client_id)
                );

                let expected_response = vec![];
                let expected_response: HashSet<_> = expected_response.into_iter().collect();
                assert_eq!(topology.edges().get(&client_id), Some(&expected_response));
                break;
            }
        }
    }

    // Test remove sender from server
    #[test]
    fn test_server_remove_receivers() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );

        let drone_1_id: u8 = 1;
        let drone_2_id: u8 = 2;
        let drone_3_id: u8 = 3;
        let client_id: u8 = 4;
        let client_2_id: u8 = 6;
        let server_id = 9;

        let server_command_channel = simulation_controller
            .nodes_channels
            .get(&server_id)
            .unwrap()
            .send_command_channel
            .clone();

        let server_response_channel = simulation_controller
            .nodes_channels
            .get(&server_id)
            .unwrap()
            .receive_response_channel
            .clone();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();

        let client_2_command_channel = simulation_controller
            .nodes_channels
            .get(&client_2_id)
            .unwrap()
            .send_command_channel
            .clone();

        // Wait for flood request
        thread::sleep(std::time::Duration::from_secs(5));

        // Register client to server
        let res = client_command_channel.send(SimControllerCommand::Register(server_id));
        assert!(res.is_ok());

        // Register client 2 to server
        let res = client_2_command_channel.send(SimControllerCommand::Register(server_id));
        assert!(res.is_ok());

        // leave time for registration
        thread::sleep(std::time::Duration::from_secs(2));

        // Instruct server to remove sender 1
        let res = server_command_channel.send(SimControllerCommand::RemoveSender(drone_1_id));
        assert!(res.is_ok());
        thread::sleep(std::time::Duration::from_secs(2));

        // Instruct server to remove sender 2
        let res = server_command_channel.send(SimControllerCommand::RemoveSender(drone_2_id));
        assert!(res.is_ok());
        thread::sleep(std::time::Duration::from_secs(2));

        // Instruct server to remove sender 3
        let res = server_command_channel.send(SimControllerCommand::RemoveSender(drone_3_id));
        assert!(res.is_ok());

        // Wait for the command to take effect
        thread::sleep(std::time::Duration::from_secs(2));

        // Send message to client
        let res = client_command_channel.send(SimControllerCommand::SendMessage(
            "Hello".to_string(),
            server_id,
            client_2_id,
        ));
        assert!(res.is_ok());

        for response in server_response_channel.iter() {
            if let SimControllerResponseWrapper::Event(SimControllerEvent::FloodRequestSent) =
                response.clone()
            {
                println!("TEST - Flood response {:?}", response);
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Event(SimControllerEvent::FloodRequestSent)
                    ),
                    "Expected Flood response"
                );
                break;
            }
        }
    }

    #[test]
    fn test_topology_using_config_file_setup() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );

        let client_id = 4;

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();
        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .receive_response_channel
            .clone();
        // Wait for the flood to finish
        std::thread::sleep(std::time::Duration::from_secs(1));
        //send topology request
        let res = client_command_channel.send(SimControllerCommand::Topology);
        assert!(res.is_ok());

        // Listen for topology response
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
                topology,
            )) = response
            {
                println!("TEST - Topology response {:?}", topology);
                let expected_response = vec![1, 2];
                let expected_response: HashSet<_> = expected_response.into_iter().collect();
                assert_eq!(topology.edges().get(&client_id), Some(&expected_response));
                break;
            }
        }
    }
}
