//! Daylight sensor: provides horizontal and vertical sun angles.

use std::fmt::{Debug, Display};
use std::{cell::RefCell, f64};

use crate::conversions::fmt_trim;
use crate::{
    CableNetwork, allocate_global_id,
    devices::{Device, LogicType, SimulationSettings},
    error::{SimulationError, SimulationResult},
    parser::string_to_hash,
    types::{OptShared, Shared, shared},
};

/// Daylight sensor: tracks sun position
pub struct DaylightSensor {
    /// Device name
    name: String,
    /// Connected network
    network: OptShared<CableNetwork>,

    /// The device reference ID
    reference_id: i32,
    /// The On state
    on: RefCell<f64>,
    /// The horizontal angle (degrees)
    horizontal: RefCell<f64>,
    /// The vertical angle (degrees)
    vertical: RefCell<f64>,

    /// Simulation settings
    settings: SimulationSettings,
}

/// Constructors and helpers
impl DaylightSensor {
    /// Compile-time prefab hash constant for this device
    pub const PREFAB_HASH: i32 = string_to_hash("StructureDaylightSensor");

    /// Create a new `DaylightSensor`
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Shared<Self> {
        shared(Self {
            name: "Daylight Sensor".to_string(),
            network: None,
            reference_id: allocate_global_id(),
            on: RefCell::new(1.0),
            horizontal: RefCell::new(0.0),
            vertical: RefCell::new(0.0),
            settings: simulation_settings.unwrap_or_default(),
        })
    }

    /// Current horizontal angle (degrees)
    pub fn horizontal(&self) -> f64 {
        *self.horizontal.borrow()
    }

    /// Current vertical angle (degrees)
    pub fn vertical(&self) -> f64 {
        *self.vertical.borrow()
    }

    /// Prefab hash for `DaylightSensor`
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }
}

/// `Device` trait implementation for `DaylightSensor` providing logic access and day-cycle updates.
impl Device for DaylightSensor {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        DaylightSensor::prefab_hash()
    }

    fn get_name_hash(&self) -> i32 {
        string_to_hash(&self.name)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_network(&self) -> OptShared<CableNetwork> {
        self.network.clone()
    }

    fn set_network(&mut self, network: OptShared<CableNetwork>) {
        self.network = network;
    }

    fn rename(&mut self, name: &str) {
        let old_name_hash = self.get_name_hash();
        self.name = name.to_string();

        if let Some(network) = &self.network {
            network.borrow_mut().update_device_name(
                self.reference_id,
                old_name_hash,
                string_to_hash(name),
            );
        }
    }

    fn can_read(&self, logic_type: LogicType) -> bool {
        matches!(
            logic_type,
            LogicType::PrefabHash
                | LogicType::ReferenceId
                | LogicType::NameHash
                | LogicType::Horizontal
                | LogicType::Vertical
                | LogicType::On
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(logic_type, LogicType::On)
    }

    fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        match logic_type {
            LogicType::PrefabHash => Ok(self.get_prefab_hash() as f64),
            LogicType::ReferenceId => Ok(self.reference_id as f64),
            LogicType::NameHash => Ok(self.get_name_hash() as f64),
            LogicType::Horizontal => Ok(*self.horizontal.borrow()),
            LogicType::Vertical => Ok(*self.vertical.borrow()),
            LogicType::On => Ok(*self.on.borrow()),
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "Daylight sensor does not support reading logic type {logic_type:?}"
                ),
                line: 0,
            }),
        }
    }

    fn write(&self, logic_type: LogicType, value: f64) -> SimulationResult<()> {
        match logic_type {
            LogicType::On => {
                *self.on.borrow_mut() = if value < 1.0 { 0.0 } else { 1.0 };
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "Daylight sensor does not support writing logic type {logic_type:?}"
                ),
                line: 0,
            }),
        }
    }

    fn update(&self, tick: u64) -> SimulationResult<()> {
        // Only update when device is On (non-zero)
        if *self.on.borrow() == 0.0 {
            return Ok(());
        }

        // Calculate position within the day cycle [0.0, 1.0)
        let day_progress =
            ((tick % self.settings.ticks_per_day as u64) as f64) / self.settings.ticks_per_day;

        // Horizontal angle: simple rotation around the compass
        // 0 degrees at tick 0, 360 degrees at tick 2400
        let horizontal = day_progress * 360.0;

        // Vertical angle: uses cosine to create smooth oscillation
        // At tick 0 (midnight): vertical = 180 (nadir)
        // At tick 600 (sunrise): vertical = 90 (horizon)
        // At tick 1200 (noon): vertical = 0 (zenith)
        // At tick 1800 (sunset): vertical = 90 (horizon)
        // At tick 2400 (midnight): vertical = 180 (nadir)
        //
        // Formula: vertical = 90 + 90 * cos(2π * progress)
        // This gives: 180 at progress=0, 0 at progress=0.5, 180 at progress=1
        let angle_radians = 2.0 * f64::consts::PI * day_progress;
        let vertical = 90.0 + 90.0 * angle_radians.cos();

        // Update the logic fields with the new angles
        *self.horizontal.borrow_mut() = horizontal;
        *self.vertical.borrow_mut() = vertical;

        Ok(())
    }
}

impl Display for DaylightSensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let on_str = if *self.on.borrow() == 0.0 {
            "Off"
        } else {
            "On"
        };
        let horiz = fmt_trim(*self.horizontal.borrow(), 2);
        let vert = fmt_trim(*self.vertical.borrow(), 2);
        write!(
            f,
            "DaylightSensor {{ name: \"{}\", id: {}, on: {}, horiz: {}, vert: {} }}",
            self.name, self.reference_id, on_str, horiz, vert
        )
    }
}

impl Debug for DaylightSensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
