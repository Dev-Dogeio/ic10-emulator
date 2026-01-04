use crate::{
    CableNetwork,
    error::{IC10Error, IC10Result},
    parser::string_to_hash,
    types::OptShared,
};
use std::cell::RefCell;
use std::sync::atomic::{AtomicI32, Ordering};

pub mod daylight_sensor;
pub mod ic_housing;
pub mod logic_memory;

/// Simulation settings for devices
#[derive(Debug, Clone)]
pub struct SimulationSettings {
    /// Number of ticks in a full day cycle
    pub ticks_per_day: f64,
    /// Maximum instructions per tick for IC10
    pub max_instructions_per_tick: usize,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            ticks_per_day: 2400.0,
            max_instructions_per_tick: 128,
        }
    }
}

pub use daylight_sensor::DaylightSensor;
pub use ic_housing::ICHousing;
pub use logic_memory::LogicMemory;

/// Global device ID counter shared by all device types
static DEVICE_ID_COUNTER: AtomicI32 = AtomicI32::new(1);

/// Allocate a new unique device ID
pub fn allocate_device_id() -> i32 {
    DEVICE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Logic types for device property access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicType {
    NameHash,
    PrefabHash,
    ReferenceId,
    Setting,
    Horizontal,
    Vertical,
}

impl LogicType {
    /// Convert from a numeric value to LogicType
    pub fn from_value(value: f64) -> Option<Self> {
        match value as i32 {
            12 => Some(LogicType::Setting),
            20 => Some(LogicType::Horizontal),
            21 => Some(LogicType::Vertical),
            84 => Some(LogicType::PrefabHash),
            217 => Some(LogicType::ReferenceId),
            268 => Some(LogicType::NameHash),
            _ => None,
        }
    }

    /// Convert LogicType to its numeric value
    pub fn to_value(self) -> f64 {
        match self {
            LogicType::Setting => 12.0,
            LogicType::Horizontal => 20.0,
            LogicType::Vertical => 21.0,
            LogicType::PrefabHash => 84.0,
            LogicType::ReferenceId => 217.0,
            LogicType::NameHash => 268.0,
        }
    }

    /// Parse LogicType from a string name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Setting" => Some(LogicType::Setting),
            "Horizontal" => Some(LogicType::Horizontal),
            "Vertical" => Some(LogicType::Vertical),
            "PrefabHash" => Some(LogicType::PrefabHash),
            "ReferenceId" => Some(LogicType::ReferenceId),
            "NameHash" => Some(LogicType::NameHash),
            _ => None,
        }
    }
}

