//! Item trait and types

use std::any::Any;
use std::fmt::Debug;
use std::str::FromStr;

use crate::items::create_item;
use crate::types::OptShared;

/// Different item kinds in the simulator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    /// The IC10 chip
    ItemIntegratedCircuit10,
    /// Filter for filtration devices
    Filter,
}

impl ItemType {
    /// Return the string name of the item type
    pub fn as_str(&self) -> &'static str {
        match self {
            ItemType::ItemIntegratedCircuit10 => "ItemIntegratedCircuit10",
            ItemType::Filter => "Filter",
        }
    }
}

impl FromStr for ItemType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ItemIntegratedCircuit10" => Ok(ItemType::ItemIntegratedCircuit10),
            "Filter" => Ok(ItemType::Filter),
            _ => Err(()),
        }
    }
}

/// Trait for items; supports quantity, prefab, and merging
pub trait Item: Debug {
    /// Get the type of this item
    fn item_type(&self) -> ItemType;

    /// Unique global ID
    fn get_id(&self) -> i32;

    /// Get the prefab hash for this item
    fn get_prefab_hash(&self) -> i32;

    /// Get the quantity of this item
    fn quantity(&self) -> u32;

    /// Set the quantity of this item
    fn set_quantity(&mut self, _quantity: u32) -> bool;

    /// Maximum stack quantity for this item
    fn max_quantity(&self) -> u32;

    /// Merge another item into this one; return true if any merged
    fn merge(&mut self, other: &mut dyn Item) -> bool;

    /// Returns self as Any for downcasting to concrete types
    fn as_any(&self) -> &dyn Any;

    /// Returns self as mutable Any for downcasting to concrete types
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Create an item by prefab hash. Default implementation delegates to the module-level factory.
    fn create(prefab_hash: i32) -> OptShared<dyn Item>
    where
        Self: Sized,
    {
        create_item(prefab_hash)
    }
}
