mod content_communication {
    use ::rustafarian_chat_server::chat_server;
    use ::rustafarian_content_server::content_server;
    use rustafarian_shared::messages::commander_messages::{
        SimControllerCommand, SimControllerEvent, SimControllerMessage,
        SimControllerResponseWrapper,
    };
    use std::collections::HashSet;
    use std::thread;
    use wg_2024::controller::DroneEvent;
    use wg_2024::packet::PacketType;

    use crate::simulation_controller::{self, SimulationController, TICKS};
    use crate::tests::setup;
    use crossbeam_channel::unbounded;
    use rustafarian_client::client::Client;

    #[test]
    fn test_file_list_from_to_server() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/test_complex_config.toml");
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
                _,
            )) = response
            {
                println!("TEST - Message received {:?}", response);
                let expected_list: Vec<u8> = vec![0,1];
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::FileListResponse(expected_list)
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
        let simulation_controller =
            SimulationController::build("src/tests/configurations/simple_config_for_content_tests.toml");
       
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
                _,_
            )) = response
            {
                println!("TEST - Message received {:?}", response);
                let expected_text = "test".to_string();

                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::TextFileResponse(1, expected_text)
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
