use std::collections::HashMap;
use std::thread::JoinHandle;
use crossbeam_channel::{Receiver, Sender};
use crossbeam_channel::internal::SelectHandle;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::network::NodeId;
use crate::lib::network_initializer::NetworkInitializer;

// TODO Add description
pub struct SimulationController {

    // List of all drones' channels, used by controller to send DroneCommand
    // Communication Controller -> Drone
    drones: HashMap<NodeId, Sender<DroneCommand>>,

    // Channel where controller listens for Drones Event
    // Communication Drone -> Controller
    drone_event_recv: Receiver<DroneEvent>,

    // Vector with handles for the node threads
    pub(crate) handles: Vec<JoinHandle<()>>
}

impl SimulationController {

    /// Create a new Simulation Controller
    ///
    /// # Arguments
    ///
    /// * `drones: HashMap<NodeId, Sender<DroneCommand>>` -  List of all drones' channels,
    /// used by controller to send DroneCommand
    /// * `drone_event_recv: Receiver<DroneEvent>` - Channel where controller listens for Drones Event
    ///
    /// # Returns
    /// A new Simulation Controller
    pub fn new(config_file: Option<&str>) -> Self {

        let (
            drones,
            drone_event_recv,
            handles
        ) = NetworkInitializer::init(config_file);

        Self{ drones, drone_event_recv, handles }
    }

    pub fn crash_all(&self) {
        self.drones.iter().for_each(|(id, sender)| {
            println!("Sending crash to drone {}", id);
            sender.send(DroneCommand::Crash).unwrap();
        })
    }

    pub fn show_drones(&self) {

        self.drones.iter().for_each(|(id, sender)| {
            println!("DRONE: {} - Sender: {:?}", id, sender)
        })
    }

    pub fn is_alive(&self) -> bool {
        !self.handles.is_empty()
    }
}