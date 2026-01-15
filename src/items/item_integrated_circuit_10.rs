//! IC10 programmable chip item implementation

use crate::constants::{REGISTER_COUNT, RETURN_ADDRESS_INDEX, STACK_POINTER_INDEX, STACK_SIZE};
use crate::devices::ChipSlot;
use crate::error::{SimulationError, SimulationResult};
use crate::instruction::{Instruction, ParsedInstruction};
use crate::items::SimulationItemSettings;
use crate::parser::{preprocess, string_to_hash};
use crate::types::{OptShared, OptWeakShared, Shared};
use crate::{CableNetwork, Item, ItemType, get_builtin_constants};
use crate::{LogicType, logic};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The IC10 programmable chip
#[derive(Debug)]
pub struct ItemIntegratedCircuit10 {
    /// Unique global ID
    id: i32,

    /// Program counter - current line being executed
    pc: RefCell<usize>,

    /// Compiled program lines
    program: RefCell<Vec<ParsedInstruction>>,

    /// Aliases mapping names to register/device indices
    aliases: RefCell<HashMap<String, AliasTarget>>,

    /// Jump labels mapping names to line numbers
    labels: RefCell<HashMap<String, usize>>,

    /// Compile-time constants
    defines: RefCell<HashMap<String, f64>>,

    /// Original source text
    source: RefCell<Option<String>>,

    /// Chip slot reference (optional)
    chip_slot: OptWeakShared<ChipSlot>,

    /// Registers (r0-r17)
    registers: RefCell<[f64; REGISTER_COUNT]>,

    /// Stack memory
    stack: RefCell<[f64; STACK_SIZE]>,

    /// Execution state
    halted: RefCell<bool>,

    /// Remaining sleep ticks (if sleeping)
    sleep_ticks: RefCell<u64>,

    /// Error state
    error_line: RefCell<Option<usize>>,
}

/// Alias target - can reference a register or device
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasTarget {
    Register(usize),
    Device(i32),   // Stores device reference ID
    Alias(String), // References another alias by name
}

impl ItemIntegratedCircuit10 {
    /// Create a new `ItemIntegratedCircuit10`
    pub fn new(settings: SimulationItemSettings) -> Self {
        let mut aliases = HashMap::new();

        aliases.insert("sp".to_string(), AliasTarget::Register(STACK_POINTER_INDEX));
        aliases.insert(
            "ra".to_string(),
            AliasTarget::Register(RETURN_ADDRESS_INDEX),
        );

        Self {
            id: settings.id.unwrap(),
            pc: RefCell::new(0),
            program: RefCell::new(Vec::new()),
            aliases: RefCell::new(aliases),
            labels: RefCell::new(HashMap::new()),
            defines: RefCell::new(get_builtin_constants()),
            source: RefCell::new(None),
            chip_slot: None,
            registers: RefCell::new([0.0; REGISTER_COUNT]),
            stack: RefCell::new([0.0; STACK_SIZE]),
            halted: RefCell::new(false),
            error_line: RefCell::new(None),
            sleep_ticks: RefCell::new(0),
        }
    }

    /// Load IC10 source code into the chip
    pub fn load_program(&mut self, source: &str) -> SimulationResult<()> {
        self.program.borrow_mut().clear();
        self.labels.borrow_mut().clear();
        self.registers.borrow_mut().fill(0.0);
        self.stack.borrow_mut().fill(0.0);
        *self.pc.borrow_mut() = 0;
        *self.halted.borrow_mut() = false;
        *self.error_line.borrow_mut() = None;
        *self.source.borrow_mut() = Some(source.to_string());

        // Preprocess the source
        let preprocessed = preprocess(source)?;

        // First pass: collect labels
        for (line_num, line) in preprocessed.lines().enumerate() {
            let trimmed = line.trim();

            // Check if this is a label (ends with ':')
            if trimmed.ends_with(':') && !trimmed.starts_with('#') {
                let label_name = trimmed[..trimmed.len() - 1].trim().to_string();
                if self.labels.borrow().contains_key(&label_name) {
                    return Err(SimulationError::IC10ParseError {
                        line: line_num,
                        message: format!("Duplicate label: {label_name}"),
                    });
                }
                self.labels.borrow_mut().insert(label_name, line_num);
            }
        }

        // Second pass: parse instructions
        for (line_num, line) in preprocessed.lines().enumerate() {
            let parsed = ParsedInstruction::parse(line, line_num)?;

            // If this is an alias instruction for a device, validate the device pin
            if let Instruction::Alias {
                name: _,
                target: AliasTarget::Device(pin_idx),
            } = parsed.instruction
            {
                // pin_idx in parsed instruction is the pin index (stored as i32)
                let pin = pin_idx as usize;
                if pin >= 6 {
                    return Err(SimulationError::IC10ParseError {
                        line: line_num,
                        message: format!("Device pin out of range: d{} (max d{})", pin, 6 - 1),
                    });
                }
            }

            self.program.borrow_mut().push(parsed);
        }

        Ok(())
    }

