//! AirConditioner device: transfers heat and gas between atmospheric networks.

use crate::{
    CableNetwork,
    atmospherics::{CELSIUS_TO_KELVIN, GasType, MatterState, ONE_ATMOSPHERE, calculate_moles},
    constants::DEFAULT_MAX_INSTRUCTIONS_PER_TICK,
    devices::{
        AtmosphericDevice, ChipSlot, Device, DeviceAtmosphericNetworkType, ICHostDevice,
        ICHostDeviceMemoryOverride, LogicType, SimulationDeviceSettings,
        property_descriptor::{PropertyDescriptor, PropertyRegistry},
    },
    error::{SimulationError, SimulationResult},
    networks::AtmosphericNetwork,
    parser::string_to_hash,
    prop_ro, prop_rw_bool, prop_rw_clamped,
    types::{OptShared, OptWeakShared, Shared, shared},
};

use crate::animation_curve::AnimationCurve;
use crate::conversions::fmt_trim;
use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    rc::Rc,
    sync::{Arc, OnceLock},
};

/// Pressure per tick used for mole transfer (kPa).
pub(crate) const PRESSURE_PER_TICK: f64 = 750.0;

/// Internal buffer volume in litres
const INTERNAL_VOLUME_LITRES: f64 = 100.0;

/// Energy coefficient
const ENERGY_COEFFICIENT: f64 = 14000.0;

/// AirConditioner device: transfers heat and gas between networks
pub struct AirConditioner {
    /// Device name
    name: String,
    /// Connected network
    network: OptWeakShared<CableNetwork>,

    /// The device reference ID
    reference_id: i32,
    /// The On state
    on: RefCell<f64>,
    /// The Mode state (0 = off, 1 = on)
    mode: RefCell<f64>,
    /// The Setting state (target temperature)
    setting: RefCell<f64>,

    /// The input network
    input_network: OptWeakShared<AtmosphericNetwork>,
    /// The output network
    output_network: OptWeakShared<AtmosphericNetwork>,
    /// The waste network
    waste_network: OptWeakShared<AtmosphericNetwork>,
    /// Internal buffer used to hold transferred gas
    internal: Shared<AtmosphericNetwork>,

    /// Last computed temperature differential efficiency
    temperature_differential_efficiency: RefCell<f64>,
    /// Last computed operational temperature limitor
    operational_temperature_limitor: RefCell<f64>,
    /// Last computed optimal pressure scalar
    optimal_pressure_scalar: RefCell<f64>,
    /// Last processed mole amount
    processed_moles: RefCell<f64>,
    /// Energy moved in last tick (J)
    energy_moved: RefCell<f64>,

    /// Temperature delta efficiency curve
    temperature_delta_curve: Arc<AnimationCurve>,
    /// Operating temperature efficiency curve
    input_and_waste_curve: Arc<AnimationCurve>,

    /// Max instructions an installed IC can execute per tick
    max_instructions_per_tick: usize,

    /// Chip hosting helper (slot 0 may hold an IC10 chip)
    chip_host: Shared<ChipSlot>,
}

/// Constructors and helpers
impl AirConditioner {
    /// Compile-time prefab hash constant for this device
    pub const PREFAB_HASH: i32 = string_to_hash("StructureAirConditioner");

    /// Create a new `AirConditioner`.
    pub fn new(settings: SimulationDeviceSettings) -> Shared<Self> {
        // Load curves once and share them across instances
        static TEMPERATURE_DELTA_CURVE: OnceLock<Arc<AnimationCurve>> = OnceLock::new();
        static INPUT_AND_WASTE_CURVE: OnceLock<Arc<AnimationCurve>> = OnceLock::new();

        let temperature_delta_curve = Arc::clone(TEMPERATURE_DELTA_CURVE.get_or_init(|| {
            Arc::new(
                AnimationCurve::from_json(include_str!(
                    "../curves/AirConditioner/TemperatureDeltaEfficiency.json"
                ))
                .expect("Failed to parse temperature delta curve"),
            )
        }));

        let input_and_waste_curve = Arc::clone(INPUT_AND_WASTE_CURVE.get_or_init(|| {
            Arc::new(
                AnimationCurve::from_json(include_str!(
                    "../curves/AirConditioner/OperationalTemperatureEfficiency.json"
                ))
                .expect("Failed to parse operational temperature curve"),
            )
        }));

        let internal = if let Some(net) = settings.internal_atmospheric_network.as_ref() {
            net.clone()
        } else {
            AtmosphericNetwork::new(INTERNAL_VOLUME_LITRES)
        };

        let name = if let Some(n) = settings.name.as_ref() {
            n.to_string()
        } else {
            Self::display_name_static().to_string()
        };

        let max_instructions_per_tick = settings
            .max_instructions_per_tick
            .unwrap_or(DEFAULT_MAX_INSTRUCTIONS_PER_TICK);

        let s = shared(Self {
            name,
            network: None,
            setting: RefCell::new(20.0),
            on: RefCell::new(1.0),
            mode: RefCell::new(0.0),
            reference_id: settings.id.unwrap(),
            max_instructions_per_tick,
            input_network: None,
            output_network: None,
            waste_network: None,
            internal,
            processed_moles: RefCell::new(0.0),
            temperature_differential_efficiency: RefCell::new(0.0),
            operational_temperature_limitor: RefCell::new(0.0),
            optimal_pressure_scalar: RefCell::new(0.0),
            energy_moved: RefCell::new(0.0),
            temperature_delta_curve,
            input_and_waste_curve,
            chip_host: ChipSlot::new(2),
        });

        s.borrow()
            .chip_host
            .borrow_mut()
            .set_host_device(Some(s.clone()));

        s
    }

