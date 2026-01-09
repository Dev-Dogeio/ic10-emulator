//! Filtration device: separates specified gases from an input mixture.

use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    sync::OnceLock,
};

use crate::{
    CableNetwork, Filter, Item, ItemType, LogicSlotType, Slot, allocate_global_id,
    atmospherics::{GasType, MAX_PRESSURE_GAS_PIPE, MatterState, PIPE_VOLUME, calculate_moles},
    constants::DEFAULT_MAX_INSTRUCTIONS_PER_TICK,
    conversions::lerp,
    devices::{
        AtmosphericDevice, ChipSlot, Device, DeviceAtmosphericNetworkType, ICHostDevice,
        ICHostDeviceMemoryOverride, LogicType, SimulationDeviceSettings, SlotHostDevice,
        property_descriptor::{
            PropertyDescriptor, PropertyRegistry, SlotPropertyDescriptor, SlotPropertyRegistry,
        },
    },
    error::{SimulationError, SimulationResult},
    networks::AtmosphericNetwork,
    parser::string_to_hash,
    prop_ro, prop_rw_bool, prop_slot_ro, reserve_global_id,
    types::{OptShared, Shared, shared},
};

/// Maximum number of filter slots on a Filtration device
const MAX_FILTERS: usize = 2;

const PRESSURE_PER_TICK: f64 = 1000.0;

/// Filtration device: separates specified gases
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

    /// Device slots (2x Filter)
    slots: Vec<Slot>,

    /// Max instructions an installed IC can execute per tick
    max_instructions_per_tick: usize,

    /// Chip hosting helper (slot 0 may hold an IC10 chip)
    chip_host: Shared<ChipSlot>,
}

/// Minimum mole fraction threshold to also siphon remaining gas from the input atmosphere
const MIN_RATIO_TO_FILTER_ALL: f64 = 1.0 / 1000.0;

/// Constructors and helper methods
impl Filtration {
    /// Compile-time prefab hash constant for this device
    pub const PREFAB_HASH: i32 = string_to_hash("StructureFiltration");

    /// Create a new `Filtration`. Optionally accepts simulation settings.
    pub fn new(simulation_settings: Option<SimulationDeviceSettings>) -> Shared<Self> {
        let settings = simulation_settings.unwrap_or_default();
        let reference_id = if let Some(id) = settings.id {
            reserve_global_id(id)
        } else {
            allocate_global_id()
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
            on: RefCell::new(1.0),
            mode: RefCell::new(0.0),
            reference_id,
            max_instructions_per_tick,
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

    /// Get the currently active filters from inserted physical filter items (quantity > 0)
    pub fn active_filters(&self) -> Vec<GasType> {
        let mut out = Vec::new();

        for slot in &self.slots {
            if let Some(item) = slot.get_item() {
                let item_ref = item.borrow();
                if item_ref.item_type() == ItemType::Filter
                    && let Some(filter_item) = item_ref.as_any().downcast_ref::<Filter>()
                {
                    out.push(filter_item.gas_type());
                }
            }
        }

        out
    }

    /// Return the prefab hash for `Filtration`.
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }

    pub fn display_name_static() -> &'static str {
        "Filtration"
    }

    /// Get the property registry for this device type
    #[rustfmt::skip]
    pub fn properties() -> &'static PropertyRegistry<Self> {
        use LogicType::*;
        use DeviceAtmosphericNetworkType::*;
        use GasType::*;
        static REGISTRY: OnceLock<PropertyRegistry<Filtration>> = OnceLock::new();

