//! Error types for the IC10 emulator

use thiserror::Error;

/// Simulation error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SimulationError {
    #[error("Parse error at line {line}: {message}")]
    IC10ParseError { line: usize, message: String },

    #[error("Runtime error at line {line}: {message}")]
    RuntimeError { line: usize, message: String },

    #[error("Register index {0} out of bounds (valid range: 0-17)")]
    RegisterOutOfBounds(usize),

    #[error("Stack index {0} out of bounds (valid range: 0-511)")]
    StackOutOfBounds(usize),

    #[error("Unrecognized instruction: {0}")]
    UnrecognizedInstruction(String),

    #[error("Incorrect argument count for {instruction}: expected {expected}, got {actual}")]
    IncorrectArgumentCount {
        instruction: String,
        expected: usize,
        actual: usize,
    },
}

/// Result type for simulation operations
pub type SimulationResult<T> = Result<T, SimulationError>;