    /// Energy moved in the last tick
    pub fn energy_moved_last_tick(&self) -> f64 {
        *self.energy_moved.borrow()
    }

    /// Processed moles in the last tick
    pub fn processed_moles_last_tick(&self) -> f64 {
        *self.processed_moles.borrow()
    }

    /// Get the optimal pressure scalar
    pub fn optimal_pressure_scalar_last_tick(&self) -> f64 {
        *self.optimal_pressure_scalar.borrow()
    }

    /// Get the temperature differential efficiency
    pub fn temperature_differential_efficiency_last_tick(&self) -> f64 {
        *self.temperature_differential_efficiency.borrow()
    }

    /// Get the operational temperature limitor efficiency
    pub fn operational_temperature_limitor_last_tick(&self) -> f64 {
        *self.operational_temperature_limitor.borrow()
    }

    /// Get the temperature delta curve
    pub fn get_temperature_delta_curve(&self) -> Arc<AnimationCurve> {
        Arc::clone(&self.temperature_delta_curve)
    }

    /// Get the input and waste operational efficiency curve
    pub fn get_input_and_waste_curve(&self) -> Arc<AnimationCurve> {
        Arc::clone(&self.input_and_waste_curve)
    }

    /// Return the prefab hash for `AirConditioner`.
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }

    /// Human-readable display name
    pub fn display_name_static() -> &'static str {
        "Air Conditioner"
    }

    /// Get the property registry for this device type
    #[rustfmt::skip]
    pub fn properties() -> &'static PropertyRegistry<Self> {
        use LogicType::*;
        use DeviceAtmosphericNetworkType::*;
        use GasType::*;
        static REGISTRY: OnceLock<PropertyRegistry<AirConditioner>> = OnceLock::new();

        REGISTRY.get_or_init(|| {
            const DESCRIPTORS: &[PropertyDescriptor<AirConditioner>] = &[
                prop_ro!(ReferenceId, |device, _| Ok(device.reference_id as f64)),
                prop_ro!(PrefabHash, |device, _| Ok(device.get_prefab_hash() as f64)),
                prop_ro!(NameHash, |device, _| Ok(device.get_name_hash() as f64)),
                prop_rw_bool!(On, on),
                prop_rw_bool!(Mode, mode),
                prop_rw_clamped!(Setting, setting, 0.0, 999.0 + CELSIUS_TO_KELVIN),

                prop_ro!(PressureInput, |device, _| device.read_network_prop(Input, |net| net.pressure())),
                prop_ro!(TemperatureInput, |device, _| device.read_network_prop(Input, |net| net.temperature())),
                prop_ro!(TotalMolesInput, |device, _| device.read_network_prop(Input, |net| net.total_moles())),
                prop_ro!(CombustionInput, |_, _| Err(SimulationError::RuntimeError { message: "CombustionInput not implemented".to_string(), line: 0 })),
                prop_ro!(RatioOxygenInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(Oxygen))),
                prop_ro!(RatioCarbonDioxideInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(CarbonDioxide))),
                prop_ro!(RatioNitrogenInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(Nitrogen))),
                prop_ro!(RatioPollutantInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(Pollutant))),
                prop_ro!(RatioVolatilesInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(Volatiles))),
                prop_ro!(RatioWaterInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(Water))),
                prop_ro!(RatioSteamInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(Steam))),
                prop_ro!(RatioNitrousOxideInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(NitrousOxide))),
                prop_ro!(RatioLiquidNitrogenInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(LiquidNitrogen))),
                prop_ro!(RatioLiquidOxygenInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(LiquidOxygen))),
                prop_ro!(RatioLiquidVolatilesInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(LiquidVolatiles))),
                prop_ro!(RatioLiquidCarbonDioxideInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(LiquidCarbonDioxide))),
                prop_ro!(RatioLiquidPollutantInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(LiquidPollutant))),
                prop_ro!(RatioLiquidNitrousOxideInput, |device, _| device.read_network_prop(Input, |net| net.gas_ratio(LiquidNitrousOxide))),

                prop_ro!(PressureOutput, |device, _| device.read_network_prop(Output, |net| net.pressure())),
                prop_ro!(TemperatureOutput, |device, _| device.read_network_prop(Output, |net| net.temperature())),
                prop_ro!(TotalMolesOutput, |device, _| device.read_network_prop(Output, |net| net.total_moles())),
                prop_ro!(CombustionOutput, |_, _| Err(SimulationError::RuntimeError { message: "CombustionOutput not implemented".to_string(), line: 0 })),
                prop_ro!(RatioOxygenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Oxygen))),
                prop_ro!(RatioCarbonDioxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(CarbonDioxide))),
                prop_ro!(RatioNitrogenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Nitrogen))),
                prop_ro!(RatioPollutantOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Pollutant))),
                prop_ro!(RatioVolatilesOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Volatiles))),
                prop_ro!(RatioWaterOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Water))),
                prop_ro!(RatioNitrousOxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(NitrousOxide))),
                prop_ro!(RatioLiquidNitrogenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidNitrogen))),
                prop_ro!(RatioLiquidOxygenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidOxygen))),
                prop_ro!(RatioLiquidVolatilesOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidVolatiles))),
                prop_ro!(RatioSteamOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Steam))),
                prop_ro!(RatioLiquidCarbonDioxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidCarbonDioxide))),
                prop_ro!(RatioLiquidPollutantOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidPollutant))),
                prop_ro!(RatioLiquidNitrousOxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidNitrousOxide))),

                prop_ro!(PressureOutput2, |device, _| device.read_network_prop(Output2, |net| net.pressure())),
                prop_ro!(TemperatureOutput2, |device, _| device.read_network_prop(Output2, |net| net.temperature())),
                prop_ro!(TotalMolesOutput2, |device, _| device.read_network_prop(Output2, |net| net.total_moles())),
                prop_ro!(CombustionOutput2, |_, _| Err(SimulationError::RuntimeError { message: "CombustionOutput2 not implemented".to_string(), line: 0 })),
                prop_ro!(RatioOxygenOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Oxygen))),
                prop_ro!(RatioCarbonDioxideOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(CarbonDioxide))),
                prop_ro!(RatioNitrogenOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Nitrogen))),
                prop_ro!(RatioPollutantOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Pollutant))),
                prop_ro!(RatioVolatilesOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Volatiles))),
                prop_ro!(RatioWaterOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Water))),
                prop_ro!(RatioNitrousOxideOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(NitrousOxide))),
                prop_ro!(RatioLiquidNitrogenOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidNitrogen))),
                prop_ro!(RatioLiquidOxygenOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidOxygen))),
                prop_ro!(RatioLiquidVolatilesOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidVolatiles))),
                prop_ro!(RatioSteamOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Steam))),
                prop_ro!(RatioLiquidCarbonDioxideOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidCarbonDioxide))),
                prop_ro!(RatioLiquidPollutantOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidPollutant))),
                prop_ro!(RatioLiquidNitrousOxideOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidNitrousOxide))),

                prop_ro!(OperationalTemperatureEfficiency, |device, _| Ok(*device.operational_temperature_limitor.borrow())),
                prop_ro!(TemperatureDifferentialEfficiency, |device, _| Ok(*device.temperature_differential_efficiency.borrow())),
                prop_ro!(PressureEfficiency, |device, _| Ok(*device.optimal_pressure_scalar.borrow())),
            ];

            PropertyRegistry::new(DESCRIPTORS)
        })
    }

    // Helper methods to get atmospheric networks
    fn require_network(
        &self,
        connection: DeviceAtmosphericNetworkType,
    ) -> SimulationResult<Shared<AtmosphericNetwork>> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Internal => Ok(self.internal.clone()),
            Input => self.input_network.as_ref().and_then(|w| w.upgrade()).ok_or(
                SimulationError::RuntimeError {
                    message: "AirConditioner device has no input atmospheric network".to_string(),
                    line: 0,
                },
            ),
            Output => self
                .output_network
                .as_ref()
                .and_then(|w| w.upgrade())
                .ok_or(SimulationError::RuntimeError {
                    message: "AirConditioner device has no output atmospheric network".to_string(),
                    line: 0,
                }),
            Output2 => self.waste_network.as_ref().and_then(|w| w.upgrade()).ok_or(
                SimulationError::RuntimeError {
                    message: "AirConditioner device has no output2 atmospheric network".to_string(),
                    line: 0,
                },
            ),
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "AirConditioner does not support atmospheric connection type {:?}",
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

    /// Helper to get a mutable reference to an atmospheric network slot
    fn network_slot_mut(
        &mut self,
        connection: DeviceAtmosphericNetworkType,
    ) -> Result<&mut OptWeakShared<AtmosphericNetwork>, SimulationError> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Input => Ok(&mut self.input_network),
            Output => Ok(&mut self.output_network),
            Output2 => Ok(&mut self.waste_network),
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "AirConditioner does not support atmospheric connection type {:?}",
                    connection
                ),
                line: 0,
            }),
        }
    }
}

