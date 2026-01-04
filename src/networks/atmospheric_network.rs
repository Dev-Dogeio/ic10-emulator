//! Atmospheric Network - manages connected atmospheric devices
//!
//! An atmospheric network connects devices that handle atmospheric gases.
//!
//! The network maintains a shared gas mixture that all connected devices
//! can access. Devices can add or remove gas from the network, and the
//! network automatically equalizes pressure and temperature across all
//! connected devices.

use crate::atmospherics::{GasMixture, GasType, Mole};
use std::collections::HashSet;

/// Identifier for devices on the atmospheric network
pub type DeviceId = usize;

/// An atmospheric network that manages a shared gas mixture.
///
/// The network maintains a gas mixture that devices can directly interact with.
/// Devices added to a network are tracked for enumeration purposes.
#[derive(Debug, Clone)]
pub struct AtmosphericNetwork {
    /// The shared gas mixture for this network
    mixture: GasMixture,

    /// Set of device IDs connected to this network (for enumeration only)
    devices: HashSet<DeviceId>,
}

impl AtmosphericNetwork {
    /// Create a new atmospheric network with specified volume
    /// Panics if volume is 0 or negative
    pub fn new(volume: f64) -> Self {
        assert!(
            volume > 0.0,
            "Atmospheric networks must have positive volume"
        );
        Self {
            mixture: GasMixture::new(volume),
            devices: HashSet::new(),
        }
    }

    /// Get immutable reference to the network's gas mixture
    pub fn mixture(&self) -> &GasMixture {
        &self.mixture
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

    /// Get the number of devices connected to this network
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Check if a device is connected to this network
    pub fn has_device(&self, device_id: DeviceId) -> bool {
        self.devices.contains(&device_id)
    }

    /// Get all device IDs connected to this network
    pub fn device_ids(&self) -> Vec<DeviceId> {
        self.devices.iter().copied().collect()
    }

    /// Add a device to the network
    /// The device will be stored in the device set
    /// Returns true if the device was added, false if it was already present
    #[allow(dead_code)]
    pub(crate) fn add_device(&mut self, device_id: DeviceId) -> bool {
        self.devices.insert(device_id)
    }

    /// Remove a device from the network
    /// Returns true if the device was present and removed
    #[allow(dead_code)]
    pub(crate) fn remove_device(&mut self, device_id: DeviceId) -> bool {
        self.devices.remove(&device_id)
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
    pub fn remove_moles(&mut self, moles: f64) -> GasMixture {
        self.mixture.remove_moles(moles)
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

    /// Merge another atmospheric network into this one
    /// The other network will be emptied and its devices will be transferred
    /// Returns the list of devices that were transferred
    pub fn merge_network(&mut self, other: &mut AtmosphericNetwork) -> Vec<DeviceId> {
        // Collect devices to transfer
        let transferred_devices: Vec<DeviceId> = other.devices.iter().copied().collect();

        // Transfer all devices
        for &device_id in &transferred_devices {
            self.devices.insert(device_id);
        }

        // Merge gas mixtures
        self.mixture.merge(&other.mixture);

        // Clear the other network
        other.devices.clear();
        other.mixture.clear();

        transferred_devices
    }

    /// Equalize with another atmospheric network
    /// This transfers gas between networks until pressures are equal
    pub fn equalize_with(&mut self, other: &mut AtmosphericNetwork) {
        self.mixture.equalize_with(&mut other.mixture);
    }

    /// Transfer a specific amount of gas to another network
    pub fn transfer_to(&mut self, other: &mut AtmosphericNetwork, moles: f64) {
        let transferred = self.mixture.remove_moles(moles);
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
