//! Logic Memory device - stores a single value
//!
//! The Logic Memory provides:
//! - Setting (read/write): stores a single numeric value

use crate::{
    CableNetwork,
    devices::{Device, DeviceBase, LogicType, SimulationSettings},
    error::{IC10Error, IC10Result},
    types::OptShared,
};

/// Logic Memory - stores a single value that can be read and written
#[derive(Debug)]
pub struct LogicMemory {
    base: DeviceBase,
    /// Simulation settings
    #[allow(dead_code)]
    settings: SimulationSettings,
}

impl LogicMemory {
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Self {
        let base = DeviceBase::new(
            "Logic Memory".to_string(),
            "StructureLogicMemory".to_string(),
        );

        base.logic_types.borrow_mut().setting = Some(0.0);
        Self {
            base,
            settings: simulation_settings.unwrap_or_default(),
        }
    }
}

impl Device for LogicMemory {
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

    fn get_network(&self) -> OptShared<CableNetwork> {
        self.base.network.clone()
    }

    fn set_network(&mut self, network: OptShared<CableNetwork>) {
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
                | LogicType::Setting
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(logic_type, LogicType::Setting)
    }

    fn read(&self, logic_type: LogicType) -> IC10Result<f64> {
        match logic_type {
            LogicType::PrefabHash => Ok(self.base.logic_types.borrow().prefab_hash as f64),
            LogicType::ReferenceId => Ok(self.base.logic_types.borrow().reference_id as f64),
            LogicType::NameHash => Ok(self.base.logic_types.borrow().name_hash as f64),
            LogicType::Setting => {
                self.base
                    .logic_types
                    .borrow()
                    .setting
                    .ok_or(IC10Error::RuntimeError {
                        message: "Setting value not set".to_string(),
                        line: 0,
                    })
            }
            _ => Err(IC10Error::RuntimeError {
                message: format!("Logic Memory does not support reading logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }

    fn write(&self, logic_type: LogicType, value: f64) -> IC10Result<()> {
        match logic_type {
            LogicType::Setting => {
                self.base.logic_types.borrow_mut().setting = Some(value);
                Ok(())
            }
            _ => Err(IC10Error::RuntimeError {
                message: format!("Logic Memory does not support writing logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }

    fn get_supported_logic_types(&self) -> Vec<LogicType> {
        vec![LogicType::Setting]
    }
}

impl Default for LogicMemory {
    fn default() -> Self {
        Self::new(None)
    }
}
