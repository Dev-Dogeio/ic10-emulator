//! Atmospheric device trait for devices that interact with gas networks
//!
//! Devices that handle gases (tanks, filters, ACs, vents, etc.) should
//! implement this trait to manage their connections to atmospheric networks.

use std::fmt::Debug;

use crate::SimulationError;
use crate::devices::DeviceAtmosphericNetworkType;
use crate::networks::AtmosphericNetwork;
use crate::types::OptShared;

pub trait AtmosphericDevice: Debug {
    /// Set the atmospheric network for a specific connection on this device
    fn set_atmospheric_network(
        &mut self,
        connection: DeviceAtmosphericNetworkType,
        network: OptShared<AtmosphericNetwork>,
    ) -> Result<(), SimulationError>;

    /// Get the atmospheric network for a specific connection on this device
    fn get_atmospheric_network(
        &self,
        connection: DeviceAtmosphericNetworkType,
    ) -> OptShared<AtmosphericNetwork>;
}
