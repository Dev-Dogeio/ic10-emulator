//! IC10 emulator library
//!
//! This library provides a complete emulator for the IC10 programmable chip.
//! It also simulates cable and atmospheric (pipe/grids) networks.

pub mod animation_curve;
pub mod atmospherics;
pub mod constants;
pub mod conversions;
pub mod devices;
pub mod error;
pub mod id;
pub mod instruction;
pub mod items;
pub mod logic;
pub mod networks;
pub mod parser;
pub mod simulation_manager;
pub mod types;

#[cfg(test)]
pub mod tests;

pub use constants::get_builtin_constants;
pub use devices::{DaylightSensor, Device, ICHousing, LogicSlotType, LogicType};
pub use error::{SimulationError, SimulationResult};

pub use id::{allocate_global_id, reserve_global_id, reset_global_id_counter};
pub use instruction::Instruction;
pub use items::{Filter, Item, ItemIntegratedCircuit10, ItemType, Slot};
pub use networks::{AtmosphericNetwork, BatchMode, CableNetwork};
pub use simulation_manager::SimulationManager;

#[cfg(feature = "wasm")]
pub mod wasm;
