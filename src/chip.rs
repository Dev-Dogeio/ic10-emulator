use crate::constants::{
    DEVICE_PIN_COUNT, REGISTER_COUNT, RETURN_ADDRESS_INDEX, STACK_POINTER_INDEX, STACK_SIZE,
};
use crate::devices::ICHousing;
use crate::error::{IC10Error, IC10Result};
use crate::get_builtin_constants;
use crate::instruction::{Instruction, ParsedInstruction};
use crate::logic;
use crate::network::CableNetwork;
use crate::parser::preprocess;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The main IC10 programmable chip
#[derive(Debug)]
pub struct ProgrammableChip {
    /// Program counter - current line being executed
    pc: usize,

    /// Compiled program lines
    program: Vec<ParsedInstruction>,

    /// Aliases mapping names to register/device indices
    aliases: HashMap<String, AliasTarget>,

    /// Jump labels mapping names to line numbers
    labels: HashMap<String, usize>,

    /// Compile-time constants
    defines: HashMap<String, f64>,

    /// Circuit housing (the IC housing device itself)
    housing: Rc<RefCell<ICHousing>>,

    /// Execution state
    halted: bool,

    /// Remaining sleep ticks (if sleeping)
    sleep_ticks: u64,

    /// Error state
    error_line: Option<usize>,
}

/// Alias target - can reference a register or device
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AliasTarget {
    Register(usize),
    Device(i32), // Stores device reference ID
}

impl ProgrammableChip {
    /// Create a new programmable chip with a housing
    pub fn new(housing: Rc<RefCell<ICHousing>>) -> Self {
        let mut aliases = HashMap::new();
        // db is a default alias that points to the housing itself
        let housing_id = housing.borrow().id();
        aliases.insert("db".to_string(), AliasTarget::Device(housing_id));

        Self {
            pc: 0,
            program: Vec::new(),
            aliases,
            labels: HashMap::new(),
            defines: get_builtin_constants(),
            housing,
            halted: false,
            error_line: None,
            sleep_ticks: 0,
        }
    }

    /// Create a chip with a new network and housing
    /// Sets up a chip, network, and housing, and connects them
    pub fn new_with_network() -> (
        Rc<RefCell<Self>>,
        Rc<RefCell<ICHousing>>,
        Rc<RefCell<CableNetwork>>,
    ) {
        let network = Rc::new(RefCell::new(CableNetwork::new()));
        let housing = Rc::new(RefCell::new(ICHousing::new(None, None)));
        let chip = Rc::new(RefCell::new(ProgrammableChip::new(housing.clone())));

        // Connect chip to housing
        housing.borrow_mut().set_chip(chip.clone());

        // Connect housing to network (which also adds it as a device)
        network
            .borrow_mut()
            .add_device(housing.clone(), network.clone());

        (chip, housing, network)
    }

    /// Load a program from source code
    pub fn load_program(&mut self, source: &str) -> IC10Result<()> {
        self.program.clear();
        self.labels.clear();
        self.pc = 0;
        self.halted = false;
        self.error_line = None;

        // Preprocess the source
        let preprocessed = preprocess(source)?;

        // First pass: collect labels
        for (line_num, line) in preprocessed.lines().enumerate() {
            let trimmed = line.trim();

            // Check if this is a label (ends with ':')
            if trimmed.ends_with(':') && !trimmed.starts_with('#') {
                let label_name = trimmed[..trimmed.len() - 1].trim().to_string();
                if self.labels.contains_key(&label_name) {
                    return Err(IC10Error::ParseError {
                        line: line_num,
                        message: format!("Duplicate label: {}", label_name),
                    });
                }
                self.labels.insert(label_name, line_num);
            }
        }

        // Second pass: parse instructions
        for (line_num, line) in preprocessed.lines().enumerate() {
            let parsed = ParsedInstruction::parse(line, line_num)?;
            // If this is an alias instruction for a device, validate the device pin
            if let Instruction::Alias {
                name: _,
                ref target,
            } = parsed.instruction
            {
                if let AliasTarget::Device(pin_idx) = target {
                    // pin_idx in parsed instruction is the pin index (stored as i32)
                    let pin = *pin_idx as usize;
                    if pin >= DEVICE_PIN_COUNT {
                        return Err(IC10Error::ParseError {
                            line: line_num,
                            message: format!(
                                "Device pin out of range: d{} (max d{})",
                                pin,
                                DEVICE_PIN_COUNT - 1
                            ),
                        });
                    }
                }
            }
            self.program.push(parsed);
        }

        Ok(())
    }

    /// Execute one instruction
    pub fn step(&mut self) -> IC10Result<bool> {
        if self.halted {
            return Ok(false);
        }

        if self.pc >= self.program.len() {
            self.halted = true;
            return Ok(false);
        }

        let instruction = &self.program[self.pc].clone();

        match self.execute_instruction(instruction) {
            Ok(next_pc) => {
                self.pc = next_pc;
                Ok(true)
            }
            Err(e) => {
                self.error_line = Some(self.pc);
                self.halted = true;
                Err(e)
            }
        }
    }

