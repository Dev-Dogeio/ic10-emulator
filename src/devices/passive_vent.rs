//! Passive vent: equalizes gas between to atmospheres

use crate::{
    CableNetwork,
    devices::{
        AtmosphericDevice, Device, DeviceAtmosphericNetworkType, LogicType,
        SimulationDeviceSettings, property_descriptor::PropertyRegistry,
    },
    error::{SimulationError, SimulationResult},
    networks::AtmosphericNetwork,
    parser::string_to_hash,
    types::{OptShared, OptWeakShared, Shared, shared},
};

use std::{
    fmt::{Debug, Display},
    rc::Rc,
    sync::OnceLock,
};

/// Passive vent device
pub struct PassiveVent {
    name: String,

    reference_id: i32,

    input_network: OptWeakShared<AtmosphericNetwork>,
    output_network: OptWeakShared<AtmosphericNetwork>,
}

impl PassiveVent {
    pub const PREFAB_HASH: i32 = string_to_hash("StructurePassiveVent");

    pub fn new(settings: SimulationDeviceSettings) -> Shared<Self> {
        let name = settings
            .name
            .clone()
            .unwrap_or_else(|| Self::display_name_static().to_string());
        shared(PassiveVent {
            name,
            reference_id: settings.id.unwrap(),
            input_network: None,
            output_network: None,
        })
    }

    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }

    pub fn display_name_static() -> &'static str {
        "Passive Vent"
    }

    #[rustfmt::skip]
    pub fn properties() -> &'static PropertyRegistry<Self> {
        static REGISTRY: OnceLock<PropertyRegistry<PassiveVent>> = OnceLock::new();

        REGISTRY.get_or_init(|| {
            PropertyRegistry::new(&[])
        })
    }

    fn require_network(
        &self,
        connection: DeviceAtmosphericNetworkType,
    ) -> SimulationResult<Shared<AtmosphericNetwork>> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Input => self.input_network.as_ref().and_then(|w| w.upgrade()).ok_or(
                SimulationError::RuntimeError {
                    message: "PassiveVent device has no input atmospheric network".to_string(),
                    line: 0,
                },
            ),
            Output => self
                .output_network
                .as_ref()
                .and_then(|w| w.upgrade())
                .ok_or(SimulationError::RuntimeError {
                    message: "PassiveVent device has no output atmospheric network".to_string(),
                    line: 0,
                }),
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "PassiveVent does not support atmospheric connection type {:?}",
                    connection
                ),
                line: 0,
            }),
        }
    }
}

/// `Device` trait implementation for `PassiveVent`
impl Device for PassiveVent {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        PassiveVent::prefab_hash()
    }

    fn get_name_hash(&self) -> i32 {
        string_to_hash(self.name.as_str())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_network(&self) -> OptShared<CableNetwork> {
        None
    }

    fn set_network(&mut self, _network: OptWeakShared<CableNetwork>) -> SimulationResult<()> {
        Err(SimulationError::RuntimeError {
            message: "PassiveVent cannot be connected to a cable network".to_string(),
            line: 0,
        })
    }

    fn rename(&mut self, name: &str) {
        let old_name_hash = self.get_name_hash();
        self.name = name.to_string();
        if let Some(net_rc) = self.get_network() {
            net_rc.borrow_mut().update_device_name(
                self.reference_id,
                old_name_hash,
                string_to_hash(name),
            );
        }
    }

    fn can_read(&self, logic_type: LogicType) -> bool {
        Self::properties().can_read(logic_type)
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        Self::properties().can_write(logic_type)
    }

    fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        Self::properties().read(self, logic_type)
    }

    fn write(&self, logic_type: LogicType, value: f64) -> SimulationResult<()> {
        Self::properties().write(self, logic_type, value)
    }

    fn update(&self, _tick: u64) -> SimulationResult<bool> {
        let input_rc = self.require_network(DeviceAtmosphericNetworkType::Input)?;
        let output_rc = self.require_network(DeviceAtmosphericNetworkType::Output)?;

        let in_pressure = input_rc.borrow().pressure();
        let out_pressure = output_rc.borrow().pressure();

        // If pressures are already equal (within tiny epsilon), do nothing
        if (in_pressure - out_pressure).abs() < 1e-6 {
            return Ok(false);
        }

        // Equalize gas between environment and output network (opening the vent)
        // This modifies both networks to reach equilibrium like an open connection
        input_rc
            .borrow_mut()
            .equalize_with(&mut output_rc.borrow_mut());

        Ok(true)
    }

    fn supported_types(&self) -> Vec<LogicType> {
        Self::properties().supported_types()
    }

    fn properties() -> &'static PropertyRegistry<Self> {
        PassiveVent::properties()
    }

    fn display_name_static() -> &'static str {
        PassiveVent::display_name_static()
    }

    fn required_atmospheric_connections() -> Vec<DeviceAtmosphericNetworkType> {
        use DeviceAtmosphericNetworkType::*;
        vec![Input, Output]
    }

    fn supports_cable_network() -> bool
    where
        Self: Sized,
    {
        false
    }

    fn as_atmospheric_device(&self) -> Option<&dyn AtmosphericDevice> {
        Some(self)
    }

    fn as_atmospheric_device_mut(&mut self) -> Option<&mut dyn AtmosphericDevice> {
        Some(self)
    }
}

impl Display for PassiveVent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PassiveVent {{ name: \"{}\", id: {} }}",
            self.name, self.reference_id,
        )
    }
}

impl Debug for PassiveVent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl AtmosphericDevice for PassiveVent {
    fn set_atmospheric_network(
        &mut self,
        connection: DeviceAtmosphericNetworkType,
        network: OptShared<AtmosphericNetwork>,
    ) -> SimulationResult<()> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Input => {
                self.input_network = network.as_ref().map(Rc::downgrade);
                Ok(())
            }
            Output => {
                self.output_network = network.as_ref().map(Rc::downgrade);
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "PassiveVent does not support atmospheric connection {:?}",
                    connection
                ),
                line: 0,
            }),
        }
    }

    fn get_atmospheric_network(
        &self,
        connection: DeviceAtmosphericNetworkType,
    ) -> OptShared<AtmosphericNetwork> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Input => self.input_network.as_ref().and_then(|w| w.upgrade()),
            Output => self.output_network.as_ref().and_then(|w| w.upgrade()),
            _ => None,
        }
    }
}