    /// Execute a single instruction; returns whether executed
    pub fn step(&self) -> SimulationResult<bool> {
        if *self.halted.borrow() {
            return Ok(false);
        }

        if *self.pc.borrow() >= self.program.borrow().len() {
            *self.halted.borrow_mut() = true;
            return Ok(false);
        }

        let instruction = &self.program.borrow()[*self.pc.borrow()].clone();

        match self.execute_instruction(instruction) {
            Ok(next_pc) => {
                *self.pc.borrow_mut() = next_pc;
                Ok(true)
            }
            Err(e) => {
                *self.error_line.borrow_mut() = Some(*self.pc.borrow());
                *self.halted.borrow_mut() = true;
                Err(e)
            }
        }
    }

    /// Run up to `max_steps`, stopping at yield or sleep
    pub fn run(&self, max_steps: usize) -> SimulationResult<usize> {
        let mut steps = 0;

        while steps < max_steps {
            if *self.halted.borrow() {
                return Ok(0);
            } else if *self.pc.borrow() >= self.program.borrow().len() {
                *self.halted.borrow_mut() = true;
                return Ok(0);
            }

            steps += 1;
            if *self.sleep_ticks.borrow() > 0 {
                *self.sleep_ticks.borrow_mut() -= 1;
                return Ok(steps);
            }

            let current_instruction = self.program.borrow()[*self.pc.borrow()].clone();

            self.step()?;

            match current_instruction.instruction {
                Instruction::Yield | Instruction::Sleep { duration: _ } => {
                    return Ok(steps);
                }
                _ => {}
            }
        }

        Ok(steps)
    }

    /// Execute `instruction` and return next program counter
    fn execute_instruction(&self, instruction: &ParsedInstruction) -> SimulationResult<usize> {
        logic::execute_instruction(self, instruction)
    }

    /// Resolve an `Operand` to a runtime value
    pub(crate) fn resolve_value(&self, operand: &Operand) -> SimulationResult<f64> {
        match operand {
            Operand::Register(idx) => self.get_register(*idx),
            Operand::Immediate(val) => Ok(*val),
            Operand::DevicePin(_) => Err(SimulationError::RuntimeError {
                line: *self.pc.borrow(),
                message: "Cannot use device pin as a value".to_string(),
            }),
            Operand::Alias(name) => {
                // First check defines (compile-time constants)
                if let Some(val) = self.defines.borrow().get(name) {
                    return Ok(*val);
                }
                // Then check aliases
                match self.aliases.borrow().get(name) {
                    Some(AliasTarget::Register(idx)) => self.get_register(*idx),
                    Some(AliasTarget::Device(_)) => Err(SimulationError::RuntimeError {
                        line: *self.pc.borrow(),
                        message: format!("Cannot use device alias '{name}' as a value"),
                    }),
                    Some(AliasTarget::Alias(other_name)) => Err(SimulationError::RuntimeError {
                        line: *self.pc.borrow(),
                        message: format!(
                            "Alias '{name}' referencing another alias '{other_name}' at runtime"
                        ),
                    }),
                    None => {
                        // Check for labels
                        if let Some(&line) = self.labels.borrow().get(name) {
                            return Ok(line as f64);
                        }

                        Err(SimulationError::RuntimeError {
                            line: *self.pc.borrow(),
                            message: format!("Undefined alias, define, or label: {name}"),
                        })
                    }
                }
            }
        }
    }

    /// Resolve an `Operand` to a register index
    pub(crate) fn resolve_register(&self, operand: &Operand) -> SimulationResult<usize> {
        match operand {
            Operand::Register(idx) => Ok(*idx),
            Operand::Immediate(_) => Err(SimulationError::RuntimeError {
                line: *self.pc.borrow(),
                message: "Cannot use immediate value as a register destination".to_string(),
            }),
            Operand::DevicePin(_) => Err(SimulationError::RuntimeError {
                line: *self.pc.borrow(),
                message: "Cannot use device pin as a register destination".to_string(),
            }),
            Operand::Alias(name) => {
                // Check aliases first
                match self.aliases.borrow().get(name) {
                    Some(AliasTarget::Register(idx)) => Ok(*idx),
                    Some(AliasTarget::Device(_)) => Err(SimulationError::RuntimeError {
                        line: *self.pc.borrow(),
                        message: format!(
                            "Cannot use device alias '{name}' as a register destination"
                        ),
                    }),
                    _ => Err(SimulationError::RuntimeError {
                        line: *self.pc.borrow(),
                        message: format!("Undefined alias: {name}"),
                    }),
                }
            }
        }
    }

