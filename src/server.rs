use crossbeam_channel::{select, Receiver, Sender};
use std::collections::HashMap;
use wg_2024::network::SourceRoutingHeader;
use wg_2024::packet::{FloodResponse, Fragment, NodeType, Packet, PacketType};

const FRAGMENT_DSIZE: usize = 128;

pub struct Server {
    server_id: u8,
    receiver: Receiver<Packet>,
    senders: HashMap<u8, Sender<Packet>>,
}

impl Server {
    pub fn new(
        server_id: u8,
        receiver: Receiver<Packet>,
        senders: HashMap<u8, Sender<Packet>>,
    ) -> Self {
        Server {
            server_id,
            receiver,
            senders,
        }
    }

    pub fn run(&mut self) {
        println!("Created server {:?}", self.server_id);

        loop {
            println!("Server {} waiting for packet", self.server_id);
            select! {
                recv(self.receiver) -> packet => {
                    println!("Received packet {:?}", packet);
                    match packet {
                        Ok(packet) => {
                            // check if flood request
                            match packet.pack_type {
                                PacketType::FloodRequest(mut flood_request) => {
                                    println!("Received flood request from node {} with flood id flood_id: {}", flood_request.initiator_id, flood_request.flood_id);

                                    //create flood response
                                    flood_request.path_trace.push((self.server_id,NodeType::Server));
                                    let path_trace_reverse = flood_request.path_trace.into_iter().rev().collect::<Vec<_>>();

                                    let flood_response = FloodResponse {
                                        flood_id: flood_request.flood_id,
                                        path_trace: path_trace_reverse.clone(),
                                    };

                                    let hops: Vec<u8> = path_trace_reverse.iter().map(|(node_id, _)| *node_id).collect();

                                    // create response packet
                                    let response_packet = Packet {
                                        routing_header: SourceRoutingHeader {
                                            hop_index: 1,
                                            hops
                                        },
                                        session_id: packet.session_id,
                                        pack_type: PacketType::FloodResponse(flood_response),
                                    };

                                    println!("Packet response created {:?}", response_packet);

                                    // send flood response
                                    if let Some(sender) = self.senders.get(&response_packet.routing_header.hops[1]) {
                                        if let Err(err) = sender.send(response_packet.clone()) {
                                            eprintln!("Error in sending FloodResponse to Node {}: {}", &response_packet.routing_header.hops[1], err);
                                        } else {
                                            println!("Server sent a flood response to node {}", &response_packet.routing_header.hops[1]);
                                        }
                                    } else {
                                        eprintln!("Sender didn't found node {}", &response_packet.routing_header.hops[1]);
                                    }
                                }

                                PacketType::MsgFragment(_) => {
                                    println!("Received chat request");
                                }

                                _ => println!("Received packet type not handled")
                            }
                        }

                        Err(_) => {
                            println!("Error in receiving packet");
                        }
                    }
                }
            }
        }
    }

    pub fn run2(&mut self) {
        println!("Created server {:?}", self.server_id);

        loop {
            select! {
                recv(self.receiver) -> packet => {
                    match packet {
                        Ok(packet) => {
                            if let PacketType::MsgFragment(_) = packet.pack_type {
                            let content = format!("Messaggio dal server {}", self.server_id).into_bytes();
                            //response fragment
                            let fragment = Fragment {
                                fragment_index: 0,
                                total_n_fragments: 1,
                                length: FRAGMENT_DSIZE as u8,
                                data: {
                                    let mut buffer = [0u8; FRAGMENT_DSIZE];
                                    buffer[..content.len()].copy_from_slice(&content);
                                    buffer
                                },
                            };

                            let route: Vec<u8> = vec![self.server_id, 5, 2, 1, 7];


                            let packet_resp = Packet {
                                routing_header: SourceRoutingHeader {
                                    hop_index: 1,
                                    hops: route.clone(),
                                },
                                session_id: packet.session_id,
                                pack_type: PacketType::MsgFragment(fragment),
                            };

                            println!("Inverse route {:?}", packet_resp.routing_header.hops);

                            let next_hop=route[1];
                            println!("Next hop:{:?}", next_hop);
                            if let Some(sender) = self.senders.get(&next_hop) {
                                if let Err(err) = sender.send(packet_resp) {
                                    eprintln!("Error in sending fragment to Node {}: {}", next_hop, err);
                                } else {
                                    println!("Server sent a fragment to node {}", next_hop);
                                }
                            } else {
                                eprintln!("Server didn't found node {}", next_hop);
                            }
                        }
                    }
                        Err(_) => {

                        }
                    }
                }
            }
        }
    }
}
