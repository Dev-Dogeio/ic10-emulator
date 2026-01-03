//! IC Housing device - a housing unit for integrated circuits
//!
//! The IC Housing provides:
//! - 6 device pins (d0-d5) for connecting to other devices on the network
//! - 18 registers (r0-r15 general purpose, r16=sp stack pointer, r17=ra return address)
//! - 512 stack memory entries

use crate::{
    CableNetwork, ProgrammableChip,
    constants::{DEVICE_PIN_COUNT, REGISTER_COUNT, STACK_SIZE},
    devices::{Device, DeviceBase, LogicType, SimulationSettings},
    error::{IC10Error, IC10Result},
};
use std::{cell::RefCell, rc::Rc};

/// IC Housing - a housing unit that holds an IC10 chip and connects to devices
/// that can reference ANY device on the cable network by its reference ID.
#[derive(Debug)]
pub struct ICHousing {
    base: DeviceBase,
    chip: Option<Rc<RefCell<ProgrammableChip>>>,
    /// Device pins (d0-d5) - store reference IDs to devices on the network
    /// None means no device is assigned to that pin
    device_pins: [Option<i32>; DEVICE_PIN_COUNT],
    /// 18 registers (r0-r15 general purpose, r16=sp, r17=ra)
    registers: RefCell<[f64; REGISTER_COUNT]>,
    /// Stack memory (512 entries) - accessed via get/put instructions
    stack: RefCell<[f64; STACK_SIZE]>,
    /// Simulation settings
    settings: SimulationSettings,
}

impl ICHousing {
    pub fn new(
        simulation_settings: Option<SimulationSettings>,
        chip: Option<Rc<RefCell<ProgrammableChip>>>,
    ) -> Self {
        let base = DeviceBase::new(
            "IC Housing".to_string(),
            "StructureCircuitHousing".to_string(),
        );

        base.logic_types.borrow_mut().setting = Some(0.0);

        Self {
            base,
            chip: chip,
            device_pins: [None; DEVICE_PIN_COUNT],
            registers: RefCell::new([0.0; REGISTER_COUNT]),
            stack: RefCell::new([0.0; STACK_SIZE]),
            settings: simulation_settings.unwrap_or_default(),
        }
    }

    /// Get a reference to the connected network
    pub fn get_network(&self) -> Option<Rc<RefCell<CableNetwork>>> {
        self.base.get_network()
    }

    /// Check if the housing is connected to a network
    pub fn is_connected(&self) -> bool {
        self.base.is_connected()
    }

    /// Get a register value
    pub fn get_register(&self, index: usize) -> IC10Result<f64> {
        if index >= REGISTER_COUNT {
            return Err(IC10Error::RuntimeError {
                message: format!("Register index {} out of bounds", index),
                line: 0,
            });
        }
        Ok(self.registers.borrow()[index])
    }

    /// Set a register value
    pub fn set_register(&self, index: usize, value: f64) -> IC10Result<()> {
        if index >= REGISTER_COUNT {
            return Err(IC10Error::RuntimeError {
                message: format!("Register index {} out of bounds", index),
                line: 0,
            });
        }
        self.registers.borrow_mut()[index] = value;
        Ok(())
    }

    /// Read from stack memory
    pub fn read_stack(&self, address: usize) -> IC10Result<f64> {
        if address >= STACK_SIZE {
            return Err(IC10Error::RuntimeError {
                message: format!("Stack address {} out of bounds", address),
                line: 0,
            });
        }
        Ok(self.stack.borrow()[address])
    }

    /// Write to stack memory
    pub fn write_stack(&self, address: usize, value: f64) -> IC10Result<()> {
        if address >= STACK_SIZE {
            return Err(IC10Error::RuntimeError {
                message: format!("Stack address {} out of bounds", address),
                line: 0,
            });
        }
        self.stack.borrow_mut()[address] = value;
        Ok(())
    }

    /// Clear device stack memory (clr/clrd)
    pub fn clear(&self) {
        self.stack.borrow_mut().fill(0.0);
    }

    /// Get the unique id of the housing
    pub fn id(&self) -> i32 {
        self.base.logic_types.borrow().reference_id
    }

    /// Get the prefab hash of the housing
    pub fn prefab_hash(&self) -> i32 {
        self.base.logic_types.borrow().prefab_hash
    }

    /// Get the name hash of the housing
    pub fn name_hash(&self) -> i32 {
        self.base.logic_types.borrow().name_hash
    }

    /// Set a device pin to reference a device by its reference ID
    /// The device must exist on the network (caller's responsibility to verify)
    pub fn set_device_pin(&mut self, pin: usize, device_ref_id: Option<i32>) {
        if pin < DEVICE_PIN_COUNT {
            self.device_pins[pin] = device_ref_id;
        }
    }

