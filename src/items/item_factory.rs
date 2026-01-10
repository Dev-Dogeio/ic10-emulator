//! Item factory registry for item creation

use crate::atmospherics::GasType;
use crate::items::{FilterSize, SimulationItemSettings};
use crate::types::{Shared, shared};
use crate::{Filter, Item, ItemIntegratedCircuit10, ItemType, atmospherics, items};
use std::collections::HashMap;
use std::sync::Mutex;

/// Factory function type for creating items
pub type ItemFactoryFn =
    Box<dyn Fn(SimulationItemSettings) -> Shared<dyn Item> + Send + Sync + 'static>;

/// Item metadata stored as (display_name, item_type)
pub type ItemMeta = (&'static str, ItemType);

/// Global item factory registry
static ITEM_FACTORY: Mutex<Option<ItemFactoryRegistry>> = Mutex::new(None);

/// Registry for item factory functions and metadata
pub struct ItemFactoryRegistry {
    factories: HashMap<i32, ItemFactoryFn>,
    metas: HashMap<i32, ItemMeta>,
}

impl ItemFactoryRegistry {
    fn new() -> Self {
        Self {
            factories: HashMap::new(),
            metas: HashMap::new(),
        }
    }

    pub fn register(&mut self, prefab_hash: i32, factory: ItemFactoryFn) {
        self.factories.insert(prefab_hash, factory);
    }

    pub fn register_meta(&mut self, prefab_hash: i32, meta: ItemMeta) {
        self.metas.insert(prefab_hash, meta);
    }

    pub fn create_item(
        &self,
        prefab_hash: i32,
        settings: SimulationItemSettings,
    ) -> Option<Shared<dyn Item>> {
        self.factories.get(&prefab_hash).map(|f| (f)(settings))
    }

    pub fn get_meta(&self, prefab_hash: i32) -> Option<(&'static str, ItemType)> {
        self.metas.get(&prefab_hash).copied()
    }

    pub fn registered_prefabs(&self) -> Vec<i32> {
        self.factories.keys().copied().collect()
    }
}

fn get_registry() -> &'static Mutex<Option<ItemFactoryRegistry>> {
    &ITEM_FACTORY
}

fn initialize_registry() -> bool {
    let mut registry_guard = get_registry().lock().unwrap();
    if registry_guard.is_none() {
        *registry_guard = Some(ItemFactoryRegistry::new());
        return true;
    }
    false
}

/// Create an item by prefab hash using the global registry
pub fn create_item(prefab_hash: i32, settings: SimulationItemSettings) -> Option<Shared<dyn Item>> {
    initialize_item_factory();

    let registry_guard = get_registry().lock().unwrap();
    registry_guard
        .as_ref()
        .and_then(|registry| registry.create_item(prefab_hash, settings))
}

/// Get all registered item prefab hashes
pub fn get_registered_item_prefabs() -> Vec<i32> {
    initialize_item_factory();

    let registry_guard = get_registry().lock().unwrap();
    registry_guard
        .as_ref()
        .map(|registry| registry.registered_prefabs())
        .unwrap()
}

/// Register an item factory function
fn register_item_factory(prefab_hash: i32, factory: ItemFactoryFn) {
    let mut registry_guard = get_registry().lock().unwrap();
    if let Some(registry) = registry_guard.as_mut() {
        registry.register(prefab_hash, factory);
    }
}

/// Register item metadata provider
fn register_item_meta(prefab_hash: i32, meta: ItemMeta) {
    let mut registry_guard = get_registry().lock().unwrap();
    if let Some(registry) = registry_guard.as_mut() {
        registry.register_meta(prefab_hash, meta);
    }
}

/// Get metadata for a prefab hash
pub fn get_prefab_metadata(prefab_hash: i32) -> Option<(&'static str, ItemType)> {
    initialize_item_factory();

    let registry_guard = get_registry().lock().unwrap();
    registry_guard
        .as_ref()
        .and_then(|registry| registry.get_meta(prefab_hash))
}

/// Macro to auto-register a simple item type in the factory registry
#[macro_export]
macro_rules! register_item {
    ($item_type:ty, $display:expr, $item_type_enum:expr) => {
        register_item_factory(
            <$item_type>::PREFAB_HASH,
            Box::new(|settings: SimulationItemSettings| shared(<$item_type>::new(settings))),
        );
        register_item_meta(<$item_type>::PREFAB_HASH, ($display, $item_type_enum));
    };
}

