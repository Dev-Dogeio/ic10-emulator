//! Daylight sensor: provides horizontal and vertical sun angles.

use std::fmt::{Debug, Display};
use std::sync::OnceLock;
use std::{cell::RefCell, f64};

use crate::constants::DEFAULT_TICKS_PER_DAY;
use crate::conversions::fmt_trim;
use crate::types::OptWeakShared;
use crate::{
    CableNetwork, allocate_global_id,
    devices::{
        Device, LogicType, SimulationDeviceSettings,
        property_descriptor::{PropertyDescriptor, PropertyRegistry},
    },
    error::SimulationResult,
    parser::string_to_hash,
    reserve_global_id,
    types::{OptShared, Shared, shared},
};
use crate::{prop_ro, prop_rw_bool};

/// Daylight sensor: tracks sun position
pub struct DaylightSensor {
    /// Device name
    name: String,
    /// Connected network
    network: OptWeakShared<CableNetwork>,

    /// The device reference ID
    reference_id: i32,
    /// The On state
    on: RefCell<f64>,
    /// The horizontal angle (degrees)
    horizontal: RefCell<f64>,
    /// The vertical angle (degrees)
    vertical: RefCell<f64>,

    /// Number of ticks in a day cycle used to determine sun position
    ticks_per_day: f64,
}

/// Constructors and helpers
impl DaylightSensor {
    /// Compile-time prefab hash constant for this device
    pub const PREFAB_HASH: i32 = string_to_hash("StructureDaylightSensor");

    /// Create a new `DaylightSensor`
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

        let ticks_per_day = settings.ticks_per_day.unwrap_or(DEFAULT_TICKS_PER_DAY);

        shared(Self {
            name,
            network: None,
            reference_id,
            on: RefCell::new(1.0),
            horizontal: RefCell::new(0.0),
            vertical: RefCell::new(0.0),
            ticks_per_day,
        })
    }

    /// Current horizontal angle (degrees)
    pub fn horizontal(&self) -> f64 {
        *self.horizontal.borrow()
    }

    /// Current vertical angle (degrees)
    pub fn vertical(&self) -> f64 {
        *self.vertical.borrow()
    }

    /// Prefab hash for `DaylightSensor`
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }

    pub fn display_name_static() -> &'static str {
        "Daylight Sensor"
    }

    /// Get the property registry for this device type
    #[rustfmt::skip]
    pub fn properties() -> &'static PropertyRegistry<Self> {
        use LogicType::*;
        static REGISTRY: OnceLock<PropertyRegistry<DaylightSensor>> = OnceLock::new();

        REGISTRY.get_or_init(|| {
            const DESCRIPTORS: &[PropertyDescriptor<DaylightSensor>] = &[
                prop_ro!(ReferenceId, |device, _| Ok(device.reference_id as f64)),
                prop_ro!(PrefabHash, |device, _| Ok(device.get_prefab_hash() as f64)),
                prop_ro!(NameHash, |device, _| Ok(device.get_name_hash() as f64)),
                prop_ro!(Horizontal, |device, _| Ok(*device.horizontal.borrow())),
                prop_ro!(Vertical, |device, _| Ok(*device.vertical.borrow())),
                prop_rw_bool!(On, on),
            ];

            PropertyRegistry::new(DESCRIPTORS)
        })
    }
}

/// `Device` trait implementation for `DaylightSensor` providing logic access and day-cycle updates.
impl Device for DaylightSensor {
    fn get_id(&self) -> i32 {
        self.reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        DaylightSensor::prefab_hash()
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

    fn update(&self, tick: u64) -> SimulationResult<()> {
        // Only update when device is On (non-zero)
        if *self.on.borrow() == 0.0 {
            return Ok(());
        }

        // Calculate position within the day cycle [0.0, 1.0)
        let day_progress = ((tick % self.ticks_per_day as u64) as f64) / self.ticks_per_day;

        // Horizontal angle: simple rotation around the compass
        // 0 degrees at tick 0, 360 degrees at tick 2400
        let horizontal = day_progress * 360.0;

        // Vertical angle: uses cosine to create smooth oscillation
        // At tick 0 (midnight): vertical = 180 (nadir)
        // At tick 600 (sunrise): vertical = 90 (horizon)
        // At tick 1200 (noon): vertical = 0 (zenith)
        // At tick 1800 (sunset): vertical = 90 (horizon)
        // At tick 2400 (midnight): vertical = 180 (nadir)
        //
        // Formula: vertical = 90 + 90 * cos(2π * progress)
        // This gives: 180 at progress=0, 0 at progress=0.5, 180 at progress=1
        let angle_radians = 2.0 * f64::consts::PI * day_progress;
        let vertical = 90.0 + 90.0 * angle_radians.cos();

        // Update the logic fields with the new angles
        *self.horizontal.borrow_mut() = horizontal;
        *self.vertical.borrow_mut() = vertical;

        Ok(())
    }

    fn properties() -> &'static PropertyRegistry<Self> {
        DaylightSensor::properties()
    }

    fn display_name_static() -> &'static str {
        DaylightSensor::display_name_static()
    }
}

impl Display for DaylightSensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let on_str = if *self.on.borrow() == 0.0 {
            "Off"
        } else {
            "On"
        };
        let horiz = fmt_trim(*self.horizontal.borrow(), 2);
        let vert = fmt_trim(*self.vertical.borrow(), 2);
        write!(
            f,
            "DaylightSensor {{ name: \"{}\", id: {}, on: {}, horiz: {}, vert: {} }}",
            self.name, self.reference_id, on_str, horiz, vert
        )
    }
}

impl Debug for DaylightSensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
