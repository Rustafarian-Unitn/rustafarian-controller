mod content_communication {
    use rustafarian_shared::messages::commander_messages::{
        SimControllerCommand, SimControllerMessage, SimControllerResponseWrapper,
    };
    use rustafarian_shared::messages::general_messages::ServerType;
    use std::process::exit;
    use std::thread;
    use wg_2024::controller::DroneEvent;
    use wg_2024::packet::PacketType;

    use crate::simulation_controller::{SimulationController, TICKS};
    use crate::tests::setup;
    use rustafarian_client::client::Client;

    #[test]
    fn test_server_type_text() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/test_complex_config.toml", false);
        let content_server_id: u8 = 8;
        let client_id: u8 = 5;

        let _client_command_channel = simulation_controller
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
        // let res =
        //     client_command_channel.send(SimControllerCommand::RequestServerType(content_server_id));
        // assert!(res.is_ok());

        // ignore messages until message is received
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(
                SimControllerMessage::ServerTypeResponse(server_id, server_type),
            ) = response
            {
                println!("TEST - Server type {:?}", server_type);
                assert_eq!(server_id, content_server_id);
                assert!(matches!(server_type, ServerType::Text));
                exit(0);
            }
        }
    }

    #[test]
    fn test_file_list_from_to_server() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/test_complex_config.toml", false);
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
            client_command_channel.send(SimControllerCommand::RequestFileList(content_server_id));
        assert!(res.is_ok());

        // ignore messages until message is received
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::FileListResponse(
                _,_
            )) = response
            {
                println!("TEST - Message received {:?}", response);
                let _expected_list: Vec<u8> = vec![0, 1];
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::FileListResponse(content_server_id, _)
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
            false,
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

        // ignore messages until message is received
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TextFileResponse(
                _,
                _,
            )) = response
            {
                println!("TEST - Message received {:?}", response);
                let _expected_text = "test".to_string();

                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::TextFileResponse(1, _expected_text2)
                        )
                    ),
                    "Expected message received"
                );
                break;
            }
        }
    }

    #[test]
    fn test_media_file_request() {
        let ((mut client, _), _, _, drones, simulation_controller) = setup::setup();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .send_command_channel
            .clone();

        let server_receive_packet_channel = simulation_controller
            .nodes_channels
            .get(&3)
            .unwrap()
            .receive_packet_channel
            .clone();

        let drone_receive_event_channel = simulation_controller
            .drone_channels
            .get(&2)
            .unwrap()
            .receive_event_channel
            .clone();

        for mut drone in drones {
            thread::spawn(move || {
                wg_2024::drone::Drone::run(&mut drone);
            });
        }

        thread::spawn(move || {
            client.run(TICKS);
        });

        // Instruct client to request media file
        let res = client_command_channel.send(SimControllerCommand::RequestMediaFile(1, 3));
        assert!(res.is_ok());

        // Server listen for message from client
        let message = server_receive_packet_channel.recv().unwrap();

        assert!(matches!(message.pack_type, PacketType::MsgFragment(_)));

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));
    }
}
