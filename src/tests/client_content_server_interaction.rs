mod content_communication {
    use crate::simulation_controller::SimulationController;
    use crossbeam_channel::select;
    use rustafarian_shared::messages::commander_messages::{
        SimControllerCommand, SimControllerMessage, SimControllerResponseWrapper,
    };
    use rustafarian_shared::messages::general_messages::ServerType;
    use std::process::exit;
    use std::thread;
    use std::time::Duration;

    const DEBUG: bool = false;

    #[test]
    fn test_server_type_text() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/simple_config_for_content_tests.toml", true);
        let content_server_id: u8 = 8;
        let client_id: u8 = 5;

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

        // Instruct client to register to server
        let res =
            client_command_channel.send(SimControllerCommand::RequestServerType(content_server_id));
        assert!(res.is_ok());


        // ignore messages until message is received
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(
                SimControllerMessage::ServerTypeResponse(server_id, server_type),
            ) = response
            {

                println!("TEST - Server type {:?}", server_type);

                if server_id == content_server_id {
                    assert!(matches!(server_type, ServerType::Text));
                    break;
                }
            }
        }
    }

    #[test]
    fn test_file_list_from_to_server() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/simple_config_for_content_tests.toml",
            DEBUG,
        );
        let content_server_id: u8 = 8;
        let client_id: u8 = 5;

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

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(content_server_id));
        assert!(res.is_ok());

        let res =
            client_command_channel.send(SimControllerCommand::RequestFileList(content_server_id));
        assert!(res.is_ok());

        // ignore messages until message is received
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::FileListResponse(
                _,
                _,
            )) = response
            {
                println!("TEST - Message received {:?}", response);
                let _expected_list: Vec<u8> = vec![0, 1];
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::FileListResponse(_, _expected_list)
                        )
                    ),
                    "Expected message received"
                );
                break;
            }
        }
    }

    // Test text file request
    #[test]
    fn test_text_file_request() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/simple_config_for_content_tests.toml",
            DEBUG,
        );

        let client_id: u8 = 5;
        let server_id = 8;

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

        // wait for flood to finish
        thread::sleep(std::time::Duration::from_secs(1));

        // Instruct client to request text file
        let res = client_command_channel.send(SimControllerCommand::RequestTextFile(1, server_id));
        assert!(res.is_ok());

        // ignore messages until message is received or timeout
        let timeout = Duration::from_secs(5);
        loop {
            select! {
                recv(client_response_channel) -> response => {
                    if let Ok(SimControllerResponseWrapper::Message(SimControllerMessage::TextFileResponse(_, _))) = response {
                        println!("TEST - Message received {:?}", response);
                        let _expected_text = "test".to_string();

                        assert!(
                            matches!(
                                response.unwrap(),
                                SimControllerResponseWrapper::Message(
                                    SimControllerMessage::TextFileResponse(1, _expected_text)
                                )
                            ),
                            "Expected message received"
                        );
                        break;
                    }
                }
                default(timeout) => {
                    println!("TEST - Timeout reached");
                    break;
                }
            }
        }
    }

    // Test file list request
    #[test]
    fn test_file_list_request() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/simple_config_for_content_tests.toml",
            DEBUG,
        );

        let client_id: u8 = 5;
        let server_id = 8;

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

        // wait for flood to finish
        thread::sleep(std::time::Duration::from_secs(1));
        // Instruct client to request file list
        let res = client_command_channel.send(SimControllerCommand::RequestFileList(server_id));
        assert!(res.is_ok());

        // Ignore all messages until FileListResponse is received
        for message in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::FileListResponse(
                _,
                _,
            )) = message
            {
                println!("TEST - Message received {:?}", message);
                assert!(matches!(
                    message,
                    SimControllerResponseWrapper::Message(SimControllerMessage::FileListResponse(
                        _,
                        _
                    ))
                ));
                break;
            }
        }
    }

    #[test]
    fn test_media_file_request() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/simple_config_for_content_tests.toml",
            DEBUG,
        );

        let client_id: u8 = 5;
        let server_id = 7;

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

        // wait for flood to finish
        thread::sleep(std::time::Duration::from_secs(1));
        // Instruct client to request media file
        let res = client_command_channel.send(SimControllerCommand::RequestMediaFile(1, server_id));
        assert!(res.is_ok());

        // Ignore all messages until MediaFileResponse is received
        for message in client_response_channel.iter()
        {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::MediaFileResponse(
                _,
                _,
            )) = message
            {
                println!("TEST - Message received {:?}", message);
                assert!(matches!(
                    message,
                    SimControllerResponseWrapper::Message(SimControllerMessage::MediaFileResponse(
                        _,
                        _
                    ))
                ));
                break;
            }
        }
    }
}