    /// Resolve an `Operand` to a device reference ID
    /// Device pin operands consult the hosting chip slot
    pub(crate) fn resolve_device_ref_id(&self, operand: &Operand) -> SimulationResult<i32> {
        match operand {
            Operand::DevicePin(pin_idx) => {
                // Direct device pin access (d0-d5) - get the reference ID stored at this pin
                let chip_slot = self.get_chip_slot();
                if let Some(ref_id) = chip_slot.borrow().get_device_pin(*pin_idx) {
                    Ok(ref_id)
                } else {
                    Err(SimulationError::RuntimeError {
                        line: *self.pc.borrow(),
                        message: format!("No device assigned to pin d{pin_idx}"),
                    })
                }
            }
            Operand::Register(idx) => {
                // Indirect device access - register contains device reference ID
                let ref_id = self.get_register(*idx)? as i32;
                Ok(ref_id)
            }
            Operand::Immediate(val) => {
                // Immediate value is a reference ID
                Ok(*val as i32)
            }
            Operand::Alias(name) => {
                // Device aliases store reference IDs directly
                match self.aliases.borrow().get(name) {
                    Some(AliasTarget::Device(ref_id)) => Ok(*ref_id),
                    Some(AliasTarget::Register(idx)) => {
                        // Indirect device access - register contains device reference ID
                        let ref_id = self.get_register(*idx)? as i32;
                        Ok(ref_id)
                    }
                    Some(AliasTarget::Alias(_)) => Err(SimulationError::RuntimeError {
                        line: *self.pc.borrow(),
                        message: format!("Alias '{name}' referencing another alias at runtime"),
                    }),
                    None => {
                        if let Some(val) = self.defines.borrow().get(name) {
                            Ok(*val as i32)
                        } else {
                            Err(SimulationError::RuntimeError {
                                line: *self.pc.borrow(),
                                message: format!("Undefined alias or define: {name}"),
                            })
                        }
                    }
                }
            }
        }
    }

    /// Resolve an alias name to its `AliasTarget`
    pub(crate) fn resolve_alias(&self, name: &str) -> SimulationResult<AliasTarget> {
        match self.aliases.borrow().get(name) {
            Some(target) => Ok(target.clone()),
            None => Err(SimulationError::RuntimeError {
                line: *self.pc.borrow(),
                message: format!("Undefined alias: {name}"),
            }),
        }
    }

