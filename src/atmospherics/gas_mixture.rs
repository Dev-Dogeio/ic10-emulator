//! Gas Mixture - container for multiple gas types
//!
//! A GasMixture holds quantities of multiple gas types and provides
//! methods for mixing, transferring, and calculating properties like
//! pressure, temperature, and partial pressures.

use super::{GasType, Mole, chemistry};
use std::fmt;

/// A mixture of gases with their associated energies
#[derive(Debug, Clone)]
pub struct GasMixture {
    /// Individual gas moles indexed by gas type
    gases: [Mole; GasType::count()],
    /// Volume of the container in Litres
    volume: f64,
}

impl GasMixture {
    /// Create a new empty gas mixture with specified volume
    pub fn new(volume: f64) -> Self {
        Self {
            gases: [
                Mole::zero(GasType::Oxygen),
                Mole::zero(GasType::Nitrogen),
                Mole::zero(GasType::CarbonDioxide),
                Mole::zero(GasType::Volatiles),
                Mole::zero(GasType::Pollutant),
                Mole::zero(GasType::NitrousOxide),
                Mole::zero(GasType::Water),
            ],
            volume: volume.max(chemistry::MINIMUM_GAS_VOLUME),
        }
    }

    /// Get the index for a gas type
    fn gas_index(gas_type: GasType) -> usize {
        match gas_type {
            GasType::Oxygen => 0,
            GasType::Nitrogen => 1,
            GasType::CarbonDioxide => 2,
            GasType::Volatiles => 3,
            GasType::Pollutant => 4,
            GasType::NitrousOxide => 5,
            GasType::Water => 6,
        }
    }

    /// Get the volume in Litres
    pub fn volume(&self) -> f64 {
        self.volume
    }

