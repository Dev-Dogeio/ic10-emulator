//! Property descriptor system for extensible device logic types

use crate::{
    LogicSlotType, LogicType,
    error::{SimulationError, SimulationResult},
};
use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

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

// TODO: Currently slot properties are read-only only. If writable slot properties are needed in the future,
// the SlotPropertyDescriptor and SlotPropertyRegistry structs can be extended similarly to PropertyDescriptor

// Helper macros for slot property descriptors
#[macro_export]
macro_rules! prop_slot_ro {
    ($logic:expr, $slots:expr, $closure:expr) => {
        SlotPropertyDescriptor::read_only_for_slots($logic, $slots, $closure)
    };
}

/// Function type for reading a slot property value from a device
pub type SlotPropertyReadFn<T> = fn(&T, usize, LogicSlotType) -> SimulationResult<f64>;

/// Metadata descriptor for a slot property
///
/// `slots` lists the zero-based slot indices this descriptor applies to. The
/// slice **must be non-empty** and explicitly list the indices the descriptor
/// covers.
#[derive(Clone, Copy)]
pub struct SlotPropertyDescriptor<T> {
    /// The LogicSlotType enum value for this property
    pub logic_type: LogicSlotType,

    /// Whether this property can be read
    pub readable: bool,

    /// Function to read the property value for the specified slot index
    pub read_fn: Option<SlotPropertyReadFn<T>>,

    /// Which slot indices this descriptor applies to. Must be non-empty.
    pub slots: &'static [usize],
}

impl<T> SlotPropertyDescriptor<T> {
    /// Create a read-only slot property descriptor that applies only to the
    /// provided slot indices. `slots` must be non-empty.
    pub const fn read_only_for_slots(
        logic_type: LogicSlotType,
        slots: &'static [usize],
        read_fn: SlotPropertyReadFn<T>,
    ) -> Self {
        Self {
            logic_type,
            readable: true,
            read_fn: Some(read_fn),
            slots,
        }
    }
}

/// Slot property registry for a device type
pub struct SlotPropertyRegistry<T: 'static> {
    properties: &'static [SlotPropertyDescriptor<T>],
    // Map logic type -> indices into `properties` (allows multiple descriptors per logic type
    // covering different slot sets).
    lookup: HashMap<LogicSlotType, Vec<usize>>,

    // Sorted list of unique slot indices covered by the descriptors
    registered_slots: Vec<usize>,
}

impl<T: 'static> SlotPropertyRegistry<T> {
    /// Create a new slot property registry from a static slice of descriptors
    pub fn new(properties: &'static [SlotPropertyDescriptor<T>]) -> Self {
        let mut lookup: HashMap<LogicSlotType, Vec<usize>> = HashMap::new();

        // Validate descriptors for conflicting coverage and build lookup
        let mut seen_pairs: HashSet<(LogicSlotType, usize)> = HashSet::new();

        for (idx, prop) in properties.iter().enumerate() {
            let lt = prop.logic_type;

            if prop.slots.is_empty() {
                panic!(
                    "SlotPropertyDescriptor.slots must be non-empty; specify explicit slot indices"
                );
            }

            // ensure no duplicate (logic_type, slot) pairs
            for &s in prop.slots {
                if !seen_pairs.insert((lt, s)) {
                    panic!("Duplicate logic slot type for same slot in SlotPropertyRegistry");
                }
            }

            lookup.entry(lt).or_default().push(idx);
        }

        // Build the registered slot indices set (unique, sorted)
        let mut reg_set: HashSet<usize> = HashSet::new();
        for prop in properties.iter() {
            for &s in prop.slots {
                reg_set.insert(s);
            }
        }

        let mut registered_slots: Vec<usize> = reg_set.into_iter().collect();
        registered_slots.sort_unstable();

        Self {
            properties,
            lookup,
            registered_slots,
        }
    }

    /// Read a slot property value
    pub fn read(
        &self,
        device: &T,
        index: usize,
        logic_type: LogicSlotType,
    ) -> SimulationResult<f64> {
        match self.lookup.get(&logic_type) {
            Some(indices) => {
                // Find the first descriptor that applies to this index
                for &idx in indices {
                    let prop = &self.properties[idx];
                    if prop.slots.contains(&index) {
                        if let Some(read_fn) = prop.read_fn {
                            return read_fn(device, index, logic_type);
                        } else {
                            return Err(SimulationError::RuntimeError {
                                message: format!("Slot property {:?} is not readable", logic_type),
                                line: 0,
                            });
                        }
                    }
                }

                Err(SimulationError::RuntimeError {
                    message: format!(
                        "Slot property {:?} not supported for slot {}",
                        logic_type, index
                    ),
                    line: 0,
                })
            }
            None => Err(SimulationError::RuntimeError {
                message: format!("Unknown logic slot type {:?}", logic_type),
                line: 0,
            }),
        }
    }

    /// Get all supported logic slot types (union across slots)
    pub fn supported_types(&self) -> Vec<LogicSlotType> {
        self.properties.iter().map(|p| p.logic_type).collect()
    }

    /// Get all supported logic slot types for a specific slot index
    pub fn supported_types_for(&self, index: usize) -> Vec<LogicSlotType> {
        let mut out = Vec::new();
        for prop in self.properties.iter() {
            if prop.slots.contains(&index) {
                out.push(prop.logic_type);
            }
        }
        out
    }

    /// Return the sorted list of registered slot indices covered by at least one descriptor.
    /// The returned vector may be empty if there are no descriptors.
    pub fn registered_slot_indices(&self) -> Vec<usize> {
        self.registered_slots.clone()
    }

    /// Return the underlying descriptors slice (read-only)
    pub fn descriptors(&self) -> &'static [SlotPropertyDescriptor<T>] {
        self.properties
    }
}

/// Return a static empty `SlotPropertyRegistry<T>` instance for types that have no slot properties.
pub fn empty_slot_registry<T>() -> &'static SlotPropertyRegistry<T> {
    static EMPTY: OnceLock<SlotPropertyRegistry<()>> = OnceLock::new();
    let r = EMPTY.get_or_init(|| SlotPropertyRegistry::new(&[]));
    let ptr: *const SlotPropertyRegistry<()> = r as *const SlotPropertyRegistry<()>;
    let ptr_t = ptr as *const SlotPropertyRegistry<T>;
    unsafe { &*ptr_t }
}
