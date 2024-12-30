#[cfg(test)]
mod client_communication {
    use std::thread;

    use rustafarian_shared::messages::commander_messages::{
        SimControllerCommand, SimControllerEvent, SimControllerMessage,
        SimControllerResponseWrapper,
    };
    use rustafarian_shared::messages::general_messages::Response;
    use wg_2024::controller::DroneEvent;
    use wg_2024::packet::PacketType;

    use crate::simulation_controller::TICKS;
    use crate::tests::setup;
    use rustafarian_client::client::Client;

    #[test]
    fn test_message_from_client_to_server() {
        let ((mut client, mut client_2), _, mut chat_server, mut drone, simulation_controller) =
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

        let client_response_channel = simulation_controller
            .nodes_channels
            .get(&1)
            .unwrap()
            .receive_response_channel
            .clone();
        
        thread::spawn(move || {
            wg_2024::drone::Drone::run(&mut drone);
        });

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

        // Server listen for message from client
        let message = server_receive_packet_channel.recv().unwrap();
        println!("first message {:?}", message);
        assert!(matches!(message.pack_type, PacketType::MsgFragment(_)));

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        println!("ack {:?}", ack);
        assert!(matches!(ack, DroneEvent::PacketSent(_)));

        // Listen for packets from client
        let first_response = client_response_channel.recv().unwrap();

        println!("{:?}", first_response);
        println!("first response {:?}", first_response);
        assert!(matches!(
            first_response,
            SimControllerResponseWrapper::Event(SimControllerEvent::PacketSent { .. })
        ));

        let second_response = client_response_channel.recv().unwrap();
        println!("second response {:?}", second_response);
        println!("{:?}", second_response);

        assert!(
            matches!(
                second_response,
                SimControllerResponseWrapper::Event(SimControllerEvent::MessageSent(_, _, _))
            ),
            "Expected message sent"
        );

        let third_response = client_response_channel.recv().unwrap();
        println!("third response {:?}", third_response);
        assert!(
            matches!(
                third_response,
                SimControllerResponseWrapper::Event(SimControllerEvent::PacketReceived(_))
            ),
            "Expected packet received"
        );

        let fourth_response = client_response_channel.recv().unwrap();
        println!("Fourth response {:?}", fourth_response);
        assert!(
            matches!(
                fourth_response,
                SimControllerResponseWrapper::Message(SimControllerMessage::MessageReceived(
                    _,
                    _,
                    _
                ))
            ),
            "Expected message received"
        );
    }

    #[test]
    fn test_client_register() {
        let ((mut client, _), _, _, mut drone, simulation_controller) = setup::setup();

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

        thread::spawn(move || {
            wg_2024::drone::Drone::run(&mut drone);
        });

        thread::spawn(move || {
            client.run(TICKS);
        });

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::Register(3));
        assert!(res.is_ok());

        // Server listen for message from client
        let message = server_receive_packet_channel.recv().unwrap();

        assert!(matches!(message.pack_type, PacketType::MsgFragment(_)));

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));
    }

    // Test client list
    #[test]
    fn test_client_list() {
        let ((mut client, _), _, _, mut drone, simulation_controller) = setup::setup();

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

        thread::spawn(move || {
            wg_2024::drone::Drone::run(&mut drone);
        });

        thread::spawn(move || {
            client.run(TICKS);
        });

        // Instruct client to register to server
        let res = client_command_channel.send(SimControllerCommand::ClientList(3));
        assert!(res.is_ok());

        // Server listen for message from client
        let message = server_receive_packet_channel.recv().unwrap();

        assert!(matches!(message.pack_type, PacketType::MsgFragment(_)));

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));
    }

    //Test flood request
    #[test]
    fn test_flood_request() {
        let ((mut client, _), mut server, _, mut drone, simulation_controller) = setup::setup();

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

        let drone_receive_event_channel = simulation_controller
            .drone_channels
            .get(&2)
            .unwrap()
            .receive_event_channel
            .clone();

        thread::spawn(move || {
            wg_2024::drone::Drone::run(&mut drone);
        });

        thread::spawn(move || {
            client.run(TICKS);
        });

        thread::spawn(move || {
            server.run();
        });

        // Instruct client to request flood
        let res = client_command_channel.send(SimControllerCommand::FloodRequest);
        assert!(res.is_ok());

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));

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
        let ((mut client, _), _, _, mut drone, simulation_controller) = setup::setup();

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

        let drone_receive_event_channel = simulation_controller
            .drone_channels
            .get(&2)
            .unwrap()
            .receive_event_channel
            .clone();

        thread::spawn(move || {
            wg_2024::drone::Drone::run(&mut drone);
        });

        thread::spawn(move || {
            client.run(TICKS);
        });

        // Instruct client to request known servers
        let res = client_command_channel.send(SimControllerCommand::KnownServers);
        assert!(res.is_ok());

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));

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
        let ((mut client, _), _, _, mut drone, simulation_controller) = setup::setup();

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

        let drone_receive_event_channel = simulation_controller
            .drone_channels
            .get(&2)
            .unwrap()
            .receive_event_channel
            .clone();

        thread::spawn(move || {
            wg_2024::drone::Drone::run(&mut drone);
        });

        thread::spawn(move || {
            client.run(TICKS);
        });

        // Instruct client to request registered servers
        let res = client_command_channel.send(SimControllerCommand::RegisteredServers);
        assert!(res.is_ok());

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));

        // Listen for packets from client
        // First packet is the registered servers request sent
        let first_packet = controller_response_channel.recv().unwrap();
        assert!(
            matches!(first_packet, SimControllerResponseWrapper::Event(SimControllerEvent::PacketSent{packet_type, ..}) if packet_type == "RegisteredServers"),
            "Expected registered servers request sent"
        );
        // Second packet is RegisteredServers response
        let second_packet = controller_response_channel.recv().unwrap();
        assert!(
            matches!(
                second_packet,
                SimControllerResponseWrapper::Message(
                    SimControllerMessage::RegisteredServersResponse(_)
                )
            ),
            "Expected registered servers response"
        );
    }

    // Test text file request
    #[test]
    fn test_text_file_request() {
        let ((mut client, _), _, _, mut drone, simulation_controller) = setup::setup();

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

        thread::spawn(move || {
            wg_2024::drone::Drone::run(&mut drone);
        });

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
        let ((mut client, _), _, _, mut drone, simulation_controller) = setup::setup();

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

        thread::spawn(move || {
            wg_2024::drone::Drone::run(&mut drone);
        });

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
        let ((mut client, _), _, _, mut drone, simulation_controller) = setup::setup();

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

        thread::spawn(move || {
            wg_2024::drone::Drone::run(&mut drone);
        });

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
