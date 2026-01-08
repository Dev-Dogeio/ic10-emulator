//! Property descriptor system for extensible device logic types

use crate::{
    devices::LogicType,
    error::{SimulationError, SimulationResult},
};
use std::collections::HashMap;

// Helper macros for creating `PropertyDescriptor`s.
#[macro_export]
macro_rules! prop_ro {
    ($logic:expr, $closure:expr) => {
        PropertyDescriptor::read_only($logic, $closure)
    };
}

#[macro_export]
macro_rules! prop_rw_bool {
    ($logic:expr, $field:ident) => {
        PropertyDescriptor::read_write(
            $logic,
            |device, _| Ok(*device.$field.borrow()),
            |device, _, value| {
                *device.$field.borrow_mut() = if value < 1.0 { 0.0 } else { 1.0 };
                Ok(())
            },
        )
    };
}

#[macro_export]
macro_rules! prop_rw_clamped {
    ($logic:expr, $field:ident, $min:expr, $max:expr) => {
        PropertyDescriptor::read_write(
            $logic,
            |device, _| Ok(*device.$field.borrow()),
            |device, _, value| {
                *device.$field.borrow_mut() = value.clamp($min, $max);
                Ok(())
            },
        )
    };
}

/// Function type for reading a property value from a device
pub type PropertyReadFn<T> = fn(&T, LogicType) -> SimulationResult<f64>;

/// Function type for writing a property value to a device
pub type PropertyWriteFn<T> = fn(&T, LogicType, f64) -> SimulationResult<()>;

/// Metadata descriptor for a device property
#[derive(Clone, Copy)]
pub struct PropertyDescriptor<T> {
    /// The LogicType enum value for this property
    pub logic_type: LogicType,

    /// Whether this property can be read
    pub readable: bool,
    /// Whether this property can be written
    pub writable: bool,

    /// Function to read the property value
    pub read_fn: Option<PropertyReadFn<T>>,
    /// Function to write the property value
    pub write_fn: Option<PropertyWriteFn<T>>,
}

impl<T> PropertyDescriptor<T> {
    /// Create a read-only property descriptor
    pub const fn read_only(logic_type: LogicType, read_fn: PropertyReadFn<T>) -> Self {
        Self {
            logic_type,
            readable: true,
            writable: false,
            read_fn: Some(read_fn),
            write_fn: None,
        }
    }

    /// Create a read-write property descriptor
    pub const fn read_write(
        logic_type: LogicType,
        read_fn: PropertyReadFn<T>,
        write_fn: PropertyWriteFn<T>,
    ) -> Self {
        Self {
            logic_type,
            readable: true,
            writable: true,
            read_fn: Some(read_fn),
            write_fn: Some(write_fn),
        }
    }

    /// Create a write-only property descriptor
    pub const fn write_only(logic_type: LogicType, write_fn: PropertyWriteFn<T>) -> Self {
        Self {
            logic_type,
            readable: false,
            writable: true,
            read_fn: None,
            write_fn: Some(write_fn),
        }
    }
}

/// Property registry for a device type
pub struct PropertyRegistry<T: 'static> {
    properties: &'static [PropertyDescriptor<T>],
    lookup: HashMap<LogicType, usize>,
}

impl<T: 'static> PropertyRegistry<T> {
    /// Create a new property registry from a static slice of descriptors
    pub fn new(properties: &'static [PropertyDescriptor<T>]) -> Self {
        let lookup: HashMap<LogicType, usize> = properties
            .iter()
            .enumerate()
            .map(|(idx, prop)| (prop.logic_type, idx))
            .collect();

        // Properties must have unique LogicType values
        if lookup.len() != properties.len() {
            panic!("Duplicate LogicType values in PropertyRegistry");
        }

        Self { properties, lookup }
    }

    /// Check if a property can be read
    pub fn can_read(&self, logic_type: LogicType) -> bool {
        self.lookup
            .get(&logic_type)
            .and_then(|&idx| self.properties.get(idx))
            .map(|prop| prop.readable)
            .unwrap_or(false)
    }

    /// Check if a property can be written
    pub fn can_write(&self, logic_type: LogicType) -> bool {
        self.lookup
            .get(&logic_type)
            .and_then(|&idx| self.properties.get(idx))
            .map(|prop| prop.writable)
            .unwrap_or(false)
    }

    /// Read a property value
    pub fn read(&self, device: &T, logic_type: LogicType) -> SimulationResult<f64> {
        match self.lookup.get(&logic_type) {
            Some(&idx) => {
                let prop = &self.properties[idx];
                if let Some(read_fn) = prop.read_fn {
                    read_fn(device, logic_type)
                } else {
                    Err(SimulationError::RuntimeError {
                        message: format!("Property {:?} is not readable", logic_type),
                        line: 0,
                    })
                }
            }
            None => Err(SimulationError::RuntimeError {
                message: format!("Unknown logic type {:?}", logic_type),
                line: 0,
            }),
        }
    }

    /// Write a property value
    pub fn write(&self, device: &T, logic_type: LogicType, value: f64) -> SimulationResult<()> {
        match self.lookup.get(&logic_type) {
            Some(&idx) => {
                let prop = &self.properties[idx];
                if let Some(write_fn) = prop.write_fn {
                    write_fn(device, logic_type, value)
                } else {
                    Err(SimulationError::RuntimeError {
                        message: format!("Property {:?} is not writable", logic_type),
                        line: 0,
                    })
                }
            }
            None => Err(SimulationError::RuntimeError {
                message: format!("Unknown logic type {:?}", logic_type),
                line: 0,
            }),
        }
    }

    /// Get all supported logic types
    pub fn supported_types(&self) -> Vec<LogicType> {
        self.properties.iter().map(|p| p.logic_type).collect()
    }
}
