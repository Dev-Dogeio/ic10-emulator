use std::fmt::Debug;

use crate::{
    CableNetwork,
    error::{SimulationError, SimulationResult},
    items::ItemIntegratedCircuit10,
    types::{OptShared, Shared},
};

pub mod air_conditioner;
pub mod atmospheric_device;
pub mod chip_slot;
pub mod daylight_sensor;
pub mod filtration;
pub mod ic_housing;
pub mod logic_memory;
pub mod volume_pump;

pub use air_conditioner::AirConditioner;
pub use atmospheric_device::AtmosphericDevice;
pub use chip_slot::ChipSlot;
pub use daylight_sensor::DaylightSensor;
pub use filtration::Filtration;
pub use ic_housing::ICHousing;
pub use logic_memory::LogicMemory;
pub use volume_pump::VolumePump;

/// Simulation settings for devices
#[derive(Clone)]
pub struct SimulationSettings {
    /// Number of ticks in a full day cycle
    pub ticks_per_day: f64,
    /// Maximum instructions per tick for IC10
    pub max_instructions_per_tick: usize,
}

impl std::fmt::Display for SimulationSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SimulationSettings {{ ticks_per_day: {}, max_instructions_per_tick: {} }}",
            self.ticks_per_day, self.max_instructions_per_tick
        )
    }
}

impl std::fmt::Debug for SimulationSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            ticks_per_day: 2400.0,
            max_instructions_per_tick: 128,
        }
    }
}

/// Slot logic types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicSlotType {
    None = 0,
    Occupied = 1,
    OccupantHash = 2,
    Quantity = 3,
    Damage = 4,
    Efficiency = 5,
    Health = 6,
    Growth = 7,
    Pressure = 8,
    Temperature = 9,
    Charge = 10,
    ChargeRatio = 11,
    Class = 12,
    PressureWaste = 13,
    PressureAir = 14,
    MaxQuantity = 15,
    Mature = 16,
    PrefabHash = 17,
    Seeding = 18,
    LineNumber = 19,
    Volume = 20,
    Open = 21,
    On = 22,
    Lock = 23,
    SortingClass = 24,
    FilterType = 25,
    ReferenceId = 26,
    HarvestedHash = 27,
    Mode = 28,
    MaturityRatio = 29,
    SeedingRatio = 30,
    FreeSlots = 31,
    TotalSlots = 32,
}

impl LogicSlotType {
    pub fn from_value(value: f64) -> Option<Self> {
        match value as i32 {
            0 => Some(LogicSlotType::None),
            1 => Some(LogicSlotType::Occupied),
            2 => Some(LogicSlotType::OccupantHash),
            3 => Some(LogicSlotType::Quantity),
            4 => Some(LogicSlotType::Damage),
            5 => Some(LogicSlotType::Efficiency),
            6 => Some(LogicSlotType::Health),
            7 => Some(LogicSlotType::Growth),
            8 => Some(LogicSlotType::Pressure),
            9 => Some(LogicSlotType::Temperature),
            10 => Some(LogicSlotType::Charge),
            11 => Some(LogicSlotType::ChargeRatio),
            12 => Some(LogicSlotType::Class),
            13 => Some(LogicSlotType::PressureWaste),
            14 => Some(LogicSlotType::PressureAir),
            15 => Some(LogicSlotType::MaxQuantity),
            16 => Some(LogicSlotType::Mature),
            17 => Some(LogicSlotType::PrefabHash),
            18 => Some(LogicSlotType::Seeding),
            19 => Some(LogicSlotType::LineNumber),
            20 => Some(LogicSlotType::Volume),
            21 => Some(LogicSlotType::Open),
            22 => Some(LogicSlotType::On),
            23 => Some(LogicSlotType::Lock),
            24 => Some(LogicSlotType::SortingClass),
            25 => Some(LogicSlotType::FilterType),
            26 => Some(LogicSlotType::ReferenceId),
            27 => Some(LogicSlotType::HarvestedHash),
            28 => Some(LogicSlotType::Mode),
            29 => Some(LogicSlotType::MaturityRatio),
            30 => Some(LogicSlotType::SeedingRatio),
            31 => Some(LogicSlotType::FreeSlots),
            32 => Some(LogicSlotType::TotalSlots),
            _ => None,
        }
    }

