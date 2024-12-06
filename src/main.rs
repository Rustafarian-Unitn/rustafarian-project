#![allow(unused)]

use crate::lib::simulation_controller::SimulationController;

mod lib;

fn main() {
    let sim_controller = SimulationController::new(Some("./config.toml"));
    sim_controller.crash_all();

    while sim_controller.is_alive() {
        
    }
}