    /// Execute multiple steps, stopping at yield, sleep, or max_steps
    pub fn run(&mut self, max_steps: usize) -> IC10Result<usize> {
        let mut steps = 0;

        while steps < max_steps {
            if self.pc >= self.program.len() {
                self.halted = true;
                break;
            }

            if self.sleep_ticks > 0 {
                self.sleep_ticks -= 1;
                return Ok(steps);
            }

            let current_instruction = self.program[self.pc].clone();

            self.step()?;
            steps += 1;

            match current_instruction.instruction {
                Instruction::Yield | Instruction::Sleep { duration: _ } => {
                    break;
                }
                _ => {}
            }
        }

        Ok(steps)
    }

    /// Execute a single instruction and return the next PC
    fn execute_instruction(&mut self, instruction: &ParsedInstruction) -> IC10Result<usize> {
        logic::execute_instruction(self, instruction)
    }

    /// Resolve a value from an operand
    pub(crate) fn resolve_value(&self, operand: &Operand) -> IC10Result<f64> {
        match operand {
            Operand::Register(idx) => self.get_register(*idx),
            Operand::Immediate(val) => Ok(*val),
            Operand::DevicePin(_) => Err(IC10Error::RuntimeError {
                line: self.pc,
                message: "Cannot use device pin as a value".to_string(),
            }),
            Operand::Alias(name) => {
                // First check defines (compile-time constants)
                if let Some(val) = self.defines.get(name) {
                    return Ok(*val);
                }
                // Then check aliases
                match self.aliases.get(name) {
                    Some(AliasTarget::Register(idx)) => self.get_register(*idx),
                    Some(AliasTarget::Device(_)) => Err(IC10Error::RuntimeError {
                        line: self.pc,
                        message: format!("Cannot use device alias '{}' as a value", name),
                    }),
                    None => {
                        // Check for labels
                        if let Some(&line) = self.labels.get(name) {
                            return Ok(line as f64);
                        }

                        // Check for default sp/ra aliases
                        match name.as_str() {
                            "sp" => self.get_register(STACK_POINTER_INDEX),
                            "ra" => self.get_register(RETURN_ADDRESS_INDEX),
                            _ => Err(IC10Error::RuntimeError {
                                line: self.pc,
                                message: format!("Undefined alias, define, or label: {}", name),
                            }),
                        }
                    }
                }
            }
        }
    }

    /// Resolve an operand to a register index (for use as a destination)
    pub(crate) fn resolve_register(&self, operand: &Operand) -> IC10Result<usize> {
        match operand {
            Operand::Register(idx) => Ok(*idx),
            Operand::Immediate(_) => Err(IC10Error::RuntimeError {
                line: self.pc,
                message: "Cannot use immediate value as a register destination".to_string(),
            }),
            Operand::DevicePin(_) => Err(IC10Error::RuntimeError {
                line: self.pc,
                message: "Cannot use device pin as a register destination".to_string(),
            }),
            Operand::Alias(name) => {
                // Check aliases first
                match self.aliases.get(name) {
                    Some(AliasTarget::Register(idx)) => Ok(*idx),
                    Some(AliasTarget::Device(_)) => Err(IC10Error::RuntimeError {
                        line: self.pc,
                        message: format!(
                            "Cannot use device alias '{}' as a register destination",
                            name
                        ),
                    }),
                    None => {
                        // Check for default sp/ra aliases
                        match name.as_str() {
                            "sp" => Ok(STACK_POINTER_INDEX),
                            "ra" => Ok(RETURN_ADDRESS_INDEX),
                            _ => Err(IC10Error::RuntimeError {
                                line: self.pc,
                                message: format!("Undefined alias: {}", name),
                            }),
                        }
                    }
                }
            }
        }
    }

