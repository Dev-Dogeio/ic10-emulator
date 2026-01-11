//! Atmospheric Network - manages a shared gas mixture that can be accessed by multiple devices.

use crate::SimulationError;
use crate::atmospherics::{GasMixture, GasType, MatterState, Mole};
use crate::types::{Shared, shared};
use std::fmt::{Debug, Display};

/// An atmospheric network that manages a shared gas mixture.
#[derive(Clone)]
pub struct AtmosphericNetwork {
    /// Optional assigned id for this network
    id: Option<i32>,

    /// The shared gas mixture for this network
    mixture: GasMixture,

    /// Constant mixture to copy after each update
    constant_mixture: Option<GasMixture>,
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
            id: None,
            mixture: GasMixture::new(volume),
            constant_mixture: None,
        })
    }

    /// Perform tasks after a mixture update
    fn after_update(&mut self) {
        if let Some(constant) = &self.constant_mixture {
            self.mixture = constant.clone();
        }
    }

    /// Set the assigned id for this network
    pub fn set_id(&mut self, id: Option<i32>) {
        self.id = id;
    }

    /// Get the assigned id for this network
    pub fn get_id(&self) -> Option<i32> {
        self.id
    }

    /// Get the total volume of the network
    pub fn total_volume(&self) -> f64 {
        self.mixture.volume()
    }

    /// Set the network volume
    /// If the network is constant, the constant mixture volume is also updated
    pub fn set_volume(&mut self, volume: f64) -> Result<(), SimulationError> {
        if volume <= 0.0 {
            return Err(SimulationError::RuntimeError {
                message: "Atmospheric network volume must be positive".to_string(),
                line: 0,
            });
        }

        self.mixture.set_volume(volume);
        if let Some(constant) = &mut self.constant_mixture {
            constant.set_volume(volume);
        }

        self.after_update();
        Ok(())
    }

    /// Add gas to the network
    pub fn add_gas(&mut self, gas_type: GasType, moles: f64, temperature: f64) {
        self.mixture.add_gas(gas_type, moles, temperature);
        self.after_update();
    }

    /// Add a gas mixture to the network
    pub fn add_mixture(&mut self, other: &GasMixture) {
        self.mixture.merge(other);
        self.after_update();
    }

    /// Add a Mole to the network
    pub fn add_mole(&mut self, mole: &Mole) {
        self.mixture.add_mole(mole);
        self.after_update();
    }

    /// Remove gas from the network
    pub fn remove_gas(&mut self, gas_type: GasType, moles: f64) -> f64 {
        let removed = self.mixture.remove_gas(gas_type, moles);
        self.after_update();
        removed.quantity()
    }

    /// Remove a specific amount of moles proportionally
    pub fn remove_moles(&mut self, moles: f64, matter_state: MatterState) -> GasMixture {
        let mixture = self.mixture.remove_moles(moles, matter_state);
        self.after_update();
        mixture
    }

    /// Remove all moles of a specific gas type and return the removed Mole
    pub fn remove_all_gas(&mut self, gas_type: GasType) -> Mole {
        let mole = self.mixture.remove_all_gas(gas_type);
        self.after_update();
        mole
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
        let changes = self.mixture.process_phase_changes();
        self.after_update();
        changes
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

    /// Get the volume of the network
    pub fn volume(&self) -> f64 {
        self.mixture.volume()
    }

    /// Get the total moles of gases in the network
    pub fn total_moles_gases(&self) -> f64 {
        self.mixture.total_moles_gases()
    }

    /// Get the total moles of liquids in the network
    pub fn total_moles_liquids(&self) -> f64 {
        self.mixture.total_moles_liquids()
    }

    /// Get the total moles in a specific matter state
    pub fn total_moles_by_state(&self, state_value: MatterState) -> f64 {
        self.mixture.total_moles_by_state(state_value)
    }

    /// Get the total volume of gases in the network
    pub fn total_volume_liquids(&self) -> f64 {
        self.mixture.total_volume_liquids()
    }

    /// Get the ratio of liquid volume to total volume (0.0 to 1.0)
    pub fn liquid_volume_ratio(&self) -> f64 {
        self.mixture.liquid_volume_ratio()
    }

    /// Get the gas volume of the network
    pub fn gas_volume(&self) -> f64 {
        self.mixture.gas_volume()
    }

    /// Get the total energy of gases in the network
    pub fn total_energy_gases(&self) -> f64 {
        self.mixture.total_energy_gases()
    }

    /// Get the total energy of liquids in the network
    pub fn total_energy_liquids(&self) -> f64 {
        self.mixture.total_energy_liquids()
    }

    /// Get the total energy of the network
    pub fn total_energy(&self) -> f64 {
        self.mixture.total_energy()
    }

    /// Get the total heat capacity of gases in the network
    pub fn total_heat_capacity_gases(&self) -> f64 {
        self.mixture.total_heat_capacity_gases()
    }

    /// Get the total heat capacity of liquids in the network
    pub fn total_heat_capacity_liquids(&self) -> f64 {
        self.mixture.total_heat_capacity_liquids()
    }

    /// Get the total heat capacity of the network
    pub fn total_heat_capacity(&self) -> f64 {
        self.mixture.total_heat_capacity()
    }

    /// Get the pressure contributed by gases only (kPa)
    /// Same as `pressure()`
    pub fn pressure_gases(&self) -> f64 {
        self.mixture.pressure_gases()
    }

    /// Check if the network is empty
    pub fn is_empty(&self) -> bool {
        self.mixture.is_empty()
    }

    /// Consume all gas from the network and return it
    pub fn consume(&mut self) -> GasMixture {
        let consumed = self.mixture.clone();
        self.mixture.clear();
        self.after_update();
        consumed
    }

    /// Clear all gas from the network
    pub fn clear(&mut self) {
        self.mixture.clear();
        self.after_update();
    }

    /// Equalize with another atmospheric network
    /// This transfers gas between networks until pressures are equal
    pub fn equalize_with(&mut self, other: &mut AtmosphericNetwork) {
        self.mixture.equalize_with(&mut other.mixture);
        self.after_update();
    }

    /// Equalize internal energy within the network
    pub fn equalize_internal_energy(&mut self) {
        self.mixture.equalize_internal_energy();
        self.after_update();
    }

    /// Transfer a specific amount of gas to another network
    pub fn transfer_to(&mut self, other: &mut AtmosphericNetwork, moles: f64) {
        let transferred = self.mixture.remove_moles(moles, MatterState::All);
        other.mixture.merge(&transferred);
        self.after_update();
    }

    /// Set the temperature of the network
    pub fn set_temperature(&mut self, temperature: f64) {
        self.mixture.set_temperature(temperature);
        self.after_update();
    }

    /// Add thermal energy to the network (in Joules)
    pub fn add_energy(&mut self, joules: f64) {
        self.mixture.add_energy(joules);
        self.after_update();
    }

    /// Remove thermal energy from the network (in Joules)
    /// Returns the actual amount removed
    pub fn remove_energy(&mut self, joules: f64) -> f64 {
        let energy = self.mixture.remove_energy(joules);
        self.after_update();
        energy
    }

    /// Scale the entire gas mixture by a factor
    pub fn scale(&mut self, factor: f64, matter_state: MatterState) {
        self.mixture.scale(factor, matter_state);
        self.after_update();
    }

    /// Check if the network is in constant mixture mode
    pub fn is_constant(&self) -> bool {
        self.constant_mixture.is_some()
    }

    /// Toggle constant mixture mode
    pub fn toggle_constant(&mut self) {
        if self.is_constant() {
            self.constant_mixture = None;
        } else {
            self.constant_mixture = Some(self.mixture.clone());
        }
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
