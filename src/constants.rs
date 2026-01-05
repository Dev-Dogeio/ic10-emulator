//! Built-in constants for IC10

use std::{collections::HashMap, f64};

use crate::{atmospherics::IDEAL_GAS_CONSTANT, parser::string_to_hash};

/// Stack size for the IC housing
pub const STACK_SIZE: usize = 512;

/// Number of registers (r0-r15 general purpose, r16=sp, r17=ra)
pub const REGISTER_COUNT: usize = 18;

/// Stack pointer register index (r16/sp)
pub const STACK_POINTER_INDEX: usize = 16;

/// Return address register index (r17/ra)
pub const RETURN_ADDRESS_INDEX: usize = 17;

/// Prefab hash for the IC Housing
pub const IC_HOUSING_PREFAB_HASH: i32 = string_to_hash("StructureCircuitHousing");

/// Built-in IC10 constants
pub fn get_builtin_constants() -> HashMap<String, f64> {
    let mut constants = HashMap::new();

    // nan - A constant representing 'not a number'
    constants.insert("nan".to_string(), f64::NAN);

    // pinf - A constant representing positive infinity
    constants.insert("pinf".to_string(), f64::INFINITY);

    // ninf - A constant representing negative infinity
    constants.insert("ninf".to_string(), f64::NEG_INFINITY);

    // pi - Ratio of circumference to diameter
    constants.insert("pi".to_string(), f64::consts::PI);

    // tau - Ratio of circumference to radius (2*pi)
    constants.insert("tau".to_string(), 2.0 * f64::consts::PI);

    // deg2rad - Degrees to radians conversion
    constants.insert("deg2rad".to_string(), f64::consts::PI / 180.0);

    // rad2deg - Radians to degrees conversion (matches C# exactly: 57.295780181884766)
    constants.insert("rad2deg".to_string(), 57.295780181884766);

    // epsilon - Smallest positive subnormal > 0
    constants.insert("epsilon".to_string(), 4.94065645841247E-324);

    // rgas - Universal gas constant (J/(mol*K))
    constants.insert("rgas".to_string(), IDEAL_GAS_CONSTANT);

    constants
}
