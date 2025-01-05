mod chat_test {

    use rustafarian_shared::messages::commander_messages::{SimControllerMessage, SimControllerResponseWrapper};

    use crate::simulation_controller::SimulationController;

    #[test]
    fn initialization_test() {}

    #[test]
    fn known_servers() {
        let chat_client_id = 4;
        let simulation_controller =
            SimulationController::build("src/tests/configurations/known_servers_test.toml", false);
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
        //send topology request
        let res = client_command_channel.send(SimControllerCommand::KnownServers);
        assert!(res.is_ok());
        // Listen for topology response
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::KnownServers(
                known_servers,
            )) = response
            {
                println!("TEST - Known servers response {:?}", known_servers);
                assert_eq!(known_servers.len(), 1);
                break;
            }
        }
    }
}
