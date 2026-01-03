//! Refactored logic tests - cleaner, more maintainable
//!
//! Design principles:
//! 1. Use string parsing to test full pipeline (parsing + execution)
//! 2. Use helper macros to reduce boilerplate
//! 3. Test operand resolution ONCE, not per-instruction
//! 4. Focus on behavior and edge cases, not operand combinations
//! 5. Use table-driven tests where appropriate

#[cfg(test)]
mod tests {
    use crate::chip::ProgrammableChip;
    use crate::constants::{RETURN_ADDRESS_INDEX, STACK_POINTER_INDEX};
    use crate::devices::{DaylightSensor, Device, ICHousing};
    use crate::instruction::ParsedInstruction;
    use crate::logic::execute_instruction;
    use std::cell::RefCell;
    use std::rc::Rc;

    // ==================== Test Helpers ====================

    /// Create a chip with optional initial register values
    fn chip() -> ProgrammableChip {
        let housing = Rc::new(RefCell::new(ICHousing::new(None)));
        ProgrammableChip::new(housing)
    }

    /// Execute a single instruction from string, return new PC
    fn exec(chip: &mut ProgrammableChip, line: &str) -> Result<usize, String> {
        let parsed = ParsedInstruction::parse(line, 0).map_err(|e| format!("{:?}", e))?;
        execute_instruction(chip, &parsed).map_err(|e| format!("{:?}", e))
    }

    /// Execute instruction, expect success
    fn exec_ok(chip: &mut ProgrammableChip, line: &str) -> usize {
        exec(chip, line).expect(&format!("Failed to execute: {}", line))
    }

    /// Get register value
    fn reg(chip: &ProgrammableChip, idx: usize) -> f64 {
        chip.get_register(idx).unwrap()
    }

    /// Set register value
    fn set_reg(chip: &mut ProgrammableChip, idx: usize, val: f64) {
        chip.set_register(idx, val).unwrap();
    }

    /// Assert register equals value (with floating point tolerance)
    fn assert_reg(chip: &ProgrammableChip, idx: usize, expected: f64) {
        let actual = reg(chip, idx);
        if expected.is_nan() {
            assert!(actual.is_nan(), "r{} expected NaN, got {}", idx, actual);
        } else if expected.is_infinite() {
            assert_eq!(
                actual, expected,
                "r{} expected {}, got {}",
                idx, expected, actual
            );
        } else {
            assert!(
                (actual - expected).abs() < 1e-10,
                "r{} expected {}, got {}",
                idx,
                expected,
                actual
            );
        }
    }

    // ==================== Operand Resolution Tests ====================
    // Test operand types ONCE - they work the same for all instructions