    pub fn to_value(self) -> f64 {
        match self {
            LogicSlotType::None => 0.0,
            LogicSlotType::Occupied => 1.0,
            LogicSlotType::OccupantHash => 2.0,
            LogicSlotType::Quantity => 3.0,
            LogicSlotType::Damage => 4.0,
            LogicSlotType::Efficiency => 5.0,
            LogicSlotType::Health => 6.0,
            LogicSlotType::Growth => 7.0,
            LogicSlotType::Pressure => 8.0,
            LogicSlotType::Temperature => 9.0,
            LogicSlotType::Charge => 10.0,
            LogicSlotType::ChargeRatio => 11.0,
            LogicSlotType::Class => 12.0,
            LogicSlotType::PressureWaste => 13.0,
            LogicSlotType::PressureAir => 14.0,
            LogicSlotType::MaxQuantity => 15.0,
            LogicSlotType::Mature => 16.0,
            LogicSlotType::PrefabHash => 17.0,
            LogicSlotType::Seeding => 18.0,
            LogicSlotType::LineNumber => 19.0,
            LogicSlotType::Volume => 20.0,
            LogicSlotType::Open => 21.0,
            LogicSlotType::On => 22.0,
            LogicSlotType::Lock => 23.0,
            LogicSlotType::SortingClass => 24.0,
            LogicSlotType::FilterType => 25.0,
            LogicSlotType::ReferenceId => 26.0,
            LogicSlotType::HarvestedHash => 27.0,
            LogicSlotType::Mode => 28.0,
            LogicSlotType::MaturityRatio => 29.0,
            LogicSlotType::SeedingRatio => 30.0,
            LogicSlotType::FreeSlots => 31.0,
            LogicSlotType::TotalSlots => 32.0,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "None" => Some(LogicSlotType::None),
            "Occupied" => Some(LogicSlotType::Occupied),
            "OccupantHash" => Some(LogicSlotType::OccupantHash),
            "Quantity" => Some(LogicSlotType::Quantity),
            "Damage" => Some(LogicSlotType::Damage),
            "Efficiency" => Some(LogicSlotType::Efficiency),
            "Health" => Some(LogicSlotType::Health),
            "Growth" => Some(LogicSlotType::Growth),
            "Pressure" => Some(LogicSlotType::Pressure),
            "Temperature" => Some(LogicSlotType::Temperature),
            "Charge" => Some(LogicSlotType::Charge),
            "ChargeRatio" => Some(LogicSlotType::ChargeRatio),
            "Class" => Some(LogicSlotType::Class),
            "PressureWaste" => Some(LogicSlotType::PressureWaste),
            "PressureAir" => Some(LogicSlotType::PressureAir),
            "MaxQuantity" => Some(LogicSlotType::MaxQuantity),
            "Mature" => Some(LogicSlotType::Mature),
            "PrefabHash" => Some(LogicSlotType::PrefabHash),
            "Seeding" => Some(LogicSlotType::Seeding),
            "LineNumber" => Some(LogicSlotType::LineNumber),
            "Volume" => Some(LogicSlotType::Volume),
            "Open" => Some(LogicSlotType::Open),
            "On" => Some(LogicSlotType::On),
            "Lock" => Some(LogicSlotType::Lock),
            "SortingClass" => Some(LogicSlotType::SortingClass),
            "FilterType" => Some(LogicSlotType::FilterType),
            "ReferenceId" => Some(LogicSlotType::ReferenceId),
            "HarvestedHash" => Some(LogicSlotType::HarvestedHash),
            "Mode" => Some(LogicSlotType::Mode),
            "MaturityRatio" => Some(LogicSlotType::MaturityRatio),
            "SeedingRatio" => Some(LogicSlotType::SeedingRatio),
            "FreeSlots" => Some(LogicSlotType::FreeSlots),
            "TotalSlots" => Some(LogicSlotType::TotalSlots),
            _ => None,
        }
    }
}

/// Logic types for device property access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicType {
    Setting,
    Horizontal,
    Vertical,
    Ratio,
    On,
    Mode,
    PrefabHash,

    // Atmospheric Input 1
    PressureInput,
    TemperatureInput,
    RatioOxygenInput,
    RatioCarbonDioxideInput,
    RatioNitrogenInput,
    RatioPollutantInput,
    RatioVolatilesInput,
    RatioSteamInput,
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
    RatioSteamInput2,
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
    RatioSteamOutput,
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
    RatioSteamOutput2,
    RatioNitrousOxideOutput2,
    TotalMolesOutput2,

    // AirConditioner
    OperationalTemperatureEfficiency,
    TemperatureDifferentialEfficiency,
    PressureEfficiency,

    ReferenceId,
    LineNumber,
    StackSize,
    NameHash,
}

