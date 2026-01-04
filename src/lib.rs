pub mod animation_curve;
pub mod atmospherics;
pub mod chip;
pub mod constants;
pub mod conversions;
pub mod devices;
pub mod error;
pub mod instruction;
pub mod logic;
pub mod networks;
pub mod parser;
pub mod types;

#[cfg(test)]
pub mod tests;

pub use chip::ProgrammableChip;
pub use constants::get_builtin_constants;
pub use devices::{DaylightSensor, Device, ICHousing, LogicType};
pub use error::{SimulationError, SimulationResult};
pub use instruction::Instruction;
pub use networks::{BatchMode, CableNetwork};
