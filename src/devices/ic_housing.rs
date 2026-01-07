//! IC housing device: hosts IC10 chips and exposes registers/memory.

use crate::conversions::fmt_trim;
use crate::{
    CableNetwork, allocate_global_id,
    constants::STACK_SIZE,
    devices::{
        ChipSlot, Device, ICHostDevice, ICHostDeviceMemoryOverride, LogicType, SimulationSettings,
    },
    error::{SimulationError, SimulationResult},
    parser::string_to_hash,
    types::{OptShared, Shared, shared},
};

use std::cell::RefCell;
use std::fmt::{Debug, Display};

/// IC housing: holds an IC10 chip and exposes host interfaces
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

/// Constructors and helpers
impl ICHousing {
    /// Compile-time prefab hash constant for this device
    pub const PREFAB_HASH: i32 = string_to_hash("StructureCircuitHousing");

    /// Create a new `ICHousing`
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

    /// Prefab hash for `ICHousing`
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }
}

/// `Device` trait implementation for `ICHousing` providing logic access, naming, and chip hosting.
impl Device for ICHousing {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        ICHousing::prefab_hash()
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
        let old_name_hash = string_to_hash(self.name.as_str());
        self.name = name.to_string();

        if let Some(network) = &self.network {
            network.borrow_mut().update_device_name(
                self.reference_id,
                old_name_hash,
                string_to_hash(name),
            );
        }
    }

    fn clear_internal_references(&mut self) {
        self.chip_host.borrow_mut().clear_internal_references();
    }

    fn can_read(&self, logic_type: LogicType) -> bool {
        matches!(
            logic_type,
            LogicType::PrefabHash
                | LogicType::ReferenceId
                | LogicType::NameHash
                | LogicType::Setting
                | LogicType::On
                | LogicType::StackSize
                | LogicType::LineNumber
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(
            logic_type,
            LogicType::Setting | LogicType::On | LogicType::LineNumber
        )
    }

    fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        match logic_type {
            LogicType::PrefabHash => Ok(self.get_prefab_hash() as f64),
            LogicType::ReferenceId => Ok(self.reference_id as f64),
            LogicType::NameHash => Ok(self.get_name_hash() as f64),
            LogicType::Setting => Ok(*self.setting.borrow()),
            LogicType::On => Ok(*self.on.borrow()),
            LogicType::StackSize => Ok(STACK_SIZE as f64),
            LogicType::LineNumber => {
                if let Some(chip) = self.chip_slot().borrow().get_chip() {
                    Ok(chip.get_pc() as f64)
                } else {
                    Ok(0.0)
                }
            }
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
            LogicType::LineNumber => {
                if value.is_nan() || value.is_infinite() || value < 0.0 {
                    return Err(SimulationError::RuntimeError {
                        message: "Invalid line number".to_string(),
                        line: 0,
                    });
                }

                let pc = value as usize;
                if let Some(chip) = self.chip_slot().borrow().get_chip() {
                    chip.set_pc(pc);
                    Ok(())
                } else {
                    Err(SimulationError::RuntimeError {
                        message: "No chip installed".to_string(),
                        line: 0,
                    })
                }
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

    fn as_ic_host_device(&mut self) -> Option<&mut dyn ICHostDevice> {
        Some(self)
    }
}

/// `ICHostDevice` helpers for `ICHousing` (chip slot and memory access).
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

impl Display for ICHousing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let on_str = if *self.on.borrow() == 0.0 {
            "Off"
        } else {
            "On"
        };
        let setting_str = fmt_trim(*self.setting.borrow(), 3);
        write!(
            f,
            "ICHousing {{ name: \"{}\", id: {}, on: {}, setting: {} }}",
            self.name, self.reference_id, on_str, setting_str
        )
    }
}

impl Debug for ICHousing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
