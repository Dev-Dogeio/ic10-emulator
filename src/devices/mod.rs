use crate::{
    CableNetwork,
    error::{SimulationError, SimulationResult},
    parser::string_to_hash,
    types::OptShared,
};
use std::cell::RefCell;
use std::sync::atomic::{AtomicI32, Ordering};

pub mod atmospheric_device;
pub mod daylight_sensor;
pub mod filtration;
pub mod ic_housing;
pub mod logic_memory;

pub use atmospheric_device::AtmosphericDevice;
pub use filtration::Filtration;

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
    Setting,
    Horizontal,
    Vertical,
    On,
    PrefabHash,

    // Atmospheric Input 1
    PressureInput,
    TemperatureInput,
    RatioOxygenInput,
    RatioCarbonDioxideInput,
    RatioNitrogenInput,
    RatioPollutantInput,
    RatioVolatilesInput,
    RatioWaterInput,
    RatioNitrousOxideInput,
    TotalMolesInput,

    // Atmospheric Input 2
    PressureInput2,
    TemperatureInput2,
    RatioOxygenInput2,
    RatioCarbonDioxideInput2,
    RatioNitrogenInput2,
    RatioPollutantInput2,
    RatioVolatilesInput2,
    RatioWaterInput2,
    RatioNitrousOxideInput2,
    TotalMolesInput2,

    // Atmospheric Output 1
    PressureOutput,
    TemperatureOutput,
    RatioOxygenOutput,
    RatioCarbonDioxideOutput,
    RatioNitrogenOutput,
    RatioPollutantOutput,
    RatioVolatilesOutput,
    RatioWaterOutput,
    RatioNitrousOxideOutput,
    TotalMolesOutput,

    // Atmospheric Output 2
    PressureOutput2,
    TemperatureOutput2,
    RatioOxygenOutput2,
    RatioCarbonDioxideOutput2,
    RatioNitrogenOutput2,
    RatioPollutantOutput2,
    RatioVolatilesOutput2,
    RatioWaterOutput2,
    RatioNitrousOxideOutput2,
    TotalMolesOutput2,

    ReferenceId,
    NameHash,
}