/// `Device` trait implementation for `AirConditioner` providing logic access, naming, and update behavior.
impl Device for AirConditioner {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        AirConditioner::prefab_hash()
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

    fn update(&self, _tick: u64) -> SimulationResult<()> {
        // Only run when device is On and Mode is enabled
        let stop = *self.on.borrow() == 0.0 || *self.mode.borrow() == 0.0;

        if stop {
            // Only processed moles is zeroed when not operating
            *self.processed_moles.borrow_mut() = 0.0;
            return Ok(());
        }

        let input_rc = self.require_network(DeviceAtmosphericNetworkType::Input)?;
        let output_rc = self.require_network(DeviceAtmosphericNetworkType::Output)?;
        let waste_rc = self.require_network(DeviceAtmosphericNetworkType::Output2)?;

        let target_temperature = *self.setting.borrow();

        let input_temperature = input_rc.borrow().temperature();

        // only operate if target temperature differs from input by at least 1K
        if (target_temperature - input_temperature).abs() < 1.0 {
            *self.processed_moles.borrow_mut() = 0.0;
            return Ok(());
        }

        // compute pressure scalar
        let pressure_offset_kpa = 0.1;

        let input_pressure_ratio =
            input_rc.borrow().pressure() / ONE_ATMOSPHERE - pressure_offset_kpa;
        let waste_pressure_ratio =
            waste_rc.borrow().pressure() / ONE_ATMOSPHERE - pressure_offset_kpa;

        let optimal_pressure_scalar = input_pressure_ratio
            .min(waste_pressure_ratio)
            .clamp(0.0, 1.0);

        // transfer moles using ideal gas law for internal volume
        let transfer_moles =
            calculate_moles(PRESSURE_PER_TICK, INTERNAL_VOLUME_LITRES, input_temperature);

        if transfer_moles <= 0.0 {
            *self.processed_moles.borrow_mut() = 0.0;
            return Ok(());
        }

        // remove that many moles from the input network
        let transferred_mixture = input_rc
            .borrow_mut()
            .remove_moles(transfer_moles, MatterState::All);

        // add to internal buffer
        {
            // Add transferred gas to internal
            self.internal.borrow_mut().add_mixture(&transferred_mixture);

            // temperature gap evaluation (between internal and waste depending on direction)
            let temperature_gap = if target_temperature > self.internal.borrow().temperature() {
                waste_rc.borrow().temperature() - self.internal.borrow().temperature()
            } else {
                self.internal.borrow().temperature() - waste_rc.borrow().temperature()
            };

            let operational_efficiency = self
                .input_and_waste_curve
                .evaluate(self.internal.borrow().temperature())
                .min(
                    self.input_and_waste_curve
                        .evaluate(waste_rc.borrow().temperature()),
                );

            let energy_joules = ENERGY_COEFFICIENT
                * self.temperature_delta_curve.evaluate(temperature_gap)
                * operational_efficiency
                * optimal_pressure_scalar
                * 1.0;

            // transfer thermal energy between internal and waste according to target direction
            if target_temperature > self.internal.borrow().temperature() {
                // need heating: remove energy from waste and add to internal
                let energy_removed = waste_rc.borrow_mut().remove_energy(energy_joules);
                self.internal.borrow_mut().add_energy(energy_removed);
            } else {
                // need cooling: remove energy from internal and add to waste
                let energy_removed = self.internal.borrow_mut().remove_energy(energy_joules);
                waste_rc.borrow_mut().add_energy(energy_removed);
            }

            // move internal gas to primary output and reset internal buffer
            output_rc
                .borrow_mut()
                .add_mixture(self.internal.borrow().mixture());
            self.internal.borrow_mut().clear();

            // store metrics
            *self.temperature_differential_efficiency.borrow_mut() =
                self.temperature_delta_curve.evaluate(temperature_gap);
            *self.operational_temperature_limitor.borrow_mut() = operational_efficiency;
            *self.optimal_pressure_scalar.borrow_mut() = optimal_pressure_scalar;
            *self.energy_moved.borrow_mut() = energy_joules;
            *self.processed_moles.borrow_mut() = transfer_moles;
        }

        Ok(())
    }

