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
        let res = client_command_channel.send(SimControllerCommand::RequestFileList(content_server_id));
        assert!(res.is_ok());

        // ignore messages until message is received
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::FileListResponse(_)) = response
            {
                println!("TEST - Message received {:?}", response);
                let expected_list:Vec<u8>  = vec![];
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

}
