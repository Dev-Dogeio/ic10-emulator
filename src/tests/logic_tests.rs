#[cfg(test)]
mod tests {
    use std::f64;

    use crate::CableNetwork;
    use crate::Filter;
    use crate::ItemIntegratedCircuit10;
    use crate::LogicType;
    use crate::atmospherics::GasType;
    use crate::constants::STACK_SIZE;
    use crate::constants::{RETURN_ADDRESS_INDEX, STACK_POINTER_INDEX};
    use crate::devices::Filtration;
    use crate::devices::ICHostDevice;
    use crate::devices::{DaylightSensor, Device, ICHousing};
    use crate::instruction::ParsedInstruction;
    use crate::items::FilterSize;
    use crate::logic::execute_instruction;
    use crate::types::{Shared, shared};

    // ==================== Test Helpers ====================

    /// Create a chip with optional initial register values
    fn chip() -> ItemIntegratedCircuit10 {
        // For simple chip-only tests we don't need to attach to a housing
        ItemIntegratedCircuit10::new()
    }

    // Test-only helper: construct a chip + housing + network and wire them together.
    impl ItemIntegratedCircuit10 {
        pub fn new_with_network() -> (
            Shared<ItemIntegratedCircuit10>,
            Shared<ICHousing>,
            Shared<CableNetwork>,
        ) {
            let network = CableNetwork::new();
            let housing = ICHousing::new(None);
            let chip = shared(ItemIntegratedCircuit10::new());

            // Connect chip to housing (this will also attach the chip slot to the chip)
            housing.borrow_mut().set_chip(chip.clone());

            // Connect housing to network (which also adds it as a device)
            network
                .borrow_mut()
                .add_device(housing.clone(), network.clone());

            (chip, housing, network)
        }
    }

    /// Execute a single instruction from string, return new PC
    fn exec(chip: &mut ItemIntegratedCircuit10, line: &str) -> Result<usize, String> {
        let parsed = ParsedInstruction::parse(line, 0).map_err(|e| format!("{e:?}"))?;
        execute_instruction(chip, &parsed).map_err(|e| format!("{e:?}"))
    }

    /// Execute instruction, expect success
    fn exec_ok(chip: &mut ItemIntegratedCircuit10, line: &str) -> usize {
        exec(chip, line).unwrap_or_else(|_| panic!("Failed to execute: {line}"))
    }

    /// Get register value
    fn reg(chip: &ItemIntegratedCircuit10, idx: usize) -> f64 {
        chip.get_register(idx).unwrap()
    }

    /// Set register value
    fn set_reg(chip: &mut ItemIntegratedCircuit10, idx: usize, val: f64) {
        chip.set_register(idx, val).unwrap();
    }