    fn run(&self) -> SimulationResult<()> {
        if *self.on.borrow() != 0.0 {
            self.chip_host
                .borrow()
                .run(self.max_instructions_per_tick)?
        }

        Ok(())
    }

    fn supported_types(&self) -> Vec<LogicType> {
        Self::properties().supported_types()
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

    fn properties() -> &'static PropertyRegistry<Self> {
        AirConditioner::properties()
    }

    fn display_name_static() -> &'static str {
        AirConditioner::display_name_static()
    }

    fn required_atmospheric_connections() -> Vec<DeviceAtmosphericNetworkType> {
        use DeviceAtmosphericNetworkType::*;
        vec![Internal, Input, Output, Output2]
    }

    fn as_ic_host_device(&self) -> Option<&dyn ICHostDevice> {
        Some(self)
    }

    fn as_ic_host_device_mut(&mut self) -> Option<&mut dyn ICHostDevice> {
        Some(self)
    }

    fn as_atmospheric_device(&self) -> Option<&dyn AtmosphericDevice> {
        Some(self)
    }

    fn as_atmospheric_device_mut(&mut self) -> Option<&mut dyn AtmosphericDevice> {
        Some(self)
    }

    fn is_ic_host() -> bool {
        true
    }
}

/// `ICHostDevice` helpers for `AirConditioner` (chip hosting and memory access helpers).
impl ICHostDevice for AirConditioner {
    fn ichost_get_id(&self) -> i32 {
        self.reference_id
    }

