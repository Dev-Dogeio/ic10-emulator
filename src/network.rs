use crate::devices::{Device, LogicType};
use crate::error::IC10Result;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

/// A cable network that connects multiple devices together.
///
/// The cable network manages all devices that are connected via data cables,
/// allowing the IC10 chip to access devices by:
/// - Reference ID (unique identifier for each device)
/// - Prefab hash (type identifier for batch operations like `lb`, `sb`)
/// - Name hash (for operations like `lbn`, `sbn`)
#[derive(Debug, Clone)]
pub struct CableNetwork {
    /// All devices on the network, keyed by their reference ID
    devices: HashMap<i32, Rc<RefCell<dyn Device>>>,

    /// Index for quick lookup by prefab hash
    /// Maps prefab_hash -> list of device reference IDs
    prefab_index: HashMap<i32, Vec<i32>>,

    /// Index for quick lookup by name hash
    /// Maps name_hash -> list of device reference IDs
    name_index: HashMap<i32, Vec<i32>>,
}

impl CableNetwork {
    /// Create a new empty cable network
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
            prefab_index: HashMap::new(),
            name_index: HashMap::new(),
        }
    }

    /// Add a device to the network and set up the bidirectional connection
    ///
    /// The device will be indexed by its reference ID, prefab hash, and name hash
    pub fn add_device(
        &mut self,
        device: Rc<RefCell<dyn Device>>,
        network_rc: Rc<RefCell<CableNetwork>>,
    ) {
        // Set the device's network reference
        device.borrow_mut().set_network(Some(network_rc));

        let borrowed = device.borrow();
        let ref_id = borrowed.get_id();
        let prefab_hash = borrowed.get_prefab_hash();
        let name_hash = borrowed.get_name_hash();
        drop(borrowed);

        // Add to main device map
        self.devices.insert(ref_id, Rc::clone(&device));

        // Add to prefab index
        self.prefab_index
            .entry(prefab_hash)
            .or_insert_with(Vec::new)
            .push(ref_id);

        // Add to name index
        self.name_index
            .entry(name_hash)
            .or_insert_with(Vec::new)
            .push(ref_id);
    }

    /// Remove a device from the network by its reference ID
    pub fn remove_device(&mut self, ref_id: i32) -> Option<Rc<RefCell<dyn Device>>> {
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

        // Add to new name index
        self.name_index
            .entry(new_name_hash)
            .or_insert_with(Vec::new)
            .push(ref_id);
    }

    /// Check if a device with the given reference ID exists on the network
    pub fn device_exists(&self, ref_id: i32) -> bool {
        self.devices.contains_key(&ref_id)
    }

    /// Get a device by its reference ID (immutable borrow)
    /// Returns the Rc so the caller can borrow as needed
    pub fn get_device(&self, ref_id: i32) -> Option<Ref<dyn Device>> {
        self.devices.get(&ref_id).map(|d| d.borrow())
    }

    /// Get a device by its reference ID (mutable borrow)
    /// Returns the Rc so the caller can borrow as needed
    pub fn get_device_mut(&self, ref_id: i32) -> Option<RefMut<dyn Device>> {
        let device = self.devices.get(&ref_id)?;
        Some(device.borrow_mut())
    }

    /// Get the raw Rc for a device (for when you need to store a reference)
    pub fn get_device_rc(&self, ref_id: i32) -> Option<Rc<RefCell<dyn Device>>> {
        self.devices.get(&ref_id).cloned()
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

    // ==================== Batch Read Operations ====================

    /// Read a logic value from all devices matching a prefab hash
    /// Used for `lb` instruction
    pub fn batch_read_by_prefab(
        &self,
        prefab_hash: i32,
        logic_type: LogicType,
        batch_mode: BatchMode,
    ) -> IC10Result<f64> {
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
    ) -> IC10Result<f64> {
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
    ) -> IC10Result<f64> {
        if device_ids.is_empty() {
            // Return 0 for empty batch (matches game behavior)
            return Ok(0.0);
        }

        let mut values: Vec<f64> = Vec::new();

        for &ref_id in device_ids {
            if let Some(device) = self.get_device(ref_id) {
                if device.can_read(logic_type) {
                    match device.read(logic_type) {
                        Ok(val) => values.push(val),
                        Err(_) => continue, // Skip devices that error
                    }
                }
            }
        }

        if values.is_empty() {
            return Ok(0.0);
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
    ) -> IC10Result<usize> {
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
    ) -> IC10Result<usize> {
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
    ) -> IC10Result<usize> {
        let mut write_count = 0;

        for &ref_id in device_ids {
            if let Some(mut device) = self.get_device_mut(ref_id) {
                if device.can_write(logic_type) {
                    if device.write(logic_type, value).is_ok() {
                        write_count += 1;
                    }
                }
            }
        }

        Ok(write_count)
    }
}

impl Default for CableNetwork {
    fn default() -> Self {
        Self::new()
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
