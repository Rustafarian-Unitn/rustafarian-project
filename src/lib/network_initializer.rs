use std::collections::HashMap;
use std::{fs, thread};
use std::thread::JoinHandle;
use crossbeam_channel::{unbounded, Sender, Receiver};
use rustafarian_drone::RustafarianDrone;
use wg_2024::config::{Config};
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::NodeId;
use wg_2024::packet::Packet;
use crate::lib::server::Server;
use crate::lib::client::Client;

// TODO Add description
pub struct NetworkInitializer;

impl NetworkInitializer {

    /// Initialize the whole network
    ///
    /// # Arguments
    ///
    /// * `config_file: Option<&str>` - Path to the config file, file must be a `.toml` file and
    /// must respect the structure in the protocol.
    /// If None is passed, then fall back to default one (./config.toml)
    pub fn init(config_file: Option<&str>) -> (
        HashMap<NodeId, Sender<DroneCommand>>,
        Receiver<DroneEvent>,
        Vec<JoinHandle<()>>
    ) {

        // Fetch config file
        let config: Config;
        if config_file.is_some() { config = Self::parse_config(config_file.unwrap()); }
        else { config = Self::parse_config("./config2.toml"); }

        // Create channels for every node in the network
        let mut packet_channels = HashMap::new();
        for drone in config.drone.iter() {
            packet_channels.insert(drone.id, unbounded());
        }
        for client in config.client.iter() {
            packet_channels.insert(client.id, unbounded());
        }
        for server in config.server.iter() {
            packet_channels.insert(server.id, unbounded());
        }

        // Map containing every NodeId in the network, and the corresponding Sender channel,
        // Channel owned by drones, communication Controller -> Drone
        let mut controller_drones = HashMap::new();

        // Communication Drone -> Controller
        // drone_event_send: Sender<DroneEvent> Used by drones to send DroneEvent to controller
        // drone_event_recv: Receiver<DroneEvent> Used by controller to receive DroneEvent
        let (drone_event_send, drone_event_recv) = unbounded();


        // Run every node in a dedicated thread
        let mut handles = Vec::new();

        //let drone_implementations=distribute_implementations(config.drone.len());

        //println!("{:?}", drone_implementations);
        for drone in config.drone.into_iter() {

            // COMMANDS
            // Communication Controller -> Drone
            let (
                drone_command_send,
                drone_command_recv
            ) = unbounded();

            controller_drones.insert(drone.id, drone_command_send);
            let drone_event_send = drone_event_send.clone();

            // PACKETS
            let packet_recv = packet_channels[&drone.id].1.clone();
            let packet_send = drone
                .connected_node_ids
                .into_iter()
                .map(|id| (id, packet_channels[&id].0.clone()))
                .collect();

            handles.push(thread::spawn(move || {

                let mut new_drone = RustafarianDrone::new(
                    drone.id,
                    drone_event_send,
                    drone_command_recv,
                    packet_recv,
                    packet_send,
                    drone.pdr
                );
                new_drone.run();
                
            }));
        };
        

        
        for server in config.server.into_iter() {
            let packet_recv = packet_channels[&server.id].1.clone();
            let packet_send = server
                .connected_drone_ids
                .iter()
                .map(|&id| (id, packet_channels[&id].0.clone()))
                .collect();

            handles.push(thread::spawn(move || {
                let mut new_server=Server::new(server.id, packet_recv, packet_send);
                
               new_server.run2();
            }));
        };

        

        for client in config.client.into_iter() {
            let packet_recv = packet_channels[&client.id].1.clone();
            let packet_send = client
                .connected_drone_ids
                .iter()
                .map(|&id| (id, packet_channels[&id].0.clone()))
                .collect();

        
            handles.push(thread::spawn(move || {
                let mut new_client=Client::new(client.id, packet_send, packet_recv);
        
                
                new_client.run2();
            }));
        }

        

        (controller_drones, drone_event_recv, handles)
    }


    fn parse_config(file: &str) -> Config {
        let file_str = fs::read_to_string(file).unwrap();
        toml::from_str(&file_str).unwrap()
    }
}








