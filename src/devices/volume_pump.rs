//! Volume pump device: moves gas between input and output networks.

use crate::{
    CableNetwork,
    atmospherics::MatterState,
    devices::{
        AtmosphericDevice, Device, DeviceAtmosphericNetworkType, LogicType,
        SimulationDeviceSettings,
        property_descriptor::{PropertyDescriptor, PropertyRegistry},
    },
    error::{SimulationError, SimulationResult},
    networks::AtmosphericNetwork,
    parser::string_to_hash,
    prop_ro, prop_rw_bool, prop_rw_clamped,
    types::{OptShared, OptWeakShared, Shared, shared},
};

use crate::conversions::fmt_trim;
use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
    sync::OnceLock,
};

/// Volume pump: moves gas between input and output networks
pub struct VolumePump {
    /// Device name
    name: String,
    /// Connected network
    network: OptWeakShared<CableNetwork>,

    /// The device reference ID
    reference_id: i32,
    /// The On state
    on: RefCell<f64>,
    /// The Setting state (volume)
    setting: RefCell<f64>,

    /// The input network
    input_network: OptWeakShared<AtmosphericNetwork>,
    /// The output network
    output_network: OptWeakShared<AtmosphericNetwork>,
}

/// Constructors for `VolumePump`.
impl VolumePump {
    /// Compile-time prefab hash constant for this device
    pub const PREFAB_HASH: i32 = string_to_hash("StructureVolumePump");

    /// Create a new `VolumePump`.
    pub fn new(settings: SimulationDeviceSettings) -> Shared<Self> {
        let name = if let Some(n) = settings.name.as_ref() {
            n.to_string()
        } else {
            Self::display_name_static().to_string()
        };

        shared(Self {
            name,
            network: None,
            setting: RefCell::new(5.0),
            on: RefCell::new(0.0),
            reference_id: settings.id.unwrap(),
            input_network: None,
            output_network: None,
        })
    }

    /// Return the prefab hash for `VolumePump`.
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }

    /// Human-readable display name
    pub fn display_name_static() -> &'static str {
        "Volume Pump"
    }

    /// Get the property registry for this device type
    #[rustfmt::skip]
    pub fn properties() -> &'static PropertyRegistry<Self> {
        use LogicType::*;
        static REGISTRY: OnceLock<PropertyRegistry<VolumePump>> = OnceLock::new();

        REGISTRY.get_or_init(|| {
            const DESCRIPTORS: &[PropertyDescriptor<VolumePump>] = &[
                prop_ro!(ReferenceId, |device, _| Ok(device.reference_id as f64)),
                prop_ro!(PrefabHash, |device, _| Ok(device.get_prefab_hash() as f64)),
                prop_ro!(NameHash, |device, _| Ok(device.get_name_hash() as f64)),
                prop_ro!(Ratio, |device, _| Ok(*device.setting.borrow() / 10.0)),
                prop_rw_bool!(On, on),
                prop_rw_clamped!(Setting, setting, 0.0, 10.0),
            ];

            PropertyRegistry::new(DESCRIPTORS)
        })
    }
}

/// `Device` trait implementation for `VolumePump` providing logic access, naming, and update behavior.
impl Device for VolumePump {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        VolumePump::prefab_hash()
    }

    fn get_name_hash(&self) -> i32 {
        string_to_hash(self.name.as_str())
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_network(&self) -> OptShared<CableNetwork> {
        self.network.as_ref().and_then(|w| w.upgrade()).clone()
    }

    fn set_network(&mut self, network: OptWeakShared<CableNetwork>) {
        self.network = network;
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

    fn supported_types(&self) -> Vec<LogicType> {
        Self::properties().supported_types()
    }

    fn update(&self, _tick: u64) -> SimulationResult<bool> {
        // Only run when device is On and Mode is enabled
        if *self.on.borrow() == 0.0 {
            return Ok(false);
        }

        let input_rc = self
            .input_network
            .as_ref()
            .and_then(|w| w.upgrade())
            .ok_or(SimulationError::RuntimeError {
                message: "VolumePump device has no input atmospheric network".to_string(),
                line: 0,
            })?;

        let output_rc = self
            .output_network
            .as_ref()
            .and_then(|w| w.upgrade())
            .ok_or(SimulationError::RuntimeError {
                message: "VolumePump device has no output atmospheric network".to_string(),
                line: 0,
            })?;

        let setting = *self.setting.borrow();

        let (input_total_volume, total_moles) = {
            let input = input_rc.borrow();
            (input.total_volume(), input.total_moles())
        };

        // Clamp setting to available volume
        let volume_to_move = setting.min(input_total_volume);

        // Proportional transfer of all matter (gases + liquids)
        if volume_to_move > 0.0 {
            let ratio = (volume_to_move / input_total_volume).clamp(0.0, 1.0);
            if ratio > 0.0 {
                let moles_to_move = total_moles * ratio;
                output_rc.borrow_mut().add_mixture(
                    &input_rc
                        .borrow_mut()
                        .remove_moles(moles_to_move, MatterState::All),
                );
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn properties() -> &'static PropertyRegistry<Self> {
        VolumePump::properties()
    }

    fn display_name_static() -> &'static str {
        VolumePump::display_name_static()
    }

    fn required_atmospheric_connections() -> Vec<DeviceAtmosphericNetworkType> {
        use DeviceAtmosphericNetworkType::*;
        vec![Input, Output]
    }

    fn as_atmospheric_device(&self) -> Option<&dyn AtmosphericDevice> {
        Some(self)
    }

    fn as_atmospheric_device_mut(&mut self) -> Option<&mut dyn AtmosphericDevice> {
        Some(self)
    }
}

impl Display for VolumePump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let on_str = if *self.on.borrow() == 0.0 {
            "Off"
        } else {
            "On"
        };
        let setting_str = fmt_trim(*self.setting.borrow(), 3);

        write!(
            f,
            "VolumePump {{ name: \"{}\", id: {}, on: {}, setting: {}",
            self.name, self.reference_id, on_str, setting_str
        )?;

        if let Some(weak) = &self.input_network
            && let Some(net) = weak.upgrade()
        {
            write!(f, ", input: {}", net.borrow())?;
        }
        if let Some(weak) = &self.output_network
            && let Some(net) = weak.upgrade()
        {
            write!(f, ", output: {}", net.borrow())?;
        }

        write!(f, " }}")
    }
}

impl Debug for VolumePump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// `AtmosphericDevice` implementation for `VolumePump` that manages input/output atmospheric network connections.
impl AtmosphericDevice for VolumePump {
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
                    "VolumePump does not support atmospheric connection {:?}",
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
