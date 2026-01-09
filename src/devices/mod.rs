//! Device implementations for the IC10 emulator

use std::fmt::{Debug, Display};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::{
    AtmosphericNetwork, CableNetwork, Item,
    devices::property_descriptor::{PropertyRegistry, SlotPropertyRegistry, empty_slot_registry},
    error::{SimulationError, SimulationResult},
    items::ItemIntegratedCircuit10,
    types::{OptShared, Shared},
};

pub mod air_conditioner;
pub mod chip_slot;
pub mod daylight_sensor;
pub mod device_factory;
pub mod filtration;
pub mod ic_housing;
pub mod logic_memory;
pub mod property_descriptor;
pub mod volume_pump;

pub use air_conditioner::AirConditioner;
pub use chip_slot::ChipSlot;
pub use daylight_sensor::DaylightSensor;
pub use filtration::Filtration;
pub use ic_housing::ICHousing;
pub use logic_memory::LogicMemory;
pub use volume_pump::VolumePump;

/// Simulation settings for devices
#[derive(Clone, Default)]
pub struct SimulationDeviceSettings {
    /// Number of ticks in a day cycle
    pub ticks_per_day: Option<f64>,
    /// Max IC10 instructions per tick
    pub max_instructions_per_tick: Option<usize>,
    /// Device name override
    pub name: Option<String>,
    /// The device will request this ID and panic if already allocated
    pub id: Option<i32>,
    /// Internal atmospheric network to use for devices that require an internal buffer, ignored otherwise
    pub internal_atmospheric_network: OptShared<AtmosphericNetwork>,
}

impl Display for SimulationDeviceSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SimulationDeviceSettings {{ ticks_per_day: {:?}, max_instructions_per_tick: {:?}, name: {:?}, id: {:?}, internal: {} }}",
            self.ticks_per_day,
            self.max_instructions_per_tick,
            self.name,
            self.id,
            if self.internal_atmospheric_network.is_some() {
                "Some"
            } else {
                "None"
            }
        )
    }
}

impl Debug for SimulationDeviceSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Slot logic types
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[repr(i32)]
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
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[repr(i32)]
pub enum LogicType {
    Mode = 3,
    Setting = 12,
    Horizontal = 20,
    Vertical = 21,
    Ratio = 24,
    On = 28,
    PrefabHash = 84,

    // Atmospheric Input 1
    PressureInput = 106,
    TemperatureInput = 107,
    TotalMolesInput = 115,
    CombustionInput = 146,
    RatioOxygenInput = 108,
    RatioCarbonDioxideInput = 109,
    RatioNitrogenInput = 110,
    RatioPollutantInput = 111,
    RatioVolatilesInput = 112,
    RatioWaterInput = 113,
    RatioNitrousOxideInput = 114,
    RatioLiquidNitrogenInput = 178,
    RatioLiquidOxygenInput = 184,
    RatioLiquidVolatilesInput = 189,
    RatioSteamInput = 194,
    RatioLiquidCarbonDioxideInput = 200,
    RatioLiquidPollutantInput = 205,
    RatioLiquidNitrousOxideInput = 210,

    // Atmospheric Input 2
    PressureInput2 = 116,
    TemperatureInput2 = 117,
    TotalMolesInput2 = 125,
    CombustionInput2 = 147,
    RatioOxygenInput2 = 118,
    RatioCarbonDioxideInput2 = 119,
    RatioNitrogenInput2 = 120,
    RatioPollutantInput2 = 121,
    RatioVolatilesInput2 = 122,
    RatioWaterInput2 = 123,
    RatioNitrousOxideInput2 = 124,
    RatioLiquidNitrogenInput2 = 179,
    RatioLiquidOxygenInput2 = 185,
    RatioLiquidVolatilesInput2 = 190,
    RatioSteamInput2 = 195,
    RatioLiquidCarbonDioxideInput2 = 201,
    RatioLiquidPollutantInput2 = 206,
    RatioLiquidNitrousOxideInput2 = 211,

