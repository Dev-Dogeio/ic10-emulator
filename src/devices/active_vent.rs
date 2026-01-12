//! Active vent device: moves gas between a pipe network and an external network

use crate::{
    CableNetwork,
    atmospherics::{MatterState, ONE_ATMOSPHERE, calculate_moles},
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

use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
    sync::OnceLock,
};

/// Pressure per tick used for mole transfer (kPa).
const PRESSURE_PER_TICK: f64 = 10.0;

/// Active vent device
pub struct ActiveVent {
    name: String,
    network: OptWeakShared<CableNetwork>,

    reference_id: i32,

    /// On state
    on: RefCell<f64>,
    /// Mode (0 = 0utward, 1 = 1nward)
    mode: RefCell<f64>,

    /// External pressure target (kPa)
    external_pressure: RefCell<f64>,
    /// Internal pressure target (kPa)
    internal_pressure: RefCell<f64>,

    input_network: OptWeakShared<AtmosphericNetwork>,
    output_network: OptWeakShared<AtmosphericNetwork>,
}

impl ActiveVent {
    pub const PREFAB_HASH: i32 = string_to_hash("StructureActiveVent");

    pub fn new(settings: SimulationDeviceSettings) -> Shared<Self> {
        let name = if let Some(n) = settings.name.as_ref() {
            n.to_string()
        } else {
            Self::display_name_static().to_string()
        };

        // Default to outward mode with external = 1 atm, internal = 0
        shared(ActiveVent {
            name,
            network: None,
            reference_id: settings.id.unwrap(),
            on: RefCell::new(0.0),
            mode: RefCell::new(0.0),
            external_pressure: RefCell::new(ONE_ATMOSPHERE),
            internal_pressure: RefCell::new(0.0),
            input_network: None,
            output_network: None,
        })
    }

    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }

    pub fn display_name_static() -> &'static str {
        "Active Vent"
    }

    #[rustfmt::skip]
    pub fn properties() -> &'static PropertyRegistry<ActiveVent> {
        use LogicType::*;
        use DeviceAtmosphericNetworkType::*;
        use crate::atmospherics::GasType::*;
        static REGISTRY: OnceLock<PropertyRegistry<ActiveVent>> = OnceLock::new();

        REGISTRY.get_or_init(|| {
            const DESCRIPTORS: &[PropertyDescriptor<ActiveVent>] = &[
                prop_ro!(ReferenceId, |device: &ActiveVent, _| Ok(device.reference_id as f64)),
                prop_ro!(PrefabHash, |device: &ActiveVent, _| Ok(device.get_prefab_hash() as f64)),
                prop_ro!(NameHash, |device: &ActiveVent, _| Ok(device.get_name_hash() as f64)),
                prop_rw_bool!(On, on),
                PropertyDescriptor::read_write(
                    Mode,
                    |device, _| Ok(*device.mode.borrow()),
                    |device, _, value| {
                        let mode = if value < 1.0 { 0.0 } else { 1.0 };
                        *device.mode.borrow_mut() = mode;
                        if mode == 0.0 {
                            // Outward: external = 1 atm, internal = 0
                            *device.external_pressure.borrow_mut() = ONE_ATMOSPHERE;
                            *device.internal_pressure.borrow_mut() = 0.0;
                        } else {
                            // Inward: external = 0, internal = 500 atm
                            *device.external_pressure.borrow_mut() = 0.0;
                            *device.internal_pressure.borrow_mut() = ONE_ATMOSPHERE * 500.0;
                        }
                        Ok(())
                    },
                ),

                // Read-only properties exposing the connected pipe (Output) atmospheric state
                prop_ro!(PressureOutput, |device, _| device.read_network_prop(Output, |net| net.pressure())),
                prop_ro!(TemperatureOutput, |device, _| device.read_network_prop(Output, |net| net.temperature())),
                prop_ro!(TotalMolesOutput, |device, _| device.read_network_prop(Output, |net| net.total_moles())),
                prop_ro!(RatioOxygenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Oxygen))),
                prop_ro!(RatioCarbonDioxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(CarbonDioxide))),
                prop_ro!(RatioNitrogenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Nitrogen))),
                prop_ro!(RatioPollutantOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Pollutant))),
                prop_ro!(RatioVolatilesOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Volatiles))),
                prop_ro!(RatioWaterOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Water))),
                prop_ro!(RatioNitrousOxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(NitrousOxide))),
                prop_ro!(CombustionOutput, |_, _| Err(SimulationError::RuntimeError { message: "CombustionOutput not implemented".to_string(), line: 0 })),

                // Read-write external/internal pressure targets
                prop_rw_clamped!(PressureExternal, external_pressure, 0.0, f64::INFINITY),
                prop_rw_clamped!(PressureInternal, internal_pressure, 0.0, f64::INFINITY),
            ];

            PropertyRegistry::new(DESCRIPTORS)
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
                    message: "ActiveVent device has no input atmospheric network".to_string(),
                    line: 0,
                },
            ),
            Output => self
                .output_network
                .as_ref()
                .and_then(|w| w.upgrade())
                .ok_or(SimulationError::RuntimeError {
                    message: "ActiveVent device has no output atmospheric network".to_string(),
                    line: 0,
                }),
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "ActiveVent does not support atmospheric connection type {:?}",
                    connection
                ),
                line: 0,
            }),
        }
    }

    /// Helper to read a property from an atmospheric network
    fn read_network_prop<T, F>(
        &self,
        connection: DeviceAtmosphericNetworkType,
        f: F,
    ) -> SimulationResult<T>
    where
        F: FnOnce(&AtmosphericNetwork) -> T,
    {
        let net = self.require_network(connection)?;
        Ok(f(&net.borrow()))
    }

    /// Set external pressure target (kPa)
    pub fn set_external_pressure(&self, p: f64) {
        *self.external_pressure.borrow_mut() = p;
    }

    /// Set internal pressure target (kPa)
    pub fn set_internal_pressure(&self, p: f64) {
        *self.internal_pressure.borrow_mut() = p;
    }

    /// Get external pressure target (kPa)
    pub fn external_pressure(&self) -> f64 {
        *self.external_pressure.borrow()
    }

    /// Get internal pressure target (kPa)
    pub fn internal_pressure(&self) -> f64 {
        *self.internal_pressure.borrow()
    }

    /// Pump gas from pipe -> world (outward). Returns true if any gas was transferred.
    fn pump_gas_to_world(
        &self,
        world: &mut AtmosphericNetwork,
        pipe: &mut AtmosphericNetwork,
        total_temperature: f64,
    ) -> bool {
        let pipe_pressure = pipe.pressure();
        let internal_p = *self.internal_pressure.borrow();
        let world_pressure = world.pressure();
        let external_p = *self.external_pressure.borrow();

        let pipe_available_moles = if pipe_pressure > internal_p {
            calculate_moles(
                pipe_pressure - internal_p,
                pipe.volume(),
                pipe.temperature(),
            )
        } else {
            0.0
        };

        // Calculate transfer pressure clamped to [0, PRESSURE_PER_TICK]
        let transfer_pressure = (external_p - world_pressure)
            .min(PRESSURE_PER_TICK)
            .max(0.0);

        let candidate_moles = if transfer_pressure > 0.0 {
            calculate_moles(transfer_pressure, world.volume(), total_temperature)
        } else {
            0.0
        };

        // Transfer minimum of what pipe can provide and what world can accept
        let transfer_moles = candidate_moles.min(pipe_available_moles);

        if transfer_moles > 0.0 {
            world.add_mixture(&pipe.remove_moles(transfer_moles, MatterState::All));
            return true;
        }

        false
    }

    /// Pump gas from world -> pipe (inward). Returns true if any gas was transferred.
    fn pump_gas_to_pipe(
        &self,
        world: &mut AtmosphericNetwork,
        pipe: &mut AtmosphericNetwork,
        total_temperature: f64,
    ) -> bool {
        let world_pressure = world.pressure();
        let external_p = *self.external_pressure.borrow();
        let pipe_pressure = pipe.pressure();
        let internal_p = *self.internal_pressure.borrow();

        // Calculate how many moles we can pull from world based on pressure delta
        let world_transfer_pressure = (world_pressure - external_p).min(PRESSURE_PER_TICK);
        let candidate_world_moles = if world_transfer_pressure > 0.0 {
            calculate_moles(world_transfer_pressure, world.volume(), total_temperature)
        } else {
            0.0
        };

        // Calculate how many moles pipe can accept to reach internal pressure
        let pipe_accept_moles = if internal_p > pipe_pressure {
            calculate_moles(internal_p - pipe_pressure, pipe.volume(), total_temperature)
        } else {
            0.0
        };

        // Transfer the minimum of what world can provide and what pipe can accept
        let transfer_moles = candidate_world_moles.min(pipe_accept_moles);

        if transfer_moles > 0.0 {
            // Only transfer gas (not liquids) into pipe
            pipe.add_mixture(&world.remove_moles(transfer_moles, MatterState::Gas));
            // Return change in world pressure (how much was removed)
            return true;
        }

        false
    }
}

