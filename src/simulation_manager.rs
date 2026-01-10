//! Simulation manager - central manager for the simulation
//!
//! The SimulationManager:
//! - Creates devices (by prefab hash + settings)
//! - Creates items (by prefab hash + settings)
//! - Creates atmospheric and cable networks
//! - Runs simulation ticks in deterministic order
//!
//! Update order implemented here:
//! 1. Process atmospheric network updates
//! 2. Update all cable networks (which run device updates and IC runners)

use crate::LogicSlotType;
use crate::LogicType;
use crate::conversions::fmt_trim;
use crate::devices::device_factory;
use crate::devices::{Device, SimulationDeviceSettings};
use crate::items::item_factory;
use crate::items::{self, Item, SimulationItemSettings};
use crate::networks::{AtmosphericNetwork, CableNetwork};
use crate::types::Shared;
use std::collections::{BTreeMap, HashSet};
use std::fmt::Display;

/// Central manager for running the simulation
#[derive(Default, Clone, Debug)]
pub struct SimulationManager {
    cable_networks: Vec<Shared<CableNetwork>>,
    atmospheric_networks: Vec<Shared<AtmosphericNetwork>>,
    devices: BTreeMap<i32, Shared<dyn Device>>,
    items: BTreeMap<i32, Shared<dyn Item>>,
    next_id: i32,
    allocated_ids: HashSet<i32>,
}

impl SimulationManager {
    /// Create a new `SimulationManager` with default settings
    pub fn new() -> Self {
        Self {
            next_id: 1,
            allocated_ids: HashSet::new(),
            ..Default::default()
        }
    }

    /// Allocate the next available ID
    pub fn allocate_next_id(&mut self) -> i32 {
        let id = self.next_id;
        self.allocated_ids.insert(id);
        self.next_id += 1;
        id
    }

    /// Reserve a specific ID; returns true if successful
    pub fn reserve_id(&mut self, id: i32) -> bool {
        if self.allocated_ids.contains(&id) {
            false
        } else {
            self.allocated_ids.insert(id);
            if id >= self.next_id {
                self.next_id = id + 1;
            }
            true
        }
    }

    /// Return a slice of all devices created by this manager
    pub fn all_devices(&self) -> Vec<Shared<dyn Device>> {
        self.devices.values().cloned().collect()
    }

    /// Return a slice of all items created by this manager
    pub fn all_items(&self) -> Vec<Shared<dyn Item>> {
        self.items.values().cloned().collect()
    }

    /// Register a cable network to be updated each tick
    pub fn register_cable_network(&mut self, network: Shared<CableNetwork>) {
        self.cable_networks.push(network);
    }

    /// Register an atmospheric network to be processed each tick
    pub fn register_atmospheric_network(&mut self, network: Shared<AtmosphericNetwork>) {
        self.atmospheric_networks.push(network);
    }

