//! Simulation manager - central manager for cable and atmospheric networks
//!
//! The SimulationManager holds references to all registered cable and
//! atmospheric networks and runs them in a deterministic order each tick.
//!
//! Update order implemented here:
//! 1. Process atmospheric network updates
//! 2. Update all cable networks (which run device updates and IC runners)

use crate::networks::{AtmosphericNetwork, CableNetwork};
use crate::types::Shared;
use std::cell::RefCell;

thread_local! {
    static GLOBAL_SIM_MANAGER: RefCell<SimulationManager> = RefCell::new(SimulationManager::new());
}

/// Central manager for running the simulation
#[derive(Debug, Default, Clone)]
pub struct SimulationManager {
    cable_networks: Vec<Shared<CableNetwork>>,
    atmospheric_networks: Vec<Shared<AtmosphericNetwork>>,
}

impl SimulationManager {
    /// Create a new SimulationManager
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an atmospheric network on the global manager
    pub fn register_atmospheric_network_global(network: Shared<AtmosphericNetwork>) {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow_mut().register_atmospheric_network(network));
    }

    /// Register a cable network on the global manager
    pub fn register_cable_network_global(network: Shared<CableNetwork>) {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow_mut().register_cable_network(network));
    }

    /// Run a tick on the global manager and return the total number of phase changes that occurred.
    pub fn update_global(tick: u64) -> u32 {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow().update(tick))
    }

    /// Reset the global simulation manager: remove/cleanup all devices and atmospheric networks and reset global IDs.
    pub fn reset_global() {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow_mut().reset());
        // Also reset the global ID counter
        crate::id::reset_global_id_counter();
    }

    /// Get the count of registered cable networks on the global manager
    pub fn cable_network_count_global() -> usize {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow().cable_networks.len())
    }

    /// Get the count of registered atmospheric networks on the global manager
    pub fn atmospheric_network_count_global() -> usize {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow().atmospheric_networks.len())
    }

    /// Register a cable network to be updated each tick
    fn register_cable_network(&mut self, network: Shared<CableNetwork>) {
        self.cable_networks.push(network);
    }

    /// Register an atmospheric network to be processed each tick
    fn register_atmospheric_network(&mut self, network: Shared<AtmosphericNetwork>) {
        self.atmospheric_networks.push(network);
    }

    /// Perform a simulation tick in the correct order and return the total number of phase changes.
    fn update(&self, tick: u64) -> u32 {
        // 1) Process atmospheric updates
        let mut total_changes: u32 = 0;
        for net in &self.atmospheric_networks {
            total_changes += net.borrow_mut().process_phase_changes();
        }

        // 2) Update all cable networks (which run device updates and IC runners)
        for net in &self.cable_networks {
            net.borrow().update(tick);
        }

        total_changes
    }

    /// Reset internal manager state by removing devices and clearing networks.
    fn reset(&mut self) {
        // Remove all devices from cable networks
        for net in &self.cable_networks {
            let ids = net.borrow().all_device_ids();
            for id in ids {
                {
                    let net_mut = net.borrow_mut();
                    let mut dev_mut = net_mut
                        .get_device_mut(id)
                        .expect("Expected device to be present during reset");
                    dev_mut.clear_internal_references();
                }

                let _ = net.borrow_mut().remove_device(id);
            }
        }

        self.cable_networks.clear();
        self.atmospheric_networks.clear();
    }
}
