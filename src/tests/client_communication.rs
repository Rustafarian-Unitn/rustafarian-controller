#[cfg(test)]
mod client_communication {
    use std::thread;
    
    use rustafarian_shared::messages::commander_messages::SimControllerCommand;
    use wg_2024::controller::DroneEvent;
    use wg_2024::packet::PacketType;
    
    use crate::simulation_controller::TICKS;
    use crate::tests::setup;
    use rustafarian_client::client::Client;
    
    #[test]
    fn test_message_from_client_to_server() {
        let (mut client, mut drone, simulation_controller) = setup::setup();
    
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
}
