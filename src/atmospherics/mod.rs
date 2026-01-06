//! Atmospherics module for gas simulation
//!
//! This module provides types and utilities for simulating atmospheric
//! conditions, gas mixtures, and energy transfer. Based on the Stationeers
//! game's atmospheric simulation. Supports phase changes between gas and liquid states.

mod chemistry;
mod gas_mixture;
mod gas_type;
mod mole;

pub use chemistry::*;
pub use gas_mixture::GasMixture;
pub use gas_type::{GasType, MatterState};
pub use mole::{Mole, PhaseChangeResult};
