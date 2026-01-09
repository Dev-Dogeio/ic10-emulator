//! Atmospheric Network - manages a shared gas mixture that can be accessed by multiple devices.

use crate::atmospherics::{GasMixture, GasType, MatterState, Mole};
use crate::types::{Shared, shared};
use std::fmt::{Debug, Display};

/// An atmospheric network that manages a shared gas mixture.
#[derive(Clone)]
pub struct AtmosphericNetwork {
    /// The shared gas mixture for this network
    mixture: GasMixture,
}

impl AtmosphericNetwork {
    /// Create a new atmospheric network with specified volume.
    /// Panics if volume is 0 or negative
    pub fn new(volume: f64) -> Shared<AtmosphericNetwork> {
        assert!(
            volume > 0.0,
            "Atmospheric networks must have positive volume"
        );

        shared(AtmosphericNetwork {
            mixture: GasMixture::new(volume),
        })
    }

    /// Get immutable reference to the network's gas mixture
    pub fn mixture(&self) -> &GasMixture {
        &self.mixture
    }

    /// Get mutable reference to the network's gas mixture
    pub fn mixture_mut(&mut self) -> &mut GasMixture {
        &mut self.mixture
    }

    /// Get the total volume of the network
    pub fn total_volume(&self) -> f64 {
        self.mixture.volume()
    }

    /// Set the network volume
    /// Panics if volume is 0 or negative
    pub fn set_volume(&mut self, volume: f64) {
        assert!(
            volume > 0.0,
            "Atmospheric networks must have positive volume"
        );
        self.mixture.set_volume(volume);
    }

    /// Add gas to the network
    pub fn add_gas(&mut self, gas_type: GasType, moles: f64, temperature: f64) {
        self.mixture.add_gas(gas_type, moles, temperature);
    }

    /// Add a gas mixture to the network
    pub fn add_mixture(&mut self, other: &GasMixture) {
        self.mixture.merge(other);
    }

    /// Add a Mole to the network
    pub fn add_mole(&mut self, mole: &Mole) {
        self.mixture.add_mole(mole);
    }

    /// Remove gas from the network
    pub fn remove_gas(&mut self, gas_type: GasType, moles: f64) -> f64 {
        let removed = self.mixture.remove_gas(gas_type, moles);
        removed.quantity()
    }

    /// Remove a specific amount of moles proportionally
    pub fn remove_moles(&mut self, moles: f64, matter_state: MatterState) -> GasMixture {
        self.mixture.remove_moles(moles, matter_state)
    }

    /// Remove all moles of a specific gas type and return the removed Mole
    pub fn remove_all_gas(&mut self, gas_type: GasType) -> Mole {
        self.mixture.remove_all_gas(gas_type)
    }

    /// Get the Mole (quantity + energy) for a specific gas type (copy)
    pub fn get_gas(&self, gas_type: GasType) -> Mole {
        *self.mixture.get_gas(gas_type)
    }

    /// Get the quantity (moles) of a specific gas type
    pub fn get_moles(&self, gas_type: GasType) -> f64 {
        self.mixture.get_moles(gas_type)
    }

    /// Get the current pressure of the network (kPa)
    pub fn pressure(&self) -> f64 {
        self.mixture.pressure()
    }

    /// Get the current temperature of the network (K)
    pub fn temperature(&self) -> f64 {
        self.mixture.temperature()
    }

    /// Process post-update atmospheric tasks such as phase changes.
    /// Returns the number of phase changes that occurred in this network.
    pub fn process_phase_changes(&mut self) -> u32 {
        self.mixture.process_phase_changes()
    }

    /// Get the total moles in the network
    pub fn total_moles(&self) -> f64 {
        self.mixture.total_moles()
    }

    /// Get the ratio of a specific gas (0.0 to 1.0)
    pub fn gas_ratio(&self, gas_type: GasType) -> f64 {
        self.mixture.gas_ratio(gas_type)
    }

    /// Get the partial pressure of a specific gas (kPa)
    pub fn partial_pressure(&self, gas_type: GasType) -> f64 {
        self.mixture.partial_pressure(gas_type)
    }

    /// Check if the network is empty
    pub fn is_empty(&self) -> bool {
        self.mixture.is_empty()
    }

    /// Clear all gas from the network
    pub fn clear(&mut self) {
        self.mixture.clear();
    }

    /// Equalize with another atmospheric network
    /// This transfers gas between networks until pressures are equal
    pub fn equalize_with(&mut self, other: &mut AtmosphericNetwork) {
        self.mixture.equalize_with(&mut other.mixture);
    }

    /// Transfer a specific amount of gas to another network
    pub fn transfer_to(&mut self, other: &mut AtmosphericNetwork, moles: f64) {
        let transferred = self.mixture.remove_moles(moles, MatterState::All);
        other.mixture.merge(&transferred);
    }

    /// Set the temperature of the network
    pub fn set_temperature(&mut self, temperature: f64) {
        self.mixture.set_temperature(temperature);
    }

    /// Add thermal energy to the network (in Joules)
    pub fn add_energy(&mut self, joules: f64) {
        self.mixture.add_energy(joules);
    }

    /// Remove thermal energy from the network (in Joules)
    /// Returns the actual amount removed
    pub fn remove_energy(&mut self, joules: f64) -> f64 {
        self.mixture.remove_energy(joules)
    }
}

impl Display for AtmosphericNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mixture)
    }
}

impl Debug for AtmosphericNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
