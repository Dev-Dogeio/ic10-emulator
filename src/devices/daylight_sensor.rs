//! Daylight Sensor device - tracks sun position
//!
//! The daylight sensor provides:
//! - Horizontal angle (azimuth): 0-360 degrees, representing compass direction
//! - Vertical angle (altitude): 0-180 degrees, 0 when sun is directly overhead (zenith),
//!   180 when sun is on the opposite side of the planet (nadir)
//!
//! Note: Doesn't match the game, just a simple simulation for testing purposes.

use crate::{
    CableNetwork,
    devices::{Device, DeviceBase, LogicType, SimulationSettings},
    error::{IC10Error, IC10Result},
};
use std::{cell::RefCell, rc::Rc};

/// Daylight Sensor - tracks the sun's position in the sky
#[derive(Debug)]
pub struct DaylightSensor {
    base: DeviceBase,
    /// Sensor simulation settings
    settings: SimulationSettings,
}

impl DaylightSensor {
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Self {
        let base = DeviceBase::new(
            "Daylight Sensor".to_string(),
            "StructureDaylightSensor".to_string(),
        );

        base.logic_types.borrow_mut().horizontal = Some(0.0);
        base.logic_types.borrow_mut().vertical = Some(0.0);

        Self {
            base,
            settings: simulation_settings.unwrap_or_default(),
        }
    }

    /// Get the current horizontal angle
    pub fn horizontal(&self) -> f64 {
        self.base.logic_types.borrow().horizontal.unwrap()
    }

    /// Get the current vertical angle
    pub fn vertical(&self) -> f64 {
        self.base.logic_types.borrow().vertical.unwrap()
    }
}

impl Device for DaylightSensor {
    fn get_id(&self) -> i32 {
        self.base.logic_types.borrow().reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        self.base.logic_types.borrow().prefab_hash
    }

    fn get_name_hash(&self) -> i32 {
        self.base.logic_types.borrow().name_hash
    }

    fn get_name(&self) -> &str {
        &self.base.name
    }

    fn get_network(&self) -> Option<Rc<RefCell<CableNetwork>>> {
        self.base.network.clone()
    }

    fn set_network(&mut self, network: Option<Rc<RefCell<CableNetwork>>>) {
        self.base.network = network;
    }

    fn set_name(&mut self, name: &str) {
        self.base.set_name(name.to_string());
    }

    fn can_read(&self, logic_type: LogicType) -> bool {
        matches!(
            logic_type,
            LogicType::PrefabHash
                | LogicType::ReferenceId
                | LogicType::NameHash
                | LogicType::Horizontal
                | LogicType::Vertical
        )
    }

    fn can_write(&self, _logic_type: LogicType) -> bool {
        false // Sensors are read-only
    }

    fn read(&self, logic_type: LogicType) -> IC10Result<f64> {
        match logic_type {
            LogicType::PrefabHash => Ok(self.base.logic_types.borrow().prefab_hash as f64),
            LogicType::ReferenceId => Ok(self.base.logic_types.borrow().reference_id as f64),
            LogicType::NameHash => Ok(self.base.logic_types.borrow().name_hash as f64),
            LogicType::Horizontal => {
                self.base
                    .logic_types
                    .borrow()
                    .horizontal
                    .ok_or(IC10Error::RuntimeError {
                        message: "Horizontal value not set".to_string(),
                        line: 0,
                    })
            }
            LogicType::Vertical => {
                self.base
                    .logic_types
                    .borrow()
                    .vertical
                    .ok_or(IC10Error::RuntimeError {
                        message: "Vertical value not set".to_string(),
                        line: 0,
                    })
            }
            _ => Err(IC10Error::RuntimeError {
                message: format!(
                    "Daylight sensor does not support reading logic type {:?}",
                    logic_type
                ),
                line: 0,
            }),
        }
    }

    fn write(&self, logic_type: LogicType, value: f64) -> IC10Result<()> {
        match logic_type {
            LogicType::Horizontal => {
                self.base.logic_types.borrow_mut().horizontal = Some(value);
                Ok(())
            }
            LogicType::Vertical => {
                self.base.logic_types.borrow_mut().vertical = Some(value);
                Ok(())
            }
            _ => Err(IC10Error::RuntimeError {
                message: format!(
                    "Daylight sensor does not support writing logic type {:?}",
                    logic_type
                ),
                line: 0,
            }),
        }
    }

    fn update(&self, tick: u64) {
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
        let angle_radians = 2.0 * std::f64::consts::PI * day_progress;
        let vertical = 90.0 + 90.0 * angle_radians.cos();

        // Update the logic types with the new angles
        self.base
            .logic_types
            .borrow_mut()
            .set(LogicType::Horizontal, horizontal);
        self.base
            .logic_types
            .borrow_mut()
            .set(LogicType::Vertical, vertical);
    }
}

impl Default for DaylightSensor {
    fn default() -> Self {
        Self::new(None)
    }
}
