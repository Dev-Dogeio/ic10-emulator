//! Instruction execution logic for IC10

use crate::constants::{RETURN_ADDRESS_INDEX, STACK_POINTER_INDEX};
use crate::conversions::{double_to_long, lerp, long_to_double};
use crate::error::{SimulationError, SimulationResult};
use crate::instruction::{Instruction, ParsedInstruction};
use crate::items::item_integrated_circuit_10::AliasTarget;
use crate::networks::BatchMode;
use crate::{ItemIntegratedCircuit10, LogicSlotType, LogicType};

/// Execute a single IC10 instruction and return the next program counter
pub fn execute_instruction(
    chip: &ItemIntegratedCircuit10,
    instruction: &ParsedInstruction,
) -> SimulationResult<usize> {
    match &instruction.instruction {
        // ==================== Data Movement ====================
        Instruction::Move { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Alias { name, target } => {
            let resolved_target = match target {
                AliasTarget::Device(pin_idx) => {
                    let pin = *pin_idx as usize;
                    let chip_slot = chip.get_chip_slot();
                    if let Some(ref_id) = chip_slot.borrow().get_device_pin(pin) {
                        AliasTarget::Device(ref_id)
                    } else {
                        return Err(SimulationError::RuntimeError {
                            message: format!("No device assigned to pin d{pin}"),
                            line: instruction.line_number,
                        });
                    }
                }
                AliasTarget::Register(_) => target.clone(),
                AliasTarget::Alias(other_name) => chip.resolve_alias(other_name)?,
            };
            chip.insert_alias(name, resolved_target);
            Ok(chip.get_pc() + 1)
        }
        Instruction::Define { name, value } => {
            chip.insert_define(name, *value);
            Ok(chip.get_pc() + 1)
        }

        // ==================== Arithmetic Operations ====================
        Instruction::Add { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(chip.resolve_register(dest)?, val1 + val2)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sub { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(chip.resolve_register(dest)?, val1 - val2)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Mul { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(chip.resolve_register(dest)?, val1 * val2)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Div { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(chip.resolve_register(dest)?, val1 / val2)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Mod { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let mut result = val1 % val2;
            if result < 0.0 {
                result += val2;
            }
            chip.set_register(chip.resolve_register(dest)?, result)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sqrt { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.sqrt())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Abs { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.abs())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Exp { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.exp())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Log { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.ln())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Pow { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(chip.resolve_register(dest)?, val1.powf(val2))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Max { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(chip.resolve_register(dest)?, val1.max(val2))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Min { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(chip.resolve_register(dest)?, val1.min(val2))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Ceil { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.ceil())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Floor { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.floor())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Round { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.round())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Trunc { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.trunc())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Rand { dest } => {
            let val = rand::random::<f64>();
            chip.set_register(chip.resolve_register(dest)?, val)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Lerp {
            dest,
            arg1,
            arg2,
            arg3,
        } => {
            let a = chip.resolve_value(arg1)?;
            let b = chip.resolve_value(arg2)?;
            let t = chip.resolve_value(arg3)?;
            chip.set_register(chip.resolve_register(dest)?, lerp(a, b, t))?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Trigonometric Operations ====================
        Instruction::Sin { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.sin())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Cos { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.cos())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Tan { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.tan())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Asin { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.asin())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Acos { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.acos())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Atan { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(chip.resolve_register(dest)?, val.atan())?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Atan2 { dest, arg1, arg2 } => {
            let y = chip.resolve_value(arg1)?;
            let x = chip.resolve_value(arg2)?;
            chip.set_register(chip.resolve_register(dest)?, y.atan2(x))?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Bitwise Operations ====================
        // All bitwise ops use double_to_long/long_to_double for IC10 53-bit compatibility
        Instruction::And { dest, arg1, arg2 } => {
            let val1 = double_to_long(chip.resolve_value(arg1)?, true);
            let val2 = double_to_long(chip.resolve_value(arg2)?, true);
            chip.set_register(chip.resolve_register(dest)?, long_to_double(val1 & val2))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Or { dest, arg1, arg2 } => {
            let val1 = double_to_long(chip.resolve_value(arg1)?, true);
            let val2 = double_to_long(chip.resolve_value(arg2)?, true);
            chip.set_register(chip.resolve_register(dest)?, long_to_double(val1 | val2))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Xor { dest, arg1, arg2 } => {
            let val1 = double_to_long(chip.resolve_value(arg1)?, true);
            let val2 = double_to_long(chip.resolve_value(arg2)?, true);
            chip.set_register(chip.resolve_register(dest)?, long_to_double(val1 ^ val2))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Nor { dest, arg1, arg2 } => {
            let val1 = double_to_long(chip.resolve_value(arg1)?, true);
            let val2 = double_to_long(chip.resolve_value(arg2)?, true);
            chip.set_register(chip.resolve_register(dest)?, long_to_double(!(val1 | val2)))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Not { dest, arg } => {
            let val = double_to_long(chip.resolve_value(arg)?, true);
            chip.set_register(chip.resolve_register(dest)?, long_to_double(!val))?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Bit Shifting Operations ====================
        // Shift ops use double_to_long/long_to_double for IC10 53-bit compatibility
        Instruction::Sll { dest, arg1, arg2 } => {
            let val = double_to_long(chip.resolve_value(arg1)?, true);
            let shift = chip.resolve_value(arg2)? as i32;
            chip.set_register(chip.resolve_register(dest)?, long_to_double(val << shift))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sla { dest, arg1, arg2 } => {
            let val = double_to_long(chip.resolve_value(arg1)?, true);
            let shift = chip.resolve_value(arg2)? as i32;
            chip.set_register(chip.resolve_register(dest)?, long_to_double(val << shift))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Srl { dest, arg1, arg2 } => {
            let val = double_to_long(chip.resolve_value(arg1)?, false);
            let shift = chip.resolve_value(arg2)? as i32;
            chip.set_register(chip.resolve_register(dest)?, long_to_double(val >> shift))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sra { dest, arg1, arg2 } => {
            let val = double_to_long(chip.resolve_value(arg1)?, true);
            let shift = chip.resolve_value(arg2)? as i32;
            chip.set_register(chip.resolve_register(dest)?, long_to_double(val >> shift))?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Bit Field Operations ====================
        Instruction::Ext {
            dest,
            source,
            start,
            length,
        } => {
            let source_val = double_to_long(chip.resolve_value(source)?, false);
            let start_bit = chip.resolve_value(start)? as i32;
            let num_bits = chip.resolve_value(length)? as i32;

            if num_bits <= 0 {
                return Err(SimulationError::RuntimeError {
                    message: "EXT: length must be > 0 (ShiftUnderflow)".to_string(),
                    line: instruction.line_number,
                });
            }
            if start_bit < 0 {
                return Err(SimulationError::RuntimeError {
                    message: "EXT: start must be >= 0 (ShiftUnderflow)".to_string(),
                    line: instruction.line_number,
                });
            }
            if start_bit >= 53 {
                return Err(SimulationError::RuntimeError {
                    message: "EXT: start must be < 53 (ShiftOverflow)".to_string(),
                    line: instruction.line_number,
                });
            }
            if num_bits > 53 || start_bit + num_bits > 53 {
                return Err(SimulationError::RuntimeError {
                    message: "EXT: start + length must be <= 53 (PayloadOverflow)".to_string(),
                    line: instruction.line_number,
                });
            }

            const MANTISSA_MASK: i64 = 0x1FFFFFFFFFFFFF;

            let length_mask = if num_bits == 53 {
                MANTISSA_MASK
            } else {
                (1i64 << num_bits) - 1
            };

            let extracted =
                ((source_val & MANTISSA_MASK) & (length_mask << start_bit)) >> start_bit;
            chip.set_register(chip.resolve_register(dest)?, long_to_double(extracted))?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Ins {
            dest,
            value,
            start,
            length,
        } => {
            let dest_idx = chip.resolve_register(dest)?;
            let insert_value = double_to_long(chip.resolve_value(value)?, false);
            let bit_position = chip.resolve_value(start)? as i32;
            let num_bits = chip.resolve_value(length)? as i32;

            if num_bits <= 0 {
                return Err(SimulationError::RuntimeError {
                    message: "INS: bit count must be > 0 (ShiftUnderflow)".to_string(),
                    line: instruction.line_number,
                });
            }
            if bit_position < 0 {
                return Err(SimulationError::RuntimeError {
                    message: "INS: bit position must be >= 0 (ShiftUnderflow)".to_string(),
                    line: instruction.line_number,
                });
            }
            if bit_position >= 53 {
                return Err(SimulationError::RuntimeError {
                    message: "INS: bit position must be < 53 (ShiftOverflow)".to_string(),
                    line: instruction.line_number,
                });
            }
            if num_bits > 53 || bit_position + num_bits > 53 {
                return Err(SimulationError::RuntimeError {
                    message: "INS: position + count must be <= 53 (PayloadOverflow)".to_string(),
                    line: instruction.line_number,
                });
            }

            const MANTISSA_MASK: u64 = 0x1FFFFFFFFFFFFF;

            let current_val =
                double_to_long(chip.get_register(dest_idx)?, false) as u64 & MANTISSA_MASK;
            let value_to_insert = insert_value as u64 & MANTISSA_MASK;

            let field_mask = if num_bits == 53 {
                MANTISSA_MASK
            } else {
                (1u64 << num_bits) - 1
            };

            let positioned_mask = field_mask << bit_position;
            let inv_mask = !(positioned_mask as i64);

            let cleared = (current_val as i64 & inv_mask) as u64;
            let inserted = (((value_to_insert as i64) & (field_mask as i64)) << bit_position)
                as u64
                & positioned_mask;
            let result = (cleared | inserted) & MANTISSA_MASK;

            chip.set_register(dest_idx, long_to_double(result as i64))?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Comparison Operations ====================
        Instruction::Slt { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val1 < val2 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sgt { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val1 > val2 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sle { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val1 <= val2 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sge { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val1 >= val2 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Seq { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val1 == val2 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sne { dest, arg1, arg2 } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val1 != val2 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Select {
            dest,
            cond,
            arg1,
            arg2,
        } => {
            let cond_val = chip.resolve_value(cond)?;
            let a = chip.resolve_value(arg1)?;
            let b = chip.resolve_value(arg2)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if cond_val != 0.0 { a } else { b },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sltz { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val < 0.0 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sgtz { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val > 0.0 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Slez { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val <= 0.0 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sgez { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val >= 0.0 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Seqz { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val == 0.0 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Snez { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val != 0.0 { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Snan { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if val.is_nan() { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Snanz { dest, arg } => {
            let val = chip.resolve_value(arg)?;
            chip.set_register(
                chip.resolve_register(dest)?,
                if !val.is_nan() { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Approximate Comparison ====================
        Instruction::Sap {
            dest,
            arg1,
            arg2,
            arg3,
        } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let c = chip.resolve_value(arg3)?;
            // relative tolerance: |a - b| <= max(c * max(|a|, |b|), 1.1210387714598537E-44)
            let tolerance = (c * val1.abs().max(val2.abs())).max(1.1210387714598537E-44);
            chip.set_register(
                chip.resolve_register(dest)?,
                if (val1 - val2).abs() <= tolerance {
                    1.0
                } else {
                    0.0
                },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sna {
            dest,
            arg1,
            arg2,
            arg3,
        } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let c = chip.resolve_value(arg3)?;
            // relative tolerance: |a - b| > max(c * max(|a|, |b|), 1.1210387714598537E-44)
            let tolerance = (c * val1.abs().max(val2.abs())).max(1.1210387714598537E-44);
            chip.set_register(
                chip.resolve_register(dest)?,
                if (val1 - val2).abs() > tolerance {
                    1.0
                } else {
                    0.0
                },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sapz { dest, arg1, arg2 } => {
            let val = chip.resolve_value(arg1)?;
            let c = chip.resolve_value(arg2)?;
            // sapz is sap with second argument as 0, so tolerance = max(c * |a|, min_epsilon)
            let tolerance = (c * val.abs()).max(1.1210387714598537E-44);
            chip.set_register(
                chip.resolve_register(dest)?,
                if val.abs() <= tolerance { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Snaz { dest, arg1, arg2 } => {
            let val = chip.resolve_value(arg1)?;
            let c = chip.resolve_value(arg2)?;
            // snaz is sna with second argument as 0, so tolerance = max(c * |a|, min_epsilon)
            let tolerance = (c * val.abs()).max(1.1210387714598537E-44);
            chip.set_register(
                chip.resolve_register(dest)?,
                if val.abs() > tolerance { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Device State Detection ====================
        Instruction::Sdse { dest, device } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            let exists = chip.device_exists_by_id(ref_id);
            chip.set_register(chip.resolve_register(dest)?, if exists { 1.0 } else { 0.0 })?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sdns { dest, device } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            let exists = chip.device_exists_by_id(ref_id);
            chip.set_register(
                chip.resolve_register(dest)?,
                if !exists { 1.0 } else { 0.0 },
            )?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Branch Instructions (Absolute) ====================
        Instruction::Beq { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 == val2 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bne { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 != val2 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Blt { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 < val2 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bgt { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 > val2 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Ble { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 <= val2 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bge { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 >= val2 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Beqz { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val == 0.0 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bnez { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val != 0.0 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bltz { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val < 0.0 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bgez { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val >= 0.0 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Blez { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val <= 0.0 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bgtz { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val > 0.0 {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bnan { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val.is_nan() {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }

        // ==================== Branch Instructions (Relative) ====================
        Instruction::Breq { arg1, arg2, offset } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let off = chip.resolve_value(offset)? as i32;
            if val1 == val2 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brne { arg1, arg2, offset } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let off = chip.resolve_value(offset)? as i32;
            if val1 != val2 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brlt { arg1, arg2, offset } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let off = chip.resolve_value(offset)? as i32;
            if val1 < val2 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brgt { arg1, arg2, offset } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let off = chip.resolve_value(offset)? as i32;
            if val1 > val2 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brle { arg1, arg2, offset } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let off = chip.resolve_value(offset)? as i32;
            if val1 <= val2 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brge { arg1, arg2, offset } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let off = chip.resolve_value(offset)? as i32;
            if val1 >= val2 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Breqz { arg, offset } => {
            let val = chip.resolve_value(arg)?;
            let off = chip.resolve_value(offset)? as i32;
            if val == 0.0 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brnez { arg, offset } => {
            let val = chip.resolve_value(arg)?;
            let off = chip.resolve_value(offset)? as i32;
            if val != 0.0 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brltz { arg, offset } => {
            let val = chip.resolve_value(arg)?;
            let off = chip.resolve_value(offset)? as i32;
            if val < 0.0 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brgez { arg, offset } => {
            let val = chip.resolve_value(arg)?;
            let off = chip.resolve_value(offset)? as i32;
            if val >= 0.0 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brlez { arg, offset } => {
            let val = chip.resolve_value(arg)?;
            let off = chip.resolve_value(offset)? as i32;
            if val <= 0.0 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brgtz { arg, offset } => {
            let val = chip.resolve_value(arg)?;
            let off = chip.resolve_value(offset)? as i32;
            if val > 0.0 {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brnan { arg, offset } => {
            let val = chip.resolve_value(arg)?;
            let off = chip.resolve_value(offset)? as i32;
            if val.is_nan() {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }

        // ==================== Branch and Link Variants ====================
        Instruction::Beqal { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 == val2 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bneal { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 != val2 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bltal { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 < val2 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bgtal { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 > val2 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bleal { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 <= val2 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bgeal { arg1, arg2, line } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            if val1 >= val2 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Beqzal { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val == 0.0 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bnezal { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val != 0.0 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bltzal { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val < 0.0 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bgezal { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val >= 0.0 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Blezal { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val <= 0.0 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bgtzal { arg, line } => {
            let val = chip.resolve_value(arg)?;
            if val > 0.0 {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }

        // ==================== Approximate Branches ====================
        Instruction::Bap {
            arg1,
            arg2,
            arg3,
            line,
        } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let c = chip.resolve_value(arg3)?;
            let tolerance = (c * val1.abs().max(val2.abs())).max(1.1210387714598537E-44);
            if (val1 - val2).abs() <= tolerance {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bna {
            arg1,
            arg2,
            arg3,
            line,
        } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let c = chip.resolve_value(arg3)?;
            let tolerance = (c * val1.abs().max(val2.abs())).max(1.1210387714598537E-44);
            if (val1 - val2).abs() > tolerance {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brap {
            arg1,
            arg2,
            arg3,
            offset,
        } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let c = chip.resolve_value(arg3)?;
            let tolerance = (c * val1.abs().max(val2.abs())).max(1.1210387714598537E-44);
            if (val1 - val2).abs() <= tolerance {
                let off = chip.resolve_value(offset)? as i32;
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brna {
            arg1,
            arg2,
            arg3,
            offset,
        } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let c = chip.resolve_value(arg3)?;
            let tolerance = (c * val1.abs().max(val2.abs())).max(1.1210387714598537E-44);
            if (val1 - val2).abs() > tolerance {
                let off = chip.resolve_value(offset)? as i32;
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bapz { arg1, arg2, line } => {
            let val = chip.resolve_value(arg1)?;
            let c = chip.resolve_value(arg2)?;
            // bapz is bap with second value as 0, so tolerance = max(c * |val|, min_epsilon)
            let tolerance = (c * val.abs()).max(1.1210387714598537E-44);
            if val.abs() <= tolerance {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bnaz { arg1, arg2, line } => {
            let val = chip.resolve_value(arg1)?;
            let c = chip.resolve_value(arg2)?;
            let tolerance = (c * val.abs()).max(1.1210387714598537E-44);
            if val.abs() > tolerance {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brapz { arg1, arg2, offset } => {
            let val = chip.resolve_value(arg1)?;
            let c = chip.resolve_value(arg2)?;
            let tolerance = (c * val.abs()).max(1.1210387714598537E-44);
            let off = chip.resolve_value(offset)? as i32;
            if val.abs() <= tolerance {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brnaz { arg1, arg2, offset } => {
            let val = chip.resolve_value(arg1)?;
            let c = chip.resolve_value(arg2)?;
            let tolerance = (c * val.abs()).max(1.1210387714598537E-44);
            let off = chip.resolve_value(offset)? as i32;
            if val.abs() > tolerance {
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bapal {
            arg1,
            arg2,
            arg3,
            line,
        } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let c = chip.resolve_value(arg3)?;
            let tolerance = (c * val1.abs().max(val2.abs())).max(1.1210387714598537E-44);
            if (val1 - val2).abs() <= tolerance {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bnaal {
            arg1,
            arg2,
            arg3,
            line,
        } => {
            let val1 = chip.resolve_value(arg1)?;
            let val2 = chip.resolve_value(arg2)?;
            let c = chip.resolve_value(arg3)?;
            let tolerance = (c * val1.abs().max(val2.abs())).max(1.1210387714598537E-44);
            if (val1 - val2).abs() > tolerance {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bapzal { arg1, arg2, line } => {
            let val = chip.resolve_value(arg1)?;
            let c = chip.resolve_value(arg2)?;
            let tolerance = (c * val.abs()).max(1.1210387714598537E-44);
            if val.abs() <= tolerance {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bnazal { arg1, arg2, line } => {
            let val = chip.resolve_value(arg1)?;
            let c = chip.resolve_value(arg2)?;
            let tolerance = (c * val.abs()).max(1.1210387714598537E-44);
            if val.abs() > tolerance {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }

        // ==================== Device State Branches ====================
        Instruction::Bdse { device, line } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            if chip.device_exists_by_id(ref_id) {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bdns { device, line } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            if !chip.device_exists_by_id(ref_id) {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brdse { device, offset } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            if chip.device_exists_by_id(ref_id) {
                let off = chip.resolve_value(offset)? as i32;
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Brdns { device, offset } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            if !chip.device_exists_by_id(ref_id) {
                let off = chip.resolve_value(offset)? as i32;
                let target = (chip.get_pc() as i32 + off) as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bdseal { device, line } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            if chip.device_exists_by_id(ref_id) {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bdnsal { device, line } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            if !chip.device_exists_by_id(ref_id) {
                chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bdnvl {
            device,
            logic_type,
            line,
        } => {
            // Branch if device not valid for load (reading) the specified logic type
            let ref_id = chip.resolve_device_ref_id(device)?;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let logic_type_enum = LogicType::from_value(logic_type_val);

            // Check if device can read this logic type
            let can_load = if let Some(lt) = logic_type_enum {
                if let Some(network) = chip.get_network() {
                    let network_ref = network.borrow();

                    match network_ref.get_device(ref_id) {
                        Some(device) => device.can_read(lt),
                        None => false,
                    }
                } else {
                    false
                }
            } else {
                false // Invalid logic type
            };

            if !can_load {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }
        Instruction::Bdnvs {
            device,
            logic_type,
            line,
        } => {
            // Branch if device not valid for store (writing) the specified logic type
            let ref_id = chip.resolve_device_ref_id(device)?;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let logic_type_enum = LogicType::from_value(logic_type_val);

            // Check if device can write this logic type
            let can_store = if let Some(lt) = logic_type_enum {
                if let Some(network) = chip.get_network() {
                    let network_ref = network.borrow();

                    match network_ref.get_device(ref_id) {
                        Some(device) => device.can_write(lt),
                        None => false,
                    }
                } else {
                    false
                }
            } else {
                false // Invalid logic type
            };

            if !can_store {
                let target = chip.resolve_value(line)? as usize;
                Ok(target)
            } else {
                Ok(chip.get_pc() + 1)
            }
        }

        // ==================== Jump Instructions ====================
        Instruction::J { line } => {
            let target = chip.resolve_value(line)? as usize;
            Ok(target)
        }
        Instruction::Jr { offset } => {
            let off = chip.resolve_value(offset)? as i32;
            let target = (chip.get_pc() as i32 + off) as usize;
            Ok(target)
        }
        Instruction::Jal { line } => {
            chip.set_register(RETURN_ADDRESS_INDEX, (chip.get_pc() + 1) as f64)?;
            let target = chip.resolve_value(line)? as usize;
            Ok(target)
        }

        // ==================== Stack Operations ====================
        Instruction::Push { arg } => {
            let value = chip.resolve_value(arg)?;
            let sp = chip.get_register(STACK_POINTER_INDEX)? as usize;
            chip.write_stack(sp, value)?;
            chip.set_register(STACK_POINTER_INDEX, (sp + 1) as f64)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Pop { dest } => {
            let sp = chip.get_register(STACK_POINTER_INDEX)? as usize;
            if sp == 0 {
                return Err(SimulationError::RuntimeError {
                    line: chip.get_pc(),
                    message: "Stack underflow: cannot pop from empty stack".to_string(),
                });
            }
            let new_sp = sp - 1;
            let value = chip.read_stack(new_sp)?;
            chip.set_register(chip.resolve_register(dest)?, value)?;
            chip.set_register(STACK_POINTER_INDEX, new_sp as f64)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Peek { dest } => {
            let sp = chip.get_register(STACK_POINTER_INDEX)? as usize;
            if sp == 0 {
                return Err(SimulationError::RuntimeError {
                    line: chip.get_pc(),
                    message: "Stack underflow: cannot peek from empty stack".to_string(),
                });
            }
            let value = chip.read_stack(sp - 1)?;
            chip.set_register(chip.resolve_register(dest)?, value)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Poke { index, value } => {
            let idx = chip.resolve_value(index)? as usize;
            let val = chip.resolve_value(value)?;
            chip.write_stack(idx, val)?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Device I/O Instructions ====================
        Instruction::L {
            dest,
            device,
            logic_type,
        } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let logic_type =
                LogicType::from_value(logic_type_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid logic type: {logic_type_val}"),
                    line: instruction.line_number,
                })?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;
            let value = device.read(logic_type)?;

            chip.set_register(chip.resolve_register(dest)?, value)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::S {
            device,
            logic_type,
            value,
        } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let value = chip.resolve_value(value)?;
            let logic_type =
                LogicType::from_value(logic_type_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid logic type: {logic_type_val}"),
                    line: instruction.line_number,
                })?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;
            device.write(logic_type, value)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Ls {
            dest,
            device,
            slot_index,
            slot_logic_type,
        } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            let slot_index = chip.resolve_value(slot_index)? as usize;
            let slot_logic_val = chip.resolve_value(slot_logic_type)?;
            let slot_logic =
                LogicSlotType::from_value(slot_logic_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid slot logic type: {slot_logic_val}"),
                    line: instruction.line_number,
                })?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;

            let val = device.read_slot(slot_index, slot_logic)?;
            chip.set_register(chip.resolve_register(dest)?, val)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Ss {
            device,
            slot_index,
            slot_logic_type,
            value,
        } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            let slot_index = chip.resolve_value(slot_index)? as usize;
            let slot_logic_val = chip.resolve_value(slot_logic_type)?;
            let slot_logic =
                LogicSlotType::from_value(slot_logic_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid slot logic type: {slot_logic_val}"),
                    line: instruction.line_number,
                })?;
            let value = chip.resolve_value(value)?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;

            device.write_slot(slot_index, slot_logic, value)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Lr {
            dest: _,
            device: _,
            reagent_mode: _,
            reagent: _,
        } => {
            // Reagent operations not yet fully implemented
            Err(SimulationError::RuntimeError {
                message: "Reagent operations not yet implemented".to_string(),
                line: instruction.line_number,
            })
        }
        Instruction::Rmap {
            dest: _,
            device: _,
            reagent_hash: _,
        } => {
            // Reagent operations not yet fully implemented
            Err(SimulationError::RuntimeError {
                message: "Reagent operations not yet implemented".to_string(),
                line: instruction.line_number,
            })
        }

        // ==================== ID-Based Device Access ====================
        Instruction::Ld {
            dest,
            id,
            logic_type,
        } => {
            let ref_id = chip.resolve_value(id)? as i32;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let logic_type =
                LogicType::from_value(logic_type_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid logic type: {logic_type_val}"),
                    line: instruction.line_number,
                })?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;

            let value = device.read(logic_type)?;
            drop(device);
            chip.set_register(chip.resolve_register(dest)?, value)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sd {
            id,
            logic_type,
            value,
        } => {
            let ref_id = chip.resolve_value(id)? as i32;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let value = chip.resolve_value(value)?;
            let logic_type =
                LogicType::from_value(logic_type_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid logic type: {logic_type_val}"),
                    line: instruction.line_number,
                })?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;

            device.write(logic_type, value)?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Batch Device Access ====================
        Instruction::Lb {
            dest,
            device_hash,
            logic_type,
            batch_mode,
        } => {
            let prefab_hash = chip.resolve_value(device_hash)? as i32;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let batch_mode_val = chip.resolve_value(batch_mode)?;

            let logic_type =
                LogicType::from_value(logic_type_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid logic type: {logic_type_val}"),
                    line: instruction.line_number,
                })?;

            let batch_mode =
                BatchMode::from_value(batch_mode_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid batch mode: {batch_mode_val}"),
                    line: instruction.line_number,
                })?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let value = network
                .borrow()
                .batch_read_by_prefab(prefab_hash, logic_type, batch_mode)
                .map_err(|e| SimulationError::RuntimeError {
                    message: e.to_string(),
                    line: instruction.line_number,
                })?;

            chip.set_register(chip.resolve_register(dest)?, value)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sb {
            device_hash,
            logic_type,
            value,
        } => {
            let prefab_hash = chip.resolve_value(device_hash)? as i32;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let value = chip.resolve_value(value)?;

            let logic_type =
                LogicType::from_value(logic_type_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid logic type: {logic_type_val}"),
                    line: instruction.line_number,
                })?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            network
                .borrow()
                .batch_write_by_prefab(prefab_hash, logic_type, value)
                .map_err(|e| SimulationError::RuntimeError {
                    message: e.to_string(),
                    line: instruction.line_number,
                })?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Lbn {
            dest,
            device_hash,
            name_hash,
            logic_type,
            batch_mode,
        } => {
            let prefab_hash = chip.resolve_value(device_hash)? as i32;
            let name_hash = chip.resolve_value(name_hash)? as i32;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let batch_mode_val = chip.resolve_value(batch_mode)?;

            let logic_type =
                LogicType::from_value(logic_type_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid logic type: {logic_type_val}"),
                    line: instruction.line_number,
                })?;

            let batch_mode =
                BatchMode::from_value(batch_mode_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid batch mode: {batch_mode_val}"),
                    line: instruction.line_number,
                })?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let value = network
                .borrow()
                .batch_read_by_name(prefab_hash, name_hash, logic_type, batch_mode)
                .map_err(|e| SimulationError::RuntimeError {
                    message: e.to_string(),
                    line: instruction.line_number,
                })?;

            chip.set_register(chip.resolve_register(dest)?, value)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Sbn {
            device_hash,
            name_hash,
            logic_type,
            value,
        } => {
            let prefab_hash = chip.resolve_value(device_hash)? as i32;
            let name_hash = chip.resolve_value(name_hash)? as i32;
            let logic_type_val = chip.resolve_value(logic_type)?;
            let value = chip.resolve_value(value)?;

            let logic_type =
                LogicType::from_value(logic_type_val).ok_or(SimulationError::RuntimeError {
                    message: format!("Invalid logic type: {logic_type_val}"),
                    line: instruction.line_number,
                })?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            network
                .borrow()
                .batch_write_by_name(prefab_hash, name_hash, logic_type, value)
                .map_err(|e| SimulationError::RuntimeError {
                    message: e.to_string(),
                    line: instruction.line_number,
                })?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Memory Access Instructions ====================
        Instruction::Get {
            dest,
            device,
            stack_index,
        } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            let index = chip.resolve_value(stack_index)? as usize;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;
            let value = device
                .get_memory(index)
                .map_err(|e| SimulationError::RuntimeError {
                    message: e.to_string(),
                    line: instruction.line_number,
                })?;
            drop(device);
            chip.set_register(chip.resolve_register(dest)?, value)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Put {
            value,
            device,
            stack_index,
        } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            let index = chip.resolve_value(stack_index)? as usize;
            let val = chip.resolve_value(value)?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;
            device
                .set_memory(index, val)
                .map_err(|e| SimulationError::RuntimeError {
                    message: e.to_string(),
                    line: instruction.line_number,
                })?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Getd {
            dest,
            id,
            stack_index,
        } => {
            let ref_id = chip.resolve_value(id)? as i32;
            let index = chip.resolve_value(stack_index)? as usize;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;
            let value = device
                .get_memory(index)
                .map_err(|e| SimulationError::RuntimeError {
                    message: e.to_string(),
                    line: instruction.line_number,
                })?;
            drop(device);
            chip.set_register(chip.resolve_register(dest)?, value)?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Putd {
            value,
            id,
            stack_index,
        } => {
            let ref_id = chip.resolve_value(id)? as i32;
            let index = chip.resolve_value(stack_index)? as usize;
            let val = chip.resolve_value(value)?;

            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;
            device
                .set_memory(index, val)
                .map_err(|e| SimulationError::RuntimeError {
                    message: e.to_string(),
                    line: instruction.line_number,
                })?;
            Ok(chip.get_pc() + 1)
        }

        // ==================== Special Instructions ====================
        Instruction::Yield => Ok(chip.get_pc() + 1),
        Instruction::Sleep { duration: seconds } => {
            let ticks = chip.resolve_value(seconds)? * 2.0;
            if ticks > 1f64 {
                chip.set_sleep_ticks((ticks - 1f64) as u64);
            }
            Ok(chip.get_pc() + 1)
        }
        Instruction::Hcf => {
            chip.halt();
            Err(SimulationError::RuntimeError {
                message: "(HCF) - chip execution terminated".to_string(),
                line: instruction.line_number,
            })
        }
        Instruction::Clr { device } => {
            let ref_id = chip.resolve_device_ref_id(device)?;
            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;
            device.clear().map_err(|e| SimulationError::RuntimeError {
                message: e.to_string(),
                line: instruction.line_number,
            })?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Clrd { id } => {
            let ref_id = chip.resolve_value(id)? as i32;
            let network = chip.get_network().ok_or(SimulationError::RuntimeError {
                message: "Housing not connected to network".to_string(),
                line: instruction.line_number,
            })?;
            let network_ref = network.borrow();
            let device = network_ref
                .get_device(ref_id)
                .ok_or(SimulationError::RuntimeError {
                    message: format!("Device with reference ID {ref_id} not found"),
                    line: instruction.line_number,
                })?;
            device.clear().map_err(|e| SimulationError::RuntimeError {
                message: e.to_string(),
                line: instruction.line_number,
            })?;
            Ok(chip.get_pc() + 1)
        }
        Instruction::Noop => Ok(chip.get_pc() + 1),
        _ => Err(SimulationError::UnrecognizedInstruction(format!(
            "Unknown or unimplemented instruction {}: {:?}",
            instruction.line_number, instruction.instruction
        ))),
    }
}
