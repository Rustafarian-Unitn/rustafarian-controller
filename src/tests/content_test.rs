mod chat_test {
    use image::{open, ImageFormat};
    use ::rustafarian_chat_server::chat_server;
    use ::rustafarian_client::client;
    use rustafarian_shared::messages::browser_messages::BrowserResponse;
    use rustafarian_shared::messages::commander_messages::{
        SimControllerCommand, SimControllerEvent, SimControllerMessage,
        SimControllerResponseWrapper,
    };
    use rustafarian_shared::messages::general_messages::ServerType;
    use std::collections::HashSet;
    use std::io::{Cursor, Read};
    use std::{fs, thread};
    use wg_2024::controller::DroneEvent;
    use wg_2024::packet::PacketType;

    use crate::simulation_controller::{self, SimulationController, TICKS};
    // use crate::tests::setup;
    // use crossbeam_channel::unbounded;
    // use rustafarian_client::client::Client;

    #[test]
    fn initialization_content_test() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/topology_2.toml", false);
       
        let client_id: u8 = 7;
        let server_id_1 = 11;
        let server_id_2 = 10;

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

        let server_command_channel_1 = simulation_controller
            .nodes_channels
            .get(&server_id_1)
            .unwrap()
            .send_command_channel
            .clone();

        let server_response_channel_1 = simulation_controller
            .nodes_channels
            .get(&server_id_1)
            .unwrap()
            .receive_response_channel
            .clone();

            let server_command_channel_2 = simulation_controller
            .nodes_channels
            .get(&server_id_2)
            .unwrap()
            .send_command_channel
            .clone();

        let server_response_channel_2 = simulation_controller
            .nodes_channels
            .get(&server_id_2)
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

        let _res = server_command_channel_1.send(SimControllerCommand::Topology);

        for response in server_response_channel_1.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
                topology
            )) = response
            {   
                let mut nodes=topology.nodes().clone();
                nodes.sort();
                let mut expected_nodes=vec![1, 2, 3, 7, 4, 5, 11];
                expected_nodes.sort();
                assert_eq!(nodes, expected_nodes);
                println!("Server 1 nodes: {:?}", topology.nodes());
                break;
            }
        }

        let _res = server_command_channel_2.send(SimControllerCommand::Topology);
        for response in server_response_channel_2.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
                topology
            )) = response
            {   
                let mut nodes=topology.nodes().clone();
                nodes.sort();
                let mut expected_nodes=vec![1, 2, 3, 7, 4, 5, 11];
                expected_nodes.sort();
                assert_eq!(nodes, expected_nodes);
                println!("Server 2 nodes: {:?}", topology.nodes());
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
                node_id,file_ids
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
    fn file_media_test() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/topology_2.toml", false);
       
        let client_id: u8 = 7;
        let server_id = 10;

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

        let _res = client_command_channel.send(SimControllerCommand::RequestMediaFile(2,server_id));

        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::MediaFileResponse(
                id,media
            )) = response
            {
                match open("resources/media/0002.jpg"){
                    Ok(image)=>{
                        let mut file_content = Vec::new();
                        image.write_to(&mut Cursor::new(&mut file_content), ImageFormat::Jpeg)
                        .expect("Failed to convert image to buffer");
                        assert_eq!(file_content, media);
                        break;
                    }
                    Err(err)=>{
                        eprintln!("Error reading media {}",err);
                    }
                    
                }   
            }
        }
    }


    #[test]
    fn file_text_media_test() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/topology_2.toml", false);
       
        let client_id: u8 = 7;
        let server_1_id = 10;
        let server_2_id = 11;

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

        let _rus=client_command_channel.send(SimControllerCommand::RequestServerType(server_1_id));
 
        thread::sleep(std::time::Duration::from_secs(1));  
        let _res = client_command_channel.send(SimControllerCommand::RequestTextFile(3,server_2_id));
        
        for response in client_response_channel.iter() {
             
            
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TextWithReferences(
                id,text, media_files
            )) = response
            {   
                let text_content=fs::read_to_string("resources/files/0003.txt").expect("Failed to read file 0003.txt");
                assert_eq!(text,text_content);
                
                match open("resources/media/0004.jpg"){
                    Ok(image)=>{
                        let mut file_content = Vec::new();
                        image.write_to(&mut Cursor::new(&mut file_content), ImageFormat::Jpeg)
                        .expect("Failed to convert image to buffer");
                        assert_eq!(file_content, *media_files.get(&4).unwrap());
                        break;
                    }
                    Err(err)=>{
                        eprintln!("Error reading media {}",err);
                    }
                    
                }

                
                  
            }
        }
    }

    #[test]
    fn drop_client_test() {}

    #[test]
    fn drop_server_test() {
        //set pdr=0.9
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
    fn error_routing_client_test() {}

    #[test]
    fn error_routing_server_test() {}
}
