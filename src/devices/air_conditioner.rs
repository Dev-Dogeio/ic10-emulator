//! AirConditioner device - moves thermal energy between an input and a waste network, and gas between input and output.

use crate::{
    atmospherics::{GasMixture, GasType, ONE_ATMOSPHERE, calculate_moles},
    devices::{Device, DeviceBase, FilterConnectionType, LogicType, SimulationSettings},
    error::{SimulationError, SimulationResult},
    networks::AtmosphericNetwork,
    parser::string_to_hash,
    types::OptShared,
};

use crate::animation_curve::AnimationCurve;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, OnceLock};

/// Pressure per tick used for mole transfer (kPa).
pub(crate) const PRESSURE_PER_TICK: f64 = 750.0;

/// Internal buffer volume in litres
const INTERNAL_VOLUME_LITRES: f64 = 100.0;

/// Energy coefficient
const ENERGY_COEFFICIENT: f64 = 14000.0;

#[derive(Debug)]
pub struct AirConditioner {
    base: DeviceBase,
    #[allow(dead_code)]
    settings: SimulationSettings,

    /// The input network
    input_network: OptShared<AtmosphericNetwork>,

    /// The output network
    output_network: OptShared<AtmosphericNetwork>,

    /// The waste network
    waste_network: OptShared<AtmosphericNetwork>,

    /// Internal buffer used to hold transferred gas
    internal: RefCell<GasMixture>,

    /// Last computed metrics (for logic reads)
    temperature_differential_efficiency: Cell<f64>,
    operational_temperature_limitor: Cell<f64>,
    optimal_pressure_scalar: Cell<f64>,

    /// Last processed mole amount
    processed_moles: Cell<f64>,

    /// Energy moved in last tick (J)
    energy_moved: Cell<f64>,

    /// Curves
    temperature_delta_curve: Arc<AnimationCurve>,
    input_and_waste_curve: Arc<AnimationCurve>,
}

impl AirConditioner {
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Self {
        let base = DeviceBase::new(
            "AirConditioner".to_string(),
            string_to_hash("StructureAirConditioner"),
        );

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

        Self {
            base,
            settings: simulation_settings.unwrap_or_default(),
            input_network: None,
            output_network: None,
            waste_network: None,
            internal: RefCell::new(GasMixture::new(INTERNAL_VOLUME_LITRES)),
            processed_moles: Cell::new(0.0),
            temperature_differential_efficiency: Cell::new(0.0),
            operational_temperature_limitor: Cell::new(0.0),
            optimal_pressure_scalar: Cell::new(0.0),
            energy_moved: Cell::new(0.0),
            temperature_delta_curve,
            input_and_waste_curve,
        }
    }

    pub fn energy_moved_last_tick(&self) -> f64 {
        self.energy_moved.get()
    }

    pub fn processed_moles_last_tick(&self) -> f64 {
        self.processed_moles.get()
    }

    pub fn optimal_pressure_scalar_last_tick(&self) -> f64 {
        self.optimal_pressure_scalar.get()
    }

    pub fn temperature_differential_efficiency_last_tick(&self) -> f64 {
        self.temperature_differential_efficiency.get()
    }

    pub fn operational_temperature_limitor_last_tick(&self) -> f64 {
        self.operational_temperature_limitor.get()
    }

    pub fn get_temperature_delta_curve(&self) -> Arc<AnimationCurve> {
        Arc::clone(&self.temperature_delta_curve)
    }

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

    fn get_network(&self) -> OptShared<crate::CableNetwork> {
        self.base.get_network()
    }