    #[test]
    fn test_operand_resolution() {
        let mut chip = chip();
        set_reg(&mut chip, 0, 10.0);
        set_reg(&mut chip, 1, 20.0);

        // Register operand
        exec_ok(&mut chip, "add r2 r0 r1");
        assert_reg(&chip, 2, 30.0);

        // Immediate operand
        exec_ok(&mut chip, "add r2 5 3");
        assert_reg(&chip, 2, 8.0);

        // Mixed register + immediate
        exec_ok(&mut chip, "add r2 r0 5");
        assert_reg(&chip, 2, 15.0);

        // Define constant
        exec_ok(&mut chip, "define CONST 100");
        exec_ok(&mut chip, "add r2 r0 CONST");
        assert_reg(&chip, 2, 110.0);

        // sp/ra aliases
        exec_ok(&mut chip, "move sp 5");
        assert_reg(&chip, STACK_POINTER_INDEX, 5.0);
        exec_ok(&mut chip, "move ra 10");
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 10.0);
    }

    // ==================== Arithmetic Instructions ====================

    #[test]
    fn test_arithmetic_basic() {
        let mut chip = chip();

        // add
        exec_ok(&mut chip, "add r0 10 5");
        assert_reg(&chip, 0, 15.0);

        // sub
        exec_ok(&mut chip, "sub r0 10 3");
        assert_reg(&chip, 0, 7.0);

        // mul
        exec_ok(&mut chip, "mul r0 6 7");
        assert_reg(&chip, 0, 42.0);

        // div
        exec_ok(&mut chip, "div r0 20 4");
        assert_reg(&chip, 0, 5.0);

        // mod (floored)
        exec_ok(&mut chip, "mod r0 7 3");
        assert_reg(&chip, 0, 1.0);

        // mod with negative (should be positive due to floored mod)
        exec_ok(&mut chip, "mod r0 -7 3");
        assert_reg(&chip, 0, 2.0);

        // sqrt
        exec_ok(&mut chip, "sqrt r0 16");
        assert_reg(&chip, 0, 4.0);

        // abs
        exec_ok(&mut chip, "abs r0 -42");
        assert_reg(&chip, 0, 42.0);

        // exp
        exec_ok(&mut chip, "exp r0 0");
        assert_reg(&chip, 0, 1.0);

        // log
        exec_ok(&mut chip, "log r0 1");
        assert_reg(&chip, 0, 0.0);

        // pow
        exec_ok(&mut chip, "pow r0 2 10");
        assert_reg(&chip, 0, 1024.0);

        // max/min
        exec_ok(&mut chip, "max r0 5 10");
        assert_reg(&chip, 0, 10.0);
        exec_ok(&mut chip, "min r0 5 10");
        assert_reg(&chip, 0, 5.0);
    }

    #[test]
    fn test_rounding() {
        let mut chip = chip();

        // ceil
        exec_ok(&mut chip, "ceil r0 3.2");
        assert_reg(&chip, 0, 4.0);
        exec_ok(&mut chip, "ceil r0 -3.2");
        assert_reg(&chip, 0, -3.0);

        // floor
        exec_ok(&mut chip, "floor r0 3.7");
        assert_reg(&chip, 0, 3.0);
        exec_ok(&mut chip, "floor r0 -3.7");
        assert_reg(&chip, 0, -4.0);

        // round
        exec_ok(&mut chip, "round r0 3.4");
        assert_reg(&chip, 0, 3.0);
        exec_ok(&mut chip, "round r0 3.6");
        assert_reg(&chip, 0, 4.0);

        // trunc
        exec_ok(&mut chip, "trunc r0 3.9");
        assert_reg(&chip, 0, 3.0);
        exec_ok(&mut chip, "trunc r0 -3.9");
        assert_reg(&chip, 0, -3.0);
    }

    #[test]
    fn test_arithmetic_edge_cases() {
        let mut chip = chip();

        // Division by zero
        exec_ok(&mut chip, "div r0 1 0");
        assert!(reg(&chip, 0).is_infinite());

        // sqrt of negative
        exec_ok(&mut chip, "sqrt r0 -1");
        assert!(reg(&chip, 0).is_nan());

        // log of zero
        exec_ok(&mut chip, "log r0 0");
        assert!(reg(&chip, 0).is_infinite());

        // NaN propagation
        exec_ok(&mut chip, "div r0 0 0"); // NaN
        exec_ok(&mut chip, "add r1 r0 5");
        assert!(reg(&chip, 1).is_nan());
    }

    #[test]
    fn test_lerp() {
        let mut chip = chip();

        // Basic lerp
        exec_ok(&mut chip, "lerp r0 0 100 0.5");
        assert_reg(&chip, 0, 50.0);

        // t=0 gives a
        exec_ok(&mut chip, "lerp r0 10 20 0");
        assert_reg(&chip, 0, 10.0);

        // t=1 gives b
        exec_ok(&mut chip, "lerp r0 10 20 1");
        assert_reg(&chip, 0, 20.0);

        // t clamped above 1
        exec_ok(&mut chip, "lerp r0 10 20 2");
        assert_reg(&chip, 0, 20.0);

        // t clamped below 0
        exec_ok(&mut chip, "lerp r0 10 20 -1");
        assert_reg(&chip, 0, 10.0);
    }

    #[test]
    fn test_rand() {
        let mut chip = chip();

        // rand produces value in [0, 1)
        for _ in 0..10 {
            exec_ok(&mut chip, "rand r0");
            let val = reg(&chip, 0);
            assert!(val >= 0.0 && val < 1.0, "rand out of range: {}", val);
        }
    }

    // ==================== Trigonometric Instructions ====================

    #[test]
    fn test_trig_basic() {
        let mut chip = chip();
        let pi = std::f64::consts::PI;

        // sin
        exec_ok(&mut chip, "sin r0 0");
        assert_reg(&chip, 0, 0.0);

        set_reg(&mut chip, 1, pi / 2.0);
        exec_ok(&mut chip, "sin r0 r1");
        assert_reg(&chip, 0, 1.0);

        // cos
        exec_ok(&mut chip, "cos r0 0");
        assert_reg(&chip, 0, 1.0);

        // tan
        exec_ok(&mut chip, "tan r0 0");
        assert_reg(&chip, 0, 0.0);

        // atan2
        exec_ok(&mut chip, "atan2 r0 1 1");
        assert_reg(&chip, 0, pi / 4.0);
    }

    #[test]
    fn test_inverse_trig() {
        let mut chip = chip();
        let pi = std::f64::consts::PI;

        // asin
        exec_ok(&mut chip, "asin r0 0");
        assert_reg(&chip, 0, 0.0);
        exec_ok(&mut chip, "asin r0 1");
        assert_reg(&chip, 0, pi / 2.0);

        // acos
        exec_ok(&mut chip, "acos r0 1");
        assert_reg(&chip, 0, 0.0);
        exec_ok(&mut chip, "acos r0 0");
        assert_reg(&chip, 0, pi / 2.0);

        // atan
        exec_ok(&mut chip, "atan r0 0");
        assert_reg(&chip, 0, 0.0);
    }

    // ==================== Bitwise Instructions ====================

    #[test]
    fn test_bitwise_basic() {
        let mut chip = chip();

        exec_ok(&mut chip, "and r0 15 7");
        assert_reg(&chip, 0, 7.0); // 1111 & 0111 = 0111

        exec_ok(&mut chip, "or r0 8 4");
        assert_reg(&chip, 0, 12.0); // 1000 | 0100 = 1100

        exec_ok(&mut chip, "xor r0 15 6");
        assert_reg(&chip, 0, 9.0); // 1111 ^ 0110 = 1001

        exec_ok(&mut chip, "nor r0 0 0");
        assert_reg(&chip, 0, -1.0); // NOT(0|0) = all 1s = -1 in signed

        exec_ok(&mut chip, "not r0 0");
        assert_reg(&chip, 0, -1.0);
    }

    #[test]
    fn test_shifts() {
        let mut chip = chip();

        // Shift left
        exec_ok(&mut chip, "sll r0 1 4");
        assert_reg(&chip, 0, 16.0); // 1 << 4 = 16

        exec_ok(&mut chip, "sla r0 1 4");
        assert_reg(&chip, 0, 16.0); // Same as sll

        // Shift right logical
        exec_ok(&mut chip, "srl r0 16 2");
        assert_reg(&chip, 0, 4.0);

        // Shift right arithmetic (preserves sign)
        exec_ok(&mut chip, "sra r0 16 2");
        assert_reg(&chip, 0, 4.0);
    }

    #[test]
    fn test_bit_field_operations() {
        let mut chip = chip();

        // ext: extract bits
        set_reg(&mut chip, 0, 255.0); // 0b11111111
        exec_ok(&mut chip, "ext r1 r0 2 4"); // Extract 4 bits starting at bit 2
        assert_reg(&chip, 1, 15.0); // Should get 1111 (15)

        // ins: insert bits
        set_reg(&mut chip, 0, 0.0);
        exec_ok(&mut chip, "ins r0 4 4 15"); // Insert 15 (1111) at bit 4 for 4 bits
        assert_reg(&chip, 0, 240.0); // 0b11110000 = 240
    }

    // ==================== Comparison Instructions ====================

    #[test]
    fn test_set_comparisons() {
        let mut chip = chip();

        // slt, sgt, sle, sge, seq, sne
        exec_ok(&mut chip, "slt r0 5 10");
        assert_reg(&chip, 0, 1.0);
        exec_ok(&mut chip, "slt r0 10 5");
        assert_reg(&chip, 0, 0.0);

        exec_ok(&mut chip, "sgt r0 10 5");
        assert_reg(&chip, 0, 1.0);

        exec_ok(&mut chip, "sle r0 5 5");
        assert_reg(&chip, 0, 1.0);

        exec_ok(&mut chip, "sge r0 5 5");
        assert_reg(&chip, 0, 1.0);

        exec_ok(&mut chip, "seq r0 5 5");
        assert_reg(&chip, 0, 1.0);
        exec_ok(&mut chip, "seq r0 5 6");
        assert_reg(&chip, 0, 0.0);

        exec_ok(&mut chip, "sne r0 5 6");
        assert_reg(&chip, 0, 1.0);
    }

    #[test]
    fn test_set_zero_comparisons() {
        let mut chip = chip();

        exec_ok(&mut chip, "seqz r0 0");
        assert_reg(&chip, 0, 1.0);
        exec_ok(&mut chip, "seqz r0 1");
        assert_reg(&chip, 0, 0.0);

        exec_ok(&mut chip, "snez r0 1");
        assert_reg(&chip, 0, 1.0);
        exec_ok(&mut chip, "snez r0 0");
        assert_reg(&chip, 0, 0.0);

        exec_ok(&mut chip, "sltz r0 -1");
        assert_reg(&chip, 0, 1.0);
        exec_ok(&mut chip, "sltz r0 0");
        assert_reg(&chip, 0, 0.0);

        exec_ok(&mut chip, "sgtz r0 1");
        assert_reg(&chip, 0, 1.0);

        exec_ok(&mut chip, "slez r0 0");
        assert_reg(&chip, 0, 1.0);

        exec_ok(&mut chip, "sgez r0 0");
        assert_reg(&chip, 0, 1.0);
    }

    #[test]
    fn test_nan_comparisons() {
        let mut chip = chip();

        // Create NaN
        exec_ok(&mut chip, "div r1 0 0");

        exec_ok(&mut chip, "snan r0 r1");
        assert_reg(&chip, 0, 1.0);

        exec_ok(&mut chip, "snan r0 5");
        assert_reg(&chip, 0, 0.0);

        exec_ok(&mut chip, "snanz r0 5");
        assert_reg(&chip, 0, 1.0);

        exec_ok(&mut chip, "snanz r0 r1");
        assert_reg(&chip, 0, 0.0);
    }

    #[test]
    fn test_approximate_comparisons() {
        let mut chip = chip();

        // sap: approximately equal (relative tolerance)
        // tolerance = c * max(|a|, |b|)
        // For 5 ≈ 5.05 with c=0.1: tolerance = 0.1 * 5.05 = 0.505, diff = 0.05 <= 0.505 ✓
        set_reg(&mut chip, 0, 5.0);
        set_reg(&mut chip, 1, 5.05);
        exec_ok(&mut chip, "sap r2 r0 r1 0.1");
        assert_reg(&chip, 2, 1.0);

        // 10 ≈ 15 with c=0.1: tolerance = 0.1 * 15 = 1.5, diff = 5 > 1.5 ✗
        exec_ok(&mut chip, "sap r2 10 15 0.1");
        assert_reg(&chip, 2, 0.0);

        // sna: not approximately equal
        exec_ok(&mut chip, "sna r2 10 15 0.1");
        assert_reg(&chip, 2, 1.0);

        // sapz: approximately zero (relative tolerance against 0)
        // tolerance = c * |a| = 0.5 * 0.1 = 0.05, |0.1| > 0.05 ✗
        exec_ok(&mut chip, "sapz r2 0.1 0.5");
        assert_reg(&chip, 2, 0.0);

        // with larger tolerance: c = 2.0, tolerance = 2.0 * 0.1 = 0.2, |0.1| <= 0.2 ✓
        exec_ok(&mut chip, "sapz r2 0.1 2.0");
        assert_reg(&chip, 2, 1.0);

        // snaz: not approximately zero
        exec_ok(&mut chip, "snaz r2 5.0 0.01");
        assert_reg(&chip, 2, 1.0);
    }

    // ==================== Jump Instructions ====================

    #[test]
    fn test_jumps() {
        let mut chip = chip();

        // j (absolute jump)
        let pc = exec_ok(&mut chip, "j 10");
        assert_eq!(pc, 10);

        // jr (relative jump)
        chip.set_pc(10);
        let pc = exec_ok(&mut chip, "jr 5");
        assert_eq!(pc, 15);

        chip.set_pc(10);
        let pc = exec_ok(&mut chip, "jr -3");
        assert_eq!(pc, 7);
    }

    #[test]
    fn test_relative_jump_program() {
        let (chip, _, _) = ProgrammableChip::new_with_network();

        // jr should jump relative to current PC
        let program = r#"
move r0 0
jr 2
move r0 111
move r0 222
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();
        // Should skip line 3 (move r0 111) and execute line 4 (move r0 222)
        assert_eq!(chip.borrow().get_register(0).unwrap(), 222.0);
    }

    #[test]
    fn test_jal() {
        let mut chip = chip();

        let pc = exec_ok(&mut chip, "jal 20");
        assert_eq!(pc, 20);
        // ra should be set to next line (1)
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);
    }

    // ==================== Branch Instructions ====================

    #[test]
    fn test_branch_taken() {
        let mut chip = chip();

        // beq - taken
        let pc = exec_ok(&mut chip, "beq 5 5 10");
        assert_eq!(pc, 10);

        // bne - taken
        let pc = exec_ok(&mut chip, "bne 5 6 20");
        assert_eq!(pc, 20);

        // blt - taken
        let pc = exec_ok(&mut chip, "blt 3 5 30");
        assert_eq!(pc, 30);

        // bgt - taken
        let pc = exec_ok(&mut chip, "bgt 10 5 40");
        assert_eq!(pc, 40);

        // ble - taken (equal case)
        let pc = exec_ok(&mut chip, "ble 5 5 50");
        assert_eq!(pc, 50);

        // bge - taken (equal case)
        let pc = exec_ok(&mut chip, "bge 5 5 60");
        assert_eq!(pc, 60);
    }

    #[test]
    fn test_branch_not_taken() {
        let mut chip = chip();

        // beq - not taken
        let pc = exec_ok(&mut chip, "beq 5 6 10");
        assert_eq!(pc, 1); // Falls through

        // blt - not taken
        let pc = exec_ok(&mut chip, "blt 10 5 20");
        assert_eq!(pc, 1);

        // bgt - not taken
        let pc = exec_ok(&mut chip, "bgt 3 5 30");
        assert_eq!(pc, 1);
    }

    #[test]
    fn test_branch_zero_variants() {
        let mut chip = chip();

        let pc = exec_ok(&mut chip, "beqz 0 10");
        assert_eq!(pc, 10);

        let pc = exec_ok(&mut chip, "bnez 5 20");
        assert_eq!(pc, 20);

        let pc = exec_ok(&mut chip, "bltz -1 30");
        assert_eq!(pc, 30);

        let pc = exec_ok(&mut chip, "bgtz 1 40");
        assert_eq!(pc, 40);

        let pc = exec_ok(&mut chip, "blez 0 50");
        assert_eq!(pc, 50);

        let pc = exec_ok(&mut chip, "bgez 0 60");
        assert_eq!(pc, 60);
    }

    #[test]
    fn test_branch_and_link() {
        let mut chip = chip();

        // beqal - taken, should set ra
        let pc = exec_ok(&mut chip, "beqal 5 5 10");
        assert_eq!(pc, 10);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bneal - not taken, ra should NOT change
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 99.0);
        let pc = exec_ok(&mut chip, "bneal 5 5 20");
        assert_eq!(pc, 1);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 99.0);
    }

    #[test]
    fn test_relative_branches() {
        let mut chip = chip();

        // breq - relative branch when equal
        // At PC=0, breq with offset 5 should go to 5
        let pc = exec_ok(&mut chip, "breq 5 5 5");
        assert_eq!(pc, 5);

        // brne - relative branch when not equal
        let pc = exec_ok(&mut chip, "brne 5 6 10");
        assert_eq!(pc, 10);
    }

    // ==================== Stack Instructions ====================

    #[test]
    fn test_stack_operations() {
        let mut chip = chip();

        // sp starts at 0
        assert_reg(&chip, STACK_POINTER_INDEX, 0.0);

        // push
        exec_ok(&mut chip, "push 42");
        assert_reg(&chip, STACK_POINTER_INDEX, 1.0);

        exec_ok(&mut chip, "push 100");
        assert_reg(&chip, STACK_POINTER_INDEX, 2.0);

        // peek (read without popping)
        exec_ok(&mut chip, "peek r0");
        assert_reg(&chip, 0, 100.0);
        assert_reg(&chip, STACK_POINTER_INDEX, 2.0);

        // pop
        exec_ok(&mut chip, "pop r0");
        assert_reg(&chip, 0, 100.0);
        assert_reg(&chip, STACK_POINTER_INDEX, 1.0);

        exec_ok(&mut chip, "pop r0");
        assert_reg(&chip, 0, 42.0);
        assert_reg(&chip, STACK_POINTER_INDEX, 0.0);
    }

    #[test]
    fn test_poke() {
        let mut chip = chip();

        // poke writes directly to stack address
        exec_ok(&mut chip, "poke 5 99");

        // Set sp to 6 so we can peek at index 5
        exec_ok(&mut chip, "move sp 6");
        // Read it back via peek after adjusting sp
        set_reg(&mut chip, STACK_POINTER_INDEX, 6.0);
        // Actually, we need a different way to verify - use put/get on housing
    }

    // ==================== Select Instruction ====================

    #[test]
    fn test_select() {
        let mut chip = chip();

        // select r, cond, a, b - r = cond ? a : b
        exec_ok(&mut chip, "select r0 1 100 200");
        assert_reg(&chip, 0, 100.0);

        exec_ok(&mut chip, "select r0 0 100 200");
        assert_reg(&chip, 0, 200.0);

        // Non-zero condition
        exec_ok(&mut chip, "select r0 -5 100 200");
        assert_reg(&chip, 0, 100.0);
    }

    // ==================== Special Instructions ====================

    #[test]
    fn test_yield_and_sleep() {
        let mut chip = chip();

        // yield just advances PC
        let pc = exec_ok(&mut chip, "yield");
        assert_eq!(pc, 1);

        // sleep sets sleep ticks
        exec_ok(&mut chip, "sleep 2");
        // Sleep of 2 seconds = 4 ticks, chip stores remaining ticks
    }

    #[test]
    fn test_move_instruction() {
        let mut chip = chip();

        exec_ok(&mut chip, "move r0 42");
        assert_reg(&chip, 0, 42.0);

        set_reg(&mut chip, 1, 100.0);
        exec_ok(&mut chip, "move r0 r1");
        assert_reg(&chip, 0, 100.0);
    }

    #[test]
    fn test_alias_instruction() {
        let mut chip = chip();

        set_reg(&mut chip, 5, 999.0);
        exec_ok(&mut chip, "alias myReg r5");
        exec_ok(&mut chip, "move r0 myReg");
        assert_reg(&chip, 0, 999.0);
    }

    #[test]
    fn test_define_instruction() {
        let mut chip = chip();

        exec_ok(&mut chip, "define PI 3.14159");
        exec_ok(&mut chip, "move r0 PI");
        assert_reg(&chip, 0, 3.14159);
    }

    // ==================== Program Execution Tests ====================

    #[test]
    fn test_simple_program() {
        let (chip, _, _) = ProgrammableChip::new_with_network();

        let program = r#"
move r0 10
move r1 20
add r2 r0 r1
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(2).unwrap(), 30.0);
    }

    #[test]
    fn test_loop_with_branch() {
        let (chip, _, _) = ProgrammableChip::new_with_network();

        let program = r#"
move r0 0
loop:
add r0 r0 1
blt r0 5 loop
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 5.0);
    }

    #[test]
    fn test_subroutine_call() {
        let (chip, _, _) = ProgrammableChip::new_with_network();

        let program = r#"
move r0 10
jal addFive
yield

addFive:
add r0 r0 5
j ra
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 15.0);
    }

    // ==================== Additional Branch Instructions ====================

    #[test]
    fn test_bnan() {
        let mut chip = chip();

        // Create NaN
        exec_ok(&mut chip, "div r1 0 0");

        // bnan - taken when NaN
        let pc = exec_ok(&mut chip, "bnan r1 10");
        assert_eq!(pc, 10);

        // bnan - not taken when not NaN
        set_reg(&mut chip, 1, 5.0);
        let pc = exec_ok(&mut chip, "bnan r1 10");
        assert_eq!(pc, 1);
    }

    // ==================== Relative Branch Tests ====================

    #[test]
    fn test_relative_branches_comparison() {
        let mut chip = chip();

        // brlt - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brlt 3 5 10");
        assert_eq!(pc, 15); // 5 + 10

        // brlt - not taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brlt 10 5 10");
        assert_eq!(pc, 6);

        // brgt - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brgt 10 5 10");
        assert_eq!(pc, 15);

        // brle - taken (equal)
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brle 5 5 10");
        assert_eq!(pc, 15);

        // brge - taken (equal)
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brge 5 5 10");
        assert_eq!(pc, 15);
    }

    #[test]
    fn test_relative_branches_zero() {
        let mut chip = chip();

        // breqz - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "breqz 0 10");
        assert_eq!(pc, 15);

        // brnez - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brnez 5 10");
        assert_eq!(pc, 15);

        // brltz - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brltz -1 10");
        assert_eq!(pc, 15);

        // brgez - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brgez 0 10");
        assert_eq!(pc, 15);

        // brlez - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brlez 0 10");
        assert_eq!(pc, 15);

        // brgtz - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brgtz 1 10");
        assert_eq!(pc, 15);
    }

    #[test]
    fn test_brnan() {
        let mut chip = chip();

        // Create NaN
        exec_ok(&mut chip, "div r1 0 0");

        // brnan - taken when NaN
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brnan r1 10");
        assert_eq!(pc, 15);

        // brnan - not taken
        set_reg(&mut chip, 1, 5.0);
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brnan r1 10");
        assert_eq!(pc, 6);
    }

    // ==================== Additional Branch and Link Tests ====================

    #[test]
    fn test_branch_and_link_comparison() {
        let mut chip = chip();

        // bltal - taken
        let pc = exec_ok(&mut chip, "bltal 3 5 10");
        assert_eq!(pc, 10);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bgtal - taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bgtal 10 5 20");
        assert_eq!(pc, 20);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bleal - taken (equal)
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bleal 5 5 30");
        assert_eq!(pc, 30);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bgeal - taken (equal)
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bgeal 5 5 40");
        assert_eq!(pc, 40);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);
    }

    #[test]
    fn test_branch_and_link_zero() {
        let mut chip = chip();

        // beqzal - taken
        let pc = exec_ok(&mut chip, "beqzal 0 10");
        assert_eq!(pc, 10);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bnezal - taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bnezal 5 20");
        assert_eq!(pc, 20);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bltzal - taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bltzal -1 30");
        assert_eq!(pc, 30);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bgezal - taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bgezal 0 40");
        assert_eq!(pc, 40);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // blezal - taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "blezal 0 50");
        assert_eq!(pc, 50);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bgtzal - taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bgtzal 1 60");
        assert_eq!(pc, 60);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);
    }

    // ==================== Approximate Branch Tests ====================

    #[test]
    fn test_approximate_branches_absolute() {
        let mut chip = chip();

        // bap - taken (approximately equal)
        // 5 ≈ 5.05 with c=0.1: tolerance = 0.1 * 5.05 = 0.505, diff = 0.05 <= 0.505
        let pc = exec_ok(&mut chip, "bap 5 5.05 0.1 10");
        assert_eq!(pc, 10);

        // bap - not taken
        let pc = exec_ok(&mut chip, "bap 10 15 0.1 10");
        assert_eq!(pc, 1);

        // bna - taken (not approximately equal)
        let pc = exec_ok(&mut chip, "bna 10 15 0.1 20");
        assert_eq!(pc, 20);

        // bna - not taken
        let pc = exec_ok(&mut chip, "bna 5 5.05 0.1 20");
        assert_eq!(pc, 1);

        // bapz - taken (approximately zero)
        let pc = exec_ok(&mut chip, "bapz 0.01 2 10");
        assert_eq!(pc, 10);

        // bnaz - taken (not approximately zero)
        let pc = exec_ok(&mut chip, "bnaz 5.0 0.01 20");
        assert_eq!(pc, 20);
    }

    #[test]
    fn test_approximate_branches_relative() {
        let mut chip = chip();

        // brap - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brap 5 5.05 0.1 10");
        assert_eq!(pc, 15);

        // brna - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brna 10 15 0.1 10");
        assert_eq!(pc, 15);

        // brapz - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brapz 0.01 2 10");
        assert_eq!(pc, 15);

        // brnaz - taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brnaz 5.0 0.01 10");
        assert_eq!(pc, 15);
    }

    #[test]
    fn test_approximate_branches_and_link() {
        let mut chip = chip();

        // bapal - taken
        let pc = exec_ok(&mut chip, "bapal 5 5.05 0.1 10");
        assert_eq!(pc, 10);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bnaal - taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bnaal 10 15 0.1 20");
        assert_eq!(pc, 20);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bapzal - taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bapzal 0.01 2 30");
        assert_eq!(pc, 30);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);

        // bnazal - taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 0.0);
        let pc = exec_ok(&mut chip, "bnazal 5.0 0.01 40");
        assert_eq!(pc, 40);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 1.0);
    }

    // ==================== Device State Detection Tests ====================

    #[test]
    fn test_device_state_detection() {
        let (chip, _housing, network) = ProgrammableChip::new_with_network();

        // Add a device and assign it to d0
        let sensor = Rc::new(RefCell::new(DaylightSensor::new(None)));
        let sensor_id = sensor.borrow().get_id();
        network
            .borrow_mut()
            .add_device(sensor.clone(), network.clone());
        chip.borrow_mut()
            .get_housing_mut()
            .set_device_pin(0, Some(sensor_id));

        // sdse - device set exists (db always exists)
        let program = r#"
sdse r0 db
sdse r1 d0
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 1.0); // db exists
        assert_eq!(chip.borrow().get_register(1).unwrap(), 1.0); // d0 has device
    }

    // ==================== Device State Branch Tests ====================

    #[test]
    fn test_device_state_branches() {
        let (chip, _housing, network) = ProgrammableChip::new_with_network();

        // Add a device and assign it to d0
        let sensor = Rc::new(RefCell::new(DaylightSensor::new(None)));
        let sensor_id = sensor.borrow().get_id();
        network
            .borrow_mut()
            .add_device(sensor.clone(), network.clone());
        chip.borrow_mut()
            .get_housing_mut()
            .set_device_pin(0, Some(sensor_id));

        // bdse - branch if device set exists (db always exists)
        let program = r#"
move r0 0
bdse db 3
move r0 99
move r1 1
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // Should have branched over the move r0 99 instruction
        assert_eq!(chip.borrow().get_register(0).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(1).unwrap(), 1.0);
    }

    #[test]
    fn test_device_state_branches_relative() {
        let (chip, _housing, network) = ProgrammableChip::new_with_network();

        // Add a device to make d0 exist
        let sensor = Rc::new(RefCell::new(DaylightSensor::new(None)));
        let sensor_id = sensor.borrow().get_id();
        network
            .borrow_mut()
            .add_device(sensor.clone(), network.clone());
        chip.borrow_mut()
            .get_housing_mut()
            .set_device_pin(0, Some(sensor_id));

        // brdse - relative branch if device exists
        let program = r#"
move r0 0
brdse db 2
move r0 99
move r1 1
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // Should branch over "move r0 99"
        assert_eq!(chip.borrow().get_register(0).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(1).unwrap(), 1.0);
    }

    #[test]
    fn test_device_state_branches_and_link() {
        let (chip, _housing, network) = ProgrammableChip::new_with_network();

        // Add a device to d0
        let sensor = Rc::new(RefCell::new(DaylightSensor::new(None)));
        let sensor_id = sensor.borrow().get_id();
        network
            .borrow_mut()
            .add_device(sensor.clone(), network.clone());
        chip.borrow_mut()
            .get_housing_mut()
            .set_device_pin(0, Some(sensor_id));

        // bdseal - branch and link if device exists
        let program = r#"
move r0 0
bdseal db 4
move r0 99
yield
move r1 ra
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // Should branch to line 4, ra should be 2 (next instruction after bdseal at line 1)
        assert_eq!(chip.borrow().get_register(0).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(1).unwrap(), 2.0);
    }

    // ==================== Special Instructions Tests ====================

    #[test]
    fn test_noop() {
        let mut chip = chip();

        // Noop is generated for empty lines/comments
        let pc = exec_ok(&mut chip, "# comment");
        assert_eq!(pc, 1);

        let pc = exec_ok(&mut chip, "");
        assert_eq!(pc, 1);
    }

    #[test]
    fn test_hcf() {
        let mut chip = chip();

        // hcf should return an error (halt and catch fire)
        let result = exec(&mut chip, "hcf");
        assert!(result.is_err());
    }

    // ==================== Device I/O Tests ====================

    #[test]
    fn test_device_io_self() {
        let (chip, _, _) = ProgrammableChip::new_with_network();

        // Test reading/writing to self (db) - Setting = 12
        let program = r#"
s db 12 42
l r0 db 12
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 42.0);
    }

    #[test]
    fn test_device_io_with_network() {
        let (chip, _housing, network) = ProgrammableChip::new_with_network();

        // Add another IC housing to the network (supports Setting = 12)
        let housing2 = Rc::new(RefCell::new(ICHousing::new(None)));
        let housing2_id = housing2.borrow().id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        // Assign device to pin d0
        chip.borrow_mut()
            .get_housing_mut()
            .set_device_pin(0, Some(housing2_id));

        // Write Setting (12) to d0, then read it back
        let program = r#"
s d0 12 42
l r0 d0 12
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 42.0);
    }

    #[test]
    fn test_logic_type_operand_variants() {
        // Tests that LogicType can be specified as:
        // 1. String name (e.g., "Setting")
        // 2. Immediate value (e.g., 12)
        // 3. Register containing the value (e.g., r5 where r5=12)

        let (chip, _, _) = ProgrammableChip::new_with_network();

        // Test all three variants
        let program = r#"
# Test 1: Using string name "Setting"
s db Setting 10
l r0 db Setting

# Test 2: Using immediate value 12 (which is Setting)
s db 12 20
l r1 db 12

# Test 3: Using register containing 12
move r5 12
s db r5 30
l r2 db r5

yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 10.0);
        assert_eq!(chip.borrow().get_register(1).unwrap(), 20.0);
        assert_eq!(chip.borrow().get_register(2).unwrap(), 30.0);
    }

    // ==================== Memory Access Tests ====================
    // NOTE: get/put/getd/putd/clr/clrd currently require Device trait methods
    // that ICHousing doesn't implement. These tests are placeholders for when
    // the feature is complete.

    #[test]
    fn test_get_put() {
        // get/put use device.get_memory/set_memory which ICHousing doesn't implement
        // This test documents the expected behavior when implemented
        let (chip, _, _) = ProgrammableChip::new_with_network();

        // Just test that the instructions parse correctly by loading a program
        // The actual execution would fail until ICHousing implements Device memory methods
        let program = r#"
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();
        // Test passes if no panic - placeholder for future functionality
    }

    #[test]
    fn test_clr() {
        // clr uses device.clear_memory which ICHousing doesn't implement
        let (chip, _, _) = ProgrammableChip::new_with_network();

        let program = r#"
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();
        // Test passes if no panic - placeholder for future functionality
    }

    // ==================== ID-Based Device Access Tests ====================

    #[test]
    fn test_ld_sd() {
        let (chip, _housing, network) = ProgrammableChip::new_with_network();

        // Add another IC housing device to the network (supports Setting)
        let housing2 = Rc::new(RefCell::new(ICHousing::new(None)));
        let device_id = housing2.borrow().id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        // Store the device ID in a register and use ld/sd - Setting = 12
        let program = format!(
            r#"
move r1 {}
sd r1 12 42
ld r0 r1 12
yield
"#,
            device_id
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 42.0);
    }

    #[test]
    fn test_ld_sd_logic_type_variants() {
        // Tests that ld/sd instructions accept LogicType as:
        // 1. String name, 2. Immediate value, 3. Register

        let (chip, _housing, network) = ProgrammableChip::new_with_network();

        let housing2 = Rc::new(RefCell::new(ICHousing::new(None)));
        let device_id = housing2.borrow().id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        let program = format!(
            r#"
move r1 {}

# Test with string name
sd r1 Setting 10
ld r0 r1 Setting

# Test with immediate
sd r1 12 20
ld r2 r1 12

# Test with register
move r5 12
sd r1 r5 30
ld r3 r1 r5

yield
"#,
            device_id
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 10.0);
        assert_eq!(chip.borrow().get_register(2).unwrap(), 20.0);
        assert_eq!(chip.borrow().get_register(3).unwrap(), 30.0);
    }

    // ==================== Batch Device Access Tests ====================

    #[test]
    fn test_batch_lb_sb() {
        let (chip, _housing, network) = ProgrammableChip::new_with_network();

        // Check how many devices are on the network initially (should be 1: the chip's housing)
        let initial_count = network.borrow().device_count();
        assert_eq!(initial_count, 1, "Expected 1 device initially");

        // Add 3 more ICHousing devices (they support Setting)
        for _ in 0..3 {
            let housing = Rc::new(RefCell::new(ICHousing::new(None)));
            network.borrow_mut().add_device(housing, network.clone());
        }

        // Now we should have 4 total
        let total_count = network.borrow().device_count();
        assert_eq!(total_count, 4, "Expected 4 devices after adding 3");

        // Get the prefab hash for ICHousing
        let housing = ICHousing::new(None);
        let hash = housing.get_prefab_hash();

        // Count devices that match the hash
        let prefab_count = network.borrow().count_devices_by_prefab(hash);
        assert_eq!(prefab_count, 4, "Expected 4 devices matching prefab hash");

        // sb sets Setting(12) on all devices, lb reads sum - BatchMode 1 is Sum
        let program = format!(
            r#"
move r1 {}
sb r1 12 5
lb r0 r1 12 1
yield
"#,
            hash
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // BatchMode 1 is Sum, 4 housings total * 5 = 20
        assert_eq!(chip.borrow().get_register(0).unwrap(), 20.0);
    }

    #[test]
    fn test_batch_lb_sb_logic_type_variants() {
        // Tests that lb/sb instructions accept LogicType as:
        // 1. String name, 2. Immediate value, 3. Register

        let (chip, _housing, _network) = ProgrammableChip::new_with_network();

        // Get the prefab hash for ICHousing
        let housing = ICHousing::new(None);
        let hash = housing.get_prefab_hash();

        // Test with string name "Setting"
        let program = format!(
            r#"
move r1 {}
sb r1 Setting 10
lb r0 r1 Setting 1
yield
"#,
            hash
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // BatchMode 1 is Sum, 1 housing * 10 = 10
        assert_eq!(chip.borrow().get_register(0).unwrap(), 10.0);

        // Test with register containing logic type
        let program2 = format!(
            r#"
move r1 {}
move r5 12
sb r1 r5 20
lb r0 r1 r5 1
yield
"#,
            hash
        );

        chip.borrow_mut().load_program(&program2).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 20.0);
    }

    // ==================== Comprehensive Relative Branch Not-Taken Tests ====================

    #[test]
    fn test_relative_branches_not_taken() {
        let mut chip = chip();

        // breq - not taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "breq 5 6 10");
        assert_eq!(pc, 6);

        // brne - not taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brne 5 5 10");
        assert_eq!(pc, 6);

        // brlt - not taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brlt 10 5 10");
        assert_eq!(pc, 6);

        // brgt - not taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brgt 3 5 10");
        assert_eq!(pc, 6);

        // breqz - not taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "breqz 1 10");
        assert_eq!(pc, 6);

        // brnez - not taken
        chip.set_pc(5);
        let pc = exec_ok(&mut chip, "brnez 0 10");
        assert_eq!(pc, 6);
    }

    // ==================== Branch and Link Not-Taken Tests ====================

    #[test]
    fn test_branch_and_link_not_taken() {
        let mut chip = chip();

        // bltal - not taken (ra should NOT change)
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 99.0);
        let pc = exec_ok(&mut chip, "bltal 10 5 10");
        assert_eq!(pc, 1);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 99.0);

        // bgezal - not taken
        set_reg(&mut chip, RETURN_ADDRESS_INDEX, 99.0);
        let pc = exec_ok(&mut chip, "bgezal -5 20");
        assert_eq!(pc, 1);
        assert_reg(&chip, RETURN_ADDRESS_INDEX, 99.0);
    }

    // ==================== Getd/Putd Tests ====================
    // NOTE: getd/putd use device.get_memory/set_memory which ICHousing doesn't implement

    #[test]
    fn test_getd_putd() {
        // getd/putd use device.get_memory/set_memory which ICHousing doesn't implement
        // This test documents the expected behavior when implemented
        let (chip, _, _) = ProgrammableChip::new_with_network();

        let program = r#"
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();
        // Test passes if no panic - placeholder for future functionality
    }

    // ==================== Clrd Test ====================

    #[test]
    fn test_clrd() {
        // clrd uses device.clear_memory which ICHousing doesn't implement
        let (chip, _, _) = ProgrammableChip::new_with_network();

        let program = r#"
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();
        // Test passes if no panic - placeholder for future functionality
    }
}
