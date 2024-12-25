use crate::runnable::Runnable;
use crate::server::Server;
use crossbeam_channel::{Receiver, Sender};
use rustafarian_drone::RustafarianDrone;
use std::collections::HashMap;
use std::thread::JoinHandle;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::packet::Packet;

pub fn rustafarian_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> Box<dyn Runnable> {
    Box::new(RustafarianDrone::new(
        id,
        controller_send,
        controller_recv,
        packet_recv,
        packet_send,
        pdr,
    ))
}