impl LogicType {
    /// Convert from a numeric value to LogicType
    pub fn from_value(value: f64) -> Option<Self> {
        match value as i32 {
            3 => Some(LogicType::Mode),
            12 => Some(LogicType::Setting),
            20 => Some(LogicType::Horizontal),
            21 => Some(LogicType::Vertical),
            24 => Some(LogicType::Ratio),
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
            113 => Some(LogicType::RatioSteamInput),
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
            123 => Some(LogicType::RatioSteamInput2),
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
            133 => Some(LogicType::RatioSteamOutput),
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
            143 => Some(LogicType::RatioSteamOutput2),
            144 => Some(LogicType::RatioNitrousOxideOutput2),
            145 => Some(LogicType::TotalMolesOutput2),

            // AirConditioner
            150 => Some(LogicType::OperationalTemperatureEfficiency),
            151 => Some(LogicType::TemperatureDifferentialEfficiency),
            152 => Some(LogicType::PressureEfficiency),

            173 => Some(LogicType::LineNumber),
            217 => Some(LogicType::ReferenceId),
            280 => Some(LogicType::StackSize),
            268 => Some(LogicType::NameHash),

            _ => None,
        }
    }

    /// Convert LogicType to its numeric value
    pub fn to_value(self) -> f64 {
        match self {
            LogicType::Mode => 3.0,
            LogicType::Setting => 12.0,
            LogicType::Horizontal => 20.0,
            LogicType::Vertical => 21.0,
            LogicType::Ratio => 24.0,
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
            LogicType::RatioSteamInput => 113.0,
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
            LogicType::RatioSteamInput2 => 123.0,
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
            LogicType::RatioSteamOutput => 133.0,
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
            LogicType::RatioSteamOutput2 => 143.0,
            LogicType::RatioNitrousOxideOutput2 => 144.0,
            LogicType::TotalMolesOutput2 => 145.0,

            // AirConditioner
            LogicType::OperationalTemperatureEfficiency => 150.0,
            LogicType::TemperatureDifferentialEfficiency => 151.0,
            LogicType::PressureEfficiency => 152.0,

            LogicType::LineNumber => 173.0,
            LogicType::ReferenceId => 217.0,
            LogicType::NameHash => 268.0,
            LogicType::StackSize => 280.0,
        }
    }

    /// Parse LogicType from a string name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Mode" => Some(LogicType::Mode),
            "Setting" => Some(LogicType::Setting),
            "Horizontal" => Some(LogicType::Horizontal),
            "Vertical" => Some(LogicType::Vertical),
            "Ratio" => Some(LogicType::Ratio),
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
            "RatioSteamInput" => Some(LogicType::RatioSteamInput),
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
            "RatioSteamInput2" => Some(LogicType::RatioSteamInput2),
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
            "RatioSteamOutput" => Some(LogicType::RatioSteamOutput),
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
            "RatioSteamOutput2" => Some(LogicType::RatioSteamOutput2),
            "RatioNitrousOxideOutput2" => Some(LogicType::RatioNitrousOxideOutput2),
            "TotalMolesOutput2" => Some(LogicType::TotalMolesOutput2),

            // AirConditioner
            "OperationalTemperatureEfficiency" => Some(LogicType::OperationalTemperatureEfficiency),
            "TemperatureDifferentialEfficiency" => {
                Some(LogicType::TemperatureDifferentialEfficiency)
            }
            "PressureEfficiency" => Some(LogicType::PressureEfficiency),

            "ReferenceId" => Some(LogicType::ReferenceId),
            "LineNumber" => Some(LogicType::LineNumber),
            "StackSize" => Some(LogicType::StackSize),
            "NameHash" => Some(LogicType::NameHash),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceAtmosphericNetworkType {
    Input,
    Input2,
    Output,
    Output2,
    Internal,
}

/// Trait for devices that can be controlled by IC10
pub trait Device: Debug {
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

    /// Clear internal references (like chip slot host pointers) to break reference cycles.
    /// Default implementation does nothing; devices that hold cross-references should override.
    fn clear_internal_references(&mut self) {}

    /// Read a value from a specific slot
    fn read_slot(&self, _index: usize, _slot_logic_type: LogicSlotType) -> SimulationResult<f64> {
        Err(SimulationError::RuntimeError {
            message: "Device does not support slot operations".to_string(),
            line: 0,
        })
    }

    /// Write a value to a specific slot
    fn write_slot(
        &self,
        _index: usize,
        _slot_logic_type: LogicSlotType,
        _value: f64,
    ) -> SimulationResult<()> {
        Err(SimulationError::RuntimeError {
            message: "Device does not support slot operations".to_string(),
            line: 0,
        })
    }

