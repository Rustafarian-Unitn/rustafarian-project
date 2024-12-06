#![allow(unused)]
use crossbeam_channel::{select_biased, unbounded, Receiver, Sender};
use std::collections::{HashMap, HashSet};
use std::{fs, thread};
use wg_2024::config::Config;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::{self, FloodRequest, FloodResponse, Fragment, NodeType};
use wg_2024::packet::{Packet, PacketType};
use rand::*;

use crate::assembler::Assembler;
use crate::deassembler::Deassembler;


pub struct Server{
    deassembler:Deassembler,
    assembler:Assembler,
    resources:HashMap<String, Vec<u8>>,
}

impl Server {
    pub fn new()->Self {
        Server{
            deassembler:Deassembler::new(),
            assembler:Assembler::new(),
            resources:HashMap::new(),
        }
    }


    pub fn process_message(&mut self, packet:Packet)->Vec<Fragment> {

        match packet.pack_type {
            PacketType::MsgFragment(fragment)=>{
                if let Some(message) =  self.assembler.add_fragment(fragment, packet.session_id){
                    let resource=self.obtain_resources(message);
                    let fragments=self.deassembler.add_message(resource,  packet.session_id);
                    //inzia frammenti
                    return fragments;
                }
                else {
                    //per ora lascia così
                    return Vec::<Fragment>::new();
                }
            }
            _=>{
                //per ora lascia così
                return Vec::<Fragment>::new();
            }
        }
    }

    pub fn obtain_resources(&self, request:Vec<u8>)->Vec<u8> {

        //simula di prendere risposta e vedere se esiste
        let response_str = format!("Risposta ottenuta per: {}", String::from_utf8_lossy(&request));
        return response_str.into_bytes();
    }

    pub fn send_message(&self, fragments:Vec<Fragment>,  senders: HashMap<u8, Sender<Packet>>, packet:Packet) {
        for fragment in fragments{

            let reverse_route:Vec<u8>=packet.routing_header.hops.iter().rev().cloned().collect();
            let next_hop=reverse_route[1];
            
            let packet=Packet{
                pack_type:PacketType::MsgFragment(fragment),
                routing_header:SourceRoutingHeader{
                    hop_index:1,
                    hops: reverse_route
                },
                session_id:packet.session_id,
            };


            if let Some(sender)=senders.get(&next_hop){
                sender.send(packet).unwrap();
            }


            
        }
    }



    pub fn run(server_id:u8, receiver:Receiver<Packet>, senders:HashMap<u8, Sender<Packet>>) {
        let mut server=Server::new();

        loop {
            match receiver.recv() {
                Ok(packet)=>{

                    let response_fragments=server.process_message(packet.clone());
                    server.send_message(response_fragments, senders.clone(), packet);
                }
                Err(e)=>{
                    println!("Errore generico")
                }


            }
        }
    }




}