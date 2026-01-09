//! Filter item implementation

use super::item::{Item, ItemType};
use crate::{
    allocate_global_id, atmospherics::GasType, items::SimulationItemSettings,
    parser::string_to_hash, reserve_global_id,
};
use std::any::Any;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub enum FilterSize {
    Small,
    Medium,
    Large,
    Infinite,
}

/// Filter item for filtration devices
#[derive(Debug)]
pub struct Filter {
    id: i32,
    quantity: f64,
    gas_type: GasType,
    size: FilterSize,
}

impl Filter {
    /// Create a new `Filter` with default values
    pub fn new(settings: Option<SimulationItemSettings>) -> Self {
        let settings = settings.unwrap_or_default();

        let id = if let Some(requested_id) = settings.id {
            reserve_global_id(requested_id)
        } else {
            allocate_global_id()
        };

        let quantity = settings.quantity.unwrap_or(100) as f64;

        let gas_type = settings.gas_type.unwrap_or(GasType::CarbonDioxide);

        let size = settings.filter_size.unwrap_or(FilterSize::Small);

        Self {
            id,
            quantity,
            gas_type,
            size,
        }
    }

    /// Set the target gas type for this filter
    pub fn set_gas_type(&mut self, gas: GasType) {
        self.gas_type = gas;
    }

    /// Get the filter's target gas type
    pub fn gas_type(&self) -> GasType {
        self.gas_type
    }

    /// Set the filter size
    pub fn set_size(&mut self, size: FilterSize) {
        self.size = size;
    }

    /// Get the filter size
    pub fn size(&self) -> FilterSize {
        self.size
    }

    /// Set filter quantity (0..=100)
    pub fn set_quantity(&mut self, quantity: u32) -> bool {
        if quantity <= self.max_quantity() {
            self.quantity = quantity as f64;
            true
        } else {
            false
        }
    }

    /// Compute prefab hash for `gas` and `size`
    pub fn prefab_hash_for(gas: GasType, size: FilterSize) -> i32 {
        let gas_name = gas.filter_name();
        let suffix = match size {
            FilterSize::Small => "",
            FilterSize::Medium => "M",
            FilterSize::Large => "L",
            FilterSize::Infinite => "Infinite",
        };
        string_to_hash(&format!("ItemGasFilter{}{}", gas_name, suffix))
    }

    /// Prefab hash for this filter
    pub fn prefab_hash(&self) -> i32 {
        Filter::prefab_hash_for(self.gas_type, self.size)
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new(None)
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
