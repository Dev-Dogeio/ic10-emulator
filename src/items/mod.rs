//! Item and slot system for devices
use std::collections::HashSet;

pub mod filter;
pub mod item;
pub mod item_integrated_circuit_10;

pub use filter::Filter;
pub use filter::FilterSize;
pub use item::{Item, ItemType};
pub use item_integrated_circuit_10::ItemIntegratedCircuit10;

/// Represents a slot in a device that can hold items
/// Slots can have filters that restrict which items they accept
#[derive(Debug)]
pub struct Slot {
    /// The current item in the slot, if any
    item: Option<Box<dyn Item>>,

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
    /// - If the slot is empty the item is placed into the slot (ownership moved).
    /// - If the slot contains an item of the same `ItemType`, we attempt to merge
    ///   the incoming item into the existing one using `Item::merge`. If the
    ///   incoming item still has leftover quantity after merging it is returned
    ///   to the caller as `Err(leftover)`.
    pub fn try_insert(&mut self, mut incoming: Box<dyn Item>) -> Result<(), Box<dyn Item>> {
        if !self.is_allowed(incoming.item_type()) {
            return Err(incoming);
        }

        match self.item.as_mut() {
            None => {
                self.item = Some(incoming);
                Ok(())
            }
            Some(item) => {
                // Only same item types may be merged
                if item.item_type() != incoming.item_type() {
                    return Err(incoming);
                }

                item.merge(&mut *incoming);

                if incoming.quantity() == 0 {
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
    pub fn remove(&mut self) -> Option<Box<dyn Item + '_>> {
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
            Some(item) => item.max_quantity().saturating_sub(item.quantity()),
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
                if existing.item_type() != incoming.item_type() {
                    0
                } else {
                    existing.max_quantity().saturating_sub(existing.quantity())
                }
            }
        }
    }

    /// Get a reference to the item in the slot
    pub fn get_item(&self) -> Option<&dyn Item> {
        self.item.as_deref()
    }

    /// Get a mutable reference to the item in the slot
    pub fn get_item_mut(&mut self) -> Option<&mut (dyn Item + 'static)> {
        self.item.as_deref_mut()
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self::new(None)
    }
}
