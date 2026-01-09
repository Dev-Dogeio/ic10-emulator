//! IC housing device: hosts IC10 chips and exposes registers/memory.

use crate::constants::DEFAULT_MAX_INSTRUCTIONS_PER_TICK;
use crate::conversions::fmt_trim;
use crate::{
    CableNetwork, allocate_global_id,
    constants::STACK_SIZE,
    devices::{
        ChipSlot, Device, ICHostDevice, ICHostDeviceMemoryOverride, LogicType,
        SimulationDeviceSettings,
        property_descriptor::{PropertyDescriptor, PropertyRegistry},
    },
    error::{SimulationError, SimulationResult},
    parser::string_to_hash,
    reserve_global_id,
    types::{OptShared, Shared, shared},
};
use crate::{prop_ro, prop_rw_bool, prop_rw_clamped};

use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::sync::OnceLock;

/// IC housing: holds an IC10 chip and exposes host interfaces
pub struct ICHousing {
    /// Device name
    name: String,
    /// Connected network
    network: OptShared<CableNetwork>,

    /// The device reference ID
    reference_id: i32,
    /// The On state
    on: RefCell<f64>,
    /// The Setting state
    setting: RefCell<f64>,

    /// Chip hosting helper (shared so it can be referenced by chips)
    chip_host: Shared<ChipSlot>,

    /// Max instructions an installed IC can execute per tick
    max_instructions_per_tick: usize,
}

/// Constructors and helpers
impl ICHousing {
    /// Compile-time prefab hash constant for this device
    pub const PREFAB_HASH: i32 = string_to_hash("StructureCircuitHousing");

    /// Create a new `ICHousing`
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
            setting: RefCell::new(0.0),
            on: RefCell::new(1.0),
            reference_id,
            chip_host: ChipSlot::new(6),
            max_instructions_per_tick,
        });

        s.borrow()
            .chip_host
            .borrow_mut()
            .set_host_device(Some(s.clone()));

        s
    }

    /// Prefab hash for `ICHousing`
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }

    pub fn display_name_static() -> &'static str {
        "IC Housing"
    }

    /// Get the property registry for this device type
    #[rustfmt::skip]
    pub fn properties() -> &'static PropertyRegistry<Self> {
        use LogicType::*;
        static REGISTRY: OnceLock<PropertyRegistry<ICHousing>> = OnceLock::new();

        REGISTRY.get_or_init(|| {
            const DESCRIPTORS: &[PropertyDescriptor<ICHousing>] = &[
                prop_ro!(ReferenceId, |device, _| Ok(device.reference_id as f64)),
                prop_ro!(PrefabHash, |device, _| Ok(device.get_prefab_hash() as f64)),
                prop_ro!(NameHash, |device, _| Ok(device.get_name_hash() as f64)),
                prop_rw_clamped!(Setting, setting, -f64::INFINITY, f64::INFINITY),
                prop_rw_bool!(On, on),
                prop_ro!(StackSize, |_, _| Ok(STACK_SIZE as f64)),
                PropertyDescriptor::read_write(
                    LineNumber,
                    |device, _| {
                        if let Some(chip) = device.chip_slot().borrow().get_chip() {
                            Ok(chip.get_pc() as f64)
                        } else {
                            Ok(0.0)
                        }
                    },
                    |device, _, value| {
                        if value.is_nan() || value.is_infinite() || value < 0.0 {
                            return Err(SimulationError::RuntimeError {
                                message: "Invalid line number".to_string(),
                                line: 0,
                            });
                        }

                        let pc = value as usize;
                        if let Some(chip) = device.chip_slot().borrow().get_chip() {
                            chip.set_pc(pc);
                            Ok(())
                        } else {
                            Err(SimulationError::RuntimeError {
                                message: "No chip installed".to_string(),
                                line: 0,
                            })
                        }
                    },
                ),
            ];

            PropertyRegistry::new(DESCRIPTORS)
        })
    }
}

/// `Device` trait implementation for `ICHousing` providing logic access, naming, and chip hosting.
impl Device for ICHousing {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        ICHousing::prefab_hash()
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

    fn run(&self) -> SimulationResult<()> {
        if *self.on.borrow() != 0.0 {
            ICHostDevice::run(self)?;
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
        ICHousing::properties()
    }

    fn display_name_static() -> &'static str {
        ICHousing::display_name_static()
    }

    fn as_ic_host_device(&self) -> Option<&dyn ICHostDevice> {
        Some(self)
    }

    fn as_ic_host_device_mut(&mut self) -> Option<&mut dyn ICHostDevice> {
        Some(self)
    }
}

/// `ICHostDevice` helpers for `ICHousing` (chip slot and memory access).
impl ICHostDevice for ICHousing {
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

impl ICHostDeviceMemoryOverride for ICHousing {}

impl Display for ICHousing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let on_str = if *self.on.borrow() == 0.0 {
            "Off"
        } else {
            "On"
        };
        let setting_str = fmt_trim(*self.setting.borrow(), 3);
        write!(
            f,
            "ICHousing {{ name: \"{}\", id: {}, on: {}, setting: {} }}",
            self.name, self.reference_id, on_str, setting_str
        )
    }
}

impl Debug for ICHousing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
