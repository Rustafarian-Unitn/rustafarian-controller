mod chat_test {
    use ::rustafarian_chat_server::chat_server;
    use ::rustafarian_client::client;
    use rustafarian_shared::messages::commander_messages::{
        SimControllerCommand, SimControllerEvent, SimControllerMessage,
        SimControllerResponseWrapper,
    };
    use rustafarian_shared::messages::general_messages::ServerType;
    use std::collections::HashSet;
    use std::io::Cursor;
    use std::{fs, thread};
    use wg_2024::controller::DroneEvent;
    use wg_2024::packet::PacketType;

    use crate::simulation_controller::{self, SimulationController, TICKS};
    // use crate::tests::setup;
    // use crossbeam_channel::unbounded;
    // use rustafarian_client::client::Client;

    #[test]
    fn initialization_test() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/topology_1.toml", false);
       
        let client_id: u8 = 7;
        let server_id = 11;

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

        thread::sleep(std::time::Duration::from_secs(1));    

        let _res = client_command_channel.send(SimControllerCommand::Topology);

        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
                topology
            )) = response
            {
                let mut nodes=topology.nodes().clone();
                nodes.sort();
                let mut expected_nodes=vec![1, 2, 3, 11, 4, 5, 7];
                expected_nodes.sort();
                assert_eq!(nodes, expected_nodes);
                println!("Client nodes: {:?}", topology.nodes());
                break;
            }
        }

        let _res = server_command_channel.send(SimControllerCommand::Topology);

        for response in server_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
                topology
            )) = response
            {   
                let mut nodes=topology.nodes().clone();
                nodes.sort();
                let mut expected_nodes=vec![1, 2, 3, 7, 4, 5, 11];
                expected_nodes.sort();
                assert_eq!(nodes, expected_nodes);
                println!("Server nodes: {:?}", topology.nodes());
                break;
            }
        }
    }

    #[test]
    fn server_type_test() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/topology_1.toml", false);
       
        let client_id: u8 = 7;
        let server_id = 11;

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

        thread::sleep(std::time::Duration::from_secs(1));    

        let _res = client_command_channel.send(SimControllerCommand::RequestServerType(server_id));

        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::ServerTypeResponse(
                _,server_type
            )) = response
            {
                assert!(matches!(server_type, ServerType::Text), "Expected ServerType::Text, got {:?}", server_type);
                break;
            }
        }
    }

    #[test]
    fn file_list_test() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/topology_1.toml", false);
       
        let client_id: u8 = 7;
        let server_id = 11;

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

        thread::sleep(std::time::Duration::from_secs(1));    

        let _res = client_command_channel.send(SimControllerCommand::RequestFileList(server_id));

        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::FileListResponse(
                file_ids
            )) = response
            {
                assert_eq!(file_ids.clone().sort(), vec![1,2,3,4,5,6,7,8,9,10].sort());
                break;
            }
        }
    }

    #[test]
    fn file_text_test() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/topology_1.toml", false);
       
        let client_id: u8 = 7;
        let server_id = 11;

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

        thread::sleep(std::time::Duration::from_secs(1));    

        let _res = client_command_channel.send(SimControllerCommand::RequestTextFile(2,server_id));

        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TextFileResponse(
                id,text
            )) = response
            {
                println!("Il testo ricevuto è =>{}",text);
                let file_content=fs::read_to_string("resources/files/0002.txt").expect("Failed to read file 0002.txt");;
                assert_eq!(text,file_content);
                break;
            }
        }
    }

    #[test]
    fn file_media_test() {}

    #[test]
    fn file_text_media_test() {}

    #[test]
    fn drop_client_test() {}

    #[test]
    fn drop_server_test() {}

    #[test]
    fn error_routing_client_test() {}

    #[test]
    fn error_routing_server_test() {}
}