    /// Check whether a device with `ref_id` exists on the local network
    pub(crate) fn device_exists_by_id(&self, ref_id: i32) -> bool {
        if let Some(slot_weak) = &self.chip_slot {
            if let Some(slot_rc) = slot_weak.upgrade() {
                let slot = slot_rc.borrow();
                if let Some(network_rc) = slot.get_network() {
                    network_rc.borrow().device_exists(ref_id)
                } else if let Some(id) = slot.id() {
                    ref_id == id
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get register value at `index`
    pub fn get_register(&self, index: usize) -> SimulationResult<f64> {
        if index >= REGISTER_COUNT {
            return Err(SimulationError::RegisterOutOfBounds(index));
        }
        Ok(self.registers.borrow()[index])
    }

    /// Set register at `index` to `value`
    pub fn set_register(&self, index: usize, value: f64) -> SimulationResult<()> {
        if index >= REGISTER_COUNT {
            return Err(SimulationError::RegisterOutOfBounds(index));
        }
        self.registers.borrow_mut()[index] = value;
        Ok(())
    }

    /// Read from stack memory at `address`
    pub fn read_stack(&self, address: usize) -> SimulationResult<f64> {
        if address >= STACK_SIZE {
            return Err(SimulationError::StackOutOfBounds(address));
        }
        Ok(self.stack.borrow()[address])
    }

    /// Write `value` into stack memory at `address`
    pub fn write_stack(&self, address: usize, value: f64) -> SimulationResult<()> {
        if address >= STACK_SIZE {
            return Err(SimulationError::StackOutOfBounds(address));
        }
        self.stack.borrow_mut()[address] = value;
        Ok(())
    }

    /// Clear the stack memory
    pub fn clear_stack(&self) {
        self.stack.borrow_mut().fill(0.0);
    }

    /// Insert a compile-time define constant
    pub fn insert_define(&self, name: &str, value: f64) {
        self.defines.borrow_mut().insert(name.to_string(), value);
    }

    /// Insert an alias for a register or device pin
    pub fn insert_alias(&self, name: &str, target: AliasTarget) {
        self.aliases.borrow_mut().insert(name.to_string(), target);
    }

    /// Add a device alias (convenience method)
    /// Note: device_ref_id is the device's reference ID (from get_id()), not the pin index
    pub fn add_device_alias(&self, name: String, device_ref_id: i32) {
        self.aliases
            .borrow_mut()
            .insert(name, AliasTarget::Device(device_ref_id));
    }

    /// Get the current program counter
    pub fn get_pc(&self) -> usize {
        *self.pc.borrow()
    }

    /// Set the current program counter
    pub fn set_pc(&self, pc: usize) {
        *self.pc.borrow_mut() = pc;
    }

    /// Check if the chip is halted
    pub fn is_halted(&self) -> bool {
        *self.halted.borrow()
    }

    /// Halt the chip
    pub fn halt(&self) {
        *self.halted.borrow_mut() = true;
    }

    /// Resume the chip
    pub fn resume(&self) {
        *self.halted.borrow_mut() = false;
    }

    /// Get sleep ticks
    pub fn get_sleep_ticks(&self) -> u64 {
        *self.sleep_ticks.borrow()
    }

    /// Set sleep ticks
    pub fn set_sleep_ticks(&self, ticks: u64) {
        *self.sleep_ticks.borrow_mut() = ticks;
    }

    /// Get the `Shared<ChipSlot>` for this chip (clone of the internal Rc)
    pub fn get_chip_slot(&self) -> Shared<ChipSlot> {
        self.chip_slot.as_ref().unwrap().upgrade().unwrap().clone()
    }

    /// Get the host device's reference ID if this chip is installed
    pub fn get_host_id(&self) -> Option<i32> {
        self.chip_slot
            .as_ref()
            .and_then(|weak| weak.upgrade().and_then(|rc| rc.borrow().id()))
    }

    /// Get the stored original source text for this chip (if any)
    pub fn get_source(&self) -> Option<String> {
        self.source.borrow().clone()
    }

    /// Get the number of lines in the loaded program
    pub fn get_line_count(&self) -> usize {
        self.program.borrow().len()
    }

    /// Get the error line if execution failed (None if no error)
    pub fn get_error_line(&self) -> Option<usize> {
        *self.error_line.borrow()
    }

    /// Attach the chip to a `ChipSlot` and register self device aliases
    pub fn set_chip_slot(&mut self, slot: Shared<ChipSlot>, device_id: i32) {
        // Store weak slot reference
        self.chip_slot = Some(Rc::downgrade(&slot));

        // Add a convenient alias `db` referencing the device itself
        self.add_device_alias("db".to_string(), device_id);
    }

    /// Get a reference to the cable network (if connected)
    pub fn get_network(&self) -> OptShared<CableNetwork> {
        self.get_chip_slot().borrow().get_network()
    }

    /// Print debug information: registers and non-zero stack values
    pub fn print_debug_info(&self) {
        println!(
            "On: {}",
            if self
                .get_chip_slot()
                .borrow()
                .read(LogicType::On)
                .unwrap_or(0.0)
                == 1.0
            {
                "Yes"
            } else {
                "No"
            }
        );
        println!(
            "Halted: {}",
            if !*self.halted.borrow() { "Yes" } else { "No" }
        );
        println!("Non-zero Registers:");
        for i in 0..REGISTER_COUNT {
            let value = self.registers.borrow()[i];
            if value != 0.0 {
                let reg = if i == STACK_POINTER_INDEX {
                    "sp".to_string()
                } else if i == RETURN_ADDRESS_INDEX {
                    "ra".to_string()
                } else {
                    format!("r{}", i)
                };

                if value.fract() == 0.0 {
                    println!("{reg}: {value:.0}");
                } else {
                    println!("{reg}: {value:.6}");
                }
            }
        }

        println!("\nNon-zero Stack Values:");
        for i in 0..STACK_SIZE {
            let value = self.stack.borrow()[i];
            if value != 0.0 {
                if value.fract() == 0.0 {
                    println!("stack[{i}]: {value:.0}");
                } else {
                    println!("stack[{i}]: {value:.6}");
                }
            }
        }
    }
}

impl ItemIntegratedCircuit10 {
    /// Compile-time prefab hash constant for this item
    pub const PREFAB_HASH: i32 = string_to_hash("ItemIntegratedCircuit10");

    /// Return the prefab hash for an IC10 item
    pub fn prefab_hash() -> i32 {
        Self::PREFAB_HASH
    }
}

/// Operand types for instructions
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Register(usize),
    Immediate(f64),
    Alias(String),
    DevicePin(usize),
}

// Implement the Item trait for ItemIntegratedCircuit10
impl Item for ItemIntegratedCircuit10 {
    fn item_type(&self) -> ItemType {
        ItemType::ItemIntegratedCircuit10
    }

    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_prefab_hash(&self) -> i32 {
        ItemIntegratedCircuit10::prefab_hash()
    }

    fn quantity(&self) -> u32 {
        1
    }

    fn set_quantity(&mut self, _quantity: u32) -> bool {
        false
    }

    fn max_quantity(&self) -> u32 {
        1
    }

    fn merge(&mut self, _other: &mut dyn Item) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
