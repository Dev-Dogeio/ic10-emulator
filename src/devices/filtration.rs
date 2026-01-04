//! Filtration device - filters gases out of networks
//!
//! The Filtration device has three atmospheric connections:
//! - input: receives gas mixture
//! - filtered: outputs the filtered gas types
//! - waste: outputs the remaining gas mixture
//!
//! The device can be configured to filter up to 2 different gas types.

use crate::{
    CableNetwork,
    atmospherics::{GasType, MAX_PRESSURE_GAS_PIPE, PIPE_VOLUME, calculate_moles},
    devices::{
        AtmosphericDevice, Device, DeviceBase, FilterConnectionType, LogicType, SimulationSettings,
    },
    error::{SimulationError, SimulationResult},
    networks::AtmosphericNetwork,
    parser::string_to_hash,
    types::OptShared,
};

/// Maximum number of filter slots on a Filtration device
const MAX_FILTERS: usize = 2;

const PRESSURE_PER_TICK: f64 = 1000.0;

/// Filtration device - separates specific gases from a gas mixture
#[derive(Debug)]
pub struct Filtration {
    base: DeviceBase,
    /// Simulation settings
    #[allow(dead_code)]
    settings: SimulationSettings,

    /// Input atmospheric network connection
    input_network: OptShared<AtmosphericNetwork>,
    /// Filtered atmospheric network connection
    filtered_network: OptShared<AtmosphericNetwork>,
    /// Waste atmospheric network connection
    waste_network: OptShared<AtmosphericNetwork>,

    /// Filters list (max length defined by `MAX_FILTERS`)
    filters: Vec<GasType>,
}

/// Minimum mole fraction threshold to also siphon remaining gas from the input atmosphere
const MIN_RATIO_TO_FILTER_ALL: f64 = 1.0 / 1000.0;

impl Filtration {
    /// Create a new Filtration device
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Self {
        let base = DeviceBase::new(
            "Filtration".to_string(),
            string_to_hash("StructureFiltration"),
        );

        Self {
            base,
            settings: simulation_settings.unwrap_or_default(),
            input_network: None,
            waste_network: None,
            filtered_network: None,
            filters: Vec::new(),
        }
    }

    /// Replace the current filters with `filters` (must not exceed `MAX_FILTERS`)
    pub fn set_filters(&mut self, filters: Vec<GasType>) -> SimulationResult<()> {
        if filters.len() > MAX_FILTERS {
            return Err(SimulationError::RuntimeError {
                message: format!("Cannot set more than {} filters", MAX_FILTERS),
                line: 0,
            });
        }
        self.filters = filters;
        Ok(())
    }

    /// Add a gas type to the filters (no-op if already present)
    pub fn add_filter(&mut self, gas_type: GasType) -> SimulationResult<()> {
        if self.filters.contains(&gas_type) {
            return Ok(());
        }

        if self.filters.len() >= MAX_FILTERS {
            return Err(SimulationError::RuntimeError {
                message: format!(
                    "Cannot add filter: maximum of {} filters reached",
                    MAX_FILTERS
                ),
                line: 0,
            });
        }

        self.filters.push(gas_type);
        Ok(())
    }

    /// Remove a gas type from the filters (no-op if not present)
    pub fn remove_filter(&mut self, gas_type: GasType) -> SimulationResult<()> {
        if let Some(pos) = self.filters.iter().position(|g| *g == gas_type) {
            self.filters.remove(pos);
        }
        Ok(())
    }

    /// Get a copy of the current filters
    pub fn get_filters(&self) -> Vec<GasType> {
        self.filters.clone()
    }

    /// Get the input network connection
    pub fn get_input_network(&self) -> OptShared<AtmosphericNetwork> {
        self.input_network.clone()
    }

    /// Get the waste network connection
    pub fn get_waste_network(&self) -> OptShared<AtmosphericNetwork> {
        self.waste_network.clone()
    }

