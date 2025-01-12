mod content_test {
    use image::{open, ImageFormat};
    use rustafarian_shared::messages::commander_messages::{
        SimControllerCommand, SimControllerMessage, SimControllerResponseWrapper,
    };
    use rustafarian_shared::messages::general_messages::ServerType;
    use std::io::Cursor;
    use std::{fs, thread};

    use crate::simulation_controller::SimulationController;
    use crate::tests::utils::with_timeout;
    const TIMEOUT: u64 = 15;
    #[test]
    fn initialization_content_test() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );

        let client_id: u8 = 7;
        let server_id_1 = 8;
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
                topology,
            )) = response
            {
                let mut nodes = topology.nodes().clone();
                nodes.sort();
                let mut expected_nodes = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
                expected_nodes.sort();
                assert_eq!(nodes, expected_nodes);
                println!("Client nodes: {:?}", topology.nodes());
                break;
            }
        }

        let _res = server_command_channel_1.send(SimControllerCommand::Topology);

        for response in server_response_channel_1.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
                topology,
            )) = response
            {
                let mut nodes = topology.nodes().clone();
                nodes.sort();
                let mut expected_nodes = vec![1, 2, 3, 7, 4, 5, 6, 8, 9, 10];
                expected_nodes.sort();
                assert_eq!(nodes, expected_nodes);
                println!("Server 1 nodes: {:?}", topology.nodes());
                break;
            }
        }

        let _res = server_command_channel_2.send(SimControllerCommand::Topology);
        for response in server_response_channel_2.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
                topology,
            )) = response
            {
                let mut nodes = topology.nodes().clone();
                nodes.sort();
                let mut expected_nodes = vec![1, 2, 3, 7, 4, 5, 6, 8, 9, 10];
                expected_nodes.sort();
                assert_eq!(nodes, expected_nodes);
                println!("Server 2 nodes: {:?}", topology.nodes());
                break;
            }
        }
    }

    #[test]
    fn server_type_test() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_1.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );

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
            if let SimControllerResponseWrapper::Message(
                SimControllerMessage::ServerTypeResponse(_, server_type),
            ) = response
            {
                assert!(
                    matches!(server_type, ServerType::Text),
                    "Expected ServerType::Text, got {:?}",
                    server_type
                );
                break;
            }
        }
    }

    #[test]
    fn file_list_test() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_1.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );

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
                _,
                file_ids,
            )) = response
            {
                assert_eq!(file_ids.len(), 10);
                break;
            }
        }
    }

    #[test]
    fn file_text_test() {
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );

        let client_id: u8 = 7;
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

        thread::sleep(std::time::Duration::from_secs(5));

        let _res = client_command_channel.send(SimControllerCommand::RequestTextFile(1, server_id));

        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TextFileResponse(
                _,
                text,
            )) = response
            {
                println!("Il testo ricevuto è =>{}", text);
                let file_content = fs::read_to_string("resources/files/0001.txt")
                    .expect("Failed to read file 0001.txt");
                assert_eq!(text, file_content);
                break;
            }
        }
    }

    #[test]
    fn file_media_test() {
            let simulation_controller = SimulationController::build(
                "src/tests/configurations/topology_10_nodes.toml",
                "resources/files".to_string(),
                "resources/media".to_string(),
                false,
            );

            let client_id: u8 = 7;
            let server_id = 10;
            let media_id = 4;

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

            let _res = client_command_channel
                .send(SimControllerCommand::RequestMediaFile(media_id, server_id));

            for response in client_response_channel.iter() {
                if let SimControllerResponseWrapper::Message(
                    SimControllerMessage::MediaFileResponse(_, media),
                ) = response
                {
                    match open("resources/media/0004.jpg") {
                        Ok(image) => {
                            let mut file_content = Vec::new();
                            image
                                .write_to(&mut Cursor::new(&mut file_content), ImageFormat::Jpeg)
                                .expect("Failed to convert image to buffer");
                            assert_eq!(file_content, media);
                            break;
                        }
                        Err(err) => {
                            eprintln!("Error reading media {}", err);
                        }
                    }
                }
            }
    }

    #[test]
    fn file_text_media_test() {
            let simulation_controller = SimulationController::build(
                "src/tests/configurations/topology_10_nodes.toml",
                "resources/files".to_string(),
                "resources/media".to_string(),

                false,
            );

            let client_id: u8 = 5;
            let server_1_id = 8;
            let text_id = 3;
            let media_id = 4;
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

            thread::sleep(std::time::Duration::from_secs(5));

            let _res = client_command_channel
                .send(SimControllerCommand::RequestTextFile(text_id, server_1_id));

            for response in client_response_channel.iter() {
                println!("Response {:?}", response);
                if let SimControllerResponseWrapper::Message(
                    SimControllerMessage::TextWithReferences(_, text, media_files),
                ) = response
                {
                    let text_content = fs::read_to_string("resources/files/0003.txt")
                        .expect("Failed to read file 0003.txt");
                    assert_eq!(text, text_content);

                    match open("resources/media/0004.jpg") {
                        Ok(image) => {
                            let mut file_content = Vec::new();
                            image
                                .write_to(&mut Cursor::new(&mut file_content), ImageFormat::Jpeg)
                                .expect("Failed to convert image to buffer");
                            assert_eq!(file_content, *media_files.get(&media_id).unwrap());
                            break;
                        }
                        Err(err) => {
                            eprintln!("Error reading media {}", err);
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
        let simulation_controller = SimulationController::build(
            "src/tests/configurations/topology_10_nodes.toml",
            "resources/files".to_string(),
            "resources/media".to_string(),
            false,
        );

        let client_id: u8 = 7;
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

        let _res = client_command_channel.send(SimControllerCommand::RequestTextFile(2, server_id));
        thread::sleep(std::time::Duration::from_secs(3));

        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TextFileResponse(
                _,
                text,
            )) = response
            {
                println!("Il testo ricevuto è =>{}", text);
                let file_content: String = fs::read_to_string("resources/files/0002.txt")
                    .expect("Failed to read file 0002.txt");
                assert_eq!(text, file_content);
                break;
            }
        }
    }

    mod content_communication {
        use crate::simulation_controller::SimulationController;
        use crossbeam_channel::select;
        use rustafarian_shared::messages::commander_messages::{
            SimControllerCommand, SimControllerMessage, SimControllerResponseWrapper,
        };
        use rustafarian_shared::messages::general_messages::ServerType;
        use std::thread;
        use std::time::Duration;

        const DEBUG: bool = false;

        #[test]
        fn test_server_type_text() {
            let simulation_controller = SimulationController::build(
                "src/tests/configurations/topology_10_nodes.toml",
                "resources/files".to_string(),
                "resources/media".to_string(),
                true,
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

            // wait for flood to finish
            thread::sleep(std::time::Duration::from_secs(2));

            // Instruct client to register to server
            let res = client_command_channel
                .send(SimControllerCommand::RequestServerType(content_server_id));
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
                "src/tests/configurations/topology_10_nodes.toml",
                "resources/files".to_string(),
                "resources/media".to_string(),

                DEBUG,
            );
            let client_id: u8 = 5;
            let content_server_id: u8 = 8;

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
            thread::sleep(std::time::Duration::from_secs(2));

            let res = client_command_channel
                .send(SimControllerCommand::RequestFileList(content_server_id));
            assert!(res.is_ok());

            // ignore messages until message is received
            for response in client_response_channel.iter() {
                if let SimControllerResponseWrapper::Message(
                    SimControllerMessage::FileListResponse(_, list),
                ) = response.clone()
                {
                    println!("TEST - Message received {:?}", response);
                    assert!(
                        matches!(
                            response,
                            SimControllerResponseWrapper::Message(
                                SimControllerMessage::FileListResponse(_, _)
                            )
                        ),
                        "Expected message received"
                    );
                    assert_eq!(list.len(), 10);

                    break;
                }
            }
        }

        // Test text file request
        #[test]
        fn test_text_file_request() {
            let simulation_controller = SimulationController::build(
                "src/tests/configurations/topology_10_nodes.toml",
                "resources/files".to_string(),
                "resources/media".to_string(),
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
            let res =
                client_command_channel.send(SimControllerCommand::RequestTextFile(1, server_id));
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
                "src/tests/configurations/topology_10_nodes.toml",
                "resources/files".to_string(),
                "resources/media".to_string(),
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
            thread::sleep(std::time::Duration::from_secs(5));
            // Instruct client to request file list
            let res = client_command_channel.send(SimControllerCommand::RequestFileList(server_id));
            assert!(res.is_ok());

            // Ignore all messages until FileListResponse is received
            for message in client_response_channel.iter() {
                if let SimControllerResponseWrapper::Message(
                    SimControllerMessage::FileListResponse(_, _),
                ) = message
                {
                    println!("TEST - Message received {:?}", message);
                    assert!(matches!(
                        message,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::FileListResponse(_, _)
                        )
                    ));
                    break;
                }
            }
        }

        #[test]
        fn test_media_file_request() {
            let simulation_controller = SimulationController::build(
                "src/tests/configurations/topology_10_nodes.toml",
                "resources/files".to_string(),
                "resources/media".to_string(),
                DEBUG,
            );

            let client_id: u8 = 5;
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

            // wait for flood to finish
            thread::sleep(std::time::Duration::from_secs(5));
            // Instruct client to request media file
            let res =
                client_command_channel.send(SimControllerCommand::RequestMediaFile(4, server_id));
            assert!(res.is_ok());

            // Ignore all messages until MediaFileResponse is received
            for message in client_response_channel.iter() {
                if let SimControllerResponseWrapper::Message(
                    SimControllerMessage::MediaFileResponse(_, _),
                ) = message
                {
                    println!("TEST - Message received {:?}", message);
                    assert!(matches!(
                        message,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::MediaFileResponse(_, _)
                        )
                    ));
                    break;
                }
            }
        }

        #[test]
        fn test_text_file_with_references() {
            let simulation_controller = SimulationController::build(
                "src/tests/configurations/topology_10_nodes.toml",
                "resources/files".to_string(),
                "resources/media".to_string(),
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
            let res =
                client_command_channel.send(SimControllerCommand::RequestTextFile(2, server_id));
            assert!(res.is_ok());

            // ignore messages until message is received or timeout
            let timeout = Duration::from_secs(5);
            loop {
                select! {
                    recv(client_response_channel) -> response => {
                        if let Ok(SimControllerResponseWrapper::Message(SimControllerMessage::TextWithReferences(_, _, _))) = response {
                            println!("TEST - Message received {:?}", response);
                            assert!(
                                matches!(
                                    response.unwrap(),
                                    SimControllerResponseWrapper::Message(
                                        SimControllerMessage::TextWithReferences(2, _,_)
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

        #[test]
        fn test_request_invalid_server() {
            let simulation_controller = SimulationController::build(
                "src/tests/configurations/topology_10_nodes.toml",
                "resources/media".to_string(),
                "resources/files".to_string(),
                DEBUG,
            );

            let client_id: u8 = 5;
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

            // wait for flood to finish
            thread::sleep(std::time::Duration::from_secs(1));

            // Instruct client to request text file
            let res =
                client_command_channel.send(SimControllerCommand::RequestTextFile(1, server_id));
            assert!(res.is_ok());

            // ignore messages until message is received or timeout
            let timeout = Duration::from_secs(1);
            loop {
                select! {
                    recv(client_response_channel) -> response => {
                        if let Ok(SimControllerResponseWrapper::Message(SimControllerMessage::TextFileResponse{..})) = response {
                            println!("TEST - Message received {:?}", response);
                            assert!(false, "Server should not respond to invalid request");
                        }
                    }
                    default(timeout) => {
                        // Last message received should be an ack from the server. The server does not answer to the request
                        println!("TEST - Timeout reached");
                        assert!(true);
                        break;
                    }
                }
            }
        }
    }
}
