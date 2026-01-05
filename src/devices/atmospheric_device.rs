//! Atmospheric device trait for devices that interact with gas networks
//!
//! Devices that handle gases (tanks, filters, ACs, vents, etc.) should
//! implement this trait to manage their connections to atmospheric networks.

use std::fmt::Debug;

use crate::SimulationError;
use crate::devices::FilterConnectionType;
use crate::networks::AtmosphericNetwork;
use crate::types::OptShared;

pub trait AtmosphericDevice: Debug {
    /// Set the atmospheric network for a specific connection on this device
    /// Called by the network when the device is added or removed
    fn set_atmospheric_network(
        &mut self,
        connection: FilterConnectionType,
        network: OptShared<AtmosphericNetwork>,
    ) -> Result<(), SimulationError>;
}
