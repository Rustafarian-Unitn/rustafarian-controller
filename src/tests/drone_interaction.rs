// test set drone pdr
#[cfg(test)]
mod drone_communication_tests {
    use crate::tests::setup;
    use ::rustafarian_shared::messages::commander_messages::{
        SimControllerEvent, SimControllerMessage, SimControllerResponseWrapper,
    };
    use ::wg_2024::network::NodeId;
    use rustafarian_shared::messages::commander_messages::SimControllerCommand;
    use std::thread;
    use wg_2024::{controller::DroneCommand, drone::Drone};
    const TICKS: u64 = 1000;
    use rustafarian_client::client::Client;

    #[test]
    fn test_set_drone_pdr() {
        let ((mut client, _), _, mut content_server, drones, simulation_controller) =
            setup::setup();

        let content_server_id = 3 as NodeId;
        let client_id = 1 as NodeId;

        for mut drone in drones {
            thread::spawn(move || {
                Drone::run(&mut drone);
            });
        }

        thread::spawn(move || {
            client.run(TICKS);
        });

        thread::spawn(move || {
            content_server.run();
        });

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

        // test message sending before setting new pdr
        let client_command = SimControllerCommand::RequestFileList(content_server_id);
        let command_result = client_command_channel.send(client_command);
        assert!(command_result.is_ok());

        // ignore messages that are not the file list response
        for message in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Message(SimControllerMessage::FileListResponse(
                _,
                _,
            )) = message
            {
                println!("TEST:received file list response");
                assert!(matches!(
                    message,
                    SimControllerResponseWrapper::Message(SimControllerMessage::FileListResponse(
                        _,
                        _
                    ))
                ));
                break;
            }
        }

        let drone_command_channel = simulation_controller
            .drone_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();

        let command = DroneCommand::SetPacketDropRate(1.0);
        drone_command_channel.send(command).unwrap();

        // test message sending before setting new pdr
        let client_command = SimControllerCommand::RequestFileList(content_server_id);
        let command_result = client_command_channel.send(client_command);
        assert!(command_result.is_ok());

        // ignore messages that are not the file list response
        for message in client_response_channel.iter() {
            if let SimControllerResponseWrapper::Event(SimControllerEvent::PacketDropped {
                ..
            }) = message
            {
                println!("TEST:received file list response");
                assert!(matches!(
                    message,
                    SimControllerResponseWrapper::Event(SimControllerEvent::PacketDropped { .. })
                ));
                break;
            }
        }
    }
}
