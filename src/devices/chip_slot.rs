//! Chip slot device for IC-supported devices

use crate::{
    CableNetwork, Device, Item,
    devices::LogicType,
    error::{SimulationError, SimulationResult},
    items::{ItemIntegratedCircuit10, ItemType, Slot},
    types::{OptShared, OptWeakShared, Shared, shared},
};
use std::{
    cell::{Ref, RefCell, RefMut},
    fmt::{Debug, Display},
};

/// Chip slot for IC; manages chip, pins, and execution state
pub struct ChipSlot {
    /// The host device
    host_device: OptWeakShared<dyn Device>,

    /// The chip slot
    slot: Slot,

    /// Device pins mapping to device reference IDs
    device_pins: Vec<Option<i32>>,

    /// Last executed instruction count
    last_executed_instructions: RefCell<usize>,
}

impl ChipSlot {
    /// Create a new `ChipSlot` with `device_pin_count` pins
    pub fn new(device_pin_count: usize) -> Shared<Self> {
        shared(Self {
            host_device: None,
            device_pins: vec![None; device_pin_count],
            slot: Slot::new(Some(ItemType::ItemIntegratedCircuit10)),
            last_executed_instructions: RefCell::new(0),
        })
    }

    /// Set the host device reference
    pub fn set_host_device(&mut self, device: OptShared<dyn Device>) {
        self.host_device = device.map(|d| std::rc::Rc::downgrade(&d));
    }

    /// Get the hosted chip (if any)
    pub fn get_chip(&self) -> Option<Ref<'_, ItemIntegratedCircuit10>> {
        self.slot.borrow_item()
    }

    /// Borrow the item in the slot as type T mutably, if it matches
    pub fn get_chip_mut(&self) -> Option<RefMut<'_, ItemIntegratedCircuit10>> {
        self.slot.borrow_item_mut()
    }

    /// Install `chip` into the slot; returns Err if occupied
    pub fn set_chip(
        &mut self,
        chip: Shared<ItemIntegratedCircuit10>,
    ) -> Result<(), Shared<ItemIntegratedCircuit10>> {
        if self.slot.get_item().is_some() {
            return Err(chip);
        }

        match self.slot.try_insert(chip.clone()) {
            Ok(()) => Ok(()),
            Err(_leftover) => Err(chip),
        }
    }

    /// Remove and return the installed chip
    pub fn remove_chip(&mut self) -> OptShared<dyn Item> {
        self.slot.remove()
    }

    /// Set device pin `pin` to `device_ref_id`
    pub fn set_device_pin(&mut self, pin: usize, device_ref_id: Option<i32>) {
        if pin < self.device_pins.len() {
            self.device_pins[pin] = device_ref_id;
        }
    }

    /// Get the device reference for `pin`
    pub fn get_device_pin(&self, pin: usize) -> Option<i32> {
        if pin < self.device_pins.len() {
            self.device_pins[pin]
        } else {
            None
        }
    }

    /// Run the hosted chip up to `max_instructions_per_tick`
    pub fn run(&self, max_instructions_per_tick: usize) -> SimulationResult<()> {
        if let Some(chip) = self.get_chip() {
            let instructions = chip.run(max_instructions_per_tick)?;
            *self.last_executed_instructions.borrow_mut() = instructions;
        }

        Ok(())
    }

    /// Get last executed instruction count
    pub fn get_last_executed_instructions(&self) -> usize {
        *self.last_executed_instructions.borrow()
    }

    /// Get host device's network
    pub fn get_network(&self) -> OptShared<CableNetwork> {
        if let Some(host_weak) = &self.host_device
            && let Some(host) = host_weak.upgrade()
        {
            host.borrow().get_network()
        } else {
            None
        }
    }

    /// Host device reference ID, if any
    pub fn id(&self) -> Option<i32> {
        if let Some(host_weak) = &self.host_device {
            host_weak.upgrade().map(|host| host.borrow().get_id())
        } else {
            None
        }
    }

    /// Read a logic value from the host device
    pub fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        if let Some(host_weak) = &self.host_device
            && let Some(host) = host_weak.upgrade()
        {
            host.borrow().read(logic_type)
        } else {
            Err(SimulationError::RuntimeError {
                message: "No host device".to_string(),
                line: 0,
            })
        }
    }

    /// Write a logic value to the host device
    pub fn write(&self, logic_type: LogicType, value: f64) -> SimulationResult<()> {
        if let Some(host_weak) = &self.host_device
            && let Some(host) = host_weak.upgrade()
        {
            host.borrow().write(logic_type, value)
        } else {
            Err(SimulationError::RuntimeError {
                message: "No host device".to_string(),
                line: 0,
            })
        }
    }
}

impl Display for ChipSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let host_str = self
            .id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string());
        let mounted = self.slot.get_item().is_some();
        let pins = self
            .device_pins
            .iter()
            .map(|p| p.map(|i| i.to_string()).unwrap_or_else(|| "-".to_string()))
            .collect::<Vec<_>>()
            .join(", ");

        write!(
            f,
            "ChipSlot {{ host: {}, mounted: {}, pins: [{}] }}",
            host_str, mounted, pins
        )
    }
}

impl Debug for ChipSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