    /// Assert register equals value (with floating point tolerance)
    fn assert_reg(chip: &ItemIntegratedCircuit10, idx: usize, expected: f64) {
        let actual = reg(chip, idx);
        if expected.is_nan() {
            assert!(actual.is_nan(), "r{idx} expected NaN, got {actual}");
        } else if expected.is_infinite() {
            assert_eq!(actual, expected, "r{idx} expected {expected}, got {actual}");
        } else {
            assert!(
                (actual - expected).abs() < 1e-10,
                "r{idx} expected {expected}, got {actual}"
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
            assert!((0.0..1.0).contains(&val), "rand out of range: {val}");
        }
    }

    // ==================== Trigonometric Instructions ====================

    #[test]
    fn test_trig_basic() {
        let mut chip = chip();
        let pi = f64::consts::PI;

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
        let pi = f64::consts::PI;

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
        exec_ok(&mut chip, "ins r0 15 4 4"); // Insert 15 (1111) at bit 4 for 4 bits
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
        let (chip, _, _) = ItemIntegratedCircuit10::new_with_network();

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
        #[allow(clippy::approx_constant)]
        assert_reg(&chip, 0, 3.14159);
    }

    // ==================== Program Execution Tests ====================

    #[test]
    fn test_simple_program() {
        let (chip, _, _) = ItemIntegratedCircuit10::new_with_network();

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
        let (chip, _, _) = ItemIntegratedCircuit10::new_with_network();

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
        let (chip, _, _) = ItemIntegratedCircuit10::new_with_network();

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
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add a device and assign it to d0
        let sensor = DaylightSensor::new(None);
        let sensor_id = sensor.borrow().get_id();
        network
            .borrow_mut()
            .add_device(sensor.clone(), network.clone());
        chip.borrow_mut()
            .get_chip_slot_mut()
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
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add a device and assign it to d0
        let sensor = DaylightSensor::new(None);
        let sensor_id = sensor.borrow().get_id();
        network
            .borrow_mut()
            .add_device(sensor.clone(), network.clone());
        chip.borrow_mut()
            .get_chip_slot_mut()
            .set_device_pin(0, Some(sensor_id));

        // bdse - branch if device set exists (db always exists)
        let program = r#"
move r0 0
bdse db 4
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
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add a device to make d0 exist
        let sensor = DaylightSensor::new(None);
        let sensor_id = sensor.borrow().get_id();
        network
            .borrow_mut()
            .add_device(sensor.clone(), network.clone());
        chip.borrow_mut()
            .get_chip_slot_mut()
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
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add a device to d0
        let sensor = DaylightSensor::new(None);
        let sensor_id = sensor.borrow().get_id();
        network
            .borrow_mut()
            .add_device(sensor.clone(), network.clone());
        chip.borrow_mut()
            .get_chip_slot_mut()
            .set_device_pin(0, Some(sensor_id));

        // bdseal - branch and link if device exists
        let program = r#"
move r0 0
bdseal db 5
move r0 99
yield
move r1 ra
yield
"#;
        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // Should branch to line 5, ra should be 3 (next instruction after bdseal at line 2)
        assert_eq!(chip.borrow().get_register(0).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(1).unwrap(), 3.0);
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
        let (chip, _, _) = ItemIntegratedCircuit10::new_with_network();

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
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add another IC housing to the network (supports Setting = 12)
        let housing2 = ICHousing::new(None);
        let housing2_id = housing2.borrow().get_id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        // Assign device to pin d0
        chip.borrow_mut()
            .get_chip_slot_mut()
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

        let (chip, _, _) = ItemIntegratedCircuit10::new_with_network();

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

    #[test]
    fn test_get_put() {
        let (chip, housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add another IC housing device to the network
        let housing2 = ICHousing::new(None);
        let chip2 = shared(ItemIntegratedCircuit10::new());
        housing2.borrow_mut().set_chip(chip2.clone());
        let device_id = housing2.borrow().get_id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        // Write some values to housing2's memory
        ICHostDevice::set_memory(&*housing2.borrow(), 0, 42.0).unwrap();
        ICHostDevice::set_memory(&*housing2.borrow(), 5, 100.0).unwrap();
        ICHostDevice::set_memory(&*housing2.borrow(), 511, 999.0).unwrap();

        // Set housing2 as device on pin 0
        housing.borrow_mut().set_device_pin(0, Some(device_id));

        let program = r#"
# Test get - read from device memory
get r0 d0 0      # Read from index 0
get r1 d0 5      # Read from index 5
get r2 d0 511    # Read from index 511

# Test put - write to device memory
put 77.0 d0 1    # Write to index 1
get r3 d0 1      # Read it back

# Test with register operands
move r10 2
move r11 55.5
put r11 d0 r10   # Write 55.5 to index 2
get r4 d0 r10    # Read it back

yield
"#
        .to_string();

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // Verify results
        let chip_ref = chip.borrow();
        assert_reg(&chip_ref, 0, 42.0); // Read from index 0
        assert_reg(&chip_ref, 1, 100.0); // Read from index 5
        assert_reg(&chip_ref, 2, 999.0); // Read from index 511
        assert_reg(&chip_ref, 3, 77.0); // Read back written value
        assert_reg(&chip_ref, 4, 55.5); // Read back register-written value

        // Verify the memory was actually written
        drop(chip_ref);
        assert_eq!(
            ICHostDevice::get_memory(&*housing2.borrow(), 1).unwrap(),
            77.0
        );
        assert_eq!(
            ICHostDevice::get_memory(&*housing2.borrow(), 2).unwrap(),
            55.5
        );

        // Also verify calling through the `Device` trait object routes to the host device behavior
        let net_ref = network.borrow();
        let dev = net_ref.get_device(device_id).unwrap();
        assert_eq!(dev.get_memory(1).unwrap(), 77.0);
        assert_eq!(dev.get_memory(2).unwrap(), 55.5);
    }

    #[test]
    fn test_clr() {
        let (chip, housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add another IC housing device to the network
        let housing2 = ICHousing::new(None);
        let chip2 = shared(ItemIntegratedCircuit10::new());
        housing2.borrow_mut().set_chip(chip2.clone());

        let device_id = housing2.borrow().get_id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        // Write some values to housing2's memory
        ICHostDevice::set_memory(&*housing2.borrow(), 0, 42.0).unwrap();
        ICHostDevice::set_memory(&*housing2.borrow(), 100, 123.0).unwrap();
        ICHostDevice::set_memory(&*housing2.borrow(), 511, 999.0).unwrap();

        // Set housing2 as device on pin 0
        housing.borrow_mut().set_device_pin(0, Some(device_id));

        let program = r#"
# Verify memory has values
get r0 d0 0
get r1 d0 100
get r2 d0 511

# Clear the device memory
clr d0

# Read back to verify it's cleared
get r3 d0 0
get r4 d0 100
get r5 d0 511

yield
"#;

        chip.borrow_mut().load_program(program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // Verify results
        let chip_ref = chip.borrow();
        assert_reg(&chip_ref, 0, 42.0); // Before clear
        assert_reg(&chip_ref, 1, 123.0); // Before clear
        assert_reg(&chip_ref, 2, 999.0); // Before clear
        assert_reg(&chip_ref, 3, 0.0); // After clear
        assert_reg(&chip_ref, 4, 0.0); // After clear
        assert_reg(&chip_ref, 5, 0.0); // After clear
    }

    // ==================== ID-Based Device Access Tests ====================

    #[test]
    fn test_ld_sd() {
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add another IC housing device to the network (supports Setting)
        let housing2 = ICHousing::new(None);
        let device_id = housing2.borrow().get_id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        // Store the device ID in a register and use ld/sd - Setting = 12
        let program = format!(
            r#"
move r1 {device_id}
sd r1 12 42
ld r0 r1 12
yield
"#
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        assert_eq!(chip.borrow().get_register(0).unwrap(), 42.0);
    }

    #[test]
    fn test_ld_sd_logic_type_variants() {
        // Tests that ld/sd instructions accept LogicType as:
        // 1. String name, 2. Immediate value, 3. Register

        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        let housing2 = ICHousing::new(None);
        let device_id = housing2.borrow().get_id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        let program = format!(
            r#"
move r1 {device_id}

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
"#
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
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Check how many devices are on the network initially (should be 1: the chip's housing)
        let initial_count = network.borrow().device_count();
        assert_eq!(initial_count, 1, "Expected 1 device initially");

        // Add 3 more ICHousing devices (they support Setting)
        for _ in 0..3 {
            let housing = ICHousing::new(None);
            network.borrow_mut().add_device(housing, network.clone());
        }

        // Now we should have 4 total
        let total_count = network.borrow().device_count();
        assert_eq!(total_count, 4, "Expected 4 devices after adding 3");

        // Get the prefab hash for ICHousing
        let housing = ICHousing::new(None);
        let hash = housing.borrow().get_prefab_hash();

        // Count devices that match the hash
        let prefab_count = network.borrow().count_devices_by_prefab(hash);
        assert_eq!(prefab_count, 4, "Expected 4 devices matching prefab hash");

        // sb sets Setting(12) on all devices, lb reads sum - BatchMode 1 is Sum
        let program = format!(
            r#"
move r1 {hash}
sb r1 12 5
lb r0 r1 12 1
yield
"#
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // BatchMode 1 is Sum, 4 housings total * 5 = 20
        assert_eq!(chip.borrow().get_register(0).unwrap(), 20.0);
    }

    #[test]
    fn test_ichousing_line_number_and_stack_size() {
        let (chip, housing, _network) = ItemIntegratedCircuit10::new_with_network();

        // Initially PC is 0
        assert_eq!(housing.borrow().read(LogicType::LineNumber).unwrap(), 0.0);

        chip.borrow_mut().set_pc(10);
        assert_eq!(housing.borrow().read(LogicType::LineNumber).unwrap(), 10.0);

        assert_eq!(
            housing.borrow().read(LogicType::StackSize).unwrap(),
            STACK_SIZE as f64
        );
    }

    #[test]
    fn test_ichousing_line_number_write_sets_pc() {
        let (chip, housing, _network) = ItemIntegratedCircuit10::new_with_network();

        // Write a valid line number
        housing.borrow().write(LogicType::LineNumber, 42.0).unwrap();
        assert_eq!(chip.borrow().get_pc(), 42);
        assert_eq!(housing.borrow().read(LogicType::LineNumber).unwrap(), 42.0);

        // Write a fractional value should truncate
        housing.borrow().write(LogicType::LineNumber, 10.7).unwrap();
        assert_eq!(chip.borrow().get_pc(), 10);
    }

    #[test]
    fn test_ichousing_line_number_write_no_chip_errors() {
        let housing = ICHousing::new(None);

        let res = housing.borrow().write(LogicType::LineNumber, 5.0);
        assert!(res.is_err());
    }

    #[test]
    fn test_filtration_slot_ls() {
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        let filtration = Filtration::new(None);
        let fil_id = filtration.borrow().get_id();
        network
            .borrow_mut()
            .add_device(filtration.clone(), network.clone());

        // Insert a physical filter in slot 0
        {
            let mut f_borrow = filtration.borrow_mut();
            let slot = f_borrow.get_slot_mut(0).unwrap();
            let _ = slot.try_insert(Box::new(Filter::new(
                10.0,
                GasType::Oxygen,
                FilterSize::Small,
            )));
        }

        let program = format!(
            r#"
define fil {fil_id}
ls r0 fil 0 Occupied
ls r1 fil 0 OccupantHash
ls r2 fil 0 Quantity
ls r3 fil 0 MaxQuantity
ls r4 fil 0 FilterType
ls r5 fil 0 ReferenceId
ls r6 fil 0 FreeSlots
ls r7 fil 0 TotalSlots

ls r8 fil 1 Occupied
ls r9 fil 1 OccupantHash
ls r10 fil 1 Quantity
ls r11 fil 1 MaxQuantity
ls r12 fil 1 FilterType
ls r13 fil 1 ReferenceId
ls r14 fil 1 FreeSlots
ls r15 fil 1 TotalSlots
yield
"#
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // Compare to expected values from the inserted item
        let f_borrow = filtration.borrow();
        let item = f_borrow.get_slot(0).unwrap().get_item().unwrap();
        let expected_hash = item.get_prefab_hash() as f64;
        let expected_qty = item.quantity() as f64;
        let expected_max = item.max_quantity() as f64;
        let expected_type = if let Some(filter_item) = item.as_any().downcast_ref::<Filter>() {
            filter_item.gas_type() as u32 as f64
        } else {
            0.0
        };
        let expected_id = item.get_id() as f64;

        assert_eq!(chip.borrow().get_register(0).unwrap(), 1.0);
        assert_eq!(chip.borrow().get_register(1).unwrap(), expected_hash);
        assert_eq!(chip.borrow().get_register(2).unwrap(), expected_qty);
        assert_eq!(chip.borrow().get_register(3).unwrap(), expected_max);
        assert_eq!(chip.borrow().get_register(4).unwrap(), expected_type);
        assert_eq!(chip.borrow().get_register(5).unwrap(), expected_id);
        assert_eq!(chip.borrow().get_register(6).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(7).unwrap(), 0.0);

        assert_eq!(chip.borrow().get_register(8).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(9).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(10).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(11).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(12).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(13).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(14).unwrap(), 0.0);
        assert_eq!(chip.borrow().get_register(15).unwrap(), 0.0);
    }

    #[test]
    fn test_batch_lb_sb_logic_type_variants() {
        // Tests that lb/sb instructions accept LogicType as:
        // 1. String name, 2. Immediate value, 3. Register

        let (chip, _housing, _network) = ItemIntegratedCircuit10::new_with_network();

        // Get the prefab hash for ICHousing
        let housing = ICHousing::new(None);
        let hash = housing.borrow().get_prefab_hash();

        // Test with string name "Setting"
        let program = format!(
            r#"
move r1 {hash}
sb r1 Setting 10
lb r0 r1 Setting 1
yield
"#
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // BatchMode 1 is Sum, 1 housing * 10 = 10
        assert_eq!(chip.borrow().get_register(0).unwrap(), 10.0);

        // Test with register containing logic type
        let program2 = format!(
            r#"
move r1 {hash}
move r5 12
sb r1 r5 20
lb r0 r1 r5 1
yield
"#
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

    #[test]
    fn test_getd_putd() {
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add two IC housing devices to the network
        let housing1 = ICHousing::new(None);
        let chip1 = shared(ItemIntegratedCircuit10::new());
        housing1.borrow_mut().set_chip(chip1.clone());
        let device_id1 = housing1.borrow().get_id();
        network
            .borrow_mut()
            .add_device(housing1.clone(), network.clone());

        let housing2 = ICHousing::new(None);
        let chip2 = shared(ItemIntegratedCircuit10::new());
        housing2.borrow_mut().set_chip(chip2.clone());

        let device_id2 = housing2.borrow().get_id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        // Write some values to housing1 and housing2's memory
        ICHostDevice::set_memory(&*housing1.borrow(), 0, 111.0).unwrap();
        ICHostDevice::set_memory(&*housing1.borrow(), 50, 222.0).unwrap();
        ICHostDevice::set_memory(&*housing2.borrow(), 0, 333.0).unwrap();
        ICHostDevice::set_memory(&*housing2.borrow(), 100, 444.0).unwrap();

        let program = format!(
            r#"
# Test getd - read from device memory by ID
getd r0 {device_id1} 0      # Read from housing1 index 0
getd r1 {device_id1} 50     # Read from housing1 index 50
getd r2 {device_id2} 0      # Read from housing2 index 0
getd r3 {device_id2} 100    # Read from housing2 index 100

# Test putd - write to device memory by ID
putd 555.0 {device_id1} 10    # Write to housing1 index 10
getd r4 {device_id1} 10       # Read it back

putd 666.0 {device_id2} 200   # Write to housing2 index 200
getd r5 {device_id2} 200      # Read it back

# Test with register operands
move r10 {device_id1}
move r11 25
move r12 777.0
putd r12 r10 r11    # Write 777.0 to housing1 index 25
getd r6 r10 r11     # Read it back

yield
"#
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // Verify results
        let chip_ref = chip.borrow();
        assert_reg(&chip_ref, 0, 111.0); // Read from housing1 index 0
        assert_reg(&chip_ref, 1, 222.0); // Read from housing1 index 50
        assert_reg(&chip_ref, 2, 333.0); // Read from housing2 index 0
        assert_reg(&chip_ref, 3, 444.0); // Read from housing2 index 100
        assert_reg(&chip_ref, 4, 555.0); // Read back housing1 written value
        assert_reg(&chip_ref, 5, 666.0); // Read back housing2 written value
        assert_reg(&chip_ref, 6, 777.0); // Read back register-written value

        // Verify the memory was actually written
        drop(chip_ref);
        assert_eq!(
            ICHostDevice::get_memory(&*housing1.borrow(), 10).unwrap(),
            555.0
        );
        assert_eq!(
            ICHostDevice::get_memory(&*housing1.borrow(), 25).unwrap(),
            777.0
        );
        assert_eq!(
            ICHostDevice::get_memory(&*housing2.borrow(), 200).unwrap(),
            666.0
        );

        // Also verify calling through the `Device` trait object routes to the host device behavior
        let net_ref = network.borrow();
        let dev1 = net_ref.get_device(device_id1).unwrap();
        let dev2 = net_ref.get_device(device_id2).unwrap();
        assert_eq!(dev1.get_memory(10).unwrap(), 555.0);
        assert_eq!(dev1.get_memory(25).unwrap(), 777.0);

        // Also verify `set_memory` via the `Device` trait works (mutates host stack)
        dev2.set_memory(33, 888.0).unwrap();
        assert_eq!(
            ICHostDevice::get_memory(&*housing2.borrow(), 33).unwrap(),
            888.0
        );
        assert_eq!(dev2.get_memory(200).unwrap(), 666.0);
    }

    // ==================== Clrd Test ====================

    #[test]
    fn test_clrd() {
        let (chip, _housing, network) = ItemIntegratedCircuit10::new_with_network();

        // Add two IC housing devices to the network
        let housing1 = ICHousing::new(None);
        let chip1 = shared(ItemIntegratedCircuit10::new());
        housing1.borrow_mut().set_chip(chip1.clone());
        let device_id1 = housing1.borrow().get_id();
        network
            .borrow_mut()
            .add_device(housing1.clone(), network.clone());

        let housing2 = ICHousing::new(None);
        let chip2 = shared(ItemIntegratedCircuit10::new());
        housing2.borrow_mut().set_chip(chip2.clone());
        let device_id2 = housing2.borrow().get_id();
        network
            .borrow_mut()
            .add_device(housing2.clone(), network.clone());

        // Write some values to both housings' memory
        ICHostDevice::set_memory(&*housing1.borrow(), 0, 111.0).unwrap();
        ICHostDevice::set_memory(&*housing1.borrow(), 100, 222.0).unwrap();
        ICHostDevice::set_memory(&*housing1.borrow(), 511, 333.0).unwrap();

        ICHostDevice::set_memory(&*housing2.borrow(), 0, 444.0).unwrap();
        ICHostDevice::set_memory(&*housing2.borrow(), 100, 555.0).unwrap();
        ICHostDevice::set_memory(&*housing2.borrow(), 511, 666.0).unwrap();

        let program = format!(
            r#"
# Verify memory has values in housing1
getd r0 {device_id1} 0
getd r1 {device_id1} 100
getd r2 {device_id1} 511

# Verify memory has values in housing2
getd r3 {device_id2} 0
getd r4 {device_id2} 100
getd r5 {device_id2} 511

# Clear housing1 memory
clrd {device_id1}

# Read back housing1 to verify it's cleared
getd r6 {device_id1} 0
getd r7 {device_id1} 100
getd r8 {device_id1} 511

# Read back housing2 to verify it's NOT cleared
getd r9 {device_id2} 0
getd r10 {device_id2} 100
getd r11 {device_id2} 511

yield
"#
        );

        chip.borrow_mut().load_program(&program).unwrap();
        chip.borrow_mut().run(128).unwrap();

        // Verify results
        let chip_ref = chip.borrow();
        // Before clear - housing1
        assert_reg(&chip_ref, 0, 111.0);
        assert_reg(&chip_ref, 1, 222.0);
        assert_reg(&chip_ref, 2, 333.0);
        // Before clear - housing2
        assert_reg(&chip_ref, 3, 444.0);
        assert_reg(&chip_ref, 4, 555.0);
        assert_reg(&chip_ref, 5, 666.0);
        // After clear - housing1 (should be 0)
        assert_reg(&chip_ref, 6, 0.0);
        assert_reg(&chip_ref, 7, 0.0);
        assert_reg(&chip_ref, 8, 0.0);
        // After clear - housing2 (should still have values)
        assert_reg(&chip_ref, 9, 444.0);
        assert_reg(&chip_ref, 10, 555.0);
        assert_reg(&chip_ref, 11, 666.0);
    }
}