impl LogicType {
    /// Convert from a numeric value to LogicType
    pub fn from_value(value: f64) -> Option<Self> {
        match value as i32 {
            12 => Some(LogicType::Setting),
            20 => Some(LogicType::Horizontal),
            21 => Some(LogicType::Vertical),
            28 => Some(LogicType::On),
            84 => Some(LogicType::PrefabHash),

            // Atmospheric Input 1
            106 => Some(LogicType::PressureInput),
            107 => Some(LogicType::TemperatureInput),
            108 => Some(LogicType::RatioOxygenInput),
            109 => Some(LogicType::RatioCarbonDioxideInput),
            110 => Some(LogicType::RatioNitrogenInput),
            111 => Some(LogicType::RatioPollutantInput),
            112 => Some(LogicType::RatioVolatilesInput),
            113 => Some(LogicType::RatioWaterInput),
            114 => Some(LogicType::RatioNitrousOxideInput),
            115 => Some(LogicType::TotalMolesInput),

            // Atmospheric Input 2
            116 => Some(LogicType::PressureInput2),
            117 => Some(LogicType::TemperatureInput2),
            118 => Some(LogicType::RatioOxygenInput2),
            119 => Some(LogicType::RatioCarbonDioxideInput2),
            120 => Some(LogicType::RatioNitrogenInput2),
            121 => Some(LogicType::RatioPollutantInput2),
            122 => Some(LogicType::RatioVolatilesInput2),
            123 => Some(LogicType::RatioWaterInput2),
            124 => Some(LogicType::RatioNitrousOxideInput2),
            125 => Some(LogicType::TotalMolesInput2),

            // Atmospheric Output 1
            126 => Some(LogicType::PressureOutput),
            127 => Some(LogicType::TemperatureOutput),
            128 => Some(LogicType::RatioOxygenOutput),
            129 => Some(LogicType::RatioCarbonDioxideOutput),
            130 => Some(LogicType::RatioNitrogenOutput),
            131 => Some(LogicType::RatioPollutantOutput),
            132 => Some(LogicType::RatioVolatilesOutput),
            133 => Some(LogicType::RatioWaterOutput),
            134 => Some(LogicType::RatioNitrousOxideOutput),
            135 => Some(LogicType::TotalMolesOutput),

            // Atmospheric Output 2
            136 => Some(LogicType::PressureOutput2),
            137 => Some(LogicType::TemperatureOutput2),
            138 => Some(LogicType::RatioOxygenOutput2),
            139 => Some(LogicType::RatioCarbonDioxideOutput2),
            140 => Some(LogicType::RatioNitrogenOutput2),
            141 => Some(LogicType::RatioPollutantOutput2),
            142 => Some(LogicType::RatioVolatilesOutput2),
            143 => Some(LogicType::RatioWaterOutput2),
            144 => Some(LogicType::RatioNitrousOxideOutput2),
            145 => Some(LogicType::TotalMolesOutput2),

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
            LogicType::On => 28.0,
            LogicType::PrefabHash => 84.0,

            // Atmospheric Input 1
            LogicType::PressureInput => 106.0,
            LogicType::TemperatureInput => 107.0,
            LogicType::RatioOxygenInput => 108.0,
            LogicType::RatioCarbonDioxideInput => 109.0,
            LogicType::RatioNitrogenInput => 110.0,
            LogicType::RatioPollutantInput => 111.0,
            LogicType::RatioVolatilesInput => 112.0,
            LogicType::RatioWaterInput => 113.0,
            LogicType::RatioNitrousOxideInput => 114.0,
            LogicType::TotalMolesInput => 115.0,

            // Atmospheric Input 2
            LogicType::PressureInput2 => 116.0,
            LogicType::TemperatureInput2 => 117.0,
            LogicType::RatioOxygenInput2 => 118.0,
            LogicType::RatioCarbonDioxideInput2 => 119.0,
            LogicType::RatioNitrogenInput2 => 120.0,
            LogicType::RatioPollutantInput2 => 121.0,
            LogicType::RatioVolatilesInput2 => 122.0,
            LogicType::RatioWaterInput2 => 123.0,
            LogicType::RatioNitrousOxideInput2 => 124.0,
            LogicType::TotalMolesInput2 => 125.0,

            // Atmospheric Output 1
            LogicType::PressureOutput => 126.0,
            LogicType::TemperatureOutput => 127.0,
            LogicType::RatioOxygenOutput => 128.0,
            LogicType::RatioCarbonDioxideOutput => 129.0,
            LogicType::RatioNitrogenOutput => 130.0,
            LogicType::RatioPollutantOutput => 131.0,
            LogicType::RatioVolatilesOutput => 132.0,
            LogicType::RatioWaterOutput => 133.0,
            LogicType::RatioNitrousOxideOutput => 134.0,
            LogicType::TotalMolesOutput => 135.0,

            // Atmospheric Output 2
            LogicType::PressureOutput2 => 136.0,
            LogicType::TemperatureOutput2 => 137.0,
            LogicType::RatioOxygenOutput2 => 138.0,
            LogicType::RatioCarbonDioxideOutput2 => 139.0,
            LogicType::RatioNitrogenOutput2 => 140.0,
            LogicType::RatioPollutantOutput2 => 141.0,
            LogicType::RatioVolatilesOutput2 => 142.0,
            LogicType::RatioWaterOutput2 => 143.0,
            LogicType::RatioNitrousOxideOutput2 => 144.0,
            LogicType::TotalMolesOutput2 => 145.0,

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
            "On" => Some(LogicType::On),
            "PrefabHash" => Some(LogicType::PrefabHash),

            // Atmospheric Input 1
            "PressureInput" => Some(LogicType::PressureInput),
            "TemperatureInput" => Some(LogicType::TemperatureInput),
            "RatioOxygenInput" => Some(LogicType::RatioOxygenInput),
            "RatioCarbonDioxideInput" => Some(LogicType::RatioCarbonDioxideInput),
            "RatioNitrogenInput" => Some(LogicType::RatioNitrogenInput),
            "RatioPollutantInput" => Some(LogicType::RatioPollutantInput),
            "RatioVolatilesInput" => Some(LogicType::RatioVolatilesInput),
            "RatioWaterInput" => Some(LogicType::RatioWaterInput),
            "RatioNitrousOxideInput" => Some(LogicType::RatioNitrousOxideInput),
            "TotalMolesInput" => Some(LogicType::TotalMolesInput),

            // Atmospheric Input 2
            "PressureInput2" => Some(LogicType::PressureInput2),
            "TemperatureInput2" => Some(LogicType::TemperatureInput2),
            "RatioOxygenInput2" => Some(LogicType::RatioOxygenInput2),
            "RatioCarbonDioxideInput2" => Some(LogicType::RatioCarbonDioxideInput2),
            "RatioNitrogenInput2" => Some(LogicType::RatioNitrogenInput2),
            "RatioPollutantInput2" => Some(LogicType::RatioPollutantInput2),
            "RatioVolatilesInput2" => Some(LogicType::RatioVolatilesInput2),
            "RatioWaterInput2" => Some(LogicType::RatioWaterInput2),
            "RatioNitrousOxideInput2" => Some(LogicType::RatioNitrousOxideInput2),
            "TotalMolesInput2" => Some(LogicType::TotalMolesInput2),

            // Atmospheric Output 1
            "PressureOutput" => Some(LogicType::PressureOutput),
            "TemperatureOutput" => Some(LogicType::TemperatureOutput),
            "RatioOxygenOutput" => Some(LogicType::RatioOxygenOutput),
            "RatioCarbonDioxideOutput" => Some(LogicType::RatioCarbonDioxideOutput),
            "RatioNitrogenOutput" => Some(LogicType::RatioNitrogenOutput),
            "RatioPollutantOutput" => Some(LogicType::RatioPollutantOutput),
            "RatioVolatilesOutput" => Some(LogicType::RatioVolatilesOutput),
            "RatioWaterOutput" => Some(LogicType::RatioWaterOutput),
            "RatioNitrousOxideOutput" => Some(LogicType::RatioNitrousOxideOutput),
            "TotalMolesOutput" => Some(LogicType::TotalMolesOutput),

            // Atmospheric Output 2
            "PressureOutput2" => Some(LogicType::PressureOutput2),
            "TemperatureOutput2" => Some(LogicType::TemperatureOutput2),
            "RatioOxygenOutput2" => Some(LogicType::RatioOxygenOutput2),
            "RatioCarbonDioxideOutput2" => Some(LogicType::RatioCarbonDioxideOutput2),
            "RatioNitrogenOutput2" => Some(LogicType::RatioNitrogenOutput2),
            "RatioPollutantOutput2" => Some(LogicType::RatioPollutantOutput2),
            "RatioVolatilesOutput2" => Some(LogicType::RatioVolatilesOutput2),
            "RatioWaterOutput2" => Some(LogicType::RatioWaterOutput2),
            "RatioNitrousOxideOutput2" => Some(LogicType::RatioNitrousOxideOutput2),
            "TotalMolesOutput2" => Some(LogicType::TotalMolesOutput2),

            "ReferenceId" => Some(LogicType::ReferenceId),
            "NameHash" => Some(LogicType::NameHash),
            _ => None,
        }
    }
}

