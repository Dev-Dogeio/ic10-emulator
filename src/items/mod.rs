//! Item and slot system for devices
use std::cell::{Ref, RefMut};
use std::collections::HashSet;

use crate::atmospherics::GasType;
use crate::types::OptShared;
use crate::types::Shared;

pub mod filter;
pub mod item;
pub mod item_factory;
pub mod item_integrated_circuit_10;

pub use filter::Filter;
pub use filter::FilterSize;
pub use item::{Item, ItemType};
pub use item_factory::{get_prefab_metadata, get_registered_item_prefabs};
pub use item_integrated_circuit_10::ItemIntegratedCircuit10;

/// Settings used when creating items during simulation. Fields are optional and
/// when provided will be applied during initialization.
#[derive(Clone, Debug, Default)]
pub struct SimulationItemSettings {
    /// Optional requested ID for the item
    pub id: Option<i32>,
    /// Optional quantity to initialize to
    pub quantity: Option<u32>,
    /// Optional gas type for items that care about gas
    pub gas_type: Option<GasType>,
    /// Optional filter size for filter items
    pub filter_size: Option<FilterSize>,
}

/// Create an item by `prefab_hash`, returning `Some` when recognized.
pub fn create_item(
    prefab_hash: i32,
    settings: Option<SimulationItemSettings>,
) -> OptShared<dyn Item> {
    item_factory::create_item(prefab_hash, settings)
}

/// A device slot that can hold an item and enforces allowed types
#[derive(Debug)]
pub struct Slot {
    /// The current item in the slot, if any (shared reference)
    item: OptShared<dyn Item>,

    /// Set of allowed item types for this slot
    /// If empty, all item types are allowed
    allowed_types: HashSet<ItemType>,
}

impl Slot {
    /// Create a new unrestricted slot with unlimited capacity
    pub fn new(item_type: Option<ItemType>) -> Self {
        let mut allowed_types = HashSet::new();

        if let Some(allowed_type) = item_type {
            allowed_types.insert(allowed_type);
        }

        Self {
            item: None,
            allowed_types,
        }
    }

    /// Try to insert an item into the slot.
    ///
    /// On complete merge returns `Ok(())`.
    /// Otherwise returns `Err(item)` and the leftover item that could not be inserted.
    ///
    /// Semantics:
    /// - If the slot rejects the item type, insertion fails.
    /// - If the slot is empty the item is placed into the slot.
    /// - If the slot contains an item of the same `ItemType`, we attempt to merge
    ///   the incoming item into the existing one using `Item::merge`. If the
    ///   incoming item still has leftover quantity after merging it is returned
    ///   to the caller as `Err(leftover)`.
    pub fn try_insert(&mut self, incoming: Shared<dyn Item>) -> Result<(), Shared<dyn Item>> {
        if !self.is_allowed(incoming.borrow().item_type()) {
            return Err(incoming);
        }

        match &self.item {
            None => {
                self.item = Some(incoming);
                Ok(())
            }
            Some(existing) => {
                // Only same item types may be merged
                if existing.borrow().item_type() != incoming.borrow().item_type() {
                    return Err(incoming);
                }

                existing.borrow_mut().merge(&mut *incoming.borrow_mut());

                if incoming.borrow().quantity() == 0 {
                    Ok(())
                } else {
                    Err(incoming)
                }
            }
        }
    }

    /// Check if an item type is allowed in this slot
    pub fn is_allowed(&self, item_type: ItemType) -> bool {
        // If allowed_types is empty, all types are allowed
        self.allowed_types.is_empty() || self.allowed_types.contains(&item_type)
    }

    /// Remove an item from the slot
    pub fn remove(&mut self) -> OptShared<dyn Item> {
        self.item.take()
    }

    /// Check if the slot is empty
    pub fn is_empty(&self) -> bool {
        self.item.is_none()
    }

    /// Check if the slot is full (can't accept more of the current item)
    pub fn is_full(&self) -> bool {
        self.available_space() == 0
    }

    /// Get the available space in the slot
    pub fn available_space(&self) -> u32 {
        match &self.item {
            None => u32::MAX,
            Some(item) => item
                .borrow()
                .max_quantity()
                .saturating_sub(item.borrow().quantity()),
        }
    }

    /// Get the available space in this slot for a *specific* incoming item.
    pub fn available_space_for(&self, incoming: &dyn Item) -> u32 {
        if !self.is_allowed(incoming.item_type()) {
            return 0;
        }

        match &self.item {
            None => incoming.max_quantity(),
            Some(existing) => {
                let existing_ref = existing.borrow();
                if existing_ref.item_type() != incoming.item_type() {
                    0
                } else {
                    existing_ref
                        .max_quantity()
                        .saturating_sub(existing_ref.quantity())
                }
            }
        }
    }

    /// Get the shared item in the slot
    pub fn get_item(&self) -> OptShared<dyn Item> {
        self.item.clone()
    }

    /// Borrow the item in the slot as type T, if it matches
    pub fn borrow_item<T: Item + 'static>(&self) -> Option<Ref<'_, T>> {
        let shared = self.item.as_ref()?;
        let borrow = shared.borrow();

        if borrow.as_any().is::<T>() {
            Some(Ref::map(borrow, |item| {
                item.as_any().downcast_ref::<T>().unwrap()
            }))
        } else {
            None
        }
    }

    /// Borrow the item in the slot as type T mutably, if it matches
    pub fn borrow_item_mut<T: Item + 'static>(&self) -> Option<RefMut<'_, T>> {
        let shared = self.item.as_ref()?;
        let borrow = shared.borrow_mut();

        if borrow.as_any().is::<T>() {
            Some(RefMut::map(borrow, |item| {
                item.as_any_mut().downcast_mut::<T>().unwrap()
            }))
        } else {
            None
        }
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self::new(None)
    }
}
