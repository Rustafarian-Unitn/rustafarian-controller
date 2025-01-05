// test set drone pdr
#[cfg(test)]
mod drone_communication {
    // use crate::tests::setup;
    // use rustafarian_shared::messages::commander_messages::SimControllerCommand;
    // use std::thread;
    // use wg_2024::{controller::{DroneCommand, DroneEvent}, drone::Drone, packet::PacketType};
    // const TICKS: u64 = 1000;
    // use rustafarian_client::client::Client;

    // #[test]
    // fn test_set_drone_pdr() {
    //     let (mut client, _, mut drone, simulation_controller) = setup::setup();

    //     let drone_receive_event_channel = simulation_controller
    //         .drone_channels
    //         .get(&2)
    //         .unwrap()
    //         .receive_event_channel
    //         .clone();

    //     thread::spawn(move || {
    //         Drone::run(&mut drone);
    //     });

    //     thread::spawn(move || {
    //         client.run(TICKS);
    //     });

    //     let client_command_channel = simulation_controller
    //         .nodes_channels
    //         .get(&1)
    //         .unwrap()
    //         .send_command_channel
    //         .clone();

    //     let server_receive_packet_channel = simulation_controller
    //         .nodes_channels
    //         .get(&3)
    //         .unwrap()
    //         .receive_packet_channel
    //         .clone();

    //     // test message sending before setting new pdr
    //     let client_command = SimControllerCommand::SendMessage("Hello world".to_string(), 2, 1);
    //     let command_result = client_command_channel.send(client_command);
    //     assert_eq!(command_result.is_ok(), true);

    //     // test message receiving before setting new pdr
    //     let message = server_receive_packet_channel.recv().unwrap();

    //     assert!(matches!(message.pack_type, PacketType::MsgFragment(_)));

    //      // Listen for ack from drone
    //      let ack = drone_receive_event_channel.recv().unwrap();
    //      assert!(matches!(ack, DroneEvent::PacketSent(_)));

    //     let drone_command_channel = simulation_controller
    //         .drone_channels
    //         .get(&2)
    //         .unwrap()
    //         .send_command_channel
    //         .clone();

    //     let command = DroneCommand::SetPacketDropRate(1.0);
    //     drone_command_channel.send(command).unwrap();
    // }
}
