use crossbeam::select;
use crossbeam_channel::{unbounded, Receiver, Sender};
use rand::Rng;
use std::collections::HashMap;
use std::{thread, time::Duration};
use wg_2024::network::{SourceRoutingHeader};
use wg_2024::packet::{FloodRequest, Fragment, NodeType, Packet, PacketType};



const FRAGMENT_DSIZE: usize = 128;
pub struct Client {
    client_id: u8, senders: HashMap<u8, Sender<Packet>>, receiver: Receiver<Packet>
}

impl Client {
    pub fn new(client_id: u8, senders: HashMap<u8, Sender<Packet>>, receiver: Receiver<Packet>) -> Self {
        Client {
            client_id, senders, receiver
        }
    }

   
    pub fn run(&mut self) {
        println!("Created client {:?}", self.client_id);

        let session_id = rand::random::<u64>(); 

        
            
            //create flood request
            let flood_request=FloodRequest{
                flood_id:1,
                initiator_id:self.client_id,
                path_trace:vec![(self.client_id,NodeType::Client)]
            };

            //create packet
            let packet=Packet{
                routing_header:SourceRoutingHeader { 
                    hop_index:0, 
                    hops: vec![],
                },
                session_id,
                pack_type:PacketType::FloodRequest(flood_request),
            };


            println!("Packet request created:{:?}", packet);
            
            let next_hop=1;
            //send flood request to node 1
            if let Some(sender) = self.senders.get(&next_hop) {
                if let Err(err) = sender.send(packet.clone()) {
                    eprintln!("Error in sending FloodRequest to Node {}: {}", next_hop, err);
                } else {
                    println!("Client {} sent a FloodRequest to node {}", self.client_id, next_hop);
                }
            } else {
                eprintln!("Sender not found for next hop {}", next_hop);
            }

            loop {
                select! {
                    recv(self.receiver) -> packet => {
                        match packet {
                            Ok(packet) => {
                                match packet.pack_type {
                                    PacketType::FloodResponse(flood_response) => {
                                        println!(
                                            "Client {} received FloodResponse: {:?}",
                                            self.client_id, flood_response
                                        );
                                    }
                                    _ => {
                                        println!("Client {} received an unsupported packet type", self.client_id);
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("Client {}: Error receiving packet: {:?}", self.client_id, err);
                            }
                        }
                    }
                }
            }
        

        
    }

    
    pub fn run2(&mut self) {
        println!("Created client {:?}", self.client_id);

        let session_id = rand::random::<u64>(); 
            let content = format!("Message from client {}", self.client_id).into_bytes();

            let fragment=Fragment{
                fragment_index:0,
                total_n_fragments:1,
                length:content.len() as u8,
                data: {
                    let mut buffer = [0u8; FRAGMENT_DSIZE];
                    buffer[..content.len()].copy_from_slice(&content); 
                    buffer
                },
            };

            let route: Vec<u8> = vec![self.client_id, 1, 2, 3, 5];
            let mut hop_index = 1; 
            let packet=Packet{
                routing_header:SourceRoutingHeader { 
                    hop_index, 
                    hops: route.clone() 
                },
                session_id,
                pack_type:PacketType::MsgFragment(fragment),
            };
            println!("Full route {:?}", route);
            println!("Length route {:?}", packet.routing_header.hops.len());
            println!("Hop index:{:?}",route[hop_index]);
            
            
            if let Some(sender) = self.senders.get(&route[hop_index]) {
                if let Err(err) = sender.send(packet.clone()) {
                    eprintln!("Error in sending FloodRequest to Node {}: {}", route[hop_index], err);
                } else {
                    println!("Client {} sent a fragment to node {}", self.client_id, route[hop_index]);
                }
            } else {
                eprintln!("Sender not found for next hop {}", route[hop_index]);
            }

            loop {
                select! {
                    recv(self.receiver) -> packet => {
                        match packet {
                            Ok(packet) => {
                                match packet.pack_type {
                                    PacketType::MsgFragment(fragment) => {
                                        let message = String::from_utf8_lossy(&fragment.data[..fragment.length as usize]);
                                        println!(
                                            "Client {} received message from session {}: {}",
                                            self.client_id, packet.session_id, message
                                        );
                                    }
                                    PacketType::FloodResponse(flood_response) => {
                                        println!(
                                            "Client {} received FloodResponse: {:?}",
                                            self.client_id, flood_response
                                        );
                                    }
                                    _ => {
                                        println!("Client {} received an unsupported packet type", self.client_id);
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("Client {}: Error receiving packet: {:?}", self.client_id, err);
                            }
                        }
                    }
                }
        
            }
        
    }
    
}
