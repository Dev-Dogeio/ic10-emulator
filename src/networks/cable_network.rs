use crate::devices::{Device, LogicType};
use crate::error::SimulationResult;
use crate::types::{OptShared, Shared, shared};
use crate::{SimulationError, SimulationManager};
use std::cell::{Ref, RefMut};
use std::collections::BTreeMap;
use std::rc::Rc;

/// A cable network that connects multiple devices together.
///
/// The cable network manages all devices that are connected via data cables,
/// allowing the IC10 chip to access devices by:
/// - Reference ID (unique identifier for each device)
/// - Prefab hash (type identifier for batch operations like `lb`, `sb`)
/// - Name hash (for operations like `lbn`, `sbn`)
#[derive(Clone)]
pub struct CableNetwork {
    /// All devices on the network, keyed by their reference ID
    devices: BTreeMap<i32, Shared<dyn Device>>,

    /// Index for quick lookup by prefab hash
    /// Maps prefab_hash -> list of device reference IDs
    prefab_index: BTreeMap<i32, Vec<i32>>,

    /// Index for quick lookup by name hash
    /// Maps name_hash -> list of device reference IDs
    name_index: BTreeMap<i32, Vec<i32>>,
}

impl std::fmt::Display for CableNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CableNetwork {{ devices: {} }}", self.devices.len())
    }
}

impl std::fmt::Debug for CableNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl CableNetwork {
    /// Create a new cable network and register it with the global SimulationManager.
    pub fn new() -> Shared<CableNetwork> {
        let s = shared(CableNetwork {
            devices: BTreeMap::new(),
            prefab_index: BTreeMap::new(),
            name_index: BTreeMap::new(),
        });

        SimulationManager::register_cable_network_global(s.clone());

        s
    }

    /// Add a device to the network and set up the bidirectional connection
    /// The device will be indexed by its reference ID, prefab hash, and name hash
    /// The devices list will remain sorted by reference ID
    pub fn add_device(&mut self, device: Shared<dyn Device>, network_rc: Shared<CableNetwork>) {
        // Set the device's network reference
        device.borrow_mut().set_network(Some(network_rc));

        let borrowed = device.borrow();
        let ref_id = borrowed.get_id();
        let prefab_hash = borrowed.get_prefab_hash();
        let name_hash = borrowed.get_name_hash();
        drop(borrowed);

        // Add to main device map
        self.devices.insert(ref_id, Rc::clone(&device));

        // Add to prefab index and insert in sorted order
        let prefab_ids = self.prefab_index.entry(prefab_hash).or_default();
        match prefab_ids.binary_search(&ref_id) {
            Ok(_) => {}
            Err(pos) => prefab_ids.insert(pos, ref_id),
        }

        // Add to name index and insert in sorted order
        let name_ids = self.name_index.entry(name_hash).or_default();
        match name_ids.binary_search(&ref_id) {
            Ok(_) => {}
            Err(pos) => name_ids.insert(pos, ref_id),
        }
    }

    /// Remove a device from the network by its reference ID
    pub fn remove_device(&mut self, ref_id: i32) -> OptShared<dyn Device> {
        if let Some(device) = self.devices.remove(&ref_id) {
            let borrowed = device.borrow();
            let prefab_hash = borrowed.get_prefab_hash();
            let name_hash = borrowed.get_name_hash();
            drop(borrowed);

            // Notify the device that it is no longer part of the network
            device.borrow_mut().set_network(None);

            // Remove from prefab index
            if let Some(ids) = self.prefab_index.get_mut(&prefab_hash) {
                ids.retain(|&id| id != ref_id);
                if ids.is_empty() {
                    self.prefab_index.remove(&prefab_hash);
                }
            }

            // Remove from name index
            if let Some(ids) = self.name_index.get_mut(&name_hash) {
                ids.retain(|&id| id != ref_id);
                if ids.is_empty() {
                    self.name_index.remove(&name_hash);
                }
            }

            Some(device)
        } else {
            None
        }
    }

