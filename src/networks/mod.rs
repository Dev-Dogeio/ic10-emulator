//! Network implementations for devices and atmospherics

mod atmospheric_network;
mod cable_network;

pub use atmospheric_network::AtmosphericNetwork;
pub use cable_network::{BatchMode, CableNetwork};