    // Atmospheric Output 1
    PressureOutput = 126,
    TemperatureOutput = 127,
    TotalMolesOutput = 135,
    CombustionOutput = 148,
    RatioOxygenOutput = 128,
    RatioCarbonDioxideOutput = 129,
    RatioNitrogenOutput = 130,
    RatioPollutantOutput = 131,
    RatioVolatilesOutput = 132,
    RatioWaterOutput = 133,
    RatioNitrousOxideOutput = 134,
    RatioLiquidNitrogenOutput = 180,
    RatioLiquidOxygenOutput = 186,
    RatioLiquidVolatilesOutput = 191,
    RatioSteamOutput = 196,
    RatioLiquidCarbonDioxideOutput = 202,
    RatioLiquidPollutantOutput = 207,
    RatioLiquidNitrousOxideOutput = 212,

    // Atmospheric Output 2
    PressureOutput2 = 136,
    TemperatureOutput2 = 137,
    TotalMolesOutput2 = 145,
    CombustionOutput2 = 149,
    RatioOxygenOutput2 = 138,
    RatioCarbonDioxideOutput2 = 139,
    RatioNitrogenOutput2 = 140,
    RatioPollutantOutput2 = 141,
    RatioVolatilesOutput2 = 142,
    RatioWaterOutput2 = 143,
    RatioNitrousOxideOutput2 = 144,
    RatioLiquidNitrogenOutput2 = 181,
    RatioLiquidOxygenOutput2 = 187,
    RatioLiquidVolatilesOutput2 = 192,
    RatioSteamOutput2 = 197,
    RatioLiquidCarbonDioxideOutput2 = 203,
    RatioLiquidPollutantOutput2 = 208,
    RatioLiquidNitrousOxideOutput2 = 213,

    // AirConditioner
    OperationalTemperatureEfficiency = 150,
    TemperatureDifferentialEfficiency = 151,
    PressureEfficiency = 152,

    LineNumber = 173,
    ReferenceId = 217,
    NameHash = 268,
    StackSize = 280,
}