    fn set_network(&mut self, network: OptShared<crate::CableNetwork>) {
        self.base.set_network(network);
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
            LogicType::PrefabHash => Ok(self.base.logic_types.borrow().prefab_hash as f64),
            LogicType::ReferenceId => Ok(self.base.logic_types.borrow().reference_id as f64),
            LogicType::NameHash => Ok(self.base.logic_types.borrow().name_hash as f64),
            LogicType::On => {
                self.base
                    .logic_types
                    .borrow()
                    .on
                    .ok_or(SimulationError::RuntimeError {
                        message: "On value not set".to_string(),
                        line: 0,
                    })
            }
            LogicType::Mode => {
                self.base
                    .logic_types
                    .borrow()
                    .mode
                    .ok_or(SimulationError::RuntimeError {
                        message: "Mode value not set".to_string(),
                        line: 0,
                    })
            }
            LogicType::Setting => {
                self.base
                    .logic_types
                    .borrow()
                    .setting
                    .ok_or(SimulationError::RuntimeError {
                        message: "Setting value not set".to_string(),
                        line: 0,
                    })
            }

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

            LogicType::OperationalTemperatureEfficiency => Ok(self.operational_temperature_limitor.get()),
            LogicType::TemperatureDifferentialEfficiency => Ok(self.temperature_differential_efficiency.get()),
            LogicType::PressureEfficiency => Ok(self.optimal_pressure_scalar.get()),

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
            LogicType::On => self.base.logic_types.borrow_mut().set(LogicType::On, value),
            LogicType::Mode => self
                .base
                .logic_types
                .borrow_mut()
                .set(LogicType::Mode, value),
            LogicType::Setting => self
                .base
                .logic_types
                .borrow_mut()
                .set(LogicType::Setting, value),
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
        let stop = {
            let lt = self.base.logic_types.borrow();
            let on = lt.on.unwrap_or(1.0);
            let mode = lt.mode.unwrap_or(1.0);
            on == 0.0 || mode == 0.0
        };

        if stop {
            // Only processed moles is zeroed when not operating
            self.processed_moles.set(0.0);
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

        let target_temperature =
            self.base
                .logic_types
                .borrow()
                .setting
                .ok_or(SimulationError::RuntimeError {
                    message: "AirConditioner device has no goal temperature set".to_string(),
                    line: 0,
                })?;

        let input_temperature = input_rc.borrow().temperature();

        // only operate if target temperature differs from input by at least 1K
        if (target_temperature - input_temperature).abs() < 1.0 {
            self.processed_moles.set(0.0);
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
            self.processed_moles.set(0.0);
            return Ok(());
        }

        // remove that many moles from the input network
        let transferred_mixture = input_rc.borrow_mut().remove_moles(transfer_moles);

        // add to internal buffer
        {
            let mut internal = self.internal.borrow_mut();
            internal.merge(&transferred_mixture);

            // temperature gap evaluation (between internal and waste depending on direction)
            let temperature_gap = if target_temperature > internal.temperature() {
                waste_rc.borrow().temperature() - internal.temperature()
            } else {
                internal.temperature() - waste_rc.borrow().temperature()
            };

            let operational_efficiency = self
                .input_and_waste_curve
                .evaluate(internal.temperature())
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
            if target_temperature > internal.temperature() {
                // need heating: remove energy from waste and add to internal
                let energy_removed = waste_rc.borrow_mut().remove_energy(energy_joules);
                internal.add_energy(energy_removed);
            } else {
                // need cooling: remove energy from internal and add to waste
                let energy_removed = internal.remove_energy(energy_joules);
                waste_rc.borrow_mut().add_energy(energy_removed);
            }

            // move internal gas to primary output and reset internal buffer
            output_rc.borrow_mut().add_mixture(&internal);
            internal.clear();

            // store metrics
            self.temperature_differential_efficiency
                .set(self.temperature_delta_curve.evaluate(temperature_gap));
            self.operational_temperature_limitor
                .set(operational_efficiency);
            self.optimal_pressure_scalar.set(optimal_pressure_scalar);
            self.energy_moved.set(energy_joules);
            self.processed_moles.set(transfer_moles);
        }

        Ok(())
    }
}

impl crate::devices::AtmosphericDevice for AirConditioner {
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
}
