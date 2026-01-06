//! Gas types supported by the atmospheric simulation
//!
//! Based on Stationeers gas types, including both gaseous and liquid states.

use std::fmt;

use crate::atmospherics::FUSION_TO_VAPORIZATION_DENOMINATOR;

/// Represents the state of matter for a substance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatterState {
    /// No specific state (undefined)
    None,
    /// Gaseous state
    Gas,
    /// Liquid state
    Liquid,
    /// All states (for filtering/querying)
    All,
}

/// Represents the different types of gases and liquids in the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum GasType {
    // Gases
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
    /// Hydrogen (H2)
    Hydrogen = 16384,

    // Liquids
    /// Water (liquid H2O)
    Water = 32,
    /// Polluted Water
    PollutedWater = 65536,
    /// Liquid Nitrogen
    LiquidNitrogen = 128,
    /// Liquid Oxygen
    LiquidOxygen = 256,
    /// Liquid Volatiles
    LiquidVolatiles = 512,
    /// Liquid Carbon Dioxide
    LiquidCarbonDioxide = 2048,
    /// Liquid Pollutant
    LiquidPollutant = 4096,
    /// Liquid Nitrous Oxide
    LiquidNitrousOxide = 8192,
    /// Liquid Hydrogen
    LiquidHydrogen = 32768,
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
            GasType::Steam => "STM",
            GasType::Hydrogen => "H2",
            GasType::Water => "H2O",
            GasType::PollutedWater => "XH2O",
            GasType::LiquidNitrogen => "LN2",
            GasType::LiquidOxygen => "LOX",
            GasType::LiquidVolatiles => "LVOL",
            GasType::LiquidCarbonDioxide => "LCO2",
            GasType::LiquidPollutant => "LX",
            GasType::LiquidNitrousOxide => "LNOS",
            GasType::LiquidHydrogen => "LH2",
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
            GasType::Hydrogen => "Hydrogen",
            GasType::Water => "Water",
            GasType::PollutedWater => "Polluted Water",
            GasType::LiquidNitrogen => "Liquid Nitrogen",
            GasType::LiquidOxygen => "Liquid Oxygen",
            GasType::LiquidVolatiles => "Liquid Volatiles",
            GasType::LiquidCarbonDioxide => "Liquid Carbon Dioxide",
            GasType::LiquidPollutant => "Liquid Pollutant",
            GasType::LiquidNitrousOxide => "Liquid Nitrous Oxide",
            GasType::LiquidHydrogen => "Liquid Hydrogen",
        }
    }

    /// Get the prefab name used for ItemGasFilter prefabs
    pub fn filter_name(&self) -> &'static str {
        match self {
            GasType::Oxygen | GasType::LiquidOxygen => "Oxygen",
            GasType::Nitrogen | GasType::LiquidNitrogen => "Nitrogen",
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => "CarbonDioxide",
            GasType::Volatiles | GasType::LiquidVolatiles => "Volatiles",
            GasType::Pollutant | GasType::LiquidPollutant => "Pollutants",
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => "NitrousOxide",
            GasType::Steam | GasType::Water | GasType::PollutedWater => "Water",
            _ => panic!("No filter exists for gas type {:?}", self),
        }
    }

    /// Get the specific heat capacity (J/(mol·K)) for this gas type
    /// Used for energy calculations: E = n * Cv * T
    pub fn specific_heat(&self) -> f64 {
        match self {
            GasType::Oxygen | GasType::LiquidOxygen => 21.1,
            GasType::Nitrogen | GasType::LiquidNitrogen => 20.6,
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => 28.2,
            GasType::Volatiles | GasType::LiquidVolatiles => 20.4,
            GasType::Pollutant | GasType::LiquidPollutant => 24.8,
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => 37.2,
            GasType::Steam | GasType::Water | GasType::PollutedWater => 72.0,
            GasType::Hydrogen | GasType::LiquidHydrogen => 20.4,
        }
    }

    /// Get the matter state for this gas type
    pub fn matter_state(&self) -> MatterState {
        match self {
            GasType::Oxygen
            | GasType::Nitrogen
            | GasType::CarbonDioxide
            | GasType::Volatiles
            | GasType::Pollutant
            | GasType::NitrousOxide
            | GasType::Steam
            | GasType::Hydrogen => MatterState::Gas,

            GasType::Water
            | GasType::PollutedWater
            | GasType::LiquidNitrogen
            | GasType::LiquidOxygen
            | GasType::LiquidVolatiles
            | GasType::LiquidCarbonDioxide
            | GasType::LiquidPollutant
            | GasType::LiquidNitrousOxide
            | GasType::LiquidHydrogen => MatterState::Liquid,
        }
    }

    /// Get the freezing/triple point temperature (Kelvin)
    pub fn freezing_temperature(&self) -> f64 {
        match self {
            GasType::Oxygen | GasType::LiquidOxygen => 56.416,
            GasType::Nitrogen | GasType::LiquidNitrogen => 40.01,
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => 217.82,
            GasType::Volatiles | GasType::LiquidVolatiles => 81.6,
            GasType::Pollutant | GasType::LiquidPollutant => 173.32,
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => 252.1,
            GasType::Steam | GasType::Water => 273.15,
            GasType::PollutedWater => 276.15,
            GasType::Hydrogen | GasType::LiquidHydrogen => 16.0,
        }
    }

    /// Get the minimum pressure required for liquid phase (kPa)
    /// This is the triple point pressure
    pub fn min_liquid_pressure(&self) -> f64 {
        const ARMSTRONG_LIMIT: f64 = 6.3;
        let base: f64 = match self {
            GasType::Oxygen | GasType::LiquidOxygen => 6.3,
            GasType::Nitrogen | GasType::LiquidNitrogen => 6.3,
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => 517.0,
            GasType::Volatiles | GasType::LiquidVolatiles => 6.3,
            GasType::Pollutant | GasType::LiquidPollutant => 1800.0,
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => 800.0,
            GasType::Steam | GasType::Water | GasType::PollutedWater => 6.3,
            GasType::Hydrogen | GasType::LiquidHydrogen => 6.3,
        };
        base.max(ARMSTRONG_LIMIT)
    }

    /// Get the maximum temperature at which liquid can exist (critical temperature, Kelvin)
    pub fn max_liquid_temperature(&self) -> f64 {
        match self {
            GasType::Oxygen | GasType::LiquidOxygen => 162.2,
            GasType::Nitrogen | GasType::LiquidNitrogen => 190.0,
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => 265.0,
            GasType::Volatiles | GasType::LiquidVolatiles => 195.0,
            GasType::Pollutant | GasType::LiquidPollutant => 425.0,
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => 430.6,
            GasType::Steam | GasType::Water => 643.0,
            GasType::PollutedWater => 629.0,
            GasType::Hydrogen | GasType::LiquidHydrogen => 70.0,
        }
    }

    /// Get the critical pressure (kPa) - minimum pressure at max liquid temperature
    pub fn critical_pressure(&self) -> f64 {
        match self {
            GasType::Oxygen | GasType::LiquidOxygen => 6000.0,
            GasType::Nitrogen | GasType::LiquidNitrogen => 6000.0,
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => 6000.0,
            GasType::Volatiles | GasType::LiquidVolatiles => 6000.0,
            GasType::Pollutant | GasType::LiquidPollutant => 6000.0,
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => 2000.0,
            GasType::Steam | GasType::Water | GasType::PollutedWater => 6000.0,
            GasType::Hydrogen | GasType::LiquidHydrogen => 6000.0,
        }
    }

    /// Get the latent heat of vaporization (J/mol)
    pub fn latent_heat_of_vaporization(&self) -> f64 {
        match self {
            GasType::Oxygen | GasType::LiquidOxygen => 800.0,
            GasType::Nitrogen | GasType::LiquidNitrogen => 500.0,
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => 600.0,
            GasType::Volatiles | GasType::LiquidVolatiles => 1000.0,
            GasType::Pollutant | GasType::LiquidPollutant => 2000.0,
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => 4000.0,
            GasType::Steam | GasType::Water | GasType::PollutedWater => 8000.0,
            GasType::Hydrogen | GasType::LiquidHydrogen => 350.0,
        }
    }

    /// Get the latent heat of fusion (melting) (J/mol)
    pub fn latent_heat_of_fusion(&self) -> f64 {
        self.latent_heat_of_vaporization() / FUSION_TO_VAPORIZATION_DENOMINATOR
    }

    /// Get the molar volume for liquids (L/mol)
    /// Returns 0.0 for gases
    pub fn molar_volume(&self) -> f64 {
        match self {
            GasType::Water | GasType::PollutedWater => 0.018,
            GasType::LiquidNitrogen => 0.0348,
            GasType::LiquidOxygen => 0.03,
            GasType::LiquidVolatiles => 0.04,
            GasType::LiquidCarbonDioxide => 0.04,
            GasType::LiquidPollutant => 0.04,
            GasType::LiquidNitrousOxide => 0.026,
            GasType::LiquidHydrogen => 0.03,
            _ => 0.0, // Gases have no fixed molar volume
        }
    }

    /// Get the molar mass (g/mol)
    pub fn molar_mass(&self) -> f64 {
        match self {
            GasType::Oxygen | GasType::LiquidOxygen => 16.0,
            GasType::Nitrogen | GasType::LiquidNitrogen => 64.0,
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => 44.0,
            GasType::Volatiles | GasType::LiquidVolatiles => 16.0,
            GasType::Pollutant | GasType::LiquidPollutant => 28.0,
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => 46.0,
            GasType::Steam | GasType::Water | GasType::PollutedWater => 18.0,
            GasType::Hydrogen | GasType::LiquidHydrogen => 2.0,
        }
    }

    /// Check if this gas type can evaporate (liquid -> gas)
    pub fn can_evaporate(&self) -> bool {
        matches!(
            self,
            GasType::Water
                | GasType::PollutedWater
                | GasType::LiquidNitrogen
                | GasType::LiquidOxygen
                | GasType::LiquidVolatiles
                | GasType::LiquidCarbonDioxide
                | GasType::LiquidPollutant
                | GasType::LiquidNitrousOxide
                | GasType::LiquidHydrogen
        )
    }

    /// Check if this gas type can condense (gas -> liquid)
    pub fn can_condense(&self) -> bool {
        matches!(
            self,
            GasType::Oxygen
                | GasType::Nitrogen
                | GasType::CarbonDioxide
                | GasType::Volatiles
                | GasType::Pollutant
                | GasType::NitrousOxide
                | GasType::Steam
                | GasType::Hydrogen
        )
    }

    /// Get the gas type that this liquid evaporates into
    /// Returns None if cannot evaporate
    pub fn evaporation_type(&self) -> Option<GasType> {
        match self {
            GasType::Water | GasType::PollutedWater => Some(GasType::Steam),
            GasType::LiquidNitrogen => Some(GasType::Nitrogen),
            GasType::LiquidOxygen => Some(GasType::Oxygen),
            GasType::LiquidVolatiles => Some(GasType::Volatiles),
            GasType::LiquidCarbonDioxide => Some(GasType::CarbonDioxide),
            GasType::LiquidPollutant => Some(GasType::Pollutant),
            GasType::LiquidNitrousOxide => Some(GasType::NitrousOxide),
            GasType::LiquidHydrogen => Some(GasType::Hydrogen),
            _ => None,
        }
    }

    /// Get the liquid type that this gas condenses into
    /// Returns None if cannot condense
    pub fn condensation_type(&self) -> Option<GasType> {
        match self {
            GasType::Steam => Some(GasType::Water),
            GasType::Nitrogen => Some(GasType::LiquidNitrogen),
            GasType::Oxygen => Some(GasType::LiquidOxygen),
            GasType::Volatiles => Some(GasType::LiquidVolatiles),
            GasType::CarbonDioxide => Some(GasType::LiquidCarbonDioxide),
            GasType::Pollutant => Some(GasType::LiquidPollutant),
            GasType::NitrousOxide => Some(GasType::LiquidNitrousOxide),
            GasType::Hydrogen => Some(GasType::LiquidHydrogen),
            _ => None,
        }
    }

    /// Get all gas types as an iterator (gases only)
    pub fn all_gases() -> impl Iterator<Item = GasType> {
        [
            GasType::Oxygen,
            GasType::Nitrogen,
            GasType::CarbonDioxide,
            GasType::Volatiles,
            GasType::Pollutant,
            GasType::NitrousOxide,
            GasType::Steam,
            GasType::Hydrogen,
        ]
        .into_iter()
    }

    /// Get all liquid types as an iterator
    pub fn all_liquids() -> impl Iterator<Item = GasType> {
        [
            GasType::Water,
            GasType::PollutedWater,
            GasType::LiquidNitrogen,
            GasType::LiquidOxygen,
            GasType::LiquidVolatiles,
            GasType::LiquidCarbonDioxide,
            GasType::LiquidPollutant,
            GasType::LiquidNitrousOxide,
            GasType::LiquidHydrogen,
        ]
        .into_iter()
    }

    /// Get all gas types as an iterator
    pub fn all() -> impl Iterator<Item = GasType> {
        Self::all_gases().chain(Self::all_liquids())
    }

    /// Get the number of gas types (gases only)
    pub const fn gas_count() -> usize {
        8
    }

    /// Get the number of liquid types
    pub const fn liquid_count() -> usize {
        9
    }

    /// Get the total number of gas and liquid types
    pub const fn count() -> usize {
        Self::gas_count() + Self::liquid_count()
    }

    /// Check if this is a gas
    pub fn is_gas(&self) -> bool {
        self.matter_state() == MatterState::Gas
    }

    /// Check if this is a liquid
    pub fn is_liquid(&self) -> bool {
        self.matter_state() == MatterState::Liquid
    }

    /// Check whether this gas type matches a `MatterState` filter
    pub fn matches_state(&self, state: MatterState) -> bool {
        match state {
            MatterState::All => true,
            MatterState::Gas => self.is_gas(),
            MatterState::Liquid => self.is_liquid(),
            _ => false,
        }
    }

    /// Get evaporation coefficient A for the power law formula
    /// P = A * T^B where P is pressure in kPa and T is temperature in K
    pub fn evaporation_coefficient_a(&self) -> f64 {
        match self {
            GasType::Oxygen | GasType::LiquidOxygen => 2.6854996004e-11,
            GasType::Nitrogen | GasType::LiquidNitrogen => 5.5757107833e-07,
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => 1.579573e-26,
            GasType::Volatiles | GasType::LiquidVolatiles => 5.863496734e-15,
            GasType::Pollutant | GasType::LiquidPollutant => 2.079033884,
            GasType::Water | GasType::Steam => 3.8782059839e-19,
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => 0.065353501531,
            GasType::Hydrogen | GasType::LiquidHydrogen => 3.18041e-05,
            GasType::PollutedWater => 4e-20,
        }
    }

    /// Get evaporation coefficient B for the power law formula
    /// P = A * T^B where P is pressure in kPa and T is temperature in K
    pub fn evaporation_coefficient_b(&self) -> f64 {
        match self {
            GasType::Oxygen | GasType::LiquidOxygen => 6.49214937325,
            GasType::Nitrogen | GasType::LiquidNitrogen => 4.40221368946,
            GasType::CarbonDioxide | GasType::LiquidCarbonDioxide => 12.195837931,
            GasType::Volatiles | GasType::LiquidVolatiles => 7.8643601035,
            GasType::Pollutant | GasType::LiquidPollutant => 1.31202194555,
            GasType::Water | GasType::Steam => 7.90030107708,
            GasType::NitrousOxide | GasType::LiquidNitrousOxide => 1.70297431874,
            GasType::Hydrogen | GasType::LiquidHydrogen => 4.4843872973,
            GasType::PollutedWater => 8.27025711260823,
        }
    }
}

impl fmt::Display for GasType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}
