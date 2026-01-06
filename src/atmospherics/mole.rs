//! Mole struct for gas quantities with energy tracking
//!
//! A Mole represents a quantity of a specific gas type along with its
//! thermal energy. This allows for accurate temperature and energy
//! calculations during gas transfers and mixing.

use super::{GasType, chemistry};
use std::fmt;

/// Represents a quantity of gas with its associated thermal energy
#[derive(Debug, Clone, Copy)]
pub struct Mole {
    /// The type of gas
    gas_type: GasType,
    /// Quantity of gas in moles
    quantity: f64,
    /// Thermal energy in Joules
    energy: f64,
}

impl Mole {
    /// Create a new Mole with specified quantity and temperature
    ///
    /// # Arguments
    /// * `gas_type` - The type of gas
    /// * `quantity` - Amount in moles
    /// * `temperature` - Temperature in Kelvin
    pub fn new(gas_type: GasType, quantity: f64, temperature: f64) -> Self {
        let quantity = quantity.max(0.0);
        let temperature = temperature.max(0.0);
        let energy = quantity * gas_type.specific_heat() * temperature;

        Self {
            gas_type,
            quantity,
            energy,
        }
    }

    /// Create a new Mole with zero quantity
    pub fn zero(gas_type: GasType) -> Self {
        Self {
            gas_type,
            quantity: 0.0,
            energy: 0.0,
        }
    }

    /// Create a Mole with specified quantity and energy directly
    pub fn with_energy(gas_type: GasType, quantity: f64, energy: f64) -> Self {
        Self {
            gas_type,
            quantity: quantity.max(0.0),
            energy: energy.max(0.0),
        }
    }

    /// Get the gas type
    pub fn gas_type(&self) -> GasType {
        self.gas_type
    }

    /// Get the quantity in moles
    pub fn quantity(&self) -> f64 {
        self.quantity
    }

    /// Get the thermal energy in Joules
    pub fn energy(&self) -> f64 {
        self.energy
    }

    /// Calculate the temperature in Kelvin
    /// T = E / (n * Cv)
    pub fn temperature(&self) -> f64 {
        if self.quantity <= chemistry::MINIMUM_QUANTITY_MOLES {
            return 0.0;
        }
        let temp = self.energy / (self.quantity * self.gas_type.specific_heat());
        temp.max(0.0)
    }

    /// Get the heat capacity of this gas amount (J/K)
    /// C = n * Cv
    pub fn heat_capacity(&self) -> f64 {
        self.quantity * self.gas_type.specific_heat()
    }

    /// Check if this mole is effectively empty
    pub fn is_empty(&self) -> bool {
        self.quantity < chemistry::MINIMUM_QUANTITY_MOLES
    }

    /// Set the quantity, adjusting energy to maintain temperature
    pub fn set_quantity(&mut self, new_quantity: f64) {
        let temp = self.temperature();
        self.quantity = new_quantity.max(0.0);
        self.energy = self.quantity * self.gas_type.specific_heat() * temp;
        self.cleanup();
    }

    /// Set the temperature, adjusting energy accordingly
    pub fn set_temperature(&mut self, temperature: f64) {
        let temp = temperature.max(0.0);
        self.energy = self.quantity * self.gas_type.specific_heat() * temp;
    }

    /// Add energy (heating)
    pub fn add_energy(&mut self, joules: f64) {
        self.energy = (self.energy + joules).max(0.0);
    }

    /// Remove energy (cooling)
    /// Returns the actual amount removed
    pub fn remove_energy(&mut self, joules: f64) -> f64 {
        let removed = joules.min(self.energy);
        self.energy -= removed;
        removed
    }

    /// Add moles from another Mole of the same gas type
    /// Combines quantities and energies
    ///
    /// # Panics
    /// Panics if gas types don't match
    pub fn add(&mut self, other: &Mole) {
        assert_eq!(
            self.gas_type, other.gas_type,
            "Cannot add moles of different gas types"
        );
        self.quantity += other.quantity;
        self.energy += other.energy;
        self.cleanup();
    }

    /// Remove a specified quantity, returning the removed Mole
    /// Energy is proportionally removed
    ///
    /// # Arguments
    /// * `amount` - Amount in moles to remove
    ///
    /// # Returns
    /// A new Mole containing the removed gas
    pub fn remove(&mut self, amount: f64) -> Mole {
        let amount = amount.min(self.quantity).max(0.0);
        if amount <= 0.0 || self.quantity <= 0.0 {
            return Mole::zero(self.gas_type);
        }

        let removed_energy = self.energy * amount / self.quantity;

        self.quantity -= amount;
        self.energy -= removed_energy;

        self.cleanup();
        Mole::with_energy(self.gas_type, amount, removed_energy)
    }

    /// Remove a ratio of the gas (0.0 to 1.0)
    /// Returns the removed Mole
    pub fn remove_ratio(&mut self, ratio: f64) -> Mole {
        self.remove(self.quantity * ratio.clamp(0.0, 1.0))
    }

    /// Transfer gas to another Mole of the same type
    /// Returns the amount actually transferred
    pub fn transfer_to(&mut self, target: &mut Mole, amount: f64) -> f64 {
        let removed = self.remove(amount);
        let transferred = removed.quantity;
        target.add(&removed);
        transferred
    }

    /// Calculate the energy required to reach a target temperature
    pub fn energy_to_reach_temperature(&self, target_temp: f64) -> f64 {
        let current_temp = self.temperature();
        let delta_temp = target_temp - current_temp;
        self.heat_capacity() * delta_temp
    }

    /// Clear this mole (set to zero)
    pub fn clear(&mut self) {
        self.quantity = 0.0;
        self.energy = 0.0;
    }

    /// Cleanup small residues: zero out quantity and energy if below threshold
    pub fn cleanup(&mut self) {
        if self.quantity < chemistry::MINIMUM_QUANTITY_MOLES {
            self.clear();
        }
    }
}

impl Default for Mole {
    fn default() -> Self {
        Self {
            gas_type: GasType::Nitrogen,
            quantity: 0.0,
            energy: 0.0,
        }
    }
}

impl fmt::Display for Mole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {:.4} mol @ {:.2} K",
            self.gas_type.symbol(),
            self.quantity,
            self.temperature()
        )
    }
}
