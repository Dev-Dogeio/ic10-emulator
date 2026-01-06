//! AirConditioner device - moves thermal energy between an input and a waste network, and gas between input and output.

use crate::{
    CableNetwork, allocate_global_id,
    atmospherics::{CELSIUS_TO_KELVIN, GasType, MatterState, ONE_ATMOSPHERE, calculate_moles},
    devices::{
        AtmosphericDevice, ChipSlot, Device, DeviceAtmosphericNetworkType, ICHostDevice,
        ICHostDeviceMemoryOverride, LogicType, SimulationSettings,
    },
    error::{SimulationError, SimulationResult},
    networks::AtmosphericNetwork,
    parser::string_to_hash,
    types::{OptShared, Shared, shared},
};

use crate::animation_curve::AnimationCurve;
use std::{
    cell::RefCell,
    sync::{Arc, OnceLock},
};

/// Pressure per tick used for mole transfer (kPa).
pub(crate) const PRESSURE_PER_TICK: f64 = 750.0;

/// Internal buffer volume in litres
const INTERNAL_VOLUME_LITRES: f64 = 100.0;

/// Energy coefficient
const ENERGY_COEFFICIENT: f64 = 14000.0;

/// AirConditioner device - moves thermal energy between an input and a waste network, and gas between input and output.
#[derive(Debug)]
pub struct AirConditioner {
    /// Device name
    name: String,
    /// Connected network
    network: OptShared<CableNetwork>,

    /// The device reference ID
    reference_id: i32,
    /// The On state
    on: RefCell<f64>,
    /// The Mode state (0 = off, 1 = on)
    mode: RefCell<f64>,
    /// The Setting state (target temperature)
    setting: RefCell<f64>,

    /// The input network
    input_network: OptShared<AtmosphericNetwork>,
    /// The output network
    output_network: OptShared<AtmosphericNetwork>,
    /// The waste network
    waste_network: OptShared<AtmosphericNetwork>,
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

    /// Simulation settings
    #[allow(dead_code)]
    settings: SimulationSettings,

    /// Chip hosting helper (slot 0 may hold an IC10 chip)
    chip_host: Shared<ChipSlot>,
}