    fn chip_slot(&self) -> Shared<ChipSlot> {
        self.chip_host.clone()
    }

    fn max_instructions_per_tick(&self) -> usize {
        self.max_instructions_per_tick
    }
}

impl ICHostDeviceMemoryOverride for AirConditioner {}

/// `AtmosphericDevice` implementation for `AirConditioner` that manages input/output/waste networks.
impl AtmosphericDevice for AirConditioner {
    fn set_atmospheric_network(
        &mut self,
        connection: DeviceAtmosphericNetworkType,
        network: OptShared<AtmosphericNetwork>,
    ) -> SimulationResult<()> {
        let slot = self.network_slot_mut(connection)?;
        *slot = network.as_ref().map(Rc::downgrade);
        Ok(())
    }

    fn get_atmospheric_network(
        &self,
        connection: DeviceAtmosphericNetworkType,
    ) -> OptShared<AtmosphericNetwork> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Internal => Some(self.internal.clone()),
            Input => self.input_network.as_ref().and_then(|w| w.upgrade()),
            Output => self.output_network.as_ref().and_then(|w| w.upgrade()),
            Output2 => self.waste_network.as_ref().and_then(|w| w.upgrade()),
            _ => None,
        }
    }
}

impl Display for AirConditioner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let on_str = if *self.on.borrow() == 0.0 {
            "Off"
        } else {
            "On"
        };
        let mode_str = if *self.mode.borrow() == 0.0 {
            "Off"
        } else {
            "On"
        };
        let setting_str = fmt_trim(*self.setting.borrow(), 2);
        let processed_str = fmt_trim(*self.processed_moles.borrow(), 3);
        let energy_str = fmt_trim(*self.energy_moved.borrow(), 3);

        write!(
            f,
            "AirConditioner {{ name: \"{}\", id: {}, on: {}, mode: {}, setting: {}",
            self.name, self.reference_id, on_str, mode_str, setting_str
        )?;

        if let Some(weak) = &self.input_network
            && let Some(net) = weak.upgrade()
        {
            write!(f, ", input: {}", net.borrow().mixture())?;
        }
        if let Some(weak) = &self.output_network
            && let Some(net) = weak.upgrade()
        {
            write!(f, ", output: {}", net.borrow().mixture())?;
        }
        if let Some(weak) = &self.waste_network
            && let Some(net) = weak.upgrade()
        {
            write!(f, ", waste: {}", net.borrow().mixture())?;
        }

        write!(
            f,
            ", processed_moles: {}, energy_moved: {} }}",
            processed_str, energy_str
        )
    }
}

impl Debug for AirConditioner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
