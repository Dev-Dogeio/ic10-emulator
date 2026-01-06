//! Volume pump device - moves gas between an input and output atmospheric network based on the setting volume.

use crate::{
    CableNetwork, allocate_global_id,
    atmospherics::IDEAL_GAS_CONSTANT,
    devices::{AtmosphericDevice, Device, FilterConnectionType, LogicType, SimulationSettings},
    error::{SimulationError, SimulationResult},
    networks::AtmosphericNetwork,
    parser::string_to_hash,
    types::{OptShared, Shared, shared},
};

use std::cell::RefCell;

/// Volume pump device - moves gas between an input and output atmospheric network.
#[derive(Debug)]
pub struct VolumePump {
    /// Device name
    name: String,
    /// Connected network
    network: OptShared<CableNetwork>,

    /// The device reference ID
    reference_id: i32,
    /// The On state
    on: RefCell<f64>,
    /// The Setting state (volume)
    setting: RefCell<f64>,

    /// The input network
    input_network: OptShared<AtmosphericNetwork>,
    /// The output network
    output_network: OptShared<AtmosphericNetwork>,

    /// Simulation settings
    #[allow(dead_code)]
    settings: SimulationSettings,
}

impl VolumePump {
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Shared<Self> {
        shared(Self {
            name: "Volume Pump".to_string(),
            network: None,
            setting: RefCell::new(5.0),
            on: RefCell::new(0.0),
            reference_id: allocate_global_id(),
            settings: simulation_settings.unwrap_or_default(),
            input_network: None,
            output_network: None,
        })
    }
}

impl Device for VolumePump {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        string_to_hash("StructureVolumePump")
    }

    fn get_name_hash(&self) -> i32 {
        string_to_hash(self.name.as_str())
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
            LogicType::ReferenceId
                | LogicType::PrefabHash
                | LogicType::NameHash
                | LogicType::Ratio
                | LogicType::On
                | LogicType::Setting
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(logic_type, LogicType::On | LogicType::Setting)
    }

    #[rustfmt::skip]
    fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        match logic_type {
            LogicType::ReferenceId => Ok(self.reference_id as f64),
            LogicType::PrefabHash => Ok(self.get_prefab_hash() as f64),
            LogicType::NameHash => Ok(self.get_name_hash() as f64),
            LogicType::Ratio => Ok(*self.setting.borrow() / 10.0),
            LogicType::On => Ok(*self.on.borrow()),
            LogicType::Setting => Ok(*self.setting.borrow()),

            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "VolumePump does not support reading logic type {logic_type:?}"
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
            LogicType::Setting => {
                *self.setting.borrow_mut() = value.clamp(0.0, 10.0);
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!("VolumePump does not support writing logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }

    fn update(&self, _tick: u64) -> SimulationResult<()> {
        // Only run when device is On and Mode is enabled
        if *self.on.borrow() == 0.0 {
            return Ok(());
        }

        let input_rc = self
            .input_network
            .as_ref()
            .ok_or(SimulationError::RuntimeError {
                message: "VolumePump device has no input atmospheric network".to_string(),
                line: 0,
            })?;

        let output_rc = self
            .output_network
            .as_ref()
            .ok_or(SimulationError::RuntimeError {
                message: "VolumePump device has no output atmospheric network".to_string(),
                line: 0,
            })?;

        // n = PV/RT
        let mols = input_rc.borrow().pressure() * *self.setting.borrow()
            / (IDEAL_GAS_CONSTANT * input_rc.borrow().temperature());

        output_rc
            .borrow_mut()
            .add_mixture(&input_rc.borrow_mut().remove_moles(mols));

        Ok(())
    }
}

impl AtmosphericDevice for VolumePump {
    fn set_atmospheric_network(
        &mut self,
        connection: FilterConnectionType,
        network: OptShared<AtmosphericNetwork>,
    ) -> SimulationResult<()> {
        use FilterConnectionType::*;
        match connection {
            Input => {
                self.input_network = network;
                Ok(())
            }
            Output => {
                self.output_network = network;
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "VolumePump does not support atmospheric connection {:?}",
                    connection
                ),
                line: 0,
            }),
        }
    }
}