impl LogicType {
    /// Convert from a numeric value to LogicType
    pub fn from_value(value: f64) -> Option<Self> {
        use LogicType::*;
        match value as i32 {
            3 => Some(Mode),
            12 => Some(Setting),
            20 => Some(Horizontal),
            21 => Some(Vertical),
            24 => Some(Ratio),
            28 => Some(On),
            84 => Some(PrefabHash),

            // Atmospheric Input 1
            106 => Some(PressureInput),
            107 => Some(TemperatureInput),
            115 => Some(TotalMolesInput),
            146 => Some(CombustionInput),
            108 => Some(RatioOxygenInput),
            109 => Some(RatioCarbonDioxideInput),
            110 => Some(RatioNitrogenInput),
            111 => Some(RatioPollutantInput),
            112 => Some(RatioVolatilesInput),
            113 => Some(RatioWaterInput),
            114 => Some(RatioNitrousOxideInput),
            178 => Some(RatioLiquidNitrogenInput),
            184 => Some(RatioLiquidOxygenInput),
            189 => Some(RatioLiquidVolatilesInput),
            194 => Some(RatioSteamInput),
            200 => Some(RatioLiquidCarbonDioxideInput),
            205 => Some(RatioLiquidPollutantInput),
            210 => Some(RatioLiquidNitrousOxideInput),

            // Atmospheric Input 2
            116 => Some(PressureInput2),
            117 => Some(TemperatureInput2),
            125 => Some(TotalMolesInput2),
            147 => Some(CombustionInput2),
            118 => Some(RatioOxygenInput2),
            119 => Some(RatioCarbonDioxideInput2),
            120 => Some(RatioNitrogenInput2),
            121 => Some(RatioPollutantInput2),
            122 => Some(RatioVolatilesInput2),
            123 => Some(RatioWaterInput2),
            124 => Some(RatioNitrousOxideInput2),
            179 => Some(RatioLiquidNitrogenInput2),
            185 => Some(RatioLiquidOxygenInput2),
            190 => Some(RatioLiquidVolatilesInput2),
            195 => Some(RatioSteamInput2),
            201 => Some(RatioLiquidCarbonDioxideInput2),
            206 => Some(RatioLiquidPollutantInput2),
            211 => Some(RatioLiquidNitrousOxideInput2),

            // Atmospheric Output 1
            126 => Some(PressureOutput),
            127 => Some(TemperatureOutput),
            135 => Some(TotalMolesOutput),
            148 => Some(CombustionOutput),
            128 => Some(RatioOxygenOutput),
            129 => Some(RatioCarbonDioxideOutput),
            130 => Some(RatioNitrogenOutput),
            131 => Some(RatioPollutantOutput),
            132 => Some(RatioVolatilesOutput),
            133 => Some(RatioWaterOutput),
            134 => Some(RatioNitrousOxideOutput),
            180 => Some(RatioLiquidNitrogenOutput),
            186 => Some(RatioLiquidOxygenOutput),
            191 => Some(RatioLiquidVolatilesOutput),
            196 => Some(RatioSteamOutput),
            202 => Some(RatioLiquidCarbonDioxideOutput),
            207 => Some(RatioLiquidPollutantOutput),
            212 => Some(RatioLiquidNitrousOxideOutput),

            // Atmospheric Output 2
            136 => Some(PressureOutput2),
            137 => Some(TemperatureOutput2),
            145 => Some(TotalMolesOutput2),
            149 => Some(CombustionOutput2),
            138 => Some(RatioOxygenOutput2),
            139 => Some(RatioCarbonDioxideOutput2),
            140 => Some(RatioNitrogenOutput2),
            141 => Some(RatioPollutantOutput2),
            142 => Some(RatioVolatilesOutput2),
            143 => Some(RatioWaterOutput2),
            144 => Some(RatioNitrousOxideOutput2),
            181 => Some(RatioLiquidNitrogenOutput2),
            187 => Some(RatioLiquidOxygenOutput2),
            192 => Some(RatioLiquidVolatilesOutput2),
            197 => Some(RatioSteamOutput2),
            203 => Some(RatioLiquidCarbonDioxideOutput2),
            208 => Some(RatioLiquidPollutantOutput2),
            213 => Some(RatioLiquidNitrousOxideOutput2),

            // AirConditioner
            150 => Some(OperationalTemperatureEfficiency),
            151 => Some(TemperatureDifferentialEfficiency),
            152 => Some(PressureEfficiency),

            173 => Some(LineNumber),
            217 => Some(ReferenceId),
            268 => Some(NameHash),
            280 => Some(StackSize),

            _ => None,
        }
    }

