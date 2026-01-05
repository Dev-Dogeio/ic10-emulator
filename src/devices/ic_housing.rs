//! IC Housing device - a housing unit for integrated circuits
//!
//! The IC Housing provides:
//! - 6 device pins (d0-d5) for connecting to other devices on the network
//! - 18 registers (r0-r15 general purpose, r16=sp stack pointer, r17=ra return address)
//! - 512 stack memory entries

use crate::{
    CableNetwork, allocate_global_id,
    constants::IC_HOUSING_PREFAB_HASH,
    devices::{
        ChipSlot, Device, ICHostDevice, ICHostDeviceMemoryOverride, LogicType, SimulationSettings,
    },
    error::{SimulationError, SimulationResult},
    parser::string_to_hash,
    types::{OptShared, Shared, shared},
};
use std::cell::RefCell;

/// IC Housing - a housing unit that holds an IC10 chip and connects to devices
/// that can reference ANY device on the cable network by its reference ID.
#[derive(Debug)]
pub struct ICHousing {
    /// Device name
    name: String,
    /// Connected network
    network: OptShared<CableNetwork>,

    /// The device reference ID
    reference_id: i32,
    /// The On state
    on: RefCell<f64>,
    /// The Setting state
    setting: RefCell<f64>,

    /// Chip hosting helper (shared so it can be referenced by chips)
    chip_host: Shared<ChipSlot>,

    /// Simulation settings
    settings: SimulationSettings,
}

impl ICHousing {
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Shared<Self> {
        let s = shared(Self {
            name: "IC Housing".to_string(),
            network: None,
            setting: RefCell::new(0.0),
            on: RefCell::new(1.0),
            reference_id: allocate_global_id(),
            chip_host: ChipSlot::new(6),
            settings: simulation_settings.unwrap_or_default(),
        });

        s.borrow()
            .chip_host
            .borrow_mut()
            .set_host_device(Some(s.clone()));

        s
    }
}

impl Device for ICHousing {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        IC_HOUSING_PREFAB_HASH
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

    fn set_name(&mut self, name: &str) {
        // Update the network index if connected
        let old_name_hash = string_to_hash(&self.name);
        self.name = name.to_string();
        let new_name_hash = string_to_hash(&self.name);

        if let Some(network) = &self.network {
            let reference_id = self.reference_id;
            network
                .borrow_mut()
                .update_device_name(reference_id, old_name_hash, new_name_hash);
        }
    }

    fn can_read(&self, logic_type: LogicType) -> bool {
        matches!(
            logic_type,
            LogicType::PrefabHash
                | LogicType::ReferenceId
                | LogicType::NameHash
                | LogicType::Setting
                | LogicType::On
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(logic_type, LogicType::Setting | LogicType::On)
    }

    fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        match logic_type {
            LogicType::PrefabHash => Ok(self.get_prefab_hash() as f64),
            LogicType::ReferenceId => Ok(self.reference_id as f64),
            LogicType::NameHash => Ok(self.get_name_hash() as f64),
            LogicType::Setting => Ok(*self.setting.borrow()),
            LogicType::On => Ok(*self.on.borrow()),
            _ => Err(SimulationError::RuntimeError {
                message: format!("IC Housing does not support reading logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }

    fn write(&self, logic_type: LogicType, value: f64) -> SimulationResult<()> {
        match logic_type {
            LogicType::Setting => {
                *self.setting.borrow_mut() = value;
                Ok(())
            }
            LogicType::On => {
                *self.on.borrow_mut() = if value < 1.0 { 0.0 } else { 1.0 };
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!("IC Housing does not support writing logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }

    fn run(&self) -> SimulationResult<()> {
        if *self.on.borrow() != 0.0 {
            ICHostDevice::run(self)?;
        }

        Ok(())
    }

    fn get_memory(&self, index: usize) -> SimulationResult<f64> {
        ICHostDevice::get_memory(self, index)
    }

    fn set_memory(&self, index: usize, value: f64) -> SimulationResult<()> {
        ICHostDevice::set_memory(self, index, value)
    }

    fn clear(&self) -> SimulationResult<()> {
        ICHostDevice::clear(self)
    }
}

impl ICHostDevice for ICHousing {
    fn ichost_get_id(&self) -> i32 {
        self.reference_id
    }

    fn chip_slot(&self) -> Shared<ChipSlot> {
        self.chip_host.clone()
    }

    fn max_instructions_per_tick(&self) -> usize {
        self.settings.max_instructions_per_tick
    }
}

impl ICHostDeviceMemoryOverride for ICHousing {}