    /// Get the filtered network connection
    pub fn get_filtered_network(&self) -> OptShared<AtmosphericNetwork> {
        self.filtered_network.clone()
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

impl Device for Filtration {
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
                | LogicType::On
                | LogicType::PressureInput
                | LogicType::TemperatureInput
                | LogicType::RatioOxygenInput
                | LogicType::RatioCarbonDioxideInput
                | LogicType::RatioNitrogenInput
                | LogicType::RatioPollutantInput
                | LogicType::RatioVolatilesInput
                | LogicType::RatioWaterInput
                | LogicType::RatioNitrousOxideInput
                | LogicType::TotalMolesInput
                | LogicType::PressureOutput
                | LogicType::TemperatureOutput
                | LogicType::RatioOxygenOutput
                | LogicType::RatioCarbonDioxideOutput
                | LogicType::RatioNitrogenOutput
                | LogicType::RatioPollutantOutput
                | LogicType::RatioVolatilesOutput
                | LogicType::RatioWaterOutput
                | LogicType::RatioNitrousOxideOutput
                | LogicType::TotalMolesOutput
                | LogicType::PressureOutput2
                | LogicType::TemperatureOutput2
                | LogicType::RatioOxygenOutput2
                | LogicType::RatioCarbonDioxideOutput2
                | LogicType::RatioNitrogenOutput2
                | LogicType::RatioPollutantOutput2
                | LogicType::RatioVolatilesOutput2
                | LogicType::RatioWaterOutput2
                | LogicType::RatioNitrousOxideOutput2
                | LogicType::TotalMolesOutput2
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(logic_type, LogicType::On)
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

            LogicType::PressureInput => read!(self.input_network, pressure),
            LogicType::TemperatureInput => read!(self.input_network, temperature),
            LogicType::RatioOxygenInput => read!(self.input_network, gas_ratio, GasType::Oxygen),
            LogicType::RatioCarbonDioxideInput => read!(self.input_network, gas_ratio, GasType::CarbonDioxide),
            LogicType::RatioNitrogenInput => read!(self.input_network, gas_ratio, GasType::Nitrogen),
            LogicType::RatioPollutantInput => read!(self.input_network, gas_ratio, GasType::Pollutant),
            LogicType::RatioVolatilesInput => read!(self.input_network, gas_ratio, GasType::Volatiles),
            LogicType::RatioWaterInput => read!(self.input_network, gas_ratio, GasType::Water),
            LogicType::RatioNitrousOxideInput => read!(self.input_network, gas_ratio, GasType::NitrousOxide),
            LogicType::TotalMolesInput => read!(self.input_network, total_moles),

            LogicType::PressureOutput => read!(self.filtered_network, pressure),
            LogicType::TemperatureOutput => read!(self.filtered_network, temperature),
            LogicType::RatioOxygenOutput => read!(self.filtered_network, gas_ratio, GasType::Oxygen),
            LogicType::RatioCarbonDioxideOutput => read!(self.filtered_network, gas_ratio, GasType::CarbonDioxide),
            LogicType::RatioNitrogenOutput => read!(self.filtered_network, gas_ratio, GasType::Nitrogen),
            LogicType::RatioPollutantOutput => read!(self.filtered_network, gas_ratio, GasType::Pollutant),
            LogicType::RatioVolatilesOutput => read!(self.filtered_network, gas_ratio, GasType::Volatiles),
            LogicType::RatioWaterOutput => read!(self.filtered_network, gas_ratio, GasType::Water),
            LogicType::RatioNitrousOxideOutput => read!(self.filtered_network, gas_ratio, GasType::NitrousOxide),
            LogicType::TotalMolesOutput => read!(self.filtered_network, total_moles),

            LogicType::PressureInput2 => read!(self.waste_network, pressure),
            LogicType::TemperatureInput2 => read!(self.waste_network, temperature),
            LogicType::RatioOxygenInput2 => read!(self.waste_network, gas_ratio, GasType::Oxygen),
            LogicType::RatioCarbonDioxideInput2 => read!(self.waste_network, gas_ratio, GasType::CarbonDioxide),
            LogicType::RatioNitrogenInput2 => read!(self.waste_network, gas_ratio, GasType::Nitrogen),
            LogicType::RatioPollutantInput2 => read!(self.waste_network, gas_ratio, GasType::Pollutant),
            LogicType::RatioVolatilesInput2 => read!(self.waste_network, gas_ratio, GasType::Volatiles),
            LogicType::RatioWaterInput2 => read!(self.waste_network, gas_ratio, GasType::Water),
            LogicType::RatioNitrousOxideInput2 => read!(self.waste_network, gas_ratio, GasType::NitrousOxide),
            LogicType::TotalMolesInput2 => read!(self.waste_network, total_moles),

            _ => Err(SimulationError::RuntimeError {
                message: format!("Filtration does not support reading logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }

    fn write(&self, logic_type: LogicType, _value: f64) -> SimulationResult<()> {
        match logic_type {
            LogicType::On => self
                .base
                .logic_types
                .borrow_mut()
                .set(LogicType::On, _value),
            _ => Err(SimulationError::RuntimeError {
                message: format!("Filtration does not support writing logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }

    fn update(&self, _tick: u64) -> SimulationResult<()> {
        // Only run filtration when device is On
        if self.base.logic_types.borrow().on.unwrap() == 0.0 {
            return Ok(());
        }

        // Ensure input and both outputs exist; error if any missing
        let input_rc = self
            .input_network
            .as_ref()
            .ok_or(SimulationError::RuntimeError {
                message: "Filtration device has no input atmospheric network".to_string(),
                line: 0,
            })?;

        let filtered_rc = self
            .filtered_network
            .as_ref()
            .ok_or(SimulationError::RuntimeError {
                message: "Filtration device has no filtered atmospheric network".to_string(),
                line: 0,
            })?;

        let waste_rc = self
            .waste_network
            .as_ref()
            .ok_or(SimulationError::RuntimeError {
                message: "Filtration device has no waste atmospheric network".to_string(),
                line: 0,
            })?;

        let mut input_mut = input_rc.borrow_mut();

        // If there's nothing in the input, early out
        if input_mut.total_moles() <= 0.0 {
            return Ok(());
        }

        let input_pressure = input_mut.pressure();
        let filtered_pressure = filtered_rc.borrow().pressure();
        let waste_pressure = waste_rc.borrow().pressure();
        let max_output_pressure = filtered_pressure.max(waste_pressure);

        let input_pressure_delta = (input_pressure - max_output_pressure).max(0.0);

        let t = (input_pressure_delta / MAX_PRESSURE_GAS_PIPE).clamp(0.0, 1.0);
        let scale_pressure =
            PRESSURE_PER_TICK + (MAX_PRESSURE_GAS_PIPE / 3.0 - PRESSURE_PER_TICK) * t;

        // transfer moles using ideal gas law for pipe volume
        let transfer_moles_amount =
            calculate_moles(scale_pressure, PIPE_VOLUME, input_mut.temperature());

        if transfer_moles_amount <= 0.0 {
            return Ok(());
        }

        // Remove that many moles from the input network
        let mut transfer_mixture = input_mut.remove_moles(transfer_moles_amount);

        let mut filtered_mut = filtered_rc.borrow_mut();

        // For each configured filter, remove that gas from the transfer mixture and add to filtered output
        // Then, if the remaining input atmosphere has that gas below the min ratio, siphon all of it too
        for filter_type in &self.filters {
            let mol = transfer_mixture.remove_all_gas(*filter_type);
            if !mol.is_empty() {
                filtered_mut.add_mole(&mol);
            }

            // Check remaining input atmosphere mole fraction and optionally remove all of that gas
            let atm_total = input_mut.total_moles();
            if atm_total > 0.0 {
                let atm_gas_moles = input_mut.get_moles(*filter_type);
                if atm_gas_moles / atm_total < MIN_RATIO_TO_FILTER_ALL {
                    let extra = input_mut.remove_all_gas(*filter_type);
                    if !extra.is_empty() {
                        filtered_mut.add_mole(&extra);
                    }
                }
            }
        }

        // Remaining transfer mixture goes to the waste output
        let mut waste_mut = waste_rc.borrow_mut();
        waste_mut.add_mixture(&transfer_mixture);

        Ok(())
    }
}

impl AtmosphericDevice for Filtration {
    fn set_atmospheric_network(
        &mut self,
        connection: FilterConnectionType,
        network: OptShared<AtmosphericNetwork>,
    ) -> SimulationResult<()> {
        use crate::devices::FilterConnectionType::*;
        match connection {
            Input => {
                let _: () = self.input_network = network;
                Ok(())
            }
            Output => {
                let _: () = self.filtered_network = network;
                Ok(())
            }
            Output2 => {
                let _: () = self.waste_network = network;
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "Filtration device does not support atmospheric connection type {:?}",
                    connection
                ),
                line: 0,
            }),
        }
    }
}

impl Default for Filtration {
    fn default() -> Self {
        Self::new(None)
    }
}