/// Initialize the item factory and register all known items and prefabs
fn initialize_item_factory() {
    if initialize_registry() {
        use atmospherics::GasType::*;
        use items::FilterSize::*;

        // Register simple items
        register_item!(
            ItemIntegratedCircuit10,
            "ItemIntegratedCircuit10",
            ItemType::ItemIntegratedCircuit10
        );

        // Register filter prefabs for all gas type + size combinations
        let gas_types = [
            Oxygen,
            Nitrogen,
            CarbonDioxide,
            Volatiles,
            Pollutant,
            NitrousOxide,
            Water,
            // Hydrogen: Does not support filters
        ];

        let sizes = [Small, Medium, Large, Infinite];

        for &g in &gas_types {
            for &s in &sizes {
                let prefab = Filter::prefab_hash_for(g, s);
                register_item_factory(
                    prefab,
                    Box::new(move |mut settings: SimulationItemSettings| {
                        settings.gas_type = Some(g);
                        settings.filter_size = Some(s);
                        shared(Filter::new(settings))
                    }),
                );

                // Register metadata for filters with a human-friendly name without allocating
                fn filter_meta_name(g: GasType, s: FilterSize) -> &'static str {
                    use GasType::*;
                    use items::FilterSize::*;
                    match (g, s) {
                        (Oxygen, Small) => "Oxygen Filter (Small)",
                        (Oxygen, Medium) => "Oxygen Filter (Medium)",
                        (Oxygen, Large) => "Oxygen Filter (Large)",
                        (Oxygen, Infinite) => "Oxygen Filter (Catalytic)",

                        (Nitrogen, Small) => "Nitrogen Filter (Small)",
                        (Nitrogen, Medium) => "Nitrogen Filter (Medium)",
                        (Nitrogen, Large) => "Nitrogen Filter (Large)",
                        (Nitrogen, Infinite) => "Nitrogen Filter (Catalytic)",

                        (CarbonDioxide, Small) => "Carbon Dioxide Filter (Small)",
                        (CarbonDioxide, Medium) => "Carbon Dioxide Filter (Medium)",
                        (CarbonDioxide, Large) => "Carbon Dioxide Filter (Large)",
                        (CarbonDioxide, Infinite) => "Carbon Dioxide Filter (Catalytic)",

                        (Volatiles, Small) => "Volatiles Filter (Small)",
                        (Volatiles, Medium) => "Volatiles Filter (Medium)",
                        (Volatiles, Large) => "Volatiles Filter (Large)",
                        (Volatiles, Infinite) => "Volatiles Filter (Catalytic)",

                        (Pollutant, Small) => "Pollutant Filter (Small)",
                        (Pollutant, Medium) => "Pollutant Filter (Medium)",
                        (Pollutant, Large) => "Pollutant Filter (Large)",
                        (Pollutant, Infinite) => "Pollutant Filter (Catalytic)",

                        (NitrousOxide, Small) => "Nitrous Oxide Filter (Small)",
                        (NitrousOxide, Medium) => "Nitrous Oxide Filter (Medium)",
                        (NitrousOxide, Large) => "Nitrous Oxide Filter (Large)",
                        (NitrousOxide, Infinite) => "Nitrous Oxide Filter (Catalytic)",

                        (Steam, Small) => "Steam Filter (Small)",
                        (Steam, Medium) => "Steam Filter (Medium)",
                        (Steam, Large) => "Steam Filter (Large)",
                        (Steam, Infinite) => "Steam Filter (Catalytic)",

                        (Hydrogen, Small) => "Hydrogen Filter (Small)",
                        (Hydrogen, Medium) => "Hydrogen Filter (Medium)",
                        (Hydrogen, Large) => "Hydrogen Filter (Large)",
                        (Hydrogen, Infinite) => "Hydrogen Filter (Catalytic)",

                        (Water, Small) => "Water Filter (Small)",
                        (Water, Medium) => "Water Filter (Medium)",
                        (Water, Large) => "Water Filter (Large)",
                        (Water, Infinite) => "Water Filter (Catalytic)",

                        _ => "Unknown Filter",
                    }
                }

                register_item_meta(prefab, (filter_meta_name(g, s), ItemType::Filter));
            }
        }
    }
}
