//! Device factory registry for device creation

use crate::{
    LogicType,
    devices::{Device, SimulationSettings},
    types::Shared,
};
use std::collections::HashMap;
use std::sync::Mutex;

/// Factory function type for creating devices
pub type DeviceFactoryFn = fn(Option<SimulationSettings>) -> Shared<dyn Device>;

/// Alias for device property metadata: a vector of tuples describing a `LogicType` and
/// whether that logic type is (readable, writable) for the device.
pub type DeviceProps = Vec<(LogicType, bool, bool)>;

/// Function type that returns device metadata: `(display_name, DeviceProps)`.
pub type DeviceMetaFn = fn() -> (&'static str, DeviceProps);

/// Global device factory registry
static DEVICE_FACTORY: Mutex<Option<DeviceFactoryRegistry>> = Mutex::new(None);

/// Registry for device factory functions and metadata
pub struct DeviceFactoryRegistry {
    factories: HashMap<i32, DeviceFactoryFn>,
    metas: HashMap<i32, DeviceMetaFn>,
}

impl DeviceFactoryRegistry {
    /// Create a new empty registry
    fn new() -> Self {
        Self {
            factories: HashMap::new(),
            metas: HashMap::new(),
        }
    }

    /// Register a device factory function for a prefab hash
    pub fn register(&mut self, prefab_hash: i32, factory: DeviceFactoryFn) {
        self.factories.insert(prefab_hash, factory);
    }

    /// Register device metadata provider for a prefab hash
    pub fn register_meta(&mut self, prefab_hash: i32, meta: DeviceMetaFn) {
        self.metas.insert(prefab_hash, meta);
    }

    /// Create a device by prefab hash
    pub fn create_device(
        &self,
        prefab_hash: i32,
        simulation_settings: Option<SimulationSettings>,
    ) -> Option<Shared<dyn Device>> {
        self.factories
            .get(&prefab_hash)
            .map(|factory| factory(simulation_settings))
    }

    /// Get metadata for a prefab hash
    pub fn get_meta(&self, prefab_hash: i32) -> Option<(&'static str, DeviceProps)> {
        self.metas.get(&prefab_hash).map(|f| f())
    }

    /// Get all registered prefab hashes
    pub fn registered_prefabs(&self) -> Vec<i32> {
        self.factories.keys().copied().collect()
    }
}

/// Create a device by prefab hash using the global registry
pub fn create_device(
    prefab_hash: i32,
    simulation_settings: Option<SimulationSettings>,
) -> Option<Shared<dyn Device>> {
    initialize_device_factory();

    let registry_guard = get_registry().lock().unwrap();
    registry_guard
        .as_ref()
        .and_then(|registry| registry.create_device(prefab_hash, simulation_settings))
}

/// Get all registered prefab hashes
pub fn get_registered_device_prefabs() -> Vec<i32> {
    initialize_device_factory();

    let registry_guard = get_registry().lock().unwrap();
    registry_guard
        .as_ref()
        .map(|registry| registry.registered_prefabs())
        .unwrap()
}

/// Get or initialize the global device factory registry
fn get_registry() -> &'static Mutex<Option<DeviceFactoryRegistry>> {
    &DEVICE_FACTORY
}

/// Register a device factory function
fn register_device_factory(prefab_hash: i32, factory: DeviceFactoryFn) {
    let mut registry_guard = get_registry().lock().unwrap();
    if let Some(registry) = registry_guard.as_mut() {
        registry.register(prefab_hash, factory);
    }
}

/// Register device metadata provider
fn register_device_meta(prefab_hash: i32, meta: DeviceMetaFn) {
    let mut registry_guard = get_registry().lock().unwrap();
    if let Some(registry) = registry_guard.as_mut() {
        registry.register_meta(prefab_hash, meta);
    }
}

/// Get metadata for a prefab hash
pub fn get_prefab_metadata(prefab_hash: i32) -> Option<(&'static str, DeviceProps)> {
    initialize_device_factory();

    let registry_guard = get_registry().lock().unwrap();
    registry_guard
        .as_ref()
        .and_then(|registry| registry.get_meta(prefab_hash))
}

/// Initialize the global device registry
fn initialize_registry() -> bool {
    let mut registry_guard = get_registry().lock().unwrap();
    if registry_guard.is_none() {
        *registry_guard = Some(DeviceFactoryRegistry::new());
        return true;
    }
    false
}

/// Macro to auto-register a device type in the factory registry
#[macro_export]
macro_rules! register_device {
    ($device_type:ty) => {
        register_device_factory(<$device_type>::PREFAB_HASH, |settings| {
            <$device_type>::new(settings)
        });
        register_device_meta(<$device_type>::PREFAB_HASH, meta_from_type::<$device_type>);
    };
}

/// Produce `(display_name, DeviceProps)` for a device type `T` using its static
/// `properties()` and `display_name_static()` methods.
fn meta_from_type<T>() -> (&'static str, DeviceProps)
where
    T: crate::devices::Device + 'static,
{
    let props = T::properties()
        .supported_types()
        .into_iter()
        .map(|lt| {
            (
                lt,
                T::properties().can_read(lt),
                T::properties().can_write(lt),
            )
        })
        .collect();
    (T::display_name_static(), props)
}

/// Initialize the device factory and register all devices.
/// This function handles registry creation and registrations.
fn initialize_device_factory() {
    use crate::devices::*;

    if initialize_registry() {
        // Register each device
        register_device!(VolumePump);
        register_device!(AirConditioner);
        register_device!(Filtration);
        register_device!(DaylightSensor);
        register_device!(ICHousing);
        register_device!(LogicMemory);
    }
}
