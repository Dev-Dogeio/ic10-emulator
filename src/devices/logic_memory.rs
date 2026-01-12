//! Logic memory device: stores a single numeric value.

use std::cell::RefCell;
use std::fmt::{Debug, Display};
use std::sync::OnceLock;

use crate::conversions::fmt_trim;
use crate::types::OptWeakShared;
use crate::{
    CableNetwork,
    devices::{
        Device, LogicType, SimulationDeviceSettings,
        property_descriptor::{PropertyDescriptor, PropertyRegistry},
    },
    error::SimulationResult,
    parser::string_to_hash,
    types::{OptShared, Shared, shared},
};
use crate::{prop_ro, prop_rw_clamped};

pub struct LogicMemory {
    /// Device name
    name: String,
    /// Connected network
    network: OptWeakShared<CableNetwork>,

    /// Device reference ID
    reference_id: i32,
    /// Stored setting value
    setting: RefCell<f64>,
}

/// Constructors and helpers
impl LogicMemory {
    /// Compile-time prefab hash constant for this device
    pub const PREFAB_HASH: i32 = string_to_hash("StructureLogicMemory");

    /// Create a new `LogicMemory`.
    pub fn new(settings: SimulationDeviceSettings) -> Shared<Self> {
        let name = if let Some(n) = settings.name.as_ref() {
            n.to_string()
        } else {
            Self::display_name_static().to_string()
        };

        shared(Self {
            name,
            network: None,
            setting: RefCell::new(0.0),
            reference_id: settings.id.unwrap(),
        })
    }

    /// Prefab hash for `LogicMemory`
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }

    pub fn display_name_static() -> &'static str {
        "Logic Memory"
    }

    /// Get the property registry for this device type
    #[rustfmt::skip]
    pub fn properties() -> &'static PropertyRegistry<Self> {
        use LogicType::*;
        static REGISTRY: OnceLock<PropertyRegistry<LogicMemory>> = OnceLock::new();

        REGISTRY.get_or_init(|| {
            const DESCRIPTORS: &[PropertyDescriptor<LogicMemory>] = &[
                prop_ro!(ReferenceId, |device, _| Ok(device.reference_id as f64)),
                prop_ro!(PrefabHash, |device, _| Ok(device.get_prefab_hash() as f64)),
                prop_ro!(NameHash, |device, _| Ok(device.get_name_hash() as f64)),
                prop_rw_clamped!(Setting, setting, -f64::INFINITY, f64::INFINITY),
            ];

            PropertyRegistry::new(DESCRIPTORS)
        })
    }
}

/// `Device` trait implementation for `LogicMemory` providing memory access helpers for hosted ICs.
impl Device for LogicMemory {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        LogicMemory::prefab_hash()
    }

    fn get_name_hash(&self) -> i32 {
        string_to_hash(&self.name)
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
        LogicMemory::properties()
    }

    fn display_name_static() -> &'static str {
        LogicMemory::display_name_static()
    }
}

impl Display for LogicMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let setting = fmt_trim(*self.setting.borrow(), 3);
        write!(
            f,
            "LogicMemory {{ name: \"{}\", id: {}, setting: {} }}",
            self.name, self.reference_id, setting
        )
    }
}

impl Debug for LogicMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
