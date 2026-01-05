//! Item trait and types

use std::any::Any;
use std::fmt::Debug;
use std::str::FromStr;

/// Represents different types of items in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    /// The IC10 chip
    ItemIntegratedCircuit10,
    /// Filter for filtration devices
    Filter,
}

impl ItemType {
    /// Get the string representation of the item type
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

/// Trait for all items in the simulator
/// Items can be simple objects (ore/ingots) or complex stateful objects (chips)
pub trait Item: Debug {
    /// Get the type of this item
    fn item_type(&self) -> ItemType;

    /// Get the unique global ID of this item
    fn get_id(&self) -> i32;

    /// Get the prefab hash for this item
    fn get_prefab_hash(&self) -> i32;

    /// Get the quantity of this item
    fn quantity(&self) -> u32;

    /// Set the quantity of this item
    fn set_quantity(&mut self, _quantity: u32) -> bool;

    /// Get the maximum quantity allowed for this item (stack size).
    fn max_quantity(&self) -> u32;

    /// Merge another item into this one, edits both items in-place.
    /// Returns if the merge was successful (atleast 1 item transferred).
    fn merge(&mut self, other: &mut dyn Item) -> bool;

    /// Returns self as Any for downcasting to concrete types
    fn as_any(&self) -> &dyn Any;

    /// Returns self as mutable Any for downcasting to concrete types
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
