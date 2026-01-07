//! Logic memory device: stores a single numeric value.

use std::cell::RefCell;
use std::fmt::{Debug, Display};

use crate::conversions::fmt_trim;
use crate::{
    CableNetwork, allocate_global_id,
    devices::{Device, LogicType, SimulationSettings},
    error::{SimulationError, SimulationResult},
    parser::string_to_hash,
    types::{OptShared, Shared, shared},
};

pub struct LogicMemory {
    /// Device name
    name: String,
    /// Connected network
    network: OptShared<CableNetwork>,

    /// Device reference ID
    reference_id: i32,
    /// Stored setting value
    setting: RefCell<f64>,

    /// Simulation settings
    #[allow(dead_code)]
    settings: SimulationSettings,
}

/// Constructors and helpers
impl LogicMemory {
    /// Compile-time prefab hash constant for this device
    pub const PREFAB_HASH: i32 = string_to_hash("StructureLogicMemory");

    /// Create a new `LogicMemory`
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Shared<Self> {
        shared(Self {
            name: "Logic Memory".to_string(),
            network: None,
            setting: RefCell::new(0.0),
            reference_id: allocate_global_id(),
            settings: simulation_settings.unwrap_or_default(),
        })
    }

    /// Prefab hash for `LogicMemory`
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }
}

/// `Device` trait implementation for `LogicMemory` providing memory access helpers for hosted ICs.
impl Device for LogicMemory {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        LogicMemory::prefab_hash()
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

    fn can_read(&self, logic_type: LogicType) -> bool {
        matches!(
            logic_type,
            LogicType::PrefabHash
                | LogicType::ReferenceId
                | LogicType::NameHash
                | LogicType::Setting
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(logic_type, LogicType::Setting)
    }

    fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        match logic_type {
            LogicType::PrefabHash => Ok(self.get_prefab_hash() as f64),
            LogicType::ReferenceId => Ok(self.reference_id as f64),
            LogicType::NameHash => Ok(self.get_name_hash() as f64),
            LogicType::Setting => Ok(*self.setting.borrow()),
            _ => Err(SimulationError::RuntimeError {
                message: format!("Logic Memory does not support reading logic type {logic_type:?}"),
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
            _ => Err(SimulationError::RuntimeError {
                message: format!("Logic Memory does not support writing logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }
}

impl Display for LogicMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let setting = fmt_trim(*self.setting.borrow(), 3);
        write!(
            f,
            "LogicMemory {{ name: \"{}\", id: {}, setting: {} }}",
            self.name, self.reference_id, setting
        )
    }
}

impl Debug for LogicMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
