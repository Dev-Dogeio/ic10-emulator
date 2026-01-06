//! Chemistry constants for atmospheric simulation

/// Ideal Gas Constant R (J/(mol·K))
/// Used in ideal gas law: PV = nRT
pub const IDEAL_GAS_CONSTANT: f64 = 8.31446261815324;

/// Standard atmospheric pressure (kPa)
pub const ONE_ATMOSPHERE: f64 = 101.325;

/// Conversion from Celsius to Kelvin
pub const CELSIUS_TO_KELVIN: f64 = 273.15;

/// Minimum quantity of moles considered non-zero
/// This threshold is for tiny *per-gas* quantities.
pub const MINIMUM_QUANTITY_MOLES: f64 = 1e-5;

/// Minimum quantity of moles considered non-zero
/// This threshold is for tiny *per-mixture* quantities.
pub const MINIMUM_VALID_TOTAL_MOLES: f64 = 1e-3;

/// Volume of a pipe section (Litres)
pub const PIPE_VOLUME: f64 = 10.0;

/// Maximum pressure that gas pipes are rated for (kPa)
pub const MAX_PRESSURE_GAS_PIPE: f64 = 60_794.998_168_945_31;

/// Minimum gas volume for calculations (Litres)
pub const MINIMUM_GAS_VOLUME: f64 = 0.1;

/// Calculate pressure using ideal gas law
/// P = nRT/V
pub fn calculate_pressure(moles: f64, temperature: f64, volume: f64) -> f64 {
    if volume <= 0.0 {
        return 0.0;
    }
    // PV = nRT => P = nRT/V
    (moles * IDEAL_GAS_CONSTANT * temperature) / volume
}

/// Calculate moles using ideal gas law
/// n = PV/RT
pub fn calculate_moles(pressure: f64, volume: f64, temperature: f64) -> f64 {
    if temperature <= 0.0 {
        return 0.0;
    }
    (pressure * volume) / (IDEAL_GAS_CONSTANT * temperature)
}

/// Calculate temperature using ideal gas law
/// T = PV/nR
pub fn calculate_temperature(pressure: f64, volume: f64, moles: f64) -> f64 {
    if moles <= 0.0 {
        return 0.0;
    }
    let temp = (pressure * volume) / (moles * IDEAL_GAS_CONSTANT);
    temp.max(0.0)
}

/// Calculate volume using ideal gas law
/// V = nRT/P
pub fn calculate_volume(moles: f64, temperature: f64, pressure: f64) -> f64 {
    if pressure <= 0.0 {
        return 0.0;
    }
    (moles * IDEAL_GAS_CONSTANT * temperature) / pressure
}

/// Convert Celsius to Kelvin
pub fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + CELSIUS_TO_KELVIN
}

/// Convert Kelvin to Celsius
pub fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin - CELSIUS_TO_KELVIN
}
