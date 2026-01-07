//! Simulation manager - central manager for cable and atmospheric networks
//!
//! The SimulationManager holds references to all registered cable and
//! atmospheric networks and runs them in a deterministic order each tick.
//!
//! Update order implemented here:
//! 1. Process atmospheric network updates
//! 2. Update all cable networks (which run device updates and IC runners)

use crate::LogicType;
use crate::conversions::fmt_trim;
use crate::id::reset_global_id_counter;
use crate::networks::{AtmosphericNetwork, CableNetwork};
use crate::types::{Shared, shared};
use std::fmt::Display;

thread_local! {
    static GLOBAL_SIM_MANAGER: Shared<SimulationManager> = shared(SimulationManager::new());
}

/// Central manager for running the simulation
#[derive(Default, Clone, Debug)]
pub struct SimulationManager {
    cable_networks: Vec<Shared<CableNetwork>>,
    atmospheric_networks: Vec<Shared<AtmosphericNetwork>>,
}

impl SimulationManager {
    /// Create a new `SimulationManager` with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the global `SimulationManager` instance
    pub fn global() -> Shared<SimulationManager> {
        GLOBAL_SIM_MANAGER.with(|g| g.clone())
    }

    /// Register an atmospheric network globally
    pub fn register_atmospheric_network_global(network: Shared<AtmosphericNetwork>) {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow_mut().register_atmospheric_network(network));
    }

    /// Register a cable network globally
    pub fn register_cable_network_global(network: Shared<CableNetwork>) {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow_mut().register_cable_network(network));
    }

    /// Run a global tick and return number of atmospheric phase changes.
    pub fn update_global(tick: u64) -> u32 {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow().update(tick))
    }

    /// Reset the global simulation manager and global IDs (for tests).
    pub fn reset_global() {
        GLOBAL_SIM_MANAGER.with(|g| g.borrow_mut().reset());
        // Also reset the global ID counter
        reset_global_id_counter();
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
                net.borrow_mut().remove_device(id).unwrap();
            }
        }

        self.cable_networks.clear();
        self.atmospheric_networks.clear();
    }
}

impl Display for SimulationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SimulationManager {{")?;

        writeln!(f, "  Cable Networks ({}):", self.cable_networks.len())?;
        for (i, net) in self.cable_networks.iter().enumerate() {
            let ids = net.borrow().all_device_ids();
            writeln!(f, "    Network #{}: {} device(s)", i, ids.len())?;

            for id in ids {
                if let Some(device_ref) = net.borrow().get_device(id) {
                    let name = device_ref.get_name();
                    let prefab = device_ref.get_prefab_hash();

                    writeln!(f, "      Device #{}: \"{}\" (prefab: {})", id, name, prefab)?;

                    let mut values = Vec::new();

                    if device_ref.can_read(LogicType::On)
                        && let Ok(val) = device_ref.read(LogicType::On)
                    {
                        let state = if val == 0.0 { "Off" } else { "On" };
                        values.push(format!("On: {}", state));
                    }

                    if device_ref.can_read(LogicType::Mode)
                        && let Ok(val) = device_ref.read(LogicType::Mode)
                    {
                        let state = if val == 0.0 { "Off" } else { "On" };
                        values.push(format!("Mode: {}", state));
                    }

                    if device_ref.can_read(LogicType::Setting)
                        && let Ok(val) = device_ref.read(LogicType::Setting)
                    {
                        values.push(format!("Setting: {}", fmt_trim(val, 3)));
                    }

                    if device_ref.can_read(LogicType::Horizontal)
                        && let Ok(val) = device_ref.read(LogicType::Horizontal)
                    {
                        values.push(format!("Horizontal: {}°", fmt_trim(val, 2)));
                    }

                    if device_ref.can_read(LogicType::Vertical)
                        && let Ok(val) = device_ref.read(LogicType::Vertical)
                    {
                        values.push(format!("Vertical: {}°", fmt_trim(val, 2)));
                    }

                    if device_ref.can_read(LogicType::Ratio)
                        && let Ok(val) = device_ref.read(LogicType::Ratio)
                    {
                        values.push(format!("Ratio: {}", fmt_trim(val, 3)));
                    }

                    if !values.is_empty() {
                        writeln!(f, "        State: {}", values.join(", "))?;
                    }
                }
            }
        }

        writeln!(
            f,
            "  Atmospheric Networks ({}):",
            self.atmospheric_networks.len()
        )?;
        for (i, net) in self.atmospheric_networks.iter().enumerate() {
            let borrowed = net.borrow();
            let mixture = borrowed.mixture();
            writeln!(
                f,
                "    Network #{}: {} L, {} K, {} kPa, {} mol",
                i,
                fmt_trim(mixture.volume(), 3),
                fmt_trim(mixture.temperature(), 2),
                fmt_trim(mixture.pressure(), 3),
                fmt_trim(mixture.total_moles(), 3)
            )?;
        }

        write!(f, "}}")
    }
}