    /// Perform a simulation tick in the correct order and return the total number of phase changes.
    pub fn update(&self, tick: u64) -> u32 {
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
    pub fn reset(&mut self) {
        // Remove all devices from cable networks
        for net in &self.cable_networks {
            let mut net_mut = net.borrow_mut();
            let ids = net_mut.all_device_ids();
            for id in ids {
                net_mut.remove_device(id).unwrap();
            }
        }

        self.cable_networks.clear();
        self.atmospheric_networks.clear();

        // Clear tracked devices/items
        self.devices.clear();
        self.items.clear();
        self.next_id = 1;
        self.allocated_ids.clear();
    }

    /// Create a new device by prefab hash using the device factory and track it.
    pub fn create_device(
        &mut self,
        prefab_hash: i32,
        settings: Option<SimulationDeviceSettings>,
    ) -> Option<Shared<dyn Device>> {
        // Prepare settings and ensure an ID is set; when not provided (0) use the manager's counter
        let mut settings = settings.unwrap_or_default();

        let id = if let Some(id) = settings.id {
            if !self.reserve_id(id) {
                return None;
            }
            id
        } else {
            self.allocate_next_id()
        };

        settings.id = Some(id);

        if let Some(d) = device_factory::create_device(prefab_hash, settings) {
            // Track the created device
            self.devices.insert(d.borrow().get_id(), d.clone());
            Some(d)
        } else {
            // Creation failed, free reserved id
            self.allocated_ids.remove(&id);
            None
        }
    }

    /// Create a new item by prefab hash using the item factory and track it.
    pub fn create_item(
        &mut self,
        prefab_hash: i32,
        settings: Option<SimulationItemSettings>,
    ) -> Option<Shared<dyn Item>> {
        let mut settings = settings.unwrap_or_default();

        let id = if let Some(id) = settings.id {
            if !self.reserve_id(id) {
                return None;
            }
            id
        } else {
            self.allocate_next_id()
        };

        settings.id = Some(id);

        if let Some(it) = item_factory::create_item(prefab_hash, settings) {
            self.items.insert(it.borrow().get_id(), it.clone());
            Some(it)
        } else {
            None
        }
    }

    /// Get a device tracked by this manager by reference ID
    pub fn get_device(&self, ref_id: i32) -> Option<Shared<dyn Device>> {
        self.devices.get(&ref_id).cloned()
    }

    /// Get an item tracked by this manager by reference ID
    pub fn get_item(&self, ref_id: i32) -> Option<Shared<dyn Item>> {
        self.items.get(&ref_id).cloned()
    }

    /// Get all cable networks registered with this manager
    pub fn all_cable_networks(&self) -> Vec<Shared<CableNetwork>> {
        self.cable_networks.clone()
    }

    /// Get a cable network registered with this manager by index
    pub fn get_cable_network(&self, idx: usize) -> Option<Shared<CableNetwork>> {
        self.cable_networks.get(idx).cloned()
    }

    /// Get an atmospheric network registered with this manager by index
    pub fn get_atmospheric_network(&self, idx: usize) -> Option<Shared<AtmosphericNetwork>> {
        self.atmospheric_networks.get(idx).cloned()
    }

    /// Get all atmospheric networks registered with this manager
    pub fn all_atmospheric_networks(&self) -> Vec<Shared<AtmosphericNetwork>> {
        self.atmospheric_networks.clone()
    }

    /// Create a new cable network and register it with this manager.
    pub fn create_cable_network(&mut self) -> Shared<CableNetwork> {
        let network = CableNetwork::new();
        self.register_cable_network(network.clone());
        network
    }

    /// Create a new atmospheric network and register it with this manager.
    pub fn create_atmospheric_network(&mut self, volume: f64) -> Shared<AtmosphericNetwork> {
        let network = AtmosphericNetwork::new(volume);
        self.register_atmospheric_network(network.clone());
        network
    }
}

impl Display for SimulationManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "SimulationManager {{")?;

        writeln!(f, "  Cable Networks ({}):", self.cable_networks.len())?;
        for (i, n) in self.cable_networks.iter().enumerate() {
            let net = n.borrow();
            let ids = net.all_device_ids();
            writeln!(f, "    Network #{}: {} device(s)", i, ids.len())?;

            for id in ids {
                if let Some(device_ref) = net.get_device(id) {
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

                    // Enumerate items if device supports slots
                    if let Some(slot_host) = device_ref.as_slot_host_device() {
                        let count = slot_host.slot_count();
                        let mut items = Vec::new();

                        for slot_idx in 0..count {
                            // Try to read occupant prefab hash and quantity via slot properties
                            if let Ok(occupant_hash) =
                                device_ref.read_slot(slot_idx, LogicSlotType::OccupantHash)
                            {
                                let occupant_hash_i = occupant_hash as i32;
                                if occupant_hash_i != 0 {
                                    let name = items::get_prefab_metadata(occupant_hash_i)
                                        .map(|(n, _)| n)
                                        .unwrap_or("Unknown");
                                    let qty = device_ref
                                        .read_slot(slot_idx, LogicSlotType::Quantity)
                                        .unwrap_or(0.0)
                                        as u32;
                                    items.push(format!(
                                        "slot {}: {} ({} x{})",
                                        slot_idx, name, occupant_hash_i, qty
                                    ));
                                }
                            }
                        }

                        if !items.is_empty() {
                            writeln!(f, "        Items: {}", items.join(", "))?;
                        }
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