impl Device for ActiveVent {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        ActiveVent::prefab_hash()
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

    fn set_network(&mut self, network: OptWeakShared<CableNetwork>) -> SimulationResult<()> {
        self.network = network;
        Ok(())
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

    fn properties() -> &'static PropertyRegistry<Self> {
        ActiveVent::properties()
    }

    fn display_name_static() -> &'static str {
        ActiveVent::display_name_static()
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

    fn update(&self, _tick: u64) -> SimulationResult<bool> {
        // Only operate when On
        if *self.on.borrow() == 0.0 {
            return Ok(false);
        }

        let world_rc = self.require_network(DeviceAtmosphericNetworkType::Input)?;
        let pipe_rc = self.require_network(DeviceAtmosphericNetworkType::Output)?;

        let (input_moles, output_moles) = {
            let i = pipe_rc.borrow();
            let o = world_rc.borrow();
            (i.total_moles(), o.total_moles())
        };

        let total_temperature = if (input_moles + output_moles) > 0.0 {
            (pipe_rc.borrow().temperature() * input_moles
                + world_rc.borrow().temperature() * output_moles)
                / (input_moles + output_moles)
        } else {
            pipe_rc.borrow().temperature()
        };

        if *self.mode.borrow() == 0.0 {
            // Outward
            let did_change = self.pump_gas_to_world(
                &mut world_rc.borrow_mut(),
                &mut pipe_rc.borrow_mut(),
                total_temperature,
            );
            Ok(did_change)
        } else {
            // Inward
            let did_change = self.pump_gas_to_pipe(
                &mut world_rc.borrow_mut(),
                &mut pipe_rc.borrow_mut(),
                total_temperature,
            );
            Ok(did_change)
        }
    }
}

impl Debug for ActiveVent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {{ name: \"{}\", id: {} }}",
            Self::display_name_static(),
            self.name,
            self.reference_id
        )
    }
}

impl Display for ActiveVent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {{ name: \"{}\", id: {} }}",
            Self::display_name_static(),
            self.name,
            self.reference_id
        )
    }
}

impl AtmosphericDevice for ActiveVent {
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
                    "ActiveVent does not support atmospheric connection {:?}",
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