        REGISTRY.get_or_init(|| {
            const DESCRIPTORS: &[PropertyDescriptor<Filtration>] = &[
                prop_ro!(ReferenceId, |device, _| Ok(device.reference_id as f64)),
                prop_ro!(PrefabHash, |device, _| Ok(device.get_prefab_hash() as f64)),
                prop_ro!(NameHash, |device, _| Ok(device.get_name_hash() as f64)),
                prop_rw_bool!(On, on),
                prop_rw_bool!(Mode, mode),

                prop_ro!(PressureInput, |device, _| device.read_network_prop(Input, |net| net.pressure())),
                prop_ro!(TemperatureInput, |device, _| device.read_network_prop(Input, |net| net.temperature())),
                prop_ro!(TotalMolesInput, |device, _| device.read_network_prop(Input, |net| net.total_moles())),
                prop_ro!(CombustionInput, |_, _| Err(SimulationError::RuntimeError { message: "CombustionInput not implemented for Filtration".to_string(), line: 0 })),
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
                prop_ro!(CombustionOutput, |_, _| Err(SimulationError::RuntimeError { message: "CombustionOutput not implemented for Filtration".to_string(), line: 0 })),
                prop_ro!(RatioOxygenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Oxygen))),
                prop_ro!(RatioCarbonDioxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(CarbonDioxide))),
                prop_ro!(RatioNitrogenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Nitrogen))),
                prop_ro!(RatioPollutantOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Pollutant))),
                prop_ro!(RatioVolatilesOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Volatiles))),
                prop_ro!(RatioWaterOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Water))),
                prop_ro!(RatioSteamOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(Steam))),
                prop_ro!(RatioNitrousOxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(NitrousOxide))),
                prop_ro!(RatioLiquidNitrogenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidNitrogen))),
                prop_ro!(RatioLiquidOxygenOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidOxygen))),
                prop_ro!(RatioLiquidVolatilesOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidVolatiles))),
                prop_ro!(RatioLiquidCarbonDioxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidCarbonDioxide))),
                prop_ro!(RatioLiquidPollutantOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidPollutant))),
                prop_ro!(RatioLiquidNitrousOxideOutput, |device, _| device.read_network_prop(Output, |net| net.gas_ratio(LiquidNitrousOxide))),

                prop_ro!(PressureOutput2, |device, _| device.read_network_prop(Output2, |net| net.pressure())),
                prop_ro!(TemperatureOutput2, |device, _| device.read_network_prop(Output2, |net| net.temperature())),
                prop_ro!(TotalMolesOutput2, |device, _| device.read_network_prop(Output2, |net| net.total_moles())),
                prop_ro!(CombustionOutput2, |_, _| Err(SimulationError::RuntimeError { message: "CombustionOutput2 not implemented for Filtration".to_string(), line: 0 })),
                prop_ro!(RatioOxygenOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Oxygen))),
                prop_ro!(RatioCarbonDioxideOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(CarbonDioxide))),
                prop_ro!(RatioNitrogenOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Nitrogen))),
                prop_ro!(RatioPollutantOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Pollutant))),
                prop_ro!(RatioVolatilesOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Volatiles))),
                prop_ro!(RatioWaterOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Water))),
                prop_ro!(RatioSteamOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(Steam))),
                prop_ro!(RatioNitrousOxideOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(NitrousOxide))),
                prop_ro!(RatioLiquidNitrogenOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidNitrogen))),
                prop_ro!(RatioLiquidOxygenOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidOxygen))),
                prop_ro!(RatioLiquidVolatilesOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidVolatiles))),
                prop_ro!(RatioLiquidCarbonDioxideOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidCarbonDioxide))),
                prop_ro!(RatioLiquidPollutantOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidPollutant))),
                prop_ro!(RatioLiquidNitrousOxideOutput2, |device, _| device.read_network_prop(Output2, |net| net.gas_ratio(LiquidNitrousOxide))),
            ];

            PropertyRegistry::new(DESCRIPTORS)
        })
    }

    /// Get the slot property registry for this device type
    pub fn slot_properties() -> &'static SlotPropertyRegistry<Filtration> {
        use LogicSlotType::*;
        static SLOT_REGISTRY: OnceLock<SlotPropertyRegistry<Filtration>> = OnceLock::new();

        SLOT_REGISTRY.get_or_init(|| {
            const DESCRIPTORS: &[SlotPropertyDescriptor<Filtration>] = &[
                prop_slot_ro!(Occupied, &[0, 1], |device: &Filtration, idx, _| Ok(
                    if device.get_slot(idx).unwrap().is_empty() {
                        0.0
                    } else {
                        1.0
                    }
                )),
                prop_slot_ro!(OccupantHash, &[0, 1], |device: &Filtration, idx, _| {
                    let item = device.get_slot(idx).unwrap().get_item();
                    if let Some(i) = item {
                        Ok(i.borrow().get_prefab_hash() as f64)
                    } else {
                        Ok(0.0)
                    }
                }),
                prop_slot_ro!(Quantity, &[0, 1], |device: &Filtration, idx, _| {
                    let item = device.get_slot(idx).unwrap().get_item();
                    if let Some(i) = item {
                        Ok(i.borrow().quantity() as f64)
                    } else {
                        Ok(0.0)
                    }
                }),
                prop_slot_ro!(MaxQuantity, &[0, 1], |device: &Filtration, idx, _| {
                    let item = device.get_slot(idx).unwrap().get_item();
                    if let Some(i) = item {
                        Ok(i.borrow().max_quantity() as f64)
                    } else {
                        Ok(0.0)
                    }
                }),
                prop_slot_ro!(FilterType, &[0, 1], |device: &Filtration, idx, _| {
                    let item_opt = device.get_slot(idx).unwrap().get_item();
                    if let Some(item) = item_opt {
                        let item_ref = item.borrow();
                        if item_ref.item_type() == ItemType::Filter
                            && let Some(filter_item) = item_ref.as_any().downcast_ref::<Filter>()
                        {
                            return Ok(filter_item.gas_type() as u32 as f64);
                        }
                    }
                    Ok(0.0)
                }),
                prop_slot_ro!(ReferenceId, &[0, 1], |device: &Filtration, idx, _| {
                    let item = device.get_slot(idx).unwrap().get_item();
                    if let Some(i) = item {
                        Ok(i.borrow().get_id() as f64)
                    } else {
                        Ok(0.0)
                    }
                }),
                prop_slot_ro!(FreeSlots, &[0, 1], |_device: &Filtration, _idx, _| Ok(0.0)),
                prop_slot_ro!(TotalSlots, &[0, 1], |_device: &Filtration, _idx, _| Ok(0.0)),
            ];

            SlotPropertyRegistry::new(DESCRIPTORS)
        })
    }

    // Helper methods to get atmospheric networks
    fn require_network(
        &self,
        connection: DeviceAtmosphericNetworkType,
    ) -> SimulationResult<Shared<AtmosphericNetwork>> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Input => self
                .input_network
                .as_ref()
                .cloned()
                .ok_or(SimulationError::RuntimeError {
                    message: "Filtration device has no input atmospheric network".to_string(),
                    line: 0,
                }),
            Output => {
                self.filtered_network
                    .as_ref()
                    .cloned()
                    .ok_or(SimulationError::RuntimeError {
                        message: "Filtration device has no filtered atmospheric network"
                            .to_string(),
                        line: 0,
                    })
            }
            Output2 => self
                .waste_network
                .as_ref()
                .cloned()
                .ok_or(SimulationError::RuntimeError {
                    message: "Filtration device has no waste atmospheric network".to_string(),
                    line: 0,
                }),
            _ => Err(SimulationError::RuntimeError {
                message: format!(
                    "Filtration device does not support atmospheric connection type {:?}",
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
    ) -> Result<&mut OptShared<AtmosphericNetwork>, SimulationError> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Input => Ok(&mut self.input_network),
            Output => Ok(&mut self.filtered_network),
            Output2 => Ok(&mut self.waste_network),
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

/// `Device` trait implementation for `Filtration` providing logic access and network handling.
impl Device for Filtration {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        Filtration::prefab_hash()
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

    fn rename(&mut self, name: &str) {
        let old_name_hash = self.get_name_hash();
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

    fn supported_slot_types(&self) -> Vec<LogicSlotType> {
        Self::slot_properties().supported_types()
    }

    fn read_slot(&self, index: usize, slot_logic_type: LogicSlotType) -> SimulationResult<f64> {
        // Validate index first to preserve previous error semantics
        if index >= self.slots.len() {
            return Err(SimulationError::RuntimeError {
                message: format!("Slot index out of range: {index}"),
                line: 0,
            });
        }

        // Use a slot property registry keyed by `LogicSlotType` to handle reads
        match Self::slot_properties().read(self, index, slot_logic_type) {
            Ok(v) => Ok(v),
            Err(_) => Err(SimulationError::RuntimeError {
                message: format!(
                    "Filtration does not support reading slot logic type {slot_logic_type:?}"
                ),
                line: 0,
            }),
        }
    }

    fn write_slot(
        &self,
        _index: usize,
        slot_logic_type: LogicSlotType,
        _value: f64,
    ) -> SimulationResult<()> {
        Err(SimulationError::RuntimeError {
            message: format!(
                "Filtration does not support writing slot logic type {slot_logic_type:?}"
            ),
            line: 0,
        })
    }

    fn update(&self, _tick: u64) -> SimulationResult<()> {
        // Only run filtration when device is On and Mode is enabled
        if *self.on.borrow() == 0.0 || *self.mode.borrow() == 0.0 {
            return Ok(());
        }

        // Ensure input and both outputs exist; error if any missing
        let input_rc = self.require_network(DeviceAtmosphericNetworkType::Input)?;
        let filtered_rc = self.require_network(DeviceAtmosphericNetworkType::Output)?;
        let waste_rc = self.require_network(DeviceAtmosphericNetworkType::Output2)?;

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
        let mut transfer_mixture = input_mut.remove_moles(transfer_moles_amount, MatterState::All);

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
                .run(self.max_instructions_per_tick)?
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

    fn properties() -> &'static PropertyRegistry<Self> {
        Filtration::properties()
    }

    fn slot_properties() -> &'static SlotPropertyRegistry<Self> {
        Filtration::slot_properties()
    }

    fn display_name_static() -> &'static str {
        Filtration::display_name_static()
    }

    fn required_atmospheric_connections() -> Vec<DeviceAtmosphericNetworkType> {
        use crate::devices::DeviceAtmosphericNetworkType::*;
        vec![Input, Output, Output2]
    }

    fn as_ic_host_device(&self) -> Option<&dyn ICHostDevice> {
        Some(self)
    }

    fn as_ic_host_device_mut(&mut self) -> Option<&mut dyn ICHostDevice> {
        Some(self)
    }

    fn as_slot_host_device(&self) -> Option<&dyn SlotHostDevice> {
        Some(self)
    }

    fn as_slot_host_device_mut(&mut self) -> Option<&mut dyn SlotHostDevice> {
        Some(self)
    }

    fn as_atmospheric_device(&self) -> Option<&dyn AtmosphericDevice> {
        Some(self)
    }

    fn as_atmospheric_device_mut(&mut self) -> Option<&mut dyn AtmosphericDevice> {
        Some(self)
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
        self.max_instructions_per_tick
    }
}

impl ICHostDeviceMemoryOverride for Filtration {}

impl SlotHostDevice for Filtration {
    fn try_insert_item(
        &mut self,
        index: usize,
        incoming: Shared<dyn Item>,
    ) -> Result<(), Shared<dyn Item>> {
        if index >= self.slots.len() {
            return Err(incoming);
        }

        let slot = &mut self.slots[index];
        slot.try_insert(incoming)
    }

    fn remove_item(&mut self, index: usize) -> OptShared<dyn Item> {
        if index >= self.slots.len() {
            None
        } else {
            self.slots[index].remove()
        }
    }

    fn slot_count(&self) -> usize {
        self.slots.len()
    }
}

impl AtmosphericDevice for Filtration {
    fn set_atmospheric_network(
        &mut self,
        connection: DeviceAtmosphericNetworkType,
        network: OptShared<AtmosphericNetwork>,
    ) -> SimulationResult<()> {
        let slot = self.network_slot_mut(connection)?;
        *slot = network;
        Ok(())
    }

    fn get_atmospheric_network(
        &self,
        connection: DeviceAtmosphericNetworkType,
    ) -> OptShared<AtmosphericNetwork> {
        use DeviceAtmosphericNetworkType::*;
        match connection {
            Input => self.input_network.clone(),
            Output => self.filtered_network.clone(),
            Output2 => self.waste_network.clone(),
            _ => None,
        }
    }
}

impl Display for Filtration {
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

        write!(
            f,
            "Filtration {{ name: \"{}\", id: {}, on: {}, mode: {}",
            self.name, self.reference_id, on_str, mode_str
        )?;

        if let Some(net) = &self.input_network {
            write!(f, ", input: {}", net.borrow().mixture())?;
        }
        if let Some(net) = &self.filtered_network {
            write!(f, ", filtered: {}", net.borrow().mixture())?;
        }
        if let Some(net) = &self.waste_network {
            write!(f, ", waste: {}", net.borrow().mixture())?;
        }

        // Active filters
        let active = self.active_filters();
        if !active.is_empty() {
            let list = active
                .iter()
                .map(|g| g.symbol())
                .collect::<Vec<_>>()
                .join(", ");
            write!(f, ", filters: [{}]", list)?;
        }

        write!(f, " }}")
    }
}

impl Debug for Filtration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