// TODO: Implement slot logic for devices
// /// Logic slot types for slot-based properties
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum LogicSlotType {
//     Occupied,
//     OccupantHash,
//     Quantity,
// }

// impl LogicSlotType {
//     /// Convert from a numeric value to LogicSlotType
//     pub fn from_value(value: f64) -> Option<Self> {
//         match value as i32 {
//             0 => Some(LogicSlotType::Occupied),
//             1 => Some(LogicSlotType::OccupantHash),
//             2 => Some(LogicSlotType::Quantity),
//             _ => None,
//         }
//     }
// }

/// Shared data for all devices
#[derive(Debug)]
pub struct DeviceBase {
    pub name: String,
    pub network: OptShared<CableNetwork>,
    pub logic_types: RefCell<LogicTypes>,
}

impl DeviceBase {
    pub fn new(name: String, prefab_hash: i32) -> Self {
        let id = allocate_device_id();
        let logic_types = LogicTypes::new(id, prefab_hash, &name);
        Self {
            name,
            network: None,
            logic_types: RefCell::new(logic_types),
        }
    }

    pub fn get_network(&self) -> OptShared<CableNetwork> {
        self.network.clone()
    }

    /// Set the network reference for the device
    pub fn set_network(&mut self, network: OptShared<CableNetwork>) {
        self.network = network;
    }

    /// Set the device's name and update the network's name index
    pub fn set_name(&mut self, name: String) {
        let old_name_hash = self.logic_types.borrow().name_hash;
        self.name = name;
        let new_name_hash = string_to_hash(&self.name);
        self.logic_types.borrow_mut().name_hash = new_name_hash;

        // Update the network's name index if the device is connected
        if let Some(network) = &self.network {
            let reference_id = self.logic_types.borrow().reference_id;
            network
                .borrow_mut()
                .update_device_name(reference_id, old_name_hash, new_name_hash);
        }
    }
}

/// LogicTypes struct to hold PrefabHash, NameHash, and ReferenceId
#[derive(Debug)]
pub struct LogicTypes {
    prefab_hash: i32,
    name_hash: i32,
    reference_id: i32,
    setting: Option<f64>,
    horizontal: Option<f64>,
    vertical: Option<f64>,
}

impl LogicTypes {
    /// Get the value of a logic type
    pub fn get(&self, logic_type: LogicType) -> Option<f64> {
        match logic_type {
            LogicType::Setting => self.setting,
            LogicType::Horizontal => self.horizontal,
            LogicType::Vertical => self.vertical,
            LogicType::PrefabHash => Some(self.prefab_hash as f64),
            LogicType::ReferenceId => Some(self.reference_id as f64),
            LogicType::NameHash => Some(self.name_hash as f64),
        }
    }

    /// Set the value of a logic type
    pub fn set(&mut self, logic_type: LogicType, value: f64) {
        match logic_type {
            LogicType::Setting => self.setting = Some(value),
            LogicType::Horizontal => self.horizontal = Some(value),
            LogicType::Vertical => self.vertical = Some(value),
            LogicType::PrefabHash => self.prefab_hash = value as i32,
            LogicType::ReferenceId => self.reference_id = value as i32,
            LogicType::NameHash => self.name_hash = value as i32,
        }
    }

    pub fn new(id: i32, prefab_hash: i32, name: &str) -> Self {
        Self {
            reference_id: id,
            prefab_hash,
            name_hash: string_to_hash(name),
            setting: None,
            horizontal: None,
            vertical: None,
        }
    }
}

/// Trait for devices that can be controlled by IC10
pub trait Device: std::fmt::Debug {
    /// Get the device's unique identifier
    fn get_id(&self) -> i32;

    /// Get the device's prefab hash (type identifier)
    fn get_prefab_hash(&self) -> i32;

    /// Get the device's name hash
    fn get_name_hash(&self) -> i32;

    /// Get the device's name
    fn get_name(&self) -> &str;

    /// Get the device's network
    fn get_network(&self) -> OptShared<CableNetwork>;

    /// Check if the device can read the specified logic type
    fn can_read(&self, logic_type: LogicType) -> bool;

    /// Check if the device can write the specified logic type
    fn can_write(&self, logic_type: LogicType) -> bool;

    /// Read a logic value from the device
    fn read(&self, logic_type: LogicType) -> IC10Result<f64>;

    /// Write a logic value to the device
    fn write(&self, logic_type: LogicType, value: f64) -> IC10Result<()>;

    /// Read from device internal memory at index
    fn get_memory(&self, _index: usize) -> IC10Result<f64> {
        Err(IC10Error::RuntimeError {
            message: "Device does not support memory access".to_string(),
            line: 0,
        })
    }

    /// Write to device internal memory at index
    fn set_memory(&self, _index: usize, _value: f64) -> IC10Result<()> {
        Err(IC10Error::RuntimeError {
            message: "Device does not support memory access".to_string(),
            line: 0,
        })
    }

    /// Clear device stack memory (clr/clrd)
    fn clear(&self) -> IC10Result<()> {
        Err(IC10Error::RuntimeError {
            message: "Device does not support memory clearing".to_string(),
            line: 0,
        })
    }

    /// Set the network reference for the device
    fn set_network(&mut self, network: OptShared<CableNetwork>);

    /// Set the device's name
    fn set_name(&mut self, name: &str);

    /// Get the list of supported LogicTypes for this device
    fn get_supported_logic_types(&self) -> Vec<LogicType> {
        vec![] // Default to no supported types
    }

    /// Check if the device supports a specific LogicType
    fn supports_logic_type(&self, logic_type: LogicType) -> bool {
        self.get_supported_logic_types().contains(&logic_type)
    }

    /// Update the device state based on the global tick count
    /// Default implementation does nothing - devices can override if they need tick-based updates
    fn update(&self, _tick: u64) -> IC10Result<()> {
        Ok(())
    }
}