/// Filter connection types shared across devices
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterConnectionType {
    Input,
    Input2,
    Output,
    Output2,
}

impl std::fmt::Display for FilterConnectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            FilterConnectionType::Input => "Input",
            FilterConnectionType::Input2 => "Input2",
            FilterConnectionType::Output => "Output",
            FilterConnectionType::Output2 => "Output2",
        };
        write!(f, "{}", s)
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
        self.logic_types
            .borrow_mut()
            .set(LogicType::NameHash, new_name_hash as f64)
            .unwrap();

        // Update the network's name index if the device is connected
        if let Some(network) = &self.network {
            let reference_id = self.logic_types.borrow().reference_id;
            network
                .borrow_mut()
                .update_device_name(reference_id, old_name_hash, new_name_hash);
        }
    }
}

/// LogicTypes struct to hold settable logic values
/// Some logic types like atmospheric readings are read-only and not stored here, instead they're computed on demand
#[derive(Debug)]
pub struct LogicTypes {
    setting: Option<f64>,
    horizontal: Option<f64>,
    vertical: Option<f64>,
    on: Option<f64>,
    prefab_hash: i32,
    reference_id: i32,
    name_hash: i32,
}

impl LogicTypes {
    /// Create a new LogicTypes instance
    pub fn new(id: i32, prefab_hash: i32, name: &str) -> Self {
        Self {
            setting: None,
            horizontal: None,
            vertical: None,
            on: Some(1.0),
            prefab_hash,
            reference_id: id,
            name_hash: string_to_hash(name),
        }
    }

    /// Set the value of a logic type
    pub fn set(&mut self, logic_type: LogicType, value: f64) -> SimulationResult<()> {
        match logic_type {
            LogicType::Setting => {
                self.setting = Some(value);
                Ok(())
            }
            LogicType::Horizontal => {
                self.horizontal = Some(value);
                Ok(())
            }
            LogicType::Vertical => {
                self.vertical = Some(value);
                Ok(())
            }
            LogicType::On => {
                self.on = Some(if value < 1.0 { 0.0 } else { 1.0 });
                Ok(())
            }
            LogicType::NameHash => {
                self.name_hash = value as i32;
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "LogicType {:?} is read-only or unsupported for setting",
                    logic_type
                ),
                line: 0,
            }),
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
    fn read(&self, logic_type: LogicType) -> SimulationResult<f64>;

    /// Write a logic value to the device
    fn write(&self, logic_type: LogicType, value: f64) -> SimulationResult<()>;

    /// Read from device internal memory at index
    fn get_memory(&self, _index: usize) -> SimulationResult<f64> {
        Err(SimulationError::RuntimeError {
            message: "Device does not support memory access".to_string(),
            line: 0,
        })
    }

    /// Write to device internal memory at index
    fn set_memory(&self, _index: usize, _value: f64) -> SimulationResult<()> {
        Err(SimulationError::RuntimeError {
            message: "Device does not support memory access".to_string(),
            line: 0,
        })
    }

    /// Clear device stack memory (clr/clrd)
    fn clear(&self) -> SimulationResult<()> {
        Err(SimulationError::RuntimeError {
            message: "Device does not support memory clearing".to_string(),
            line: 0,
        })
    }

    /// Set the network reference for the device
    fn set_network(&mut self, network: OptShared<CableNetwork>);

    /// Set the device's name
    fn set_name(&mut self, name: &str);

    /// Update the device state based on the global tick count
    /// Default implementation does nothing - devices can override if they need tick-based updates
    fn update(&self, _tick: u64) -> SimulationResult<()> {
        Ok(())
    }
}