    /// Set the device's name
    fn set_name(&mut self, name: &str);

    /// Update the device state based on the global tick count
    /// Default implementation does nothing - devices can override if they need tick-based updates
    fn update(&self, _tick: u64) -> SimulationResult<()> {
        Ok(())
    }

    /// Run chip code if applicable for the device.
    /// Default implementation does nothing - devices can override if they can execute code.
    fn run(&self) -> SimulationResult<()> {
        Ok(())
    }
}

/// Marker trait to ensure implementors of `ICHostDevice` explicitly opt into providing device
/// memory access methods (`Device::get_memory`, `Device::set_memory`, and `Device::clear`).
///
/// Implementors of `ICHostDevice` MUST provide an explicit empty impl of this marker trait in
/// their module when they override `Device`'s memory methods (even if those methods simply
/// delegate to the `ICHostDevice` default implementations). Making this a supertrait enforces
/// a compile-time error for any new `ICHostDevice` implementations that forget to opt-in.
pub trait ICHostDeviceMemoryOverride {}

/// Trait for devices that host an IC10 chip and provide common helpers for chip access and execution.
///
/// Implementors of this trait should also override `Device`'s `get_memory`, `set_memory`, and `clear`
/// methods (or delegate to the `ICHostDevice` default implementations) so calls through a
/// `dyn Device` object are routed to the hosted chip's memory. Unit tests in `logic_tests`
/// verify this behavior.
pub trait ICHostDevice: ICHostDeviceMemoryOverride {
    fn ichost_get_id(&self) -> i32;

    /// Return the shared `ChipSlot` for this device.
    fn chip_slot(&self) -> Shared<ChipSlot>;

    /// Return the maximum instructions per tick setting for this host.
    fn max_instructions_per_tick(&self) -> usize;

    /// Read from device internal memory at index. Default implementation proxies to the hosted chip.
    fn get_memory(&self, address: usize) -> SimulationResult<f64> {
        if let Some(chip) = self.chip_slot().borrow().get_chip() {
            return chip.borrow().read_stack(address);
        }

        Err(SimulationError::RuntimeError {
            message: "No chip installed".to_string(),
            line: 0,
        })
    }

    /// Write to device internal memory at index. Default implementation proxies to the hosted chip.
    fn set_memory(&self, address: usize, value: f64) -> SimulationResult<()> {
        if let Some(chip) = self.chip_slot().borrow().get_chip() {
            return chip.borrow().write_stack(address, value);
        }

        Err(SimulationError::RuntimeError {
            message: "No chip installed".to_string(),
            line: 0,
        })
    }

    /// Clear device stack memory (clr/clrd). Default proxies to the hosted chip.
    fn clear(&self) -> SimulationResult<()> {
        if let Some(chip) = self.chip_slot().borrow().get_chip() {
            chip.borrow().clear_stack();
            return Ok(());
        }

        Err(SimulationError::RuntimeError {
            message: "No chip installed".to_string(),
            line: 0,
        })
    }

    /// Insert an IC chip into the host and attach it. Default implementation inserts into the slot and assigns the chip slot to the chip.
    fn set_chip(&mut self, chip: Shared<ItemIntegratedCircuit10>) {
        self.chip_slot()
            .borrow_mut()
            .set_chip(Box::new(chip.clone()))
            .unwrap();

        // Attach the slot back to the chip so it can resolve device pins/aliases
        chip.borrow_mut()
            .set_chip_slot(self.chip_slot(), self.ichost_get_id());
    }

    /// Get the hosted chip (if any).
    fn get_chip(&self) -> OptShared<ItemIntegratedCircuit10> {
        self.chip_slot().borrow().get_chip()
    }

    /// Set a device pin on the housing's chip slot (d0-dN)
    fn set_device_pin(&mut self, pin: usize, device_ref_id: Option<i32>) {
        self.chip_slot()
            .borrow_mut()
            .set_device_pin(pin, device_ref_id);
    }

    /// Get a device pin reference ID from the chip slot (d0-dN)
    fn get_device_pin(&self, pin: usize) -> Option<i32> {
        self.chip_slot().borrow().get_device_pin(pin)
    }

    /// Get the number of instructions executed during the last update.
    fn get_last_executed_instructions(&self) -> usize {
        self.chip_slot().borrow().get_last_executed_instructions()
    }

    /// Execute hosted chip code using the host's instruction limit.
    fn run(&self) -> SimulationResult<()> {
        self.chip_slot()
            .borrow()
            .run(self.max_instructions_per_tick())?;
        Ok(())
    }
}