    /// Get the reference ID of the device at a specific pin
    pub fn get_device_pin(&self, pin: usize) -> Option<i32> {
        if pin < DEVICE_PIN_COUNT {
            self.device_pins[pin]
        } else {
            None
        }
    }

    /// Check if a pin has a device assigned
    pub fn is_pin_set(&self, pin: usize) -> bool {
        pin < DEVICE_PIN_COUNT && self.device_pins[pin].is_some()
    }

    /// Clear a device pin
    pub fn clear_device_pin(&mut self, pin: usize) {
        if pin < DEVICE_PIN_COUNT {
            self.device_pins[pin] = None;
        }
    }

    /// Check if a pin index is valid (0-5)
    pub fn is_valid_pin(&self, pin: usize) -> bool {
        pin < DEVICE_PIN_COUNT
    }

    /// Set the cable network reference
    pub fn set_network(&mut self, network: Option<Rc<RefCell<CableNetwork>>>) {
        self.base.set_network(network);
    }

    /// Set the chip for the housing
    pub fn set_chip(&mut self, chip: Rc<RefCell<ProgrammableChip>>) {
        self.chip = Some(chip);
    }

    /// Get the chip of the housing
    pub fn get_chip(&self) -> Option<Rc<RefCell<ProgrammableChip>>> {
        self.chip.as_ref().map(Rc::clone)
    }

    /// Get the maximum instructions per tick setting
    pub fn get_max_instructions_per_tick(&self) -> usize {
        self.settings.max_instructions_per_tick
    }

    /// Run the chip for a specified number of steps
    pub fn update(&self, _tick: u64) -> Option<IC10Result<usize>> {
        if let Some(ref chip) = self.chip {
            let result = chip
                .borrow_mut()
                .run(self.settings.max_instructions_per_tick);
            return Some(result);
        }

        None
    }
}

impl Device for ICHousing {
    fn get_id(&self) -> i32 {
        self.base.logic_types.borrow().reference_id
    }

    fn get_prefab_hash(&self) -> i32 {
        self.base.logic_types.borrow().prefab_hash
    }

    fn get_name_hash(&self) -> i32 {
        self.base.logic_types.borrow().name_hash
    }

    fn get_name(&self) -> &str {
        &self.base.name
    }

    fn get_network(&self) -> Option<Rc<RefCell<CableNetwork>>> {
        self.base.network.clone()
    }

    fn set_network(&mut self, network: Option<Rc<RefCell<CableNetwork>>>) {
        self.base.network = network;
    }

    fn set_name(&mut self, name: &str) {
        self.base.set_name(name.to_string());
    }

    fn can_read(&self, logic_type: LogicType) -> bool {
        matches!(
            logic_type,
            LogicType::PrefabHash
                | LogicType::ReferenceId
                | LogicType::NameHash
                | LogicType::Setting
        )
    }

    fn can_write(&self, logic_type: LogicType) -> bool {
        matches!(logic_type, LogicType::Setting)
    }

    fn read(&self, logic_type: LogicType) -> IC10Result<f64> {
        match logic_type {
            LogicType::PrefabHash => Ok(self.base.logic_types.borrow().prefab_hash as f64),
            LogicType::ReferenceId => Ok(self.base.logic_types.borrow().reference_id as f64),
            LogicType::NameHash => Ok(self.base.logic_types.borrow().name_hash as f64),
            LogicType::Setting => {
                self.base
                    .logic_types
                    .borrow()
                    .setting
                    .ok_or(IC10Error::RuntimeError {
                        message: "Setting value not set".to_string(),
                        line: 0,
                    })
            }
            _ => Err(IC10Error::RuntimeError {
                message: format!(
                    "Housing does not support reading logic type {:?}",
                    logic_type
                ),
                line: 0,
            }),
        }
    }

    fn write(&self, logic_type: LogicType, value: f64) -> IC10Result<()> {
        match logic_type {
            LogicType::Setting => {
                self.base.logic_types.borrow_mut().setting = Some(value);
                Ok(())
            }
            _ => Err(IC10Error::RuntimeError {
                message: format!(
                    "Housing does not support writing logic type {:?}",
                    logic_type
                ),
                line: 0,
            }),
        }
    }

    /// Read from device internal memory at index
    fn get_memory(&self, index: usize) -> IC10Result<f64> {
        self.read_stack(index)
    }

    /// Write to device internal memory at index
    fn set_memory(&self, index: usize, value: f64) -> IC10Result<()> {
        self.write_stack(index, value)
    }

    /// Clear device stack memory (clr/clrd)
    fn clear(&self) -> IC10Result<()> {
        self.stack.borrow_mut().fill(0.0);
        Ok(())
    }
}

impl Default for ICHousing {
    fn default() -> Self {
        Self::new(None, None)
    }
}
