mod client_communication {
    use ::rustafarian_chat_server::chat_server;
    use ::rustafarian_client::client;
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
    fn test_message_from_client_to_server() {
        let controller = SimulationController::build("src/tests/configurations/simple_config_for_chat_tests.toml");
        let client_id: u8 = 6;
        let client_2_id: u8 = 8;
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

        // Wait for flood request
        thread::sleep(std::time::Duration::from_secs(5));

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(server_id));
        assert!(res.is_ok());

        // Instruct client 2 to register to server
        let res = client_2_command_channel.send(SimControllerCommand::Register(server_id));
        assert!(res.is_ok());

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

        // ignore messages until flood response is received
        for response in controller_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::FloodResponse(_)) =
                response
            {
                println!("TEST - Flood response {:?}", response);
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(SimControllerMessage::FloodResponse(_))
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
        let ((mut client, _), _, mut chat_server, drones, simulation_controller) = setup::setup();

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
            chat_server.run();
        });
        
        // Instruct client to request known servers
        let res = client_command_channel.send(SimControllerCommand::KnownServers);
        assert!(res.is_ok());
        // Listen for packets from client
        // Ignore messages until KnownServers Response is received
        for response in controller_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(
                SimControllerMessage::KnownServers(_),
            ) = response
            {
                println!("TEST - Known servers response {:?}", response);
                assert!(
                    matches!(
                        response,
                        SimControllerResponseWrapper::Message(SimControllerMessage::KnownServers(_))
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
        let ((mut client, _), _, mut chat_server, drones, simulation_controller) =
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

        // Instruct client to request registered servers
        let res = client_command_channel.send(SimControllerCommand::RegisteredServers);
        assert!(res.is_ok());

        // Ignore messages until RegisteredServersResponse is received
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
        thread::sleep(std::time::Duration::from_secs(1));

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
        let server_id = 4;

        let ((mut client, _), _, mut chat_server, drones, simulation_controller) = setup::setup();

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

        thread::sleep(std::time::Duration::from_secs(5));

        // Instruct client to remove sender 1
        let res = server_command_channel.send(SimControllerCommand::RemoveSender(2));
        assert!(res.is_ok());

        // Instruct client to remove sender 2
        let res = server_command_channel.send(SimControllerCommand::RemoveSender(6));
        assert!(res.is_ok());

        // Instruct client to remove sender 3
        let res = server_command_channel.send(SimControllerCommand::RemoveSender(7));
        assert!(res.is_ok());

        // Test server topology
        let res = server_command_channel.send(SimControllerCommand::Topology);
        assert!(res.is_ok());

        // Listen for topology response
        for response in server_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::TopologyResponse(
                topology,
            )) = response
            {
                println!(
                    "TEST - Topology response for client {} -  {:?}",
                    server_id,
                    topology.edges().get(&server_id)
                );

                let expected_response = vec![];
                let expected_response: HashSet<_> = expected_response.into_iter().collect();
                assert_eq!(topology.edges().get(&server_id), Some(&expected_response));
                break;
            }
        }
    }

    #[test]
    fn test_topology_using_config_file_setup() {
        let simulation_controller =
            SimulationController::build("src/tests/configurations/test_complex_config.toml");
        println!("Simulation controller {:?}", simulation_controller);
        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&4)
            .unwrap()
            .send_command_channel
            .clone();
        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&4)
            .unwrap()
            .receive_response_channel
            .clone();
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
                let expected_response = vec![1, 2, 3];
                let expected_response: HashSet<_> = expected_response.into_iter().collect();
                assert_eq!(topology.edges().get(&4), Some(&expected_response));
                break;
            }
        }
    } 
}
