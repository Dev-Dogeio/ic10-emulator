use crate::cable_network::BatchMode;
use crate::chip::{AliasTarget, Operand};
use crate::constants::REGISTER_COUNT;
use crate::constants::{RETURN_ADDRESS_INDEX, STACK_POINTER_INDEX};
use crate::devices::LogicType;
use crate::error::{IC10Error, IC10Result};

/// All IC10 instructions
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // ==================== Data Movement ====================
    /// move r, a - r = a
    Move { dest: Operand, arg: Operand },
    /// alias name target - Create alias for register/device
    Alias { name: String, target: AliasTarget },
    /// define name value - Define compile-time constant
    Define { name: String, value: f64 },

    // ==================== Arithmetic Operations ====================
    /// add r, a, b - r = a + b
    Add {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// sub r, a, b - r = a - b
    Sub {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// mul r, a, b - r = a * b
    Mul {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// div r, a, b - r = a / b
    Div {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// mod r, a, b - r = a mod b (adjusts negative results)
    Mod {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// sqrt r, a - r = sqrt(a)
    Sqrt { dest: Operand, arg: Operand },
    /// abs r, a - r = |a|
    Abs { dest: Operand, arg: Operand },
    /// exp r, a - r = e^a
    Exp { dest: Operand, arg: Operand },
    /// log r, a - r = ln(a)
    Log { dest: Operand, arg: Operand },
    /// pow r, a, b - r = a^b
    Pow {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// max r, a, b - r = max(a, b)
    Max {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// min r, a, b - r = min(a, b)
    Min {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// ceil r, a - r = ceil(a)
    Ceil { dest: Operand, arg: Operand },
    /// floor r, a - r = floor(a)
    Floor { dest: Operand, arg: Operand },
    /// round r, a - r = round(a)
    Round { dest: Operand, arg: Operand },
    /// trunc r, a - r = trunc(a)
    Trunc { dest: Operand, arg: Operand },
    /// rand r - r = random [0, 1)
    Rand { dest: Operand },
    /// lerp r, a, b, c - r = a + (b-a) * c
    Lerp {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
        arg3: Operand,
    },

    // ==================== Trigonometric Operations ====================
    /// sin r, a - r = sin(a) (radians)
    Sin { dest: Operand, arg: Operand },
    /// cos r, a - r = cos(a) (radians)
    Cos { dest: Operand, arg: Operand },
    /// tan r, a - r = tan(a) (radians)
    Tan { dest: Operand, arg: Operand },
    /// asin r, a - r = arcsin(a)
    Asin { dest: Operand, arg: Operand },
    /// acos r, a - r = arccos(a)
    Acos { dest: Operand, arg: Operand },
    /// atan r, a - r = arctan(a)
    Atan { dest: Operand, arg: Operand },
    /// atan2 r, y, x - r = arctan(y/x)
    Atan2 {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },

    // ==================== Bitwise Operations ====================
    /// and r, a, b - r = a AND b
    And {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// or r, a, b - r = a OR b
    Or {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// xor r, a, b - r = a XOR b
    Xor {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// nor r, a, b - r = NOT(a OR b)
    Nor {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// not r, a - r = NOT a
    Not { dest: Operand, arg: Operand },

    // ==================== Bit Shifting Operations ====================
    /// sll r, a, b - Shift left logical
    Sll {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// sla r, a, b - Shift left arithmetic (same as sll)
    Sla {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// srl r, a, b - Shift right logical (unsigned)
    Srl {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// sra r, a, b - Shift right arithmetic (signed)
    Sra {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },

    // ==================== Bit Field Operations ====================
    /// ext r, source, start, length - Extract bit field from source
    Ext {
        dest: Operand,
        source: Operand,
        start: Operand,
        length: Operand,
    },
    /// ins r, start, length, value - Insert value into bit field at start position for length bits
    Ins {
        dest: Operand,
        start: Operand,
        length: Operand,
        value: Operand,
    },

    // ==================== Comparison Operations (Set Instructions) ====================
    /// slt r, a, b - Set if a < b
    Slt {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// sgt r, a, b - Set if a > b
    Sgt {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// sle r, a, b - Set if a <= b
    Sle {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// sge r, a, b - Set if a >= b
    Sge {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// seq r, a, b - Set if a == b
    Seq {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// sne r, a, b - Set if a != b
    Sne {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// sltz r, a - Set if a < 0
    Sltz { dest: Operand, arg: Operand },
    /// sgtz r, a - Set if a > 0
    Sgtz { dest: Operand, arg: Operand },
    /// slez r, a - Set if a <= 0
    Slez { dest: Operand, arg: Operand },
    /// sgez r, a - Set if a >= 0
    Sgez { dest: Operand, arg: Operand },
    /// seqz r, a - Set if a == 0
    Seqz { dest: Operand, arg: Operand },
    /// snez r, a - Set if a != 0
    Snez { dest: Operand, arg: Operand },
    /// snan r, a - Set if isNaN(a)
    Snan { dest: Operand, arg: Operand },
    /// snanz r, a - Set if !isNaN(a)
    Snanz { dest: Operand, arg: Operand },

    // ==================== Approximate Comparison ====================
    /// sap r, a, b, c - Set if a ≈ b (within c relative tolerance)
    Sap {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
        arg3: Operand,
    },
    /// sna r, a, b, c - Set if a !≈ b
    Sna {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
        arg3: Operand,
    },
    /// sapz r, a, c - Set if a ≈ 0
    Sapz {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// snaz r, a, c - Set if a !≈ 0
    Snaz {
        dest: Operand,
        arg1: Operand,
        arg2: Operand,
    },

    // ==================== Device State Detection ====================
    /// sdse r, d - Set if device exists
    Sdse { dest: Operand, device: Operand },
    /// sdns r, d - Set if device not set
    Sdns { dest: Operand, device: Operand },

    // ==================== Branch Instructions (Absolute) ====================
    /// beq a, b, line - Branch if a == b
    Beq {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bne a, b, line - Branch if a != b
    Bne {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// blt a, b, line - Branch if a < b
    Blt {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bgt a, b, line - Branch if a > b
    Bgt {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// ble a, b, line - Branch if a <= b
    Ble {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bge a, b, line - Branch if a >= b
    Bge {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// beqz a, line - Branch if a == 0
    Beqz { arg: Operand, line: Operand },
    /// bnez a, line - Branch if a != 0
    Bnez { arg: Operand, line: Operand },
    /// bltz a, line - Branch if a < 0
    Bltz { arg: Operand, line: Operand },
    /// bgez a, line - Branch if a >= 0
    Bgez { arg: Operand, line: Operand },
    /// blez a, line - Branch if a <= 0
    Blez { arg: Operand, line: Operand },
    /// bgtz a, line - Branch if a > 0
    Bgtz { arg: Operand, line: Operand },
    /// bnan a, line - Branch if isNaN(a)
    Bnan { arg: Operand, line: Operand },

    // ==================== Branch Instructions (Relative) ====================
    /// breq a, b, offset - Branch relative if a == b
    Breq {
        arg1: Operand,
        arg2: Operand,
        offset: Operand,
    },
    /// brne a, b, offset - Branch relative if a != b
    Brne {
        arg1: Operand,
        arg2: Operand,
        offset: Operand,
    },
    /// brlt a, b, offset - Branch relative if a < b
    Brlt {
        arg1: Operand,
        arg2: Operand,
        offset: Operand,
    },
    /// brgt a, b, offset - Branch relative if a > b
    Brgt {
        arg1: Operand,
        arg2: Operand,
        offset: Operand,
    },
    /// brle a, b, offset - Branch relative if a <= b
    Brle {
        arg1: Operand,
        arg2: Operand,
        offset: Operand,
    },
    /// brge a, b, offset - Branch relative if a >= b
    Brge {
        arg1: Operand,
        arg2: Operand,
        offset: Operand,
    },
    /// breqz a, offset - Branch relative if a == 0
    Breqz { arg: Operand, offset: Operand },
    /// brnez a, offset - Branch relative if a != 0
    Brnez { arg: Operand, offset: Operand },
    /// brltz a, offset - Branch relative if a < 0
    Brltz { arg: Operand, offset: Operand },
    /// brgez a, offset - Branch relative if a >= 0
    Brgez { arg: Operand, offset: Operand },
    /// brlez a, offset - Branch relative if a <= 0
    Brlez { arg: Operand, offset: Operand },
    /// brgtz a, offset - Branch relative if a > 0
    Brgtz { arg: Operand, offset: Operand },
    /// brnan a, offset - Branch relative if isNaN(a)
    Brnan { arg: Operand, offset: Operand },

    // ==================== Branch and Link Variants ====================
    /// beqal a, b, line - Branch and link if a == b
    Beqal {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bneal a, b, line - Branch and link if a != b
    Bneal {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bltal a, b, line - Branch and link if a < b
    Bltal {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bgtal a, b, line - Branch and link if a > b
    Bgtal {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bleal a, b, line - Branch and link if a <= b
    Bleal {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bgeal a, b, line - Branch and link if a >= b
    Bgeal {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// beqzal a, line - Branch and link if a == 0
    Beqzal { arg: Operand, line: Operand },
    /// bnezal a, line - Branch and link if a != 0
    Bnezal { arg: Operand, line: Operand },
    /// bltzal a, line - Branch and link if a < 0
    Bltzal { arg: Operand, line: Operand },
    /// bgezal a, line - Branch and link if a >= 0
    Bgezal { arg: Operand, line: Operand },
    /// blezal a, line - Branch and link if a <= 0
    Blezal { arg: Operand, line: Operand },
    /// bgtzal a, line - Branch and link if a > 0
    Bgtzal { arg: Operand, line: Operand },

    // ==================== Approximate Branches ====================
    /// bap a, b, c, line - Branch if approximately equal
    Bap {
        arg1: Operand,
        arg2: Operand,
        arg3: Operand,
        line: Operand,
    },
    /// bna a, b, c, line - Branch if not approximately equal
    Bna {
        arg1: Operand,
        arg2: Operand,
        arg3: Operand,
        line: Operand,
    },
    /// brap a, b, c, offset - Branch relative if approximately equal
    Brap {
        arg1: Operand,
        arg2: Operand,
        arg3: Operand,
        offset: Operand,
    },
    /// brna a, b, c, offset - Branch relative if not approximately equal
    Brna {
        arg1: Operand,
        arg2: Operand,
        arg3: Operand,
        offset: Operand,
    },
    /// bapz a, c, line - Branch if approximately zero
    Bapz {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bnaz a, c, line - Branch if not approximately zero
    Bnaz {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// brapz a, c, offset - Branch relative if approximately zero
    Brapz {
        arg1: Operand,
        arg2: Operand,
        offset: Operand,
    },
    /// brnaz a, c, offset - Branch relative if not approximately zero
    Brnaz {
        arg1: Operand,
        arg2: Operand,
        offset: Operand,
    },
    /// bapal a, b, c, line - Branch and link if approximately equal
    Bapal {
        arg1: Operand,
        arg2: Operand,
        arg3: Operand,
        line: Operand,
    },
    /// bnaal a, b, c, line - Branch and link if not approximately equal
    Bnaal {
        arg1: Operand,
        arg2: Operand,
        arg3: Operand,
        line: Operand,
    },
    /// bapzal a, c, line - Branch and link if approximately zero
    Bapzal {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },
    /// bnazal a, c, line - Branch and link if not approximately zero
    Bnazal {
        arg1: Operand,
        arg2: Operand,
        line: Operand,
    },

    // ==================== Device State Branches ====================
    /// bdse d, line - Branch if device set/exists
    Bdse { device: Operand, line: Operand },
    /// bdns d, line - Branch if device not set
    Bdns { device: Operand, line: Operand },
    /// brdse d, offset - Branch relative if device set
    Brdse { device: Operand, offset: Operand },
    /// brdns d, offset - Branch relative if device not set
    Brdns { device: Operand, offset: Operand },
    /// bdseal d, line - Branch and link if device set
    Bdseal { device: Operand, line: Operand },
    /// bdnsal d, line - Branch and link if device not set
    Bdnsal { device: Operand, line: Operand },
    /// bdnvl d, logicType, line - Branch if device not valid for load of logicType
    Bdnvl {
        device: Operand,
        logic_type: Operand,
        line: Operand,
    },
    /// bdnvs d, logicType, line - Branch if device not valid for store of logicType
    Bdnvs {
        device: Operand,
        logic_type: Operand,
        line: Operand,
    },

    // ==================== Jump Instructions ====================
    /// j line - Unconditional jump (absolute)
    J { line: Operand },
    /// jr offset - Jump relative
    Jr { offset: Operand },
    /// jal line - Jump and link
    Jal { line: Operand },

    // ==================== Stack Operations ====================
    /// push a - Push value onto stack
    Push { arg: Operand },
    /// pop r - Pop value from stack into register
    Pop { dest: Operand },
    /// peek r - Read top of stack without popping
    Peek { dest: Operand },
    /// poke index, value - Direct stack memory write
    Poke { index: Operand, value: Operand },

    // ==================== Device I/O Instructions ====================
    /// l r, d, logicType - Load device property into register
    L {
        dest: Operand,
        device: Operand,
        logic_type: Operand,
    },
    /// s d, logicType, value - Set device property
    S {
        device: Operand,
        logic_type: Operand,
        value: Operand,
    },
    /// ls r, d, slotIndex, slotLogicType - Load slot property
    Ls {
        dest: Operand,
        device: Operand,
        slot_index: Operand,
        slot_logic_type: Operand,
    },
    /// ss d, slotIndex, slotLogicType, value - Set slot property
    Ss {
        device: Operand,
        slot_index: Operand,
        slot_logic_type: Operand,
        value: Operand,
    },
    /// lr r, d, reagentMode, reagent - Load reagent property
    Lr {
        dest: Operand,
        device: Operand,
        reagent_mode: Operand,
        reagent: Operand,
    },
    /// rmap r, d, reagentHash - Get prefab hash from reagent hash
    Rmap {
        dest: Operand,
        device: Operand,
        reagent_hash: Operand,
    },

    // ==================== ID-Based Device Access ====================
    /// ld r, id, logicType - Load from device by reference ID
    Ld {
        dest: Operand,
        id: Operand,
        logic_type: Operand,
    },
    /// sd id, logicType, value - Set device by reference ID
    Sd {
        id: Operand,
        logic_type: Operand,
        value: Operand,
    },

    // ==================== Batch Device Access ====================
    /// lb r, deviceHash, logicType, batchMode - Load batch value
    Lb {
        dest: Operand,
        device_hash: Operand,
        logic_type: Operand,
        batch_mode: Operand,
    },
    /// sb deviceHash, logicType, value - Set all matching devices
    Sb {
        device_hash: Operand,
        logic_type: Operand,
        value: Operand,
    },
    /// lbn r, deviceHash, nameHash, logicType, batchMode - Load batch by name
    Lbn {
        dest: Operand,
        device_hash: Operand,
        name_hash: Operand,
        logic_type: Operand,
        batch_mode: Operand,
    },
    /// sbn deviceHash, nameHash, logicType, value - Set batch by name
    Sbn {
        device_hash: Operand,
        name_hash: Operand,
        logic_type: Operand,
        value: Operand,
    },
    /// lbs r, deviceHash, slotIndex, slotLogicType, batchMode - Load batch slot
    Lbs {
        dest: Operand,
        device_hash: Operand,
        slot_index: Operand,
        slot_logic_type: Operand,
        batch_mode: Operand,
    },
    /// sbs deviceHash, slotIndex, slotLogicType, value - Set batch slot
    Sbs {
        device_hash: Operand,
        slot_index: Operand,
        slot_logic_type: Operand,
        value: Operand,
    },
    /// lbns r, deviceHash, nameHash, slotIndex, slotLogicType, batchMode - Load batch named slot
    Lbns {
        dest: Operand,
        device_hash: Operand,
        name_hash: Operand,
        slot_index: Operand,
        slot_logic_type: Operand,
        batch_mode: Operand,
    },

    // ==================== Memory Access Instructions ====================
    /// get r, d, stackIndex - Read from device's memory
    Get {
        dest: Operand,
        device: Operand,
        stack_index: Operand,
    },
    /// put r, d, stackIndex - Write to device's memory
    Put {
        value: Operand,
        device: Operand,
        stack_index: Operand,
    },
    /// getd r, id, stackIndex - Read from device memory by ID
    Getd {
        dest: Operand,
        id: Operand,
        stack_index: Operand,
    },
    /// putd r, id, stackIndex - Write to device memory by ID
    Putd {
        value: Operand,
        id: Operand,
        stack_index: Operand,
    },

    // ==================== Special Instructions ====================
    /// yield - End current execution cycle
    Yield,
    /// sleep seconds - Pause execution for duration
    Sleep { duration: Operand },
    /// hcf - Halt and Catch Fire (hard stop)
    Hcf,
    /// select r, cond, a, b - r = cond ? a : b
    Select {
        dest: Operand,
        cond: Operand,
        arg1: Operand,
        arg2: Operand,
    },
    /// clr d - Clear device memory
    Clr { device: Operand },
    /// clrd id - Clear device memory by ID
    Clrd { id: Operand },

    // ==================== No Operation ====================
    /// Empty line or comment
    Noop,
}

// Parse a destination operand (register or alias like sp/ra)
fn parse_dest_operand(token: &str) -> Operand {
    // Try to parse as a register first
    if let Some(stripped) = token.strip_prefix('r')
        && let Ok(idx) = stripped.parse::<usize>()
        && idx < REGISTER_COUNT
    {
        return Operand::Register(idx);
    }
    // Otherwise treat as an alias (including sp/ra)
    Operand::Alias(token.to_string())
}

fn parse_operand(token: &str) -> Operand {
    // Check for special register labels
    if token == "sp" || token == "ra" {
        return Operand::Alias(token.to_string());
    }

    // Check for device pins (d0, d1, d2, etc.)
    if let Some(stripped) = token.strip_prefix('d')
        && let Ok(idx) = stripped.parse::<usize>()
    {
        return Operand::DevicePin(idx);
    }

    if let Some(stripped) = token.strip_prefix('r')
        && let Ok(idx) = stripped.parse::<usize>()
        && idx < REGISTER_COUNT
    {
        return Operand::Register(idx);
    }
    if let Ok(val) = token.parse::<f64>() {
        return Operand::Immediate(val);
    }
    Operand::Alias(token.to_string())
}

/// Parse an operand that could be a LogicType name
fn parse_logic_type_operand(token: &str) -> Operand {
    // Check if this token matches a LogicType name
    if let Some(logic_type) = LogicType::from_name(token) {
        return Operand::Immediate(logic_type.to_value());
    }
    parse_operand(token)
}

/// Parse an operand that could be a BatchMode name
fn parse_batch_mode_operand(token: &str) -> Operand {
    // Check if this token matches a BatchMode name
    if let Some(batch_mode) = BatchMode::from_name(token) {
        return Operand::Immediate(batch_mode.to_value());
    }
    parse_operand(token)
}

fn parse_alias_target(token: &str) -> IC10Result<AliasTarget> {
    if token == "sp" {
        return Ok(AliasTarget::Register(STACK_POINTER_INDEX));
    }
    if token == "ra" {
        return Ok(AliasTarget::Register(RETURN_ADDRESS_INDEX));
    }

    if let Some(stripped) = token.strip_prefix('r') {
        let idx = stripped
            .parse::<usize>()
            .map_err(|_| IC10Error::ParseError {
                line: 0,
                message: format!("Invalid register for alias: {token}"),
            })?;
        if idx >= REGISTER_COUNT {
            return Err(IC10Error::ParseError {
                line: 0,
                message: format!("Register index out of range (r0-r17): {token}"),
            });
        }
        Ok(AliasTarget::Register(idx))
    } else if let Some(stripped) = token.strip_prefix('d') {
        let idx = stripped
            .parse::<usize>()
            .map_err(|_| IC10Error::ParseError {
                line: 0,
                message: format!("Invalid device for alias: {token}"),
            })?;
        // Store as i32 (will be interpreted as pin index during execution and resolved to ref ID)
        Ok(AliasTarget::Device(idx as i32))
    } else {
        Err(IC10Error::ParseError {
            line: 0,
            message: format!("Invalid alias target: {token}"),
        })
    }
}

/// A parsed instruction with metadata
#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    pub instruction: Instruction,
    pub line_number: usize,
    pub original_line: String,
}

impl ParsedInstruction {
    pub fn parse(line: &str, line_number: usize) -> IC10Result<Self> {
        let original_line = line.to_string();
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.ends_with(':') {
            return Ok(ParsedInstruction {
                instruction: Instruction::Noop,
                line_number,
                original_line,
            });
        }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            return Ok(ParsedInstruction {
                instruction: Instruction::Noop,
                line_number,
                original_line,
            });
        }

        match tokens[0].to_lowercase().as_str() {
            // ==================== Data Movement ====================
            "move" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "move".to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg = parse_operand(tokens[2]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Move { dest, arg },
                    line_number,
                    original_line,
                })
            }
            "alias" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "alias".to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let name = tokens[1].to_string();
                let target = parse_alias_target(tokens[2])?;
                Ok(ParsedInstruction {
                    instruction: Instruction::Alias { name, target },
                    line_number,
                    original_line,
                })
            }
            "define" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "define".to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let name = tokens[1].to_string();
                let value = tokens[2]
                    .parse::<f64>()
                    .map_err(|_| IC10Error::ParseError {
                        line: line_number,
                        message: format!("Invalid value for define: {}", tokens[2]),
                    })?;
                Ok(ParsedInstruction {
                    instruction: Instruction::Define { name, value },
                    line_number,
                    original_line,
                })
            }

            // ==================== Arithmetic Operations ====================
            "add" | "sub" | "mul" | "div" | "mod" | "pow" | "max" | "min" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg1 = parse_operand(tokens[2]);
                let arg2 = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "add" => Instruction::Add { dest, arg1, arg2 },
                    "sub" => Instruction::Sub { dest, arg1, arg2 },
                    "mul" => Instruction::Mul { dest, arg1, arg2 },
                    "div" => Instruction::Div { dest, arg1, arg2 },
                    "mod" => Instruction::Mod { dest, arg1, arg2 },
                    "pow" => Instruction::Pow { dest, arg1, arg2 },
                    "max" => Instruction::Max { dest, arg1, arg2 },
                    "min" => Instruction::Min { dest, arg1, arg2 },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "sqrt" | "abs" | "exp" | "log" | "ceil" | "floor" | "round" | "trunc" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "sqrt" => Instruction::Sqrt { dest, arg },
                    "abs" => Instruction::Abs { dest, arg },
                    "exp" => Instruction::Exp { dest, arg },
                    "log" => Instruction::Log { dest, arg },
                    "ceil" => Instruction::Ceil { dest, arg },
                    "floor" => Instruction::Floor { dest, arg },
                    "round" => Instruction::Round { dest, arg },
                    "trunc" => Instruction::Trunc { dest, arg },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "rand" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "rand".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Rand { dest },
                    line_number,
                    original_line,
                })
            }
            "lerp" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "lerp".to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg1 = parse_operand(tokens[2]);
                let arg2 = parse_operand(tokens[3]);
                let arg3 = parse_operand(tokens[4]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Lerp {
                        dest,
                        arg1,
                        arg2,
                        arg3,
                    },
                    line_number,
                    original_line,
                })
            }

            // ==================== Trigonometric Operations ====================
            "sin" | "cos" | "tan" | "asin" | "acos" | "atan" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "sin" => Instruction::Sin { dest, arg },
                    "cos" => Instruction::Cos { dest, arg },
                    "tan" => Instruction::Tan { dest, arg },
                    "asin" => Instruction::Asin { dest, arg },
                    "acos" => Instruction::Acos { dest, arg },
                    "atan" => Instruction::Atan { dest, arg },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "atan2" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "atan2".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg1 = parse_operand(tokens[2]);
                let arg2 = parse_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Atan2 { dest, arg1, arg2 },
                    line_number,
                    original_line,
                })
            }

            // ==================== Bitwise Operations ====================
            "and" | "or" | "xor" | "nor" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg1 = parse_operand(tokens[2]);
                let arg2 = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "and" => Instruction::And { dest, arg1, arg2 },
                    "or" => Instruction::Or { dest, arg1, arg2 },
                    "xor" => Instruction::Xor { dest, arg1, arg2 },
                    "nor" => Instruction::Nor { dest, arg1, arg2 },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "not" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "not".to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg = parse_operand(tokens[2]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Not { dest, arg },
                    line_number,
                    original_line,
                })
            }

            // ==================== Bit Shifting Operations ====================
            "sll" | "sla" | "srl" | "sra" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg1 = parse_operand(tokens[2]);
                let arg2 = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "sll" => Instruction::Sll { dest, arg1, arg2 },
                    "sla" => Instruction::Sla { dest, arg1, arg2 },
                    "srl" => Instruction::Srl { dest, arg1, arg2 },
                    "sra" => Instruction::Sra { dest, arg1, arg2 },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Bit Field Operations ====================
            "ext" | "ins" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg1 = parse_operand(tokens[2]);
                let arg2 = parse_operand(tokens[3]);
                let arg3 = parse_operand(tokens[4]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "ext" => Instruction::Ext {
                        dest,
                        source: arg1,
                        start: arg2,
                        length: arg3,
                    },
                    "ins" => Instruction::Ins {
                        dest,
                        start: arg1,
                        length: arg2,
                        value: arg3,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Comparison Operations (Set Instructions) ====================
            "slt" | "sgt" | "sle" | "sge" | "seq" | "sne" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg1 = parse_operand(tokens[2]);
                let arg2 = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "slt" => Instruction::Slt { dest, arg1, arg2 },
                    "sgt" => Instruction::Sgt { dest, arg1, arg2 },
                    "sle" => Instruction::Sle { dest, arg1, arg2 },
                    "sge" => Instruction::Sge { dest, arg1, arg2 },
                    "seq" => Instruction::Seq { dest, arg1, arg2 },
                    "sne" => Instruction::Sne { dest, arg1, arg2 },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "sltz" | "sgtz" | "slez" | "sgez" | "seqz" | "snez" | "snan" | "snanz" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "sltz" => Instruction::Sltz { dest, arg },
                    "sgtz" => Instruction::Sgtz { dest, arg },
                    "slez" => Instruction::Slez { dest, arg },
                    "sgez" => Instruction::Sgez { dest, arg },
                    "seqz" => Instruction::Seqz { dest, arg },
                    "snez" => Instruction::Snez { dest, arg },
                    "snan" => Instruction::Snan { dest, arg },
                    "snanz" => Instruction::Snanz { dest, arg },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Approximate Comparison ====================
            "sap" | "sna" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg1 = parse_operand(tokens[2]);
                let arg2 = parse_operand(tokens[3]);
                let arg3 = parse_operand(tokens[4]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "sap" => Instruction::Sap {
                        dest,
                        arg1,
                        arg2,
                        arg3,
                    },
                    "sna" => Instruction::Sna {
                        dest,
                        arg1,
                        arg2,
                        arg3,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "sapz" | "snaz" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let arg1 = parse_operand(tokens[2]);
                let arg2 = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "sapz" => Instruction::Sapz { dest, arg1, arg2 },
                    "snaz" => Instruction::Snaz { dest, arg1, arg2 },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Device State Detection ====================
            "sdse" | "sdns" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "sdse" => Instruction::Sdse { dest, device },
                    "sdns" => Instruction::Sdns { dest, device },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Branch Instructions (Absolute) ====================
            "beq" | "bne" | "blt" | "bgt" | "ble" | "bge" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let arg1 = parse_operand(tokens[1]);
                let arg2 = parse_operand(tokens[2]);
                let line_op = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "beq" => Instruction::Beq {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bne" => Instruction::Bne {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "blt" => Instruction::Blt {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bgt" => Instruction::Bgt {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "ble" => Instruction::Ble {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bge" => Instruction::Bge {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "beqz" | "bnez" | "bltz" | "bgez" | "blez" | "bgtz" | "bnan" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let arg = parse_operand(tokens[1]);
                let line_op = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "beqz" => Instruction::Beqz { arg, line: line_op },
                    "bnez" => Instruction::Bnez { arg, line: line_op },
                    "bltz" => Instruction::Bltz { arg, line: line_op },
                    "bgez" => Instruction::Bgez { arg, line: line_op },
                    "blez" => Instruction::Blez { arg, line: line_op },
                    "bgtz" => Instruction::Bgtz { arg, line: line_op },
                    "bnan" => Instruction::Bnan { arg, line: line_op },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Branch Instructions (Relative) ====================
            "breq" | "brne" | "brlt" | "brgt" | "brle" | "brge" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let arg1 = parse_operand(tokens[1]);
                let arg2 = parse_operand(tokens[2]);
                let offset = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "breq" => Instruction::Breq { arg1, arg2, offset },
                    "brne" => Instruction::Brne { arg1, arg2, offset },
                    "brlt" => Instruction::Brlt { arg1, arg2, offset },
                    "brgt" => Instruction::Brgt { arg1, arg2, offset },
                    "brle" => Instruction::Brle { arg1, arg2, offset },
                    "brge" => Instruction::Brge { arg1, arg2, offset },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "breqz" | "brnez" | "brltz" | "brgez" | "brlez" | "brgtz" | "brnan" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let arg = parse_operand(tokens[1]);
                let offset = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "breqz" => Instruction::Breqz { arg, offset },
                    "brnez" => Instruction::Brnez { arg, offset },
                    "brltz" => Instruction::Brltz { arg, offset },
                    "brgez" => Instruction::Brgez { arg, offset },
                    "brlez" => Instruction::Brlez { arg, offset },
                    "brgtz" => Instruction::Brgtz { arg, offset },
                    "brnan" => Instruction::Brnan { arg, offset },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Branch and Link Variants ====================
            "beqal" | "bneal" | "bltal" | "bgtal" | "bleal" | "bgeal" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let arg1 = parse_operand(tokens[1]);
                let arg2 = parse_operand(tokens[2]);
                let line_op = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "beqal" => Instruction::Beqal {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bneal" => Instruction::Bneal {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bltal" => Instruction::Bltal {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bgtal" => Instruction::Bgtal {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bleal" => Instruction::Bleal {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bgeal" => Instruction::Bgeal {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "beqzal" | "bnezal" | "bltzal" | "bgezal" | "blezal" | "bgtzal" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let arg = parse_operand(tokens[1]);
                let line_op = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "beqzal" => Instruction::Beqzal { arg, line: line_op },
                    "bnezal" => Instruction::Bnezal { arg, line: line_op },
                    "bltzal" => Instruction::Bltzal { arg, line: line_op },
                    "bgezal" => Instruction::Bgezal { arg, line: line_op },
                    "blezal" => Instruction::Blezal { arg, line: line_op },
                    "bgtzal" => Instruction::Bgtzal { arg, line: line_op },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Approximate Branches ====================
            "bap" | "bna" | "bapal" | "bnaal" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let arg1 = parse_operand(tokens[1]);
                let arg2 = parse_operand(tokens[2]);
                let arg3 = parse_operand(tokens[3]);
                let line_op = parse_operand(tokens[4]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "bap" => Instruction::Bap {
                        arg1,
                        arg2,
                        arg3,
                        line: line_op,
                    },
                    "bna" => Instruction::Bna {
                        arg1,
                        arg2,
                        arg3,
                        line: line_op,
                    },
                    "bapal" => Instruction::Bapal {
                        arg1,
                        arg2,
                        arg3,
                        line: line_op,
                    },
                    "bnaal" => Instruction::Bnaal {
                        arg1,
                        arg2,
                        arg3,
                        line: line_op,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "brap" | "brna" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let arg1 = parse_operand(tokens[1]);
                let arg2 = parse_operand(tokens[2]);
                let arg3 = parse_operand(tokens[3]);
                let offset = parse_operand(tokens[4]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "brap" => Instruction::Brap {
                        arg1,
                        arg2,
                        arg3,
                        offset,
                    },
                    "brna" => Instruction::Brna {
                        arg1,
                        arg2,
                        arg3,
                        offset,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "bapz" | "bnaz" | "bapzal" | "bnazal" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let arg1 = parse_operand(tokens[1]);
                let arg2 = parse_operand(tokens[2]);
                let line_op = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "bapz" => Instruction::Bapz {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bnaz" => Instruction::Bnaz {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bapzal" => Instruction::Bapzal {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    "bnazal" => Instruction::Bnazal {
                        arg1,
                        arg2,
                        line: line_op,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "brapz" | "brnaz" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let arg1 = parse_operand(tokens[1]);
                let arg2 = parse_operand(tokens[2]);
                let offset = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "brapz" => Instruction::Brapz { arg1, arg2, offset },
                    "brnaz" => Instruction::Brnaz { arg1, arg2, offset },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Device State Branches ====================
            "bdse" | "bdns" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let device = parse_operand(tokens[1]);
                let line_op = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "bdse" => Instruction::Bdse {
                        device,
                        line: line_op,
                    },
                    "bdns" => Instruction::Bdns {
                        device,
                        line: line_op,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "brdse" | "brdns" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let device = parse_operand(tokens[1]);
                let offset = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "brdse" => Instruction::Brdse { device, offset },
                    "brdns" => Instruction::Brdns { device, offset },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "bdseal" | "bdnsal" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let device = parse_operand(tokens[1]);
                let line_op = parse_operand(tokens[2]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "bdseal" => Instruction::Bdseal {
                        device,
                        line: line_op,
                    },
                    "bdnsal" => Instruction::Bdnsal {
                        device,
                        line: line_op,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }
            "bdnvl" | "bdnvs" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: tokens[0].to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let device = parse_operand(tokens[1]);
                let logic_type = parse_logic_type_operand(tokens[2]);
                let line_op = parse_operand(tokens[3]);
                let instruction = match tokens[0].to_lowercase().as_str() {
                    "bdnvl" => Instruction::Bdnvl {
                        device,
                        logic_type,
                        line: line_op,
                    },
                    "bdnvs" => Instruction::Bdnvs {
                        device,
                        logic_type,
                        line: line_op,
                    },
                    _ => unreachable!(),
                };
                Ok(ParsedInstruction {
                    instruction,
                    line_number,
                    original_line,
                })
            }

            // ==================== Jump Instructions ====================
            "j" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "j".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let line_op = parse_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::J { line: line_op },
                    line_number,
                    original_line,
                })
            }
            "jr" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "jr".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let offset = parse_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Jr { offset },
                    line_number,
                    original_line,
                })
            }
            "jal" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "jal".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let line_op = parse_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Jal { line: line_op },
                    line_number,
                    original_line,
                })
            }

            // ==================== Stack Operations ====================
            "push" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "push".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let arg = parse_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Push { arg },
                    line_number,
                    original_line,
                })
            }
            "pop" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "pop".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Pop { dest },
                    line_number,
                    original_line,
                })
            }
            "peek" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "peek".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Peek { dest },
                    line_number,
                    original_line,
                })
            }
            "poke" => {
                if tokens.len() != 3 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "poke".to_string(),
                        expected: 2,
                        actual: tokens.len() - 1,
                    });
                }
                let index = parse_operand(tokens[1]);
                let value = parse_operand(tokens[2]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Poke { index, value },
                    line_number,
                    original_line,
                })
            }

            // ==================== Device I/O Instructions ====================
            "l" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "l".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device = parse_operand(tokens[2]);
                let logic_type = parse_logic_type_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::L {
                        dest,
                        device,
                        logic_type,
                    },
                    line_number,
                    original_line,
                })
            }
            "s" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "s".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let device = parse_operand(tokens[1]);
                let logic_type = parse_logic_type_operand(tokens[2]);
                let value = parse_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::S {
                        device,
                        logic_type,
                        value,
                    },
                    line_number,
                    original_line,
                })
            }
            "ls" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "ls".to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device = parse_operand(tokens[2]);
                let slot_index = parse_operand(tokens[3]);
                let slot_logic_type = parse_operand(tokens[4]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Ls {
                        dest,
                        device,
                        slot_index,
                        slot_logic_type,
                    },
                    line_number,
                    original_line,
                })
            }
            "ss" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "ss".to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let device = parse_operand(tokens[1]);
                let slot_index = parse_operand(tokens[2]);
                let slot_logic_type = parse_operand(tokens[3]);
                let value = parse_operand(tokens[4]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Ss {
                        device,
                        slot_index,
                        slot_logic_type,
                        value,
                    },
                    line_number,
                    original_line,
                })
            }
            "lr" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "lr".to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device = parse_operand(tokens[2]);
                let reagent_mode = parse_operand(tokens[3]);
                let reagent = parse_operand(tokens[4]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Lr {
                        dest,
                        device,
                        reagent_mode,
                        reagent,
                    },
                    line_number,
                    original_line,
                })
            }
            "rmap" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "rmap".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device = parse_operand(tokens[2]);
                let reagent_hash = parse_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Rmap {
                        dest,
                        device,
                        reagent_hash,
                    },
                    line_number,
                    original_line,
                })
            }

            // ==================== ID-Based Device Access ====================
            "ld" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "ld".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let id = parse_operand(tokens[2]);
                let logic_type = parse_logic_type_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Ld {
                        dest,
                        id,
                        logic_type,
                    },
                    line_number,
                    original_line,
                })
            }
            "sd" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "sd".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let id = parse_operand(tokens[1]);
                let logic_type = parse_logic_type_operand(tokens[2]);
                let value = parse_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Sd {
                        id,
                        logic_type,
                        value,
                    },
                    line_number,
                    original_line,
                })
            }

            // ==================== Batch Device Access ====================
            "lb" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "lb".to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device_hash = parse_operand(tokens[2]);
                let logic_type = parse_logic_type_operand(tokens[3]);
                let batch_mode = parse_batch_mode_operand(tokens[4]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Lb {
                        dest,
                        device_hash,
                        logic_type,
                        batch_mode,
                    },
                    line_number,
                    original_line,
                })
            }
            "sb" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "sb".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let device_hash = parse_operand(tokens[1]);
                let logic_type = parse_logic_type_operand(tokens[2]);
                let value = parse_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Sb {
                        device_hash,
                        logic_type,
                        value,
                    },
                    line_number,
                    original_line,
                })
            }
            "lbn" => {
                if tokens.len() != 6 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "lbn".to_string(),
                        expected: 5,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device_hash = parse_operand(tokens[2]);
                let name_hash = parse_operand(tokens[3]);
                let logic_type = parse_logic_type_operand(tokens[4]);
                let batch_mode = parse_batch_mode_operand(tokens[5]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Lbn {
                        dest,
                        device_hash,
                        name_hash,
                        logic_type,
                        batch_mode,
                    },
                    line_number,
                    original_line,
                })
            }
            "sbn" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "sbn".to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let device_hash = parse_operand(tokens[1]);
                let name_hash = parse_operand(tokens[2]);
                let logic_type = parse_logic_type_operand(tokens[3]);
                let value = parse_operand(tokens[4]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Sbn {
                        device_hash,
                        name_hash,
                        logic_type,
                        value,
                    },
                    line_number,
                    original_line,
                })
            }
            "lbs" => {
                if tokens.len() != 6 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "lbs".to_string(),
                        expected: 5,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device_hash = parse_operand(tokens[2]);
                let slot_index = parse_operand(tokens[3]);
                let slot_logic_type = parse_operand(tokens[4]);
                let batch_mode = parse_operand(tokens[5]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Lbs {
                        dest,
                        device_hash,
                        slot_index,
                        slot_logic_type,
                        batch_mode,
                    },
                    line_number,
                    original_line,
                })
            }
            "sbs" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "sbs".to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let device_hash = parse_operand(tokens[1]);
                let slot_index = parse_operand(tokens[2]);
                let slot_logic_type = parse_operand(tokens[3]);
                let value = parse_operand(tokens[4]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Sbs {
                        device_hash,
                        slot_index,
                        slot_logic_type,
                        value,
                    },
                    line_number,
                    original_line,
                })
            }
            "lbns" => {
                if tokens.len() != 7 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "lbns".to_string(),
                        expected: 6,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device_hash = parse_operand(tokens[2]);
                let name_hash = parse_operand(tokens[3]);
                let slot_index = parse_operand(tokens[4]);
                let slot_logic_type = parse_operand(tokens[5]);
                let batch_mode = parse_operand(tokens[6]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Lbns {
                        dest,
                        device_hash,
                        name_hash,
                        slot_index,
                        slot_logic_type,
                        batch_mode,
                    },
                    line_number,
                    original_line,
                })
            }

            // ==================== Memory Access Instructions ====================
            "get" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "get".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let device = parse_operand(tokens[2]);
                let stack_index = parse_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Get {
                        dest,
                        device,
                        stack_index,
                    },
                    line_number,
                    original_line,
                })
            }
            "put" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "put".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let value = parse_operand(tokens[1]);
                let device = parse_operand(tokens[2]);
                let stack_index = parse_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Put {
                        value,
                        device,
                        stack_index,
                    },
                    line_number,
                    original_line,
                })
            }
            "getd" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "getd".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let id = parse_operand(tokens[2]);
                let stack_index = parse_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Getd {
                        dest,
                        id,
                        stack_index,
                    },
                    line_number,
                    original_line,
                })
            }
            "putd" => {
                if tokens.len() != 4 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "putd".to_string(),
                        expected: 3,
                        actual: tokens.len() - 1,
                    });
                }
                let value = parse_operand(tokens[1]);
                let id = parse_operand(tokens[2]);
                let stack_index = parse_operand(tokens[3]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Putd {
                        value,
                        id,
                        stack_index,
                    },
                    line_number,
                    original_line,
                })
            }

            // ==================== Special Instructions ====================
            "yield" => {
                if tokens.len() != 1 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "yield".to_string(),
                        expected: 0,
                        actual: tokens.len() - 1,
                    });
                }
                Ok(ParsedInstruction {
                    instruction: Instruction::Yield,
                    line_number,
                    original_line,
                })
            }
            "sleep" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "sleep".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let duration = parse_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Sleep { duration },
                    line_number,
                    original_line,
                })
            }
            "hcf" => {
                if tokens.len() != 1 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "hcf".to_string(),
                        expected: 0,
                        actual: tokens.len() - 1,
                    });
                }
                Ok(ParsedInstruction {
                    instruction: Instruction::Hcf,
                    line_number,
                    original_line,
                })
            }
            "select" => {
                if tokens.len() != 5 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "select".to_string(),
                        expected: 4,
                        actual: tokens.len() - 1,
                    });
                }
                let dest = parse_dest_operand(tokens[1]);
                let cond = parse_operand(tokens[2]);
                let arg1 = parse_operand(tokens[3]);
                let arg2 = parse_operand(tokens[4]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Select {
                        dest,
                        cond,
                        arg1,
                        arg2,
                    },
                    line_number,
                    original_line,
                })
            }
            "clr" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "clr".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let device = parse_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Clr { device },
                    line_number,
                    original_line,
                })
            }
            "clrd" => {
                if tokens.len() != 2 {
                    return Err(IC10Error::IncorrectArgumentCount {
                        instruction: "clrd".to_string(),
                        expected: 1,
                        actual: tokens.len() - 1,
                    });
                }
                let id = parse_operand(tokens[1]);
                Ok(ParsedInstruction {
                    instruction: Instruction::Clrd { id },
                    line_number,
                    original_line,
                })
            }

            _ => Err(IC10Error::UnrecognizedInstruction(tokens[0].to_string())),
        }
    }
}
