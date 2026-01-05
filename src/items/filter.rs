//! Filter item implementation

use crate::{allocate_global_id, atmospherics::GasType, parser::string_to_hash};

use super::item::{Item, ItemType};
use std::any::Any;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterSize {
    Small,
    Medium,
    Large,
    Infinite,
}

/// A filter item for filtration devices
#[derive(Debug)]
pub struct Filter {
    id: i32,
    quantity: f64,
    gas_type: GasType,
    size: FilterSize,
}

impl Filter {
    /// Create a new filter item
    pub fn new(quantity: f64, gas_type: GasType, size: FilterSize) -> Self {
        if !(0.0..=100.0).contains(&quantity) {
            panic!("Filter quantity must be between 0 and 100");
        }

        Self {
            id: allocate_global_id(),
            quantity,
            gas_type,
            size,
        }
    }

    /// Get the gas type this filter targets
    pub fn gas_type(&self) -> GasType {
        self.gas_type
    }

    /// Get the size of this filter
    pub fn size(&self) -> FilterSize {
        self.size
    }
}

impl Item for Filter {
    fn item_type(&self) -> ItemType {
        ItemType::Filter
    }

    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_prefab_hash(&self) -> i32 {
        let gas_name = self.gas_type.filter_name();
        let suffix = match self.size {
            FilterSize::Small => "",
            FilterSize::Medium => "M",
            FilterSize::Large => "L",
            FilterSize::Infinite => "Infinite",
        };
        string_to_hash(&format!("ItemGasFilter{}{}", gas_name, suffix))
    }

    fn quantity(&self) -> u32 {
        self.quantity.ceil() as u32
    }

    fn set_quantity(&mut self, quantity: u32) -> bool {
        if quantity <= self.max_quantity() {
            self.quantity = quantity as f64;
            true
        } else {
            false
        }
    }

    fn max_quantity(&self) -> u32 {
        100
    }

    fn merge(&mut self, _other: &mut dyn Item) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
