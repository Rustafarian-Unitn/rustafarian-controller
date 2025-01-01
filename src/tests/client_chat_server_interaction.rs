mod client_communication {
    use ::rustafarian_chat_server::chat_server;
    use rustafarian_shared::messages::commander_messages::{
        SimControllerCommand, SimControllerEvent, SimControllerMessage,
        SimControllerResponseWrapper,
    };
    use std::collections::HashSet;
    use std::thread;
    use wg_2024::controller::DroneEvent;
    use wg_2024::packet::PacketType;

    use crate::simulation_controller::TICKS;
    use crate::tests::setup;
    use crossbeam_channel::unbounded;
    use rustafarian_client::client::Client;

    #[test]
    fn test_message_from_client_to_server() {
        let ((mut client, mut client_2), _, mut chat_server, drones, simulation_controller) =
            setup::setup();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .send_command_channel
            .clone();

        let client_2_command_channel = simulation_controller
            .nodes_channels
            .get(&5)
            .unwrap()
            .send_command_channel
            .clone();

        let drone_receive_event_channel = simulation_controller
            .drone_channels
            .get(&2)
            .unwrap()
            .receive_event_channel
            .clone();

        let client_2_response_channel = simulation_controller
            .nodes_channels
            .get(&5)
            .unwrap()
            .receive_response_channel
            .clone();

        for mut drone in drones {
            thread::spawn(move || {
                wg_2024::drone::Drone::run(&mut drone);
            });
        }

        thread::spawn(move || {
            client.run(TICKS);
        });

        thread::spawn(move || {
            client_2.run(TICKS);
        });

        thread::spawn(move || {
            chat_server.run();
        });

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(4));
        assert!(res.is_ok());

        let res = client_2_command_channel.send(SimControllerCommand::Register(4));
        assert!(res.is_ok());

        // Instruct client to send message to server
        let res = client_command_channel.send(SimControllerCommand::SendMessage(
            "Hello".to_string(),
            4,
            5,
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
                let expected_response = "Hello".to_string();
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::MessageReceived(4, 1, expected_response)
                        )
                    ),
                    "Expected message received"
                );
                break;
            }
        }
    }

    #[test]
    fn test_client_register() {
        let ((mut client, _), _, mut chat_server, mut drones, simulation_controller) =
            setup::setup();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .send_command_channel
            .clone();

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .receive_response_channel
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

        thread::spawn(move || {
            chat_server.run();
        });

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(4));
        assert!(res.is_ok());

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));

        //1. registration to server
        let first_response = client_response_channel.recv().unwrap();
        println!("first response {:?}", first_response);
        assert!(matches!(
            first_response,
            SimControllerResponseWrapper::Event(SimControllerEvent::PacketSent { .. })
        ));

        //2. Ack from server
        let second_response = client_response_channel.recv().unwrap();
        println!("second response {:?}", second_response);

        assert!(
            matches!(
                second_response,
                SimControllerResponseWrapper::Event(SimControllerEvent::PacketReceived(_))
            ),
            "Expected packet received"
        );

        //3. Registration packet from server
        let third_response = client_response_channel.recv().unwrap();
        println!("third response {:?}", third_response);
        assert!(
            matches!(
                third_response,
                SimControllerResponseWrapper::Event(SimControllerEvent::PacketReceived(_))
            ),
            "Expected packet received"
        );
    }

    // Test client list
    #[test]
    fn test_client_list() {
        let ((mut client, mut client_2), _, mut chat_server, drones, simulation_controller) =
            setup::setup();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .send_command_channel
            .clone();

        let client_2_command_channel = simulation_controller
            .nodes_channels
            .get(&5)
            .unwrap()
            .send_command_channel
            .clone();

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .receive_response_channel
            .clone();

        for mut drone in drones {
            thread::spawn(move || {
                wg_2024::drone::Drone::run(&mut drone);
            });
        }

        thread::spawn(move || {
            client.run(TICKS);
        });

        thread::spawn(move || {
            client_2.run(TICKS);
        });

        thread::spawn(move || {
            chat_server.run();
        });

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(4));
        assert!(res.is_ok());

        // Instruction client 2 to register to server
        let res = client_2_command_channel.send(SimControllerCommand::Register(4));
        assert!(res.is_ok());

        // Instruct client to request client list
        let res = client_command_channel.send(SimControllerCommand::ClientList(4));
        assert!(res.is_ok());

        // Ignore messages until ClientListResponse is received
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(
                SimControllerMessage::ClientListResponse(_, _),
            ) = response
            {
                println!("TEST - Client list response {:?}", response);
                let expected_list = vec![1, 5];
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::ClientListResponse(4, expected_response)
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
        let ((mut client, _), _, mut server, drones, simulation_controller) = setup::setup();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .send_command_channel
            .clone();

        let controller_response_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .receive_response_channel
            .clone();

        for mut drone in drones {
            thread::spawn(move || {
                wg_2024::drone::Drone::run(&mut drone);
            });
        }

        thread::spawn(move || {
            client.run(TICKS);
        });

        thread::spawn(move || {
            server.run();
        });

        // Instruct client to request flood
        let res = client_command_channel.send(SimControllerCommand::FloodRequest);
        assert!(res.is_ok());

        // Listen for packets from client
        // First packet is the flood request sent
        let first_packet = controller_response_channel.recv().unwrap();
        assert!(
            matches!(
                first_packet,
                SimControllerResponseWrapper::Event(SimControllerEvent::FloodRequestSent)
            ),
            "Expected flood request sent"
        );
        // Second packet is PacketReceived event
        let second_packet = controller_response_channel.recv().unwrap();
        assert!(
            matches!(
                second_packet,
                SimControllerResponseWrapper::Event(SimControllerEvent::PacketReceived(_))
            ),
            "Expected packet received"
        );
        // Third packet is flood response from the server
        let third_packet = controller_response_channel.recv().unwrap();
        assert!(
            matches!(
                third_packet,
                SimControllerResponseWrapper::Message(SimControllerMessage::FloodResponse(_))
            ),
            "Expected flood response"
        );
    }

    // Test known servers
    #[test]
    fn test_known_servers() {
        let ((mut client, _), _, _, drones, simulation_controller) = setup::setup();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .send_command_channel
            .clone();

        let controller_response_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .receive_response_channel
            .clone();

        for mut drone in drones {
            thread::spawn(move || {
                wg_2024::drone::Drone::run(&mut drone);
            });
        }

        thread::spawn(move || {
            client.run(TICKS);
        });

        // Instruct client to request known servers
        let res = client_command_channel.send(SimControllerCommand::KnownServers);
        assert!(res.is_ok());

        // Listen for packets from client
        // First packet is the known servers request sent
        let first_packet = controller_response_channel.recv().unwrap();
        assert!(
            matches!(first_packet, SimControllerResponseWrapper::Event(SimControllerEvent::PacketSent{packet_type, ..}) if packet_type == "KnownServers"),
            "Expected known servers request sent"
        );
        // Second packet is KnownServers response
        let second_packet = controller_response_channel.recv().unwrap();
        assert!(
            matches!(
                second_packet,
                SimControllerResponseWrapper::Message(SimControllerMessage::KnownServers(_))
            ),
            "Expected known servers response"
        );
    }

    // Test registered servers
    #[test]
    fn test_registered_servers() {
        let ((mut client, _), _, mut chat_server, mut drones, simulation_controller) =
            setup::setup();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .send_command_channel
            .clone();

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .receive_response_channel
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

        thread::spawn(move || {
            chat_server.run();
        });

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(4));
        assert!(res.is_ok());

        // Allow time for registration
        thread::sleep(std::time::Duration::from_secs(2));

        // Instruct client to request registered servers
        let res = client_command_channel.send(SimControllerCommand::RegisteredServers);
        assert!(res.is_ok());

        // Ignore messages until RegisteredServersResponse is received
        //RegisteredServersResponse
        for response in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(
                SimControllerMessage::RegisteredServersResponse(_),
            ) = response
            {
                let expected_response = vec![4];
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(
                            SimControllerMessage::RegisteredServersResponse(expected_response)
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
        let ((mut client, mut client_2), _, mut chat_server, drones, simulation_controller) =
            setup::setup();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .send_command_channel
            .clone();

        let client_id = client.client_id();

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .receive_response_channel
            .clone();


        for mut drone in drones {
            thread::spawn(move || {
                wg_2024::drone::Drone::run(&mut drone);
            });
        }

        thread::spawn(move || {
                client.run(TICKS);
        });

        thread::spawn(move || {
            client_2.run(TICKS);
        });

        thread::spawn(move || {
            chat_server.run();
        });

        // Leave time for flood request
        thread::sleep(std::time::Duration::from_secs(5));
      
        // Instruct client to remove sender 1
        let res = client_command_channel.send(SimControllerCommand::RemoveSender(2));
        assert!(res.is_ok());

        // Instruct client to remove sender 2
        let res = client_command_channel.send(SimControllerCommand::RemoveSender(6));
        assert!(res.is_ok());

        // Instruct client to remove sender 3
        let res = client_command_channel.send(SimControllerCommand::RemoveSender(7));
        assert!(res.is_ok());

        // Send message to client 2
        let res = client_command_channel.send(SimControllerCommand::SendMessage(
            "Hello".to_string(),
            4,
            5,
        ));
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
        let ((mut client, _), _, mut chat_server, drones, simulation_controller) = setup::setup();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .send_command_channel
            .clone();

        let client_id = client.client_id();

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .receive_response_channel
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

        thread::spawn(move || {
            chat_server.run();
        });

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(4));
        assert!(res.is_ok());

        // Allow time for registration
        thread::sleep(std::time::Duration::from_secs(2));

        // Check topology before receiver removal
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

                let expected_response = vec![2, 6, 7];
                let expected_response: HashSet<_> = expected_response.into_iter().collect();
                assert_eq!(topology.edges().get(&client_id), Some(&expected_response));
                break;
            }
        }

        // // Instruct server to remove receiver 1
        // let res = client_command_channel.send(SimControllerCommand::RemoveReceiver(2));
        // assert!(res.is_ok());

        // Instruct server to remove receiver 2
        // let res = client_command
    }

    // Test text file request
    #[test]
    fn test_text_file_request() {
        let ((mut client, _), _, _, mut drones, simulation_controller) = setup::setup();

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

        // Instruct client to request text file
        let res = client_command_channel.send(SimControllerCommand::RequestTextFile(1, 3));
        assert!(res.is_ok());

        // Server listen for message from client
        let message = server_receive_packet_channel.recv().unwrap();

        assert!(matches!(message.pack_type, PacketType::MsgFragment(_)));

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));
    }

    // Test media file request
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

    // Test file list request
    #[test]
    fn test_file_list_request() {
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

        // Instruct client to request file list
        let res = client_command_channel.send(SimControllerCommand::RequestFileList(3));
        assert!(res.is_ok());

        // Server listen for message from client
        let message = server_receive_packet_channel.recv().unwrap();

        assert!(matches!(message.pack_type, PacketType::MsgFragment(_)));

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));
    }
}