    /// Parse LogicType from a string name
    pub fn from_name(name: &str) -> Option<Self> {
        use LogicType::*;
        match name {
            "Mode" => Some(Mode),
            "Setting" => Some(Setting),
            "Horizontal" => Some(Horizontal),
            "Vertical" => Some(Vertical),
            "Ratio" => Some(Ratio),
            "On" => Some(On),
            "PrefabHash" => Some(PrefabHash),

            // Atmospheric Input 1
            "PressureInput" => Some(PressureInput),
            "TemperatureInput" => Some(TemperatureInput),
            "TotalMolesInput" => Some(TotalMolesInput),
            "CombustionInput" => Some(CombustionInput),
            "RatioOxygenInput" => Some(RatioOxygenInput),
            "RatioCarbonDioxideInput" => Some(RatioCarbonDioxideInput),
            "RatioNitrogenInput" => Some(RatioNitrogenInput),
            "RatioPollutantInput" => Some(RatioPollutantInput),
            "RatioVolatilesInput" => Some(RatioVolatilesInput),
            "RatioWaterInput" => Some(RatioWaterInput),
            "RatioNitrousOxideInput" => Some(RatioNitrousOxideInput),
            "RatioLiquidNitrogenInput" => Some(RatioLiquidNitrogenInput),
            "RatioLiquidOxygenInput" => Some(RatioLiquidOxygenInput),
            "RatioLiquidVolatilesInput" => Some(RatioLiquidVolatilesInput),
            "RatioSteamInput" => Some(RatioSteamInput),
            "RatioLiquidCarbonDioxideInput" => Some(RatioLiquidCarbonDioxideInput),
            "RatioLiquidPollutantInput" => Some(RatioLiquidPollutantInput),
            "RatioLiquidNitrousOxideInput" => Some(RatioLiquidNitrousOxideInput),

            // Atmospheric Input 2
            "PressureInput2" => Some(PressureInput2),
            "TemperatureInput2" => Some(TemperatureInput2),
            "TotalMolesInput2" => Some(TotalMolesInput2),
            "CombustionInput2" => Some(CombustionInput2),
            "RatioOxygenInput2" => Some(RatioOxygenInput2),
            "RatioCarbonDioxideInput2" => Some(RatioCarbonDioxideInput2),
            "RatioNitrogenInput2" => Some(RatioNitrogenInput2),
            "RatioPollutantInput2" => Some(RatioPollutantInput2),
            "RatioVolatilesInput2" => Some(RatioVolatilesInput2),
            "RatioWaterInput2" => Some(RatioWaterInput2),
            "RatioNitrousOxideInput2" => Some(RatioNitrousOxideInput2),
            "RatioLiquidNitrogenInput2" => Some(RatioLiquidNitrogenInput2),
            "RatioLiquidOxygenInput2" => Some(RatioLiquidOxygenInput2),
            "RatioLiquidVolatilesInput2" => Some(RatioLiquidVolatilesInput2),
            "RatioSteamInput2" => Some(RatioSteamInput2),
            "RatioLiquidCarbonDioxideInput2" => Some(RatioLiquidCarbonDioxideInput2),
            "RatioLiquidPollutantInput2" => Some(RatioLiquidPollutantInput2),
            "RatioLiquidNitrousOxideInput2" => Some(RatioLiquidNitrousOxideInput2),

            // Atmospheric Output 1
            "PressureOutput" => Some(PressureOutput),
            "TemperatureOutput" => Some(TemperatureOutput),
            "TotalMolesOutput" => Some(TotalMolesOutput),
            "CombustionOutput" => Some(CombustionOutput),
            "RatioOxygenOutput" => Some(RatioOxygenOutput),
            "RatioCarbonDioxideOutput" => Some(RatioCarbonDioxideOutput),
            "RatioNitrogenOutput" => Some(RatioNitrogenOutput),
            "RatioPollutantOutput" => Some(RatioPollutantOutput),
            "RatioVolatilesOutput" => Some(RatioVolatilesOutput),
            "RatioWaterOutput" => Some(RatioWaterOutput),
            "RatioNitrousOxideOutput" => Some(RatioNitrousOxideOutput),
            "RatioLiquidNitrogenOutput" => Some(RatioLiquidNitrogenOutput),
            "RatioLiquidOxygenOutput" => Some(RatioLiquidOxygenOutput),
            "RatioLiquidVolatilesOutput" => Some(RatioLiquidVolatilesOutput),
            "RatioSteamOutput" => Some(RatioSteamOutput),
            "RatioLiquidCarbonDioxideOutput" => Some(RatioLiquidCarbonDioxideOutput),
            "RatioLiquidPollutantOutput" => Some(RatioLiquidPollutantOutput),
            "RatioLiquidNitrousOxideOutput" => Some(RatioLiquidNitrousOxideOutput),

            // Atmospheric Output 2
            "PressureOutput2" => Some(PressureOutput2),
            "TemperatureOutput2" => Some(TemperatureOutput2),
            "TotalMolesOutput2" => Some(TotalMolesOutput2),
            "CombustionOutput2" => Some(CombustionOutput2),
            "RatioOxygenOutput2" => Some(RatioOxygenOutput2),
            "RatioCarbonDioxideOutput2" => Some(RatioCarbonDioxideOutput2),
            "RatioNitrogenOutput2" => Some(RatioNitrogenOutput2),
            "RatioPollutantOutput2" => Some(RatioPollutantOutput2),
            "RatioVolatilesOutput2" => Some(RatioVolatilesOutput2),
            "RatioWaterOutput2" => Some(RatioWaterOutput2),
            "RatioNitrousOxideOutput2" => Some(RatioNitrousOxideOutput2),
            "RatioLiquidNitrogenOutput2" => Some(RatioLiquidNitrogenOutput2),
            "RatioLiquidOxygenOutput2" => Some(RatioLiquidOxygenOutput2),
            "RatioLiquidVolatilesOutput2" => Some(RatioLiquidVolatilesOutput2),
            "RatioSteamOutput2" => Some(RatioSteamOutput2),
            "RatioLiquidCarbonDioxideOutput2" => Some(RatioLiquidCarbonDioxideOutput2),
            "RatioLiquidPollutantOutput2" => Some(RatioLiquidPollutantOutput2),
            "RatioLiquidNitrousOxideOutput2" => Some(RatioLiquidNitrousOxideOutput2),

            // AirConditioner
            "OperationalTemperatureEfficiency" => Some(OperationalTemperatureEfficiency),
            "TemperatureDifferentialEfficiency" => Some(TemperatureDifferentialEfficiency),
            "PressureEfficiency" => Some(PressureEfficiency),

            "LineNumber" => Some(LineNumber),
            "ReferenceId" => Some(ReferenceId),
            "NameHash" => Some(NameHash),
            "StackSize" => Some(StackSize),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceAtmosphericNetworkType {
    Internal = 0,
    Input = 1,
    Input2 = 2,
    Output = 3,
    Output2 = 4,
}

impl DeviceAtmosphericNetworkType {
    /// Convert from a numeric value to the enum variant
    pub fn from_value(value: i32) -> Option<Self> {
        match value {
            0 => Some(DeviceAtmosphericNetworkType::Internal),
            1 => Some(DeviceAtmosphericNetworkType::Input),
            2 => Some(DeviceAtmosphericNetworkType::Input2),
            3 => Some(DeviceAtmosphericNetworkType::Output),
            4 => Some(DeviceAtmosphericNetworkType::Output2),
            _ => None,
        }
    }

    /// Parse from a string name (case-sensitive)
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Input" => Some(DeviceAtmosphericNetworkType::Input),
            "Input2" => Some(DeviceAtmosphericNetworkType::Input2),
            "Output" => Some(DeviceAtmosphericNetworkType::Output),
            "Output2" => Some(DeviceAtmosphericNetworkType::Output2),
            "Internal" => Some(DeviceAtmosphericNetworkType::Internal),
            _ => None,
        }
    }
}

impl Display for DeviceAtmosphericNetworkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DeviceAtmosphericNetworkType::Internal => "Internal",
            DeviceAtmosphericNetworkType::Input => "Input",
            DeviceAtmosphericNetworkType::Input2 => "Input2",
            DeviceAtmosphericNetworkType::Output => "Output",
            DeviceAtmosphericNetworkType::Output2 => "Output2",
        };
        write!(f, "{}", s)
    }
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

    /// Static access to the property registry for this device type.
    fn properties() -> &'static PropertyRegistry<Self>
    where
        Self: Sized;

    /// Static access to the slot property registry for this device type.
    fn slot_properties() -> &'static SlotPropertyRegistry<Self>
    where
        Self: Sized,
    {
        empty_slot_registry::<Self>()
    }

    /// Human-readable static display name for the prefab.
    fn display_name_static() -> &'static str
    where
        Self: Sized;

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
    fn rename(&mut self, name: &str);

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

    /// Get the list of supported `LogicType` values for this device.
    fn supported_types(&self) -> Vec<LogicType> {
        Vec::new()
    }

    /// Get the list of supported `LogicSlotType` values for this device.
    fn supported_slot_types(&self) -> Vec<LogicSlotType> {
        Vec::new()
    }

    /// If the device hosts an IC chip, return a reference to it.
    fn as_ic_host_device(&self) -> Option<&dyn ICHostDevice> {
        None
    }

    /// If the device hosts an IC chip, return a mutable reference to it.
    fn as_ic_host_device_mut(&mut self) -> Option<&mut dyn ICHostDevice> {
        None
    }

    /// If the device supports item slots, return a reference to it.
    fn as_slot_host_device(&self) -> Option<&dyn SlotHostDevice> {
        None
    }

    /// If the device supports item slots, return a mutable reference to it.
    fn as_slot_host_device_mut(&mut self) -> Option<&mut dyn SlotHostDevice> {
        None
    }

    /// If the device is an AtmosphericDevice, return a reference to it.
    fn as_atmospheric_device(&self) -> Option<&dyn AtmosphericDevice> {
        None
    }

    /// If the device is an AtmosphericDevice, return a mutable reference to it.
    fn as_atmospheric_device_mut(&mut self) -> Option<&mut dyn AtmosphericDevice> {
        None
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
            return chip.read_stack(address);
        }

        Err(SimulationError::RuntimeError {
            message: "No chip installed".to_string(),
            line: 0,
        })
    }

    /// Write to device internal memory at index. Default implementation proxies to the hosted chip.
    fn set_memory(&self, address: usize, value: f64) -> SimulationResult<()> {
        if let Some(chip) = self.chip_slot().borrow().get_chip_mut() {
            return chip.write_stack(address, value);
        }

        Err(SimulationError::RuntimeError {
            message: "No chip installed".to_string(),
            line: 0,
        })
    }

    /// Clear device stack memory (clr/clrd). Default proxies to the hosted chip.
    fn clear(&self) -> SimulationResult<()> {
        if let Some(chip) = self.chip_slot().borrow().get_chip() {
            chip.clear_stack();
            return Ok(());
        }

        Err(SimulationError::RuntimeError {
            message: "No chip installed".to_string(),
            line: 0,
        })
    }

    /// Insert an IC chip into the host and attach it. Default implementation inserts into the slot and assigns the chip slot to the chip.
    fn set_chip(&self, chip: Shared<ItemIntegratedCircuit10>) {
        self.chip_slot()
            .borrow_mut()
            .set_chip(chip.clone())
            .unwrap();

        // Attach the slot back to the chip so it can resolve device pins/aliases
        chip.borrow_mut()
            .set_chip_slot(self.chip_slot(), self.ichost_get_id());
    }

    /// Set a device pin on the housing's chip slot (d0-dN)
    fn set_device_pin(&self, pin: usize, device_ref_id: Option<i32>) {
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

/// Trait for devices that expose normal item slot behaviour (inserting/removing items)
pub trait SlotHostDevice {
    /// Try to insert an item into a slot by index. Returns Ok(()) on success or Err(leftover) on failure.
    fn try_insert_item(
        &mut self,
        index: usize,
        incoming: Shared<dyn Item>,
    ) -> Result<(), Shared<dyn Item>>;

    /// Remove an item from a slot and return it if present.
    fn remove_item(&mut self, index: usize) -> OptShared<dyn Item>;

    /// Optional helper to return the number of slots
    fn slot_count(&self) -> usize {
        0
    }
}

/// Trait for devices that connect to atmospheric networks
pub trait AtmosphericDevice: Debug {
    /// Set the atmospheric network for a specific connection on this device
    fn set_atmospheric_network(
        &mut self,
        connection: DeviceAtmosphericNetworkType,
        network: OptShared<AtmosphericNetwork>,
    ) -> Result<(), SimulationError>;

    /// Get the atmospheric network for a specific connection on this device
    fn get_atmospheric_network(
        &self,
        connection: DeviceAtmosphericNetworkType,
    ) -> OptShared<AtmosphericNetwork>;
}
