use crate::runnable::Runnable;
use ap2024_unitn_cppenjoyers_drone::CppEnjoyersDrone;
use crossbeam_channel::{Receiver, Sender};
use d_r_o_n_e_drone::MyDrone;
use dr_ones::Drone as DrOne;
use rustbusters_drone::RustBustersDrone;
use getdroned::GetDroned;
use lockheedrustin_drone::LockheedRustin;
use rust_do_it::RustDoIt;
// use rustastic_drone::RustasticDrone;
use rusteze_drone::RustezeDrone;
use rusty_drones::RustyDrone;
use std::collections::HashMap;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::packet::Packet;

pub fn d_r_o_n_e_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String) {
    (
        Box::new(MyDrone::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        )),
        "D.R.O.N.E. drone".to_string(),
    )
}

pub fn rusteze_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String) {
    (
        Box::new(RustezeDrone::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        )),
        "Rusteeze drone".to_string(),
    )
}

pub fn cpp_enjoyers_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String) {
    (
        Box::new(CppEnjoyersDrone::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        )),
        "C++ Enjoyers drone".to_string(),
    )
}

pub fn rust_do_it_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String) {
    (
        Box::new(RustDoIt::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        )),
        "Rust Do It drone".to_string(),
    )
}

pub fn rust_busters_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String) {
    (
        Box::new(RustBustersDrone::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        )),
        "Rust Busters drone".to_string(),
    )
}

pub fn rusty_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String) {
    (
        Box::new(RustyDrone::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        )),
        "Rusty drone".to_string(),
    )
}

pub fn dr_one_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String) {
    (
        Box::new(DrOne::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        )),
        "Dr One drone".to_string(),
    )
}

// pub fn rustastic_drone(
//     id: u8,
//     controller_send: Sender<DroneEvent>,
//     controller_recv: Receiver<DroneCommand>,
//     packet_recv: Receiver<Packet>,
//     packet_send: HashMap<u8, Sender<Packet>>,
//     pdr: f32,
// ) -> (Box<dyn Runnable>, String) {
//     (
//         Box::new(RustasticDrone::new(
//             id,
//             controller_send,
//             controller_recv,
//             packet_recv,
//             packet_send,
//             pdr,
//         )),
//         "Rustastic drone".to_string(),
//     )
// }

pub fn lockheed_rustin_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String) {
    (
        Box::new(LockheedRustin::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        )),
        "Lockheed Rustin drone".to_string(),
    )
}

pub fn get_droned_drone(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
) -> (Box<dyn Runnable>, String) {
    (
        Box::new(GetDroned::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        )),
        "Get Droned drone".to_string(),
    )
}
