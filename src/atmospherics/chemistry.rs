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

/// Minimum moles for world-level phase changes
pub const MINIMUM_WORLD_VALID_TOTAL_MOLES: f64 = 0.1;

/// Volume of a pipe section (Litres)
pub const PIPE_VOLUME: f64 = 10.0;

/// Volume of a liquid pipe section (Litres)
pub const LIQUID_PIPE_VOLUME: f64 = 20.0;

/// Maximum pressure that gas pipes are rated for (kPa)
pub const MAX_PRESSURE_GAS_PIPE: f64 = 60_794.998_168_945_31;

/// Maximum pressure that liquid pipes are rated for (kPa)
pub const MAX_PRESSURE_LIQUID_PIPE: f64 = 6_079.499_816_894_531;

/// Minimum gas volume for calculations (Litres)
pub const MINIMUM_GAS_VOLUME: f64 = 0.1;

/// Armstrong limit - minimum pressure for liquids to exist without flash boiling (kPa)
pub const ARMSTRONG_LIMIT: f64 = 6.3;

/// Low state change quantity bound for rate limiting
pub const LOW_STATE_CHANGE_QUANTITY_BOUND: f64 = 0.1;

/// Default ratio applied to state changes per tick for normal conditions
pub const DEFAULT_STATE_CHANGE_RATIO: f64 = 0.1;

/// Force ratio applied for very small quantities/world-level changes
pub const FULL_STATE_CHANGE_RATIO: f64 = 1.0;

/// Rate applied when changing small quantities (halved)
pub const SMALL_STATE_CHANGE_RATE: f64 = 0.5;

/// Temperature delta used when interpolating evaporation temperature
pub const EVAP_INTERPOLATION_TEMP_DELTA: f64 = 10.0;

/// Temperature margin used near freezing for special handling (Kelvin)
pub const NEAR_FREEZING_MARGIN: f64 = 1.0;

/// Factor for halving freezing temperature
pub const HALF_FREEZING_FACTOR: f64 = 0.5;

/// Pressure epsilon for pressure equalization (kPa)
pub const PRESSURE_EQUALIZATION_EPSILON: f64 = 0.001;

/// Denominator for fusion to vaporization latent heat ratio
/// Latent heat of fusion = Latent heat of vaporization / 5.0
pub const FUSION_TO_VAPORIZATION_DENOMINATOR: f64 = 5.0;

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

/// Calculate the energy required to change temperature
/// E = n * Cv * deltaT
pub fn calculate_energy_for_temperature_change(
    moles: f64,
    specific_heat: f64,
    temperature_delta: f64,
) -> f64 {
    moles * specific_heat * temperature_delta
}

/// Calculate the moles that can change state given available energy
/// n = E / latent_heat
pub fn calculate_moles_for_state_change(energy: f64, latent_heat: f64) -> f64 {
    if latent_heat <= 0.0 {
        return 0.0;
    }
    (energy / latent_heat).max(0.0)
}

/// Calculate energy required for state change
/// E = n * latent_heat
pub fn calculate_energy_for_state_change(moles: f64, latent_heat: f64) -> f64 {
    moles * latent_heat
}

/// Map a value from one range to another
pub fn map_to_scale(min: f64, max: f64, out_min: f64, out_max: f64, value: f64) -> f64 {
    let range_in = max - min;
    let range_out = out_max - out_min;
    if range_in == 0.0 {
        return out_min;
    }
    (value - min) * range_out / range_in + out_min
}