impl AirConditioner {
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Shared<Self> {
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

        let s = shared(Self {
            name: "Air Conditioner".to_string(),
            network: None,
            setting: RefCell::new(20.0),
            on: RefCell::new(1.0),
            mode: RefCell::new(0.0),
            reference_id: allocate_global_id(),
            settings: simulation_settings.unwrap_or_default(),
            input_network: None,
            output_network: None,
            waste_network: None,
            internal: AtmosphericNetwork::new(INTERNAL_VOLUME_LITRES),
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
    /// Get the energy moved in the last tick
    pub fn energy_moved_last_tick(&self) -> f64 {
        *self.energy_moved.borrow()
    }

    /// Get the processed moles in the last tick
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
}

macro_rules! read {
    ($net:expr, $method:ident) => {
        Ok($net.as_ref().unwrap().borrow().$method())
    };
    ($net:expr, $method:ident, $($arg:expr),+) => {
        Ok($net.as_ref().unwrap().borrow().$method($($arg),+))
    };
}

impl Device for AirConditioner {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        string_to_hash("StructureAirConditioner")
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
                | LogicType::On
                | LogicType::Mode
                | LogicType::Setting
                | LogicType::PressureInput
                | LogicType::TemperatureInput
                | LogicType::RatioOxygenInput
                | LogicType::RatioCarbonDioxideInput
                | LogicType::RatioNitrogenInput
                | LogicType::RatioPollutantInput
                | LogicType::RatioVolatilesInput
                | LogicType::RatioSteamInput
                | LogicType::RatioNitrousOxideInput
                | LogicType::TotalMolesInput
                | LogicType::PressureOutput
                | LogicType::TemperatureOutput
                | LogicType::RatioOxygenOutput
                | LogicType::RatioCarbonDioxideOutput
                | LogicType::RatioNitrogenOutput
                | LogicType::RatioPollutantOutput
                | LogicType::RatioVolatilesOutput
                | LogicType::RatioSteamOutput
                | LogicType::RatioNitrousOxideOutput
                | LogicType::TotalMolesOutput
                | LogicType::PressureOutput2
                | LogicType::TemperatureOutput2
                | LogicType::RatioOxygenOutput2
                | LogicType::RatioCarbonDioxideOutput2
                | LogicType::RatioNitrogenOutput2
                | LogicType::RatioPollutantOutput2
                | LogicType::RatioVolatilesOutput2
                | LogicType::RatioSteamOutput2
                | LogicType::RatioNitrousOxideOutput2
                | LogicType::TotalMolesOutput2
                | LogicType::OperationalTemperatureEfficiency
                | LogicType::TemperatureDifferentialEfficiency
                | LogicType::PressureEfficiency
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(
            logic_type,
            LogicType::On | LogicType::Mode | LogicType::Setting
        )
    }

    #[rustfmt::skip]
    fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        match logic_type {
            LogicType::ReferenceId => Ok(self.reference_id as f64),
            LogicType::PrefabHash => Ok(self.get_prefab_hash() as f64),
            LogicType::NameHash => Ok(self.get_name_hash() as f64),
            LogicType::On => Ok(*self.on.borrow()),
            LogicType::Mode => Ok(*self.mode.borrow()),
            LogicType::Setting => Ok(*self.setting.borrow()),

            LogicType::PressureInput => read!(self.input_network, pressure),
            LogicType::TemperatureInput => read!(self.input_network, temperature),
            LogicType::RatioOxygenInput => read!(self.input_network, gas_ratio, GasType::Oxygen),
            LogicType::RatioCarbonDioxideInput => read!(self.input_network, gas_ratio, GasType::CarbonDioxide),
            LogicType::RatioNitrogenInput => read!(self.input_network, gas_ratio, GasType::Nitrogen),
            LogicType::RatioPollutantInput => read!(self.input_network, gas_ratio, GasType::Pollutant),
            LogicType::RatioVolatilesInput => read!(self.input_network, gas_ratio, GasType::Volatiles),
            LogicType::RatioSteamInput => read!(self.input_network, gas_ratio, GasType::Steam),
            LogicType::RatioNitrousOxideInput => read!(self.input_network, gas_ratio, GasType::NitrousOxide),
            LogicType::TotalMolesInput => read!(self.input_network, total_moles),

            LogicType::PressureOutput => read!(self.output_network, pressure),
            LogicType::TemperatureOutput => read!(self.output_network, temperature),
            LogicType::RatioOxygenOutput => read!(self.output_network, gas_ratio, GasType::Oxygen),
            LogicType::RatioCarbonDioxideOutput => read!(self.output_network, gas_ratio, GasType::CarbonDioxide),
            LogicType::RatioNitrogenOutput => read!(self.output_network, gas_ratio, GasType::Nitrogen),
            LogicType::RatioPollutantOutput => read!(self.output_network, gas_ratio, GasType::Pollutant),
            LogicType::RatioVolatilesOutput => read!(self.output_network, gas_ratio, GasType::Volatiles),
            LogicType::RatioSteamOutput => read!(self.output_network, gas_ratio, GasType::Steam),
            LogicType::RatioNitrousOxideOutput => read!(self.output_network, gas_ratio, GasType::NitrousOxide),
            LogicType::TotalMolesOutput => read!(self.output_network, total_moles),

            LogicType::PressureOutput2 => read!(self.waste_network, pressure),
            LogicType::TemperatureOutput2 => read!(self.waste_network, temperature),
            LogicType::RatioOxygenOutput2 => read!(self.waste_network, gas_ratio, GasType::Oxygen),
            LogicType::RatioCarbonDioxideOutput2 => read!(self.waste_network, gas_ratio, GasType::CarbonDioxide),
            LogicType::RatioNitrogenOutput2 => read!(self.waste_network, gas_ratio, GasType::Nitrogen),
            LogicType::RatioPollutantOutput2 => read!(self.waste_network, gas_ratio, GasType::Pollutant),
            LogicType::RatioVolatilesOutput2 => read!(self.waste_network, gas_ratio, GasType::Volatiles),
            LogicType::RatioSteamOutput2 => read!(self.waste_network, gas_ratio, GasType::Steam),
            LogicType::RatioNitrousOxideOutput2 => read!(self.waste_network, gas_ratio, GasType::NitrousOxide),
            LogicType::TotalMolesOutput2 => read!(self.waste_network, total_moles),

            LogicType::OperationalTemperatureEfficiency => Ok(*self.operational_temperature_limitor.borrow()),
            LogicType::TemperatureDifferentialEfficiency => Ok(*self.temperature_differential_efficiency.borrow()),
            LogicType::PressureEfficiency => Ok(*self.optimal_pressure_scalar.borrow()),

            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "AirConditioner does not support reading logic type {logic_type:?}"
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
            LogicType::Mode => {
                *self.mode.borrow_mut() = if value < 1.0 { 0.0 } else { 1.0 };
                Ok(())
            }
            LogicType::Setting => {
                *self.setting.borrow_mut() = value.clamp(0.0, 999.0 + CELSIUS_TO_KELVIN);
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "AirConditioner does not support writing logic type {logic_type:?}"
                ),
                line: 0,
            }),
        }
    }

    fn update(&self, _tick: u64) -> SimulationResult<()> {
        // Only run when device is On and Mode is enabled
        let stop = *self.on.borrow() == 0.0 || *self.mode.borrow() == 0.0;

        if stop {
            // Only processed moles is zeroed when not operating
            *self.processed_moles.borrow_mut() = 0.0;
            return Ok(());
        }

        let input_rc = self
            .input_network
            .as_ref()
            .ok_or(SimulationError::RuntimeError {
                message: "AirConditioner device has no input atmospheric network".to_string(),
                line: 0,
            })?;

        let output_rc = self
            .output_network
            .as_ref()
            .ok_or(SimulationError::RuntimeError {
                message: "AirConditioner device has no output atmospheric network".to_string(),
                line: 0,
            })?;

        let waste_rc = self
            .waste_network
            .as_ref()
            .ok_or(SimulationError::RuntimeError {
                message: "AirConditioner device has no output2 atmospheric network".to_string(),
                line: 0,
            })?;

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
        let transferred_mixture = input_rc.borrow_mut().remove_moles(transfer_moles, MatterState::All);

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
                .run(self.settings.max_instructions_per_tick)?
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

    fn clear_internal_references(&mut self) {
        self.chip_host.borrow_mut().set_host_device(None);
    }
}

impl ICHostDevice for AirConditioner {
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

impl ICHostDeviceMemoryOverride for AirConditioner {}

impl AtmosphericDevice for AirConditioner {
    fn set_atmospheric_network(
        &mut self,
        connection: DeviceAtmosphericNetworkType,
        network: OptShared<AtmosphericNetwork>,
    ) -> SimulationResult<()> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Input => {
                self.input_network = network;
                Ok(())
            }
            Output => {
                self.output_network = network;
                Ok(())
            }
            Output2 => {
                self.waste_network = network;
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "AirConditioner does not support atmospheric connection {:?}",
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
            Input => self.input_network.clone(),
            Output => self.output_network.clone(),
            Output2 => self.waste_network.clone(),
            Internal => Some(self.internal.clone()),
            _ => None,
        }
    }
}
