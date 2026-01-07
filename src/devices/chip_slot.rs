use crate::{
    CableNetwork, Device, Item,
    devices::LogicType,
    error::{SimulationError, SimulationResult},
    items::{ItemIntegratedCircuit10, ItemType, Slot},
    types::{OptShared, Shared, shared},
};
use std::cell::RefCell;

/// Helper that encapsulates chip slot, device pins and chip execution.
pub struct ChipSlot {
    /// The host device
    host_device: OptShared<dyn Device>,

    /// The chip slot
    slot: Slot,

    /// Device pins mapping to device reference IDs
    device_pins: Vec<Option<i32>>,

    /// Last executed instruction count
    last_executed_instructions: RefCell<usize>,
}

impl ChipSlot {
    /// Create a new ChipHost with one IC chip slot and device pins.
    pub fn new(device_pin_count: usize) -> Shared<Self> {
        shared(Self {
            host_device: None,
            slot: Slot::new(Some(ItemType::ItemIntegratedCircuit10)),
            device_pins: vec![None; device_pin_count],
            last_executed_instructions: RefCell::new(0),
        })
    }

    /// Set the host device
    pub fn set_host_device(&mut self, device: OptShared<dyn Device>) {
        self.host_device = device;
    }

    /// Insert a chip (will not replace existing chip)
    /// Returns `Ok(())` on success, or `Err(item)` if insertion failed.
    pub fn set_chip(
        &mut self,
        chip: Box<Shared<ItemIntegratedCircuit10>>,
    ) -> Result<(), Box<dyn Item>> {
        self.slot.try_insert(chip)
    }

    /// Get the hosted chip (if any)
    pub fn get_chip(&self) -> OptShared<ItemIntegratedCircuit10> {
        if let Some(item) = self.slot.get_item()
            && let Some(shared_chip) = item
                .as_any()
                .downcast_ref::<Shared<ItemIntegratedCircuit10>>()
        {
            return Some(shared_chip.clone());
        }

        None
    }

    pub fn set_device_pin(&mut self, pin: usize, device_ref_id: Option<i32>) {
        if pin < self.device_pins.len() {
            self.device_pins[pin] = device_ref_id;
        }
    }

    pub fn get_device_pin(&self, pin: usize) -> Option<i32> {
        if pin < self.device_pins.len() {
            self.device_pins[pin]
        } else {
            None
        }
    }

    pub fn run(&self, max_instructions_per_tick: usize) -> SimulationResult<()> {
        if let Some(chip) = self.get_chip() {
            let instructions = chip.borrow().run(max_instructions_per_tick)?;
            *self.last_executed_instructions.borrow_mut() = instructions;
        }

        Ok(())
    }

    pub fn get_last_executed_instructions(&self) -> usize {
        *self.last_executed_instructions.borrow()
    }

    /// Get the network of the hosting device (if present)
    pub fn get_network(&self) -> OptShared<CableNetwork> {
        if let Some(host) = &self.host_device {
            host.borrow().get_network()
        } else {
            None
        }
    }

    /// Get the id of the hosting device (or -1 if not present) TODO
    pub fn id(&self) -> Option<i32> {
        self.host_device.as_ref().map(|host| host.borrow().get_id())
    }

    /// Proxy read to the hosting device
    pub fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
        if let Some(host) = &self.host_device {
            host.borrow().read(logic_type)
        } else {
            Err(SimulationError::RuntimeError {
                message: "No host device".to_string(),
                line: 0,
            })
        }
    }

    /// Proxy write to the hosting device
    pub fn write(&self, logic_type: LogicType, value: f64) -> SimulationResult<()> {
        if let Some(host) = &self.host_device {
            host.borrow().write(logic_type, value)
        } else {
            Err(SimulationError::RuntimeError {
                message: "No host device".to_string(),
                line: 0,
            })
        }
    }
}

impl std::fmt::Display for ChipSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let host_str = self
            .id()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "none".to_string());
        let mounted = self.get_chip().is_some();
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

impl std::fmt::Debug for ChipSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