    /// Set the volume in Litres
    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume.max(chemistry::MINIMUM_GAS_VOLUME);
    }

    /// Get the mole for a specific gas type
    pub fn get_gas(&self, gas_type: GasType) -> &Mole {
        &self.gases[Self::gas_index(gas_type)]
    }

    /// Get the quantity of a specific gas in moles
    pub fn get_moles(&self, gas_type: GasType) -> f64 {
        self.gases[Self::gas_index(gas_type)].quantity()
    }

    /// Add gas to the mixture
    pub fn add_gas(&mut self, gas_type: GasType, moles: f64, temperature: f64) {
        let new_mole = Mole::new(gas_type, moles, temperature);
        self.gases[Self::gas_index(gas_type)].add(&new_mole);

        // Equalize temperature across the mixture
        let equilibrium_temp = self.temperature();
        self.set_temperature(equilibrium_temp);
    }

    /// Add a Mole to the mixture
    pub fn add_mole(&mut self, mole: &Mole) {
        self.gases[Self::gas_index(mole.gas_type())].add(mole);

        // Equalize temperature across the mixture
        let equilibrium_temp = self.temperature();
        self.set_temperature(equilibrium_temp);
    }

    /// Remove gas from the mixture, returning the removed Mole
    pub fn remove_gas(&mut self, gas_type: GasType, moles: f64) -> Mole {
        self.gases[Self::gas_index(gas_type)].remove(moles)
    }

    /// Remove all moles of a specific gas type and return them
    pub fn remove_all_gas(&mut self, gas_type: GasType) -> Mole {
        let idx = Self::gas_index(gas_type);
        let qty = self.gases[idx].quantity();
        if qty <= 0.0 {
            return Mole::zero(gas_type);
        }
        self.gases[idx].remove(qty)
    }

    /// Get total moles of all gases
    pub fn total_moles(&self) -> f64 {
        self.gases.iter().map(|m| m.quantity()).sum()
    }

    /// Get total thermal energy in Joules
    pub fn total_energy(&self) -> f64 {
        self.gases.iter().map(|m| m.energy()).sum()
    }

    /// Get total heat capacity (J/K)
    pub fn total_heat_capacity(&self) -> f64 {
        self.gases.iter().map(|m| m.heat_capacity()).sum()
    }

    /// Calculate the average temperature of the mixture
    pub fn temperature(&self) -> f64 {
        let total_capacity = self.total_heat_capacity();
        if total_capacity <= 0.0 {
            return 0.0;
        }
        let temp = self.total_energy() / total_capacity;
        temp.max(0.0)
    }

    /// Calculate the total pressure using ideal gas law (kPa)
    /// P = nRT/V
    pub fn pressure(&self) -> f64 {
        chemistry::calculate_pressure(self.total_moles(), self.temperature(), self.volume)
    }

    /// Calculate partial pressure for a specific gas (kPa)
    pub fn partial_pressure(&self, gas_type: GasType) -> f64 {
        let moles = self.get_moles(gas_type);
        chemistry::calculate_pressure(moles, self.temperature(), self.volume)
    }

    /// Get the ratio of a specific gas (0.0 to 1.0)
    pub fn gas_ratio(&self, gas_type: GasType) -> f64 {
        let total = self.total_moles();
        if total <= 0.0 {
            return 0.0;
        }
        self.get_moles(gas_type) / total
    }

    /// Add energy to the mixture (distributed by heat capacity)
    pub fn add_energy(&mut self, joules: f64) {
        let total_capacity = self.total_heat_capacity();
        if total_capacity <= 0.0 {
            return;
        }

        for gas in &mut self.gases {
            let ratio = gas.heat_capacity() / total_capacity;
            gas.add_energy(joules * ratio);
        }
    }

    /// Remove energy from the mixture
    /// Returns the actual amount removed
    pub fn remove_energy(&mut self, joules: f64) -> f64 {
        let total_energy = self.total_energy();
        let to_remove = joules.min(total_energy);

        if to_remove <= 0.0 {
            return 0.0;
        }

        let total_capacity = self.total_heat_capacity();
        let mut removed = 0.0;

        for gas in &mut self.gases {
            let ratio = gas.heat_capacity() / total_capacity;
            removed += gas.remove_energy(to_remove * ratio);
        }

        removed
    }

    /// Set the temperature of all gases
    pub fn set_temperature(&mut self, temperature: f64) {
        for gas in &mut self.gases {
            gas.set_temperature(temperature);
        }
    }

    /// Transfer a ratio of all gases to another mixture
    /// Returns the total moles transferred
    pub fn transfer_ratio_to(&mut self, target: &mut GasMixture, ratio: f64) -> f64 {
        let ratio = ratio.clamp(0.0, 1.0);
        let mut transferred = 0.0;

        for i in 0..GasType::count() {
            let removed = self.gases[i].remove_ratio(ratio);
            transferred += removed.quantity();
            target.gases[i].add(&removed);
        }

        transferred
    }

    /// Transfer gas by pressure difference
    /// Moves gas from higher to lower pressure until equilibrium
    pub fn equalize_with(&mut self, other: &mut GasMixture) {
        // First, equalize temperatures by transferring thermal energy
        let self_heat_capacity = self.total_heat_capacity();
        let other_heat_capacity = other.total_heat_capacity();
        let total_heat_capacity = self_heat_capacity + other_heat_capacity;

        if total_heat_capacity > 0.0 {
            let total_energy = self.total_energy() + other.total_energy();
            let equilibrium_temp = total_energy / total_heat_capacity;

            self.set_temperature(equilibrium_temp);
            other.set_temperature(equilibrium_temp);
        }

        // Then, equalize pressures
        let self_pressure = self.pressure();
        let other_pressure = other.pressure();

        if (self_pressure - other_pressure).abs() < 0.001 {
            return;
        }

        // Calculate the amount to transfer to reach equilibrium
        let total_moles = self.total_moles() + other.total_moles();
        let total_volume = self.volume + other.volume;

        // Target moles for each based on volume ratio
        let self_target = total_moles * (self.volume / total_volume);
        let other_target = total_moles * (other.volume / total_volume);

        let self_current = self.total_moles();

        if self_current > self_target {
            // Transfer from self to other
            let to_transfer = self_current - self_target;
            let ratio = to_transfer / self_current;
            self.transfer_ratio_to(other, ratio);
        } else {
            // Transfer from other to self
            let to_transfer = other.total_moles() - other_target;
            let ratio = to_transfer / other.total_moles();
            other.transfer_ratio_to(self, ratio);
        }
    }

    /// Remove a specific amount of moles, proportionally from all gases
    /// Returns a new GasMixture with the removed gas
    pub fn remove_moles(&mut self, moles: f64) -> GasMixture {
        let total = self.total_moles();
        if total <= 0.0 || moles <= 0.0 {
            return GasMixture::new(0.0);
        }

        let ratio = (moles / total).min(1.0);
        let mut removed = GasMixture::new(0.0);

        for i in 0..GasType::count() {
            let gas = self.gases[i].remove_ratio(ratio);
            removed.gases[i].add(&gas);
        }

        removed
    }

    /// Merge another gas mixture into this one
    pub fn merge(&mut self, other: &GasMixture) {
        for i in 0..GasType::count() {
            self.gases[i].add(&other.gases[i]);
        }
    }

    /// Clear all gases
    pub fn clear(&mut self) {
        for gas in &mut self.gases {
            gas.clear();
        }
    }

    /// Check if the mixture is empty
    pub fn is_empty(&self) -> bool {
        self.total_moles() < chemistry::MINIMUM_QUANTITY_MOLES
    }

    /// Get an iterator over all gas types and their moles
    pub fn iter(&self) -> impl Iterator<Item = (GasType, &Mole)> {
        GasType::all().zip(self.gases.iter())
    }

    /// Get a mutable iterator over all gas types and their moles
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (GasType, &mut Mole)> {
        GasType::all().zip(self.gases.iter_mut())
    }
}

impl fmt::Display for GasMixture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "GasMixture ({:.1} L):", self.volume)?;
        writeln!(
            f,
            "  Temperature: {:.2} K ({:.2} °C)",
            self.temperature(),
            chemistry::kelvin_to_celsius(self.temperature())
        )?;
        writeln!(f, "  Pressure: {:.2} kPa", self.pressure())?;
        writeln!(f, "  Total Moles: {:.4}", self.total_moles())?;
        writeln!(f, "  Gases:")?;

        for (gas_type, mole) in self.iter() {
            if !mole.is_empty() {
                writeln!(
                    f,
                    "    {}: {:.4} mol ({:.1}%)",
                    gas_type.symbol(),
                    mole.quantity(),
                    self.gas_ratio(gas_type) * 100.0
                )?;
            }
        }

        Ok(())
    }
}
