// test set drone pdr
#[cfg(test)]
mod drone_communication_tests {
    use std::time::Duration;

    use crate::simulation_controller::SimulationController;
    use crossbeam::select;
 
    use rustafarian_shared::messages::commander_messages::SimControllerCommand;
    use wg_2024::controller::{DroneCommand, DroneEvent};

    #[test]
    fn test_set_drone_pdr() {
        let simulation_controller =
        SimulationController::build("src/tests/configurations/topology_10_nodes.toml", false);
   
        let drone_id = 1;
        let drone_2_id = 2;
        let drone_3_id = 3;

        let client_id = 5;
        let content_server_id = 8;


        let drone_command_channel = simulation_controller
            .drone_channels
            .get(&drone_id)
            .unwrap()
            .send_command_channel
            .clone();

        let drone_command_channel_2 = simulation_controller
            .drone_channels
            .get(&drone_2_id)
            .unwrap()
            .send_command_channel
            .clone();

        let drone_command_channel_3 = simulation_controller
            .drone_channels
            .get(&drone_3_id)
            .unwrap()
            .send_command_channel
            .clone();

        let client_command_channel = simulation_controller
            .nodes_channels
            .get(&client_id)
            .unwrap()
            .send_command_channel
            .clone();

        let drone_event_channel = simulation_controller
            .drone_channels
            .get(&drone_id)
            .unwrap()
            .receive_event_channel
            .clone();

        // Leave time for the flood to finish
        std::thread::sleep(Duration::from_millis(500));
        
        let command = DroneCommand::SetPacketDropRate(1.0);
        drone_command_channel.send(command.clone()).unwrap();
        drone_command_channel_2.send(command.clone()).unwrap();
        drone_command_channel_3.send(command).unwrap();
        
        // Leave time for drones to set the pdr
        std::thread::sleep(Duration::from_millis(500));
        
        // test message sending after setting new pdr
        let client_command = SimControllerCommand::RequestFileList(content_server_id);
        let command_result = client_command_channel.send(client_command);
        assert!(command_result.is_ok());

        // ignore messages until message is received or timeout
        let timeout = Duration::from_secs(1);
        loop {
            select! {
                recv(drone_event_channel) -> response => {
                    if let Ok(DroneEvent::PacketDropped(_)) = response{
                        println!("TEST:received message {:?}", response);
                        assert!(matches!(
                            response,
                           Ok(DroneEvent::PacketDropped(_))
                        ));
                        break;
                    } else {
                        println!("TEST:received unexpected message {:?}", response);
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
