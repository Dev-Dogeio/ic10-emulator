//! Filtration device - filters gases out of networks
//!
//! The Filtration device has three atmospheric connections:
//! - input: receives gas mixture
//! - filtered: outputs the filtered gas types
//! - waste: outputs the remaining gas mixture
//!
//! The device can be configured to filter up to 2 different gas types.

use std::cell::RefCell;

use crate::{
    CableNetwork, Filter, ItemType, Slot, allocate_global_id,
    atmospherics::{GasType, MAX_PRESSURE_GAS_PIPE, PIPE_VOLUME, calculate_moles},
    conversions::lerp,
    devices::{
        AtmosphericDevice, ChipSlot, Device, FilterConnectionType, ICHostDevice,
        ICHostDeviceMemoryOverride, LogicType, SimulationSettings,
    },
    error::{SimulationError, SimulationResult},
    networks::AtmosphericNetwork,
    parser::string_to_hash,
    types::{OptShared, Shared, shared},
};

/// Maximum number of filter slots on a Filtration device
const MAX_FILTERS: usize = 2;

const PRESSURE_PER_TICK: f64 = 1000.0;

/// Filtration device - separates specific gases from a gas mixture
#[derive(Debug)]
pub struct Filtration {
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

    /// The input network
    input_network: OptShared<AtmosphericNetwork>,
    /// The filtered network
    filtered_network: OptShared<AtmosphericNetwork>,
    /// The waste network
    waste_network: OptShared<AtmosphericNetwork>,

    /// Device slots: Filter, Filter
    slots: Vec<Slot>,

    /// Simulation settings
    #[allow(dead_code)]
    settings: SimulationSettings,

    /// Chip hosting helper (slot 0 may hold an IC10 chip)
    chip_host: Shared<ChipSlot>,
}

/// Minimum mole fraction threshold to also siphon remaining gas from the input atmosphere
const MIN_RATIO_TO_FILTER_ALL: f64 = 1.0 / 1000.0;

impl Filtration {
    /// Create a new Filtration device
    pub fn new(simulation_settings: Option<SimulationSettings>) -> Shared<Self> {
        let s = shared(Self {
            name: "Filtration".to_string(),
            network: None,
            on: RefCell::new(1.0),
            mode: RefCell::new(0.0),
            reference_id: allocate_global_id(),
            settings: simulation_settings.unwrap_or_default(),
            input_network: None,
            waste_network: None,
            filtered_network: None,
            slots: {
                let mut s = Vec::with_capacity(MAX_FILTERS);
                for _ in 0..MAX_FILTERS {
                    s.push(Slot::new(Some(ItemType::Filter)));
                }
                s
            },
            chip_host: ChipSlot::new(6),
        });

        // Attach the shared ChipSlot back to the chip so it can query network/device pins
        s.borrow()
            .chip_host
            .borrow_mut()
            .set_host_device(Some(s.clone()));

        s
    }

    /// Get a reference to a slot by index
    pub fn get_slot(&self, index: usize) -> Option<&Slot> {
        self.slots.get(index)
    }

    /// Get a mutable reference to a slot by index
    pub fn get_slot_mut(&mut self, index: usize) -> Option<&mut Slot> {
        self.slots.get_mut(index)
    }

    /// Number of filter slots
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Get the currently active filters from inserted physical filter items (quantity > 0)
    pub fn active_filters(&self) -> Vec<GasType> {
        let mut out = Vec::new();

        for slot in &self.slots {
            if let Some(item) = slot.get_item()
                && item.item_type() == ItemType::Filter
                && let Some(filter_item) = item.as_any().downcast_ref::<Filter>()
            {
                out.push(filter_item.gas_type());
            }
        }

        out
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
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        string_to_hash("StructureFiltration")
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
                | LogicType::On
                | LogicType::Mode
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
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(logic_type, LogicType::On | LogicType::Mode)
    }

    #[rustfmt::skip]
    fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        match logic_type {
            LogicType::PrefabHash => Ok(self.get_prefab_hash() as f64),
            LogicType::ReferenceId => Ok(self.reference_id as f64),
            LogicType::NameHash => Ok(self.get_name_hash() as f64),
            LogicType::On => Ok(*self.on.borrow()),
            LogicType::Mode => Ok(*self.mode.borrow()),

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

            LogicType::PressureOutput => read!(self.filtered_network, pressure),
            LogicType::TemperatureOutput => read!(self.filtered_network, temperature),
            LogicType::RatioOxygenOutput => read!(self.filtered_network, gas_ratio, GasType::Oxygen),
            LogicType::RatioCarbonDioxideOutput => read!(self.filtered_network, gas_ratio, GasType::CarbonDioxide),
            LogicType::RatioNitrogenOutput => read!(self.filtered_network, gas_ratio, GasType::Nitrogen),
            LogicType::RatioPollutantOutput => read!(self.filtered_network, gas_ratio, GasType::Pollutant),
            LogicType::RatioVolatilesOutput => read!(self.filtered_network, gas_ratio, GasType::Volatiles),
            LogicType::RatioSteamOutput => read!(self.filtered_network, gas_ratio, GasType::Steam),
            LogicType::RatioNitrousOxideOutput => read!(self.filtered_network, gas_ratio, GasType::NitrousOxide),
            LogicType::TotalMolesOutput => read!(self.filtered_network, total_moles),

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

            _ => Err(SimulationError::RuntimeError {
                message: format!("Filtration does not support reading logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }

    fn write(&self, logic_type: LogicType, _value: f64) -> SimulationResult<()> {
        match logic_type {
            LogicType::On => {
                *self.on.borrow_mut() = if _value < 1.0 { 0.0 } else { 1.0 };
                Ok(())
            }
            LogicType::Mode => {
                *self.mode.borrow_mut() = if _value < 1.0 { 0.0 } else { 1.0 };
                Ok(())
            }
            _ => Err(SimulationError::RuntimeError {
                message: format!("Filtration does not support writing logic type {logic_type:?}"),
                line: 0,
            }),
        }
    }

    fn update(&self, _tick: u64) -> SimulationResult<()> {
        // Only run filtration when device is On and Mode is enabled
        if *self.on.borrow() == 0.0 || *self.mode.borrow() == 0.0 {
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

        let scale_pressure = lerp(
            PRESSURE_PER_TICK,
            MAX_PRESSURE_GAS_PIPE / 3.0,
            input_pressure_delta / MAX_PRESSURE_GAS_PIPE,
        );

        // transfer moles using ideal gas law for pipe volume
        let transfer_moles_amount =
            calculate_moles(scale_pressure, PIPE_VOLUME, input_mut.temperature());

        if transfer_moles_amount <= 0.0 {
            return Ok(());
        }

        // Remove that many moles from the input network
        let mut transfer_mixture = input_mut.remove_moles(transfer_moles_amount);

        let mut filtered_mut = filtered_rc.borrow_mut();

        // Determine the filters to apply (physical slots with quantity > 0 take precedence)
        let filters_to_apply = self.active_filters();

        // For each configured filter, remove that gas from the transfer mixture and add to filtered output
        // Then, if the remaining input atmosphere has that gas below the min ratio, siphon all of it too
        for filter_type in &filters_to_apply {
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
}

impl ICHostDevice for Filtration {
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

impl ICHostDeviceMemoryOverride for Filtration {}

impl AtmosphericDevice for Filtration {
    fn set_atmospheric_network(
        &mut self,
        connection: FilterConnectionType,
        network: OptShared<AtmosphericNetwork>,
    ) -> SimulationResult<()> {
        use FilterConnectionType::*;
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