    /// Update the device name index when a device's name changes
    pub fn update_device_name(&mut self, ref_id: i32, old_name_hash: i32, new_name_hash: i32) {
        // Remove from old name index
        if let Some(ids) = self.name_index.get_mut(&old_name_hash) {
            ids.retain(|&id| id != ref_id);
            if ids.is_empty() {
                self.name_index.remove(&old_name_hash);
            }
        }

        // Add to new name index and insert in sorted order
        let ids = self.name_index.entry(new_name_hash).or_default();
        match ids.binary_search(&ref_id) {
            Ok(_) => {}
            Err(pos) => ids.insert(pos, ref_id),
        }
    }

    /// Check if a device with the given reference ID exists on the network
    pub fn device_exists(&self, ref_id: i32) -> bool {
        self.devices.contains_key(&ref_id)
    }

    /// Get a device by its reference ID (immutable borrow)
    pub fn get_device(&self, ref_id: i32) -> Option<Ref<'_, dyn Device>> {
        let device = self.devices.get(&ref_id)?;
        Some(device.borrow())
    }

    /// Get a device by its reference ID (mutable borrow)
    pub fn get_device_mut(&self, ref_id: i32) -> Option<RefMut<'_, dyn Device>> {
        let device = self.devices.get(&ref_id)?;
        Some(device.borrow_mut())
    }

    /// Get all devices with a specific prefab hash
    /// Returns a vector of reference IDs
    pub fn get_devices_by_prefab(&self, prefab_hash: i32) -> Vec<i32> {
        self.prefab_index
            .get(&prefab_hash)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all devices with a specific name hash
    /// Returns a vector of reference IDs
    pub fn get_devices_by_name(&self, name_hash: i32) -> Vec<i32> {
        self.name_index.get(&name_hash).cloned().unwrap_or_default()
    }

    /// Count devices with a specific prefab hash
    pub fn count_devices_by_prefab(&self, prefab_hash: i32) -> usize {
        self.prefab_index
            .get(&prefab_hash)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Count devices with a specific name hash
    pub fn count_devices_by_name(&self, name_hash: i32) -> usize {
        self.name_index
            .get(&name_hash)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Get total number of devices on the network
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Get all device reference IDs on the network
    pub fn all_device_ids(&self) -> Vec<i32> {
        self.devices.keys().cloned().collect()
    }

    /// Clear all devices from the network
    pub fn clear(&mut self) {
        self.devices.clear();
        self.prefab_index.clear();
        self.name_index.clear();
    }

    /// Update all devices in the network
    /// Devices are updated in ascending order of their reference IDs
    /// After updating all devices, IC runners are executed in the same order
    pub fn update(&self, tick: u64) {
        // Iterate over all devices in ascending order and run update
        for device in self.devices.values() {
            device.borrow().update(tick).unwrap();
        }

        // Iterate over all devices again and execute IC runners
        for device in self.devices.values() {
            device.borrow().run().unwrap();
        }
    }

    // ==================== Batch Read Operations ====================

    /// Read a logic value from all devices matching a prefab hash
    /// Used for `lb` instruction
    pub fn batch_read_by_prefab(
        &self,
        prefab_hash: i32,
        logic_type: LogicType,
        batch_mode: BatchMode,
    ) -> SimulationResult<f64> {
        let device_ids = self.get_devices_by_prefab(prefab_hash);
        self.batch_read_from_ids(&device_ids, logic_type, batch_mode)
    }

    /// Read a logic value from all devices matching a name hash
    /// Used for `lbn` instruction
    pub fn batch_read_by_name(
        &self,
        prefab_hash: i32,
        name_hash: i32,
        logic_type: LogicType,
        batch_mode: BatchMode,
    ) -> SimulationResult<f64> {
        // Get devices that match both prefab and name hash
        let prefab_devices = self.get_devices_by_prefab(prefab_hash);
        let name_devices = self.get_devices_by_name(name_hash);

        // Intersection of both sets
        let device_ids: Vec<i32> = prefab_devices
            .into_iter()
            .filter(|id| name_devices.contains(id))
            .collect();

        self.batch_read_from_ids(&device_ids, logic_type, batch_mode)
    }

    /// Internal helper to perform batch read from a list of device IDs
    fn batch_read_from_ids(
        &self,
        device_ids: &[i32],
        logic_type: LogicType,
        batch_mode: BatchMode,
    ) -> SimulationResult<f64> {
        if device_ids.is_empty() {
            // Return 0 for empty batch
            return Ok(0.0);
        }

        let mut values = Vec::with_capacity(device_ids.len());

        for &ref_id in device_ids {
            let device = self
                .get_device(ref_id)
                .ok_or_else(|| SimulationError::RuntimeError {
                    message: format!(
                        "Device with reference ID {} not found for batch read",
                        ref_id
                    ),
                    line: 0,
                })?;

            values.push(device.read(logic_type)?);
        }

        Ok(batch_mode.aggregate(&values))
    }

    // ==================== Batch Write Operations ====================

    /// Write a logic value to all devices matching a prefab hash
    /// Used for `sb` instruction
    pub fn batch_write_by_prefab(
        &self,
        prefab_hash: i32,
        logic_type: LogicType,
        value: f64,
    ) -> SimulationResult<usize> {
        let device_ids = self.get_devices_by_prefab(prefab_hash);
        self.batch_write_to_ids(&device_ids, logic_type, value)
    }

    /// Write a logic value to all devices matching a name hash
    /// Used for `sbn` instruction
    pub fn batch_write_by_name(
        &self,
        prefab_hash: i32,
        name_hash: i32,
        logic_type: LogicType,
        value: f64,
    ) -> SimulationResult<usize> {
        // Get devices that match both prefab and name hash
        let prefab_devices = self.get_devices_by_prefab(prefab_hash);
        let name_devices = self.get_devices_by_name(name_hash);

        // Intersection of both sets
        let device_ids: Vec<i32> = prefab_devices
            .into_iter()
            .filter(|id| name_devices.contains(id))
            .collect();

        self.batch_write_to_ids(&device_ids, logic_type, value)
    }

    /// Internal helper to perform batch write to a list of device IDs
    fn batch_write_to_ids(
        &self,
        device_ids: &[i32],
        logic_type: LogicType,
        value: f64,
    ) -> SimulationResult<usize> {
        let mut write_count = 0;

        for &ref_id in device_ids {
            let device = self
                .get_device(ref_id)
                .ok_or_else(|| SimulationError::RuntimeError {
                    message: format!(
                        "Device with reference ID {} not found for batch write",
                        ref_id
                    ),
                    line: 0,
                })?;

            device.write(logic_type, value)?;
            write_count += 1;
        }

        Ok(write_count)
    }
}

/// Batch mode for aggregating values from multiple devices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchMode {
    /// Average of all values
    Average,
    /// Sum of all values
    Sum,
    /// Minimum value
    Minimum,
    /// Maximum value
    Maximum,
}

impl BatchMode {
    /// Parse batch mode from a numeric value (as used in IC10 instructions)
    pub fn from_value(value: f64) -> Option<Self> {
        match value as i32 {
            0 => Some(BatchMode::Average),
            1 => Some(BatchMode::Sum),
            2 => Some(BatchMode::Minimum),
            3 => Some(BatchMode::Maximum),
            _ => None,
        }
    }

    /// Convert batch mode to its numeric value
    pub fn to_value(self) -> f64 {
        match self {
            BatchMode::Average => 0.0,
            BatchMode::Sum => 1.0,
            BatchMode::Minimum => 2.0,
            BatchMode::Maximum => 3.0,
        }
    }

    /// Parse BatchMode from a string name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Average" => Some(BatchMode::Average),
            "Sum" => Some(BatchMode::Sum),
            "Minimum" => Some(BatchMode::Minimum),
            "Maximum" => Some(BatchMode::Maximum),
            _ => None,
        }
    }

    /// Aggregate a list of values according to the batch mode
    pub fn aggregate(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        match self {
            BatchMode::Average => values.iter().sum::<f64>() / values.len() as f64,
            BatchMode::Sum => values.iter().sum(),
            BatchMode::Minimum => values.iter().cloned().fold(f64::INFINITY, f64::min),
            BatchMode::Maximum => values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        }
    }
}
