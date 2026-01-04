//! Gas types supported by the atmospheric simulation
//!
//! Based on Stationeers gas types, focusing only on gaseous states.

use std::fmt;

/// Represents the different types of gases in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GasType {
    /// Oxygen (O2)
    Oxygen = 1,
    /// Nitrogen (N2)
    Nitrogen = 2,
    /// Carbon Dioxide (CO2)
    CarbonDioxide = 4,
    /// Volatiles (VOL..?)
    Volatiles = 8,
    /// Pollutant (X)
    Pollutant = 16,
    /// Nitrous Oxide (N2O/NOS)
    NitrousOxide = 64,
    /// Steam
    Steam = 1024,
}

impl GasType {
    /// Get the chemical symbol for this gas type
    pub fn symbol(&self) -> &'static str {
        match self {
            GasType::Oxygen => "O2",
            GasType::Nitrogen => "N2",
            GasType::CarbonDioxide => "CO2",
            GasType::Volatiles => "VOL",
            GasType::Pollutant => "X",
            GasType::NitrousOxide => "N2O",
            GasType::Steam => "H2O",
        }
    }

    /// Get the display name for this gas type
    pub fn display_name(&self) -> &'static str {
        match self {
            GasType::Oxygen => "Oxygen",
            GasType::Nitrogen => "Nitrogen",
            GasType::CarbonDioxide => "Carbon Dioxide",
            GasType::Volatiles => "Volatiles",
            GasType::Pollutant => "Pollutant",
            GasType::NitrousOxide => "Nitrous Oxide",
            GasType::Steam => "Steam",
        }
    }

    /// Get the specific heat capacity (J/(mol·K)) for this gas type
    /// Used for energy calculations: E = n * Cv * T
    pub fn specific_heat(&self) -> f64 {
        match self {
            GasType::Oxygen => 21.1,
            GasType::Nitrogen => 20.6,
            GasType::CarbonDioxide => 28.2,
            GasType::Volatiles => 20.4,
            GasType::Pollutant => 24.8,
            GasType::NitrousOxide => 37.2,
            GasType::Steam => 72.0,
        }
    }

    /// Get all gas types as an iterator
    pub fn all() -> impl Iterator<Item = GasType> {
        [
            GasType::Oxygen,
            GasType::Nitrogen,
            GasType::CarbonDioxide,
            GasType::Volatiles,
            GasType::Pollutant,
            GasType::NitrousOxide,
            GasType::Steam,
        ]
        .into_iter()
    }

    /// Get the number of gas types
    pub const fn count() -> usize {
        7
    }
}

impl fmt::Display for GasType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