    /// Resolve an operand to a device reference ID
    /// For device pins (d0-d5), looks up the reference ID stored in the housing's pin
    pub(crate) fn resolve_device_ref_id(&self, operand: &Operand) -> IC10Result<i32> {
        match operand {
            Operand::DevicePin(pin_idx) => {
                // Direct device pin access (d0-d5) - get the reference ID stored at this pin
                let housing = self.housing.borrow();
                if let Some(ref_id) = housing.get_device_pin(*pin_idx) {
                    Ok(ref_id)
                } else {
                    Err(IC10Error::RuntimeError {
                        line: self.pc,
                        message: format!("No device assigned to pin d{}", pin_idx),
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
                match self.aliases.get(name) {
                    Some(AliasTarget::Device(ref_id)) => Ok(*ref_id),
                    Some(AliasTarget::Register(idx)) => {
                        // Indirect device access - register contains device reference ID
                        let ref_id = self.get_register(*idx)? as i32;
                        Ok(ref_id)
                    }
                    None => Err(IC10Error::RuntimeError {
                        line: self.pc,
                        message: format!("Undefined device alias: {}", name),
                    }),
                }
            }
        }
    }

    /// Check if a device with the given reference ID exists on the network
    pub(crate) fn device_exists_by_id(&self, ref_id: i32) -> bool {
        let housing = self.housing.borrow();
        if let Some(network) = housing.get_network() {
            network.borrow().device_exists(ref_id)
        } else {
            // If not connected to network, only the housing itself exists
            ref_id == housing.id()
        }
    }

    /// Get a register value
    pub fn get_register(&self, index: usize) -> IC10Result<f64> {
        self.housing
            .borrow()
            .get_register(index)
            .map_err(|_| IC10Error::RegisterOutOfBounds(index))
    }

    /// Set a register value
    pub fn set_register(&self, index: usize, value: f64) -> IC10Result<()> {
        self.housing
            .borrow()
            .set_register(index, value)
            .map_err(|_| IC10Error::RegisterOutOfBounds(index))
    }

    /// Read from stack memory
    pub fn read_stack(&self, address: usize) -> IC10Result<f64> {
        self.housing
            .borrow()
            .read_stack(address)
            .map_err(|_| IC10Error::StackOutOfBounds(address))
    }

    /// Write to stack memory
    pub fn write_stack(&self, address: usize, value: f64) -> IC10Result<()> {
        self.housing
            .borrow()
            .write_stack(address, value)
            .map_err(|_| IC10Error::StackOutOfBounds(address))
    }

    /// Insert a define (compile-time constant)
    pub fn insert_define(&mut self, name: &str, value: f64) {
        self.defines.insert(name.to_string(), value);
    }

    /// Insert an alias for a register or device pin
    pub fn insert_alias(&mut self, name: &str, target: AliasTarget) {
        self.aliases.insert(name.to_string(), target);
    }

    /// Add a device alias (convenience method)
    /// Note: device_ref_id is the device's reference ID (from get_id()), not the pin index
    pub fn add_device_alias(&mut self, name: String, device_ref_id: i32) {
        self.aliases
            .insert(name, AliasTarget::Device(device_ref_id));
    }

    /// Get the current program counter
    pub fn get_pc(&self) -> usize {
        self.pc
    }

    /// Set the current program counter
    pub fn set_pc(&mut self, pc: usize) {
        self.pc = pc;
    }

    /// Check if the chip is halted
    pub fn is_halted(&self) -> bool {
        self.halted
    }

    /// Halt the chip
    pub fn halt(&mut self) {
        self.halted = true;
    }

    /// Get sleep ticks
    pub fn get_sleep_ticks(&self) -> u64 {
        self.sleep_ticks
    }

    /// Set sleep ticks
    pub fn set_sleep_ticks(&mut self, ticks: u64) {
        self.sleep_ticks = ticks;
    }

    /// Get a reference to the housing
    pub fn get_housing(&self) -> std::cell::Ref<ICHousing> {
        self.housing.borrow()
    }

    /// Get a mutable reference to the housing
    pub fn get_housing_mut(&self) -> std::cell::RefMut<ICHousing> {
        self.housing.borrow_mut()
    }

    /// Get the Rc to the housing (for when you need to clone the reference)
    pub fn get_housing_rc(&self) -> Rc<RefCell<ICHousing>> {
        self.housing.clone()
    }

    /// Get a reference to the cable network (if connected)
    pub fn get_network(&self) -> Option<Rc<RefCell<CableNetwork>>> {
        self.housing.borrow().get_network()
    }

    /// Get the housing's reference ID
    pub fn get_housing_id(&self) -> i32 {
        self.housing.borrow().id()
    }

    /// Print debug information: registers and non-zero stack values
    pub fn print_debug_info(&self) {
        let housing = self.housing.borrow();
        println!("Non-zero Registers:");
        for i in 0..REGISTER_COUNT {
            if let Ok(value) = housing.get_register(i) {
                if value != 0.0 {
                    if value.fract() == 0.0 {
                        println!("r{}: {:.0}", i, value);
                    } else {
                        println!("r{}: {:.6}", i, value);
                    }
                }
            }
        }

        println!("\nNon-zero Stack Values:");
        for i in 0..STACK_SIZE {
            if let Ok(value) = housing.read_stack(i) {
                if value != 0.0 {
                    if value.fract() == 0.0 {
                        println!("stack[{}]: {:.0}", i, value);
                    } else {
                        println!("stack[{}]: {:.6}", i, value);
                    }
                }
            }
        }
    }

    /// Check if the script is over
    pub fn is_script_over(&self) -> bool {
        self.halted || self.pc >= self.program.len()
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
