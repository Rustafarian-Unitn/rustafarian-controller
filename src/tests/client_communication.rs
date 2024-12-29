#[cfg(test)]
mod client_communication {
    use std::thread;

    use rustafarian_shared::messages::commander_messages::{SimControllerCommand, SimControllerEvent, SimControllerMessage, SimControllerResponseWrapper};
    use wg_2024::controller::DroneEvent;
    use wg_2024::packet::PacketType;

    use crate::simulation_controller::TICKS;
    use crate::tests::setup;
    use rustafarian_client::client::Client;

    #[test]
    fn test_message_from_client_to_server() {
        let (mut client, _, mut drone, simulation_controller) = setup::setup();

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

        // Instruct client to send message to server
        let res = client_command_channel.send(SimControllerCommand::SendMessage(
            "Hello world".to_string(),
            3,
            3,
        ));
        assert!(res.is_ok());

        // Server listen for message from client
        let message = server_receive_packet_channel.recv().unwrap();

        assert!(matches!(message.pack_type, PacketType::MsgFragment(_)));

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));
    }

    #[test]
    fn test_client_register() {
        let (mut client, _, mut drone, simulation_controller) = setup::setup();

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
        let (mut client, _, mut drone, simulation_controller) = setup::setup();

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
        let (mut client, mut server, mut drone, simulation_controller) = setup::setup();

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

        thread::spawn(move || {
            server.run();
        });

        // Instruct client to request flood
        let res = client_command_channel.send(SimControllerCommand::FloodRequest);
        assert!(res.is_ok());

        // Server listen for message from client
        // let message = server_receive_packet_channel.recv().unwrap();

        // assert!(matches!(message.pack_type, PacketType::FloodRequest(_)));

        // Listen for ack from drone
        let ack = drone_receive_event_channel.recv().unwrap();
        assert!(matches!(ack, DroneEvent::PacketSent(_)));

        // Listen for packets from client
        // First packet is the flood request sent
        let first_packet = controller_response_channel.recv().unwrap();
        assert!(matches!(first_packet, SimControllerResponseWrapper::Event(SimControllerEvent::FloodRequestSent)),"Expected flood request sent");
        // Second packet is PacketReceived event
        let second_packet = controller_response_channel.recv().unwrap();
        assert!(matches!(second_packet, SimControllerResponseWrapper::Event(SimControllerEvent::PacketReceived(_))),"Expected packet received");
        // Third packet is flood response from the server
        let third_packet = controller_response_channel.recv().unwrap();
        assert!(matches!(third_packet, SimControllerResponseWrapper::Message(SimControllerMessage::FloodResponse(_))),"Expected flood response");

    }

    
}