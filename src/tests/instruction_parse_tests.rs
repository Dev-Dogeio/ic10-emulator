#[cfg(test)]
mod tests {
    use crate::{
        instruction::{Instruction, ParsedInstruction},
        items::item_integrated_circuit_10::{AliasTarget, Operand},
    };

    fn parse(line: &str) -> Instruction {
        ParsedInstruction::parse(line, 1).unwrap().instruction
    }

    fn alias(s: &str) -> Operand {
        Operand::Alias(s.to_string())
    }

    fn immediate(i: f64) -> Operand {
        Operand::Immediate(i)
    }

    fn register(i: usize) -> Operand {
        Operand::Register(i)
    }

    #[test]
    fn test_data_movement() {
        // Move
        assert_eq!(
            parse("move r1 r2"),
            Instruction::Move {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("move r1 42"),
            Instruction::Move {
                dest: Operand::Register(1),
                arg: immediate(42.0)
            }
        );
        assert_eq!(
            parse("move r1 a3"),
            Instruction::Move {
                dest: Operand::Register(1),
                arg: alias("a3")
            }
        );

        // Alias
        assert_eq!(
            parse("alias foo r4"),
            Instruction::Alias {
                name: "foo".to_string(),
                target: AliasTarget::Register(4)
            }
        );
        assert_eq!(
            parse("alias bar d5"),
            Instruction::Alias {
                name: "bar".to_string(),
                target: AliasTarget::Device(5)
            }
        );

        // Define
        assert_eq!(
            parse("define PI 3.14"),
            Instruction::Define {
                name: "PI".to_string(),
                #[allow(clippy::approx_constant)]
                value: 3.14
            }
        );
    }

    #[test]
    fn test_arithmetic() {
        // Add
        assert_eq!(
            parse("add r1 r2 r3"),
            Instruction::Add {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("add r1 1 2"),
            Instruction::Add {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("add r1 a1 a2"),
            Instruction::Add {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("add r1 r2 3"),
            Instruction::Add {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("add r1 4 r5"),
            Instruction::Add {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("add r1 r2 a3"),
            Instruction::Add {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("add r1 a2 r3"),
            Instruction::Add {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("add r1 7 a8"),
            Instruction::Add {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("add r1 a9 10"),
            Instruction::Add {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Sub
        assert_eq!(
            parse("sub r1 r2 r3"),
            Instruction::Sub {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sub r1 1 2"),
            Instruction::Sub {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("sub r1 a1 a2"),
            Instruction::Sub {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("sub r1 r2 3"),
            Instruction::Sub {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sub r1 4 r5"),
            Instruction::Sub {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("sub r1 r2 a3"),
            Instruction::Sub {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("sub r1 a2 r3"),
            Instruction::Sub {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sub r1 7 a8"),
            Instruction::Sub {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("sub r1 a9 10"),
            Instruction::Sub {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Mul
        assert_eq!(
            parse("mul r1 r2 r3"),
            Instruction::Mul {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("mul r1 1 2"),
            Instruction::Mul {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("mul r1 a1 a2"),
            Instruction::Mul {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("mul r1 r2 3"),
            Instruction::Mul {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("mul r1 4 r5"),
            Instruction::Mul {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("mul r1 r2 a3"),
            Instruction::Mul {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("mul r1 a2 r3"),
            Instruction::Mul {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("mul r1 7 a8"),
            Instruction::Mul {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("mul r1 a9 10"),
            Instruction::Mul {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Div
        assert_eq!(
            parse("div r1 r2 r3"),
            Instruction::Div {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("div r1 1 2"),
            Instruction::Div {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("div r1 a1 a2"),
            Instruction::Div {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("div r1 r2 3"),
            Instruction::Div {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("div r1 4 r5"),
            Instruction::Div {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("div r1 r2 a3"),
            Instruction::Div {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("div r1 a2 r3"),
            Instruction::Div {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("div r1 7 a8"),
            Instruction::Div {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("div r1 a9 10"),
            Instruction::Div {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Mod
        assert_eq!(
            parse("mod r1 r2 r3"),
            Instruction::Mod {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("mod r1 1 2"),
            Instruction::Mod {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("mod r1 a1 a2"),
            Instruction::Mod {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("mod r1 r2 3"),
            Instruction::Mod {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("mod r1 4 r5"),
            Instruction::Mod {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("mod r1 r2 a3"),
            Instruction::Mod {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("mod r1 a2 r3"),
            Instruction::Mod {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("mod r1 7 a8"),
            Instruction::Mod {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("mod r1 a9 10"),
            Instruction::Mod {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Sqrt
        assert_eq!(
            parse("sqrt r1 r2"),
            Instruction::Sqrt {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("sqrt r1 4"),
            Instruction::Sqrt {
                dest: Operand::Register(1),
                arg: immediate(4.0)
            }
        );
        assert_eq!(
            parse("sqrt r1 a2"),
            Instruction::Sqrt {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Abs
        assert_eq!(
            parse("abs r1 r2"),
            Instruction::Abs {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("abs r1 -5"),
            Instruction::Abs {
                dest: Operand::Register(1),
                arg: immediate(-5.0)
            }
        );
        assert_eq!(
            parse("abs r1 a2"),
            Instruction::Abs {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Exp
        assert_eq!(
            parse("exp r1 r2"),
            Instruction::Exp {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("exp r1 1"),
            Instruction::Exp {
                dest: Operand::Register(1),
                arg: immediate(1.0)
            }
        );
        assert_eq!(
            parse("exp r1 a2"),
            Instruction::Exp {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Log
        assert_eq!(
            parse("log r1 r2"),
            Instruction::Log {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("log r1 10"),
            Instruction::Log {
                dest: Operand::Register(1),
                arg: immediate(10.0)
            }
        );
        assert_eq!(
            parse("log r1 a2"),
            Instruction::Log {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Pow
        assert_eq!(
            parse("pow r1 r2 r3"),
            Instruction::Pow {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("pow r1 2 3"),
            Instruction::Pow {
                dest: Operand::Register(1),
                arg1: immediate(2.0),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("pow r1 a1 a2"),
            Instruction::Pow {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("pow r1 r2 3"),
            Instruction::Pow {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("pow r1 2 r3"),
            Instruction::Pow {
                dest: Operand::Register(1),
                arg1: immediate(2.0),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("pow r1 r2 a3"),
            Instruction::Pow {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("pow r1 a2 r3"),
            Instruction::Pow {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("pow r1 2 a3"),
            Instruction::Pow {
                dest: Operand::Register(1),
                arg1: immediate(2.0),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("pow r1 a2 3"),
            Instruction::Pow {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: immediate(3.0)
            }
        );

        // Max
        assert_eq!(
            parse("max r1 r2 r3"),
            Instruction::Max {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("max r1 1 2"),
            Instruction::Max {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("max r1 a1 a2"),
            Instruction::Max {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("max r1 r2 3"),
            Instruction::Max {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("max r1 4 r5"),
            Instruction::Max {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("max r1 r2 a3"),
            Instruction::Max {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("max r1 a2 r3"),
            Instruction::Max {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("max r1 7 a8"),
            Instruction::Max {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("max r1 a9 10"),
            Instruction::Max {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Min
        assert_eq!(
            parse("min r1 r2 r3"),
            Instruction::Min {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("min r1 1 2"),
            Instruction::Min {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("min r1 a1 a2"),
            Instruction::Min {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("min r1 r2 3"),
            Instruction::Min {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("min r1 4 r5"),
            Instruction::Min {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("min r1 r2 a3"),
            Instruction::Min {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("min r1 a2 r3"),
            Instruction::Min {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("min r1 7 a8"),
            Instruction::Min {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("min r1 a9 10"),
            Instruction::Min {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Ceil
        assert_eq!(
            parse("ceil r1 r2"),
            Instruction::Ceil {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("ceil r1 1.5"),
            Instruction::Ceil {
                dest: Operand::Register(1),
                arg: immediate(1.5)
            }
        );
        assert_eq!(
            parse("ceil r1 a2"),
            Instruction::Ceil {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Floor
        assert_eq!(
            parse("floor r1 r2"),
            Instruction::Floor {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("floor r1 1.5"),
            Instruction::Floor {
                dest: Operand::Register(1),
                arg: immediate(1.5)
            }
        );
        assert_eq!(
            parse("floor r1 a2"),
            Instruction::Floor {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Round
        assert_eq!(
            parse("round r1 r2"),
            Instruction::Round {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("round r1 1.5"),
            Instruction::Round {
                dest: Operand::Register(1),
                arg: immediate(1.5)
            }
        );
        assert_eq!(
            parse("round r1 a2"),
            Instruction::Round {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Trunc
        assert_eq!(
            parse("trunc r1 r2"),
            Instruction::Trunc {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("trunc r1 1.9"),
            Instruction::Trunc {
                dest: Operand::Register(1),
                arg: immediate(1.9)
            }
        );
        assert_eq!(
            parse("trunc r1 a2"),
            Instruction::Trunc {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Rand
        assert_eq!(
            parse("rand r1"),
            Instruction::Rand {
                dest: Operand::Register(1)
            }
        );

        // Lerp
        assert_eq!(
            parse("lerp r1 r2 r3 r4"),
            Instruction::Lerp {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3),
                arg3: register(4)
            }
        );
        assert_eq!(
            parse("lerp r1 0 1 0.5"),
            Instruction::Lerp {
                dest: Operand::Register(1),
                arg1: immediate(0.0),
                arg2: immediate(1.0),
                arg3: immediate(0.5)
            }
        );
        assert_eq!(
            parse("lerp r1 a1 a2 a3"),
            Instruction::Lerp {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2"),
                arg3: alias("a3")
            }
        );
        assert_eq!(
            parse("lerp r1 r2 3 a4"),
            Instruction::Lerp {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0),
                arg3: alias("a4")
            }
        );
        assert_eq!(
            parse("lerp r1 5 a6 r7"),
            Instruction::Lerp {
                dest: Operand::Register(1),
                arg1: immediate(5.0),
                arg2: alias("a6"),
                arg3: register(7)
            }
        );
        assert_eq!(
            parse("lerp r1 a8 r9 10"),
            Instruction::Lerp {
                dest: Operand::Register(1),
                arg1: alias("a8"),
                arg2: register(9),
                arg3: immediate(10.0)
            }
        );
    }

    #[test]
    fn test_trigonometric() {
        // Sin
        assert_eq!(
            parse("sin r1 r2"),
            Instruction::Sin {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("sin r1 1.5"),
            Instruction::Sin {
                dest: Operand::Register(1),
                arg: immediate(1.5)
            }
        );
        assert_eq!(
            parse("sin r1 a1"),
            Instruction::Sin {
                dest: Operand::Register(1),
                arg: alias("a1")
            }
        );

        // Cos
        assert_eq!(
            parse("cos r1 r2"),
            Instruction::Cos {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("cos r1 1.5"),
            Instruction::Cos {
                dest: Operand::Register(1),
                arg: immediate(1.5)
            }
        );
        assert_eq!(
            parse("cos r1 a1"),
            Instruction::Cos {
                dest: Operand::Register(1),
                arg: alias("a1")
            }
        );

        // Tan
        assert_eq!(
            parse("tan r1 r2"),
            Instruction::Tan {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("tan r1 1.5"),
            Instruction::Tan {
                dest: Operand::Register(1),
                arg: immediate(1.5)
            }
        );
        assert_eq!(
            parse("tan r1 a1"),
            Instruction::Tan {
                dest: Operand::Register(1),
                arg: alias("a1")
            }
        );

        // Asin
        assert_eq!(
            parse("asin r1 r2"),
            Instruction::Asin {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("asin r1 0.5"),
            Instruction::Asin {
                dest: Operand::Register(1),
                arg: immediate(0.5)
            }
        );
        assert_eq!(
            parse("asin r1 a1"),
            Instruction::Asin {
                dest: Operand::Register(1),
                arg: alias("a1")
            }
        );

        // Acos
        assert_eq!(
            parse("acos r1 r2"),
            Instruction::Acos {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("acos r1 0.5"),
            Instruction::Acos {
                dest: Operand::Register(1),
                arg: immediate(0.5)
            }
        );
        assert_eq!(
            parse("acos r1 a1"),
            Instruction::Acos {
                dest: Operand::Register(1),
                arg: alias("a1")
            }
        );

        // Atan
        assert_eq!(
            parse("atan r1 r2"),
            Instruction::Atan {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("atan r1 1.0"),
            Instruction::Atan {
                dest: Operand::Register(1),
                arg: immediate(1.0)
            }
        );
        assert_eq!(
            parse("atan r1 a1"),
            Instruction::Atan {
                dest: Operand::Register(1),
                arg: alias("a1")
            }
        );

        // Atan2
        assert_eq!(
            parse("atan2 r1 r2 r3"),
            Instruction::Atan2 {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("atan2 r1 1 2"),
            Instruction::Atan2 {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("atan2 r1 a1 a2"),
            Instruction::Atan2 {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("atan2 r1 r2 3"),
            Instruction::Atan2 {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("atan2 r1 4 r5"),
            Instruction::Atan2 {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("atan2 r1 r2 a3"),
            Instruction::Atan2 {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("atan2 r1 a2 r3"),
            Instruction::Atan2 {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("atan2 r1 7 a8"),
            Instruction::Atan2 {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("atan2 r1 a9 10"),
            Instruction::Atan2 {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );
    }

    #[test]
    fn test_bitwise() {
        // And
        assert_eq!(
            parse("and r1 r2 r3"),
            Instruction::And {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("and r1 1 2"),
            Instruction::And {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("and r1 a1 a2"),
            Instruction::And {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("and r1 r2 3"),
            Instruction::And {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("and r1 4 r5"),
            Instruction::And {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("and r1 r2 a3"),
            Instruction::And {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("and r1 a2 r3"),
            Instruction::And {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("and r1 7 a8"),
            Instruction::And {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("and r1 a9 10"),
            Instruction::And {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Or
        assert_eq!(
            parse("or r1 r2 r3"),
            Instruction::Or {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("or r1 1 2"),
            Instruction::Or {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("or r1 a1 a2"),
            Instruction::Or {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("or r1 r2 3"),
            Instruction::Or {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("or r1 4 r5"),
            Instruction::Or {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("or r1 r2 a3"),
            Instruction::Or {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("or r1 a2 r3"),
            Instruction::Or {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("or r1 7 a8"),
            Instruction::Or {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("or r1 a9 10"),
            Instruction::Or {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Xor
        assert_eq!(
            parse("xor r1 r2 r3"),
            Instruction::Xor {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("xor r1 1 2"),
            Instruction::Xor {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("xor r1 a1 a2"),
            Instruction::Xor {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("xor r1 r2 3"),
            Instruction::Xor {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("xor r1 4 r5"),
            Instruction::Xor {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("xor r1 r2 a3"),
            Instruction::Xor {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("xor r1 a2 r3"),
            Instruction::Xor {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("xor r1 7 a8"),
            Instruction::Xor {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("xor r1 a9 10"),
            Instruction::Xor {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Nor
        assert_eq!(
            parse("nor r1 r2 r3"),
            Instruction::Nor {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("nor r1 1 2"),
            Instruction::Nor {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("nor r1 a1 a2"),
            Instruction::Nor {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("nor r1 r2 3"),
            Instruction::Nor {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("nor r1 4 r5"),
            Instruction::Nor {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("nor r1 r2 a3"),
            Instruction::Nor {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("nor r1 a2 r3"),
            Instruction::Nor {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("nor r1 7 a8"),
            Instruction::Nor {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("nor r1 a9 10"),
            Instruction::Nor {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Not
        assert_eq!(
            parse("not r1 r2"),
            Instruction::Not {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("not r1 5"),
            Instruction::Not {
                dest: Operand::Register(1),
                arg: immediate(5.0)
            }
        );
        assert_eq!(
            parse("not r1 a2"),
            Instruction::Not {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );
    }

    #[test]
    fn test_bit_shifting() {
        // Sll
        assert_eq!(
            parse("sll r1 r2 r3"),
            Instruction::Sll {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sll r1 1 2"),
            Instruction::Sll {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("sll r1 a1 a2"),
            Instruction::Sll {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("sll r1 r2 3"),
            Instruction::Sll {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sll r1 4 r5"),
            Instruction::Sll {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("sll r1 r2 a3"),
            Instruction::Sll {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("sll r1 a2 r3"),
            Instruction::Sll {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sll r1 7 a8"),
            Instruction::Sll {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("sll r1 a9 10"),
            Instruction::Sll {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Sla
        assert_eq!(
            parse("sla r1 r2 r3"),
            Instruction::Sla {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sla r1 1 2"),
            Instruction::Sla {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("sla r1 a1 a2"),
            Instruction::Sla {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("sla r1 r2 3"),
            Instruction::Sla {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sla r1 4 r5"),
            Instruction::Sla {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("sla r1 r2 a3"),
            Instruction::Sla {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("sla r1 a2 r3"),
            Instruction::Sla {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sla r1 7 a8"),
            Instruction::Sla {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("sla r1 a9 10"),
            Instruction::Sla {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Srl
        assert_eq!(
            parse("srl r1 r2 r3"),
            Instruction::Srl {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("srl r1 1 2"),
            Instruction::Srl {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("srl r1 a1 a2"),
            Instruction::Srl {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("srl r1 r2 3"),
            Instruction::Srl {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("srl r1 4 r5"),
            Instruction::Srl {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("srl r1 r2 a3"),
            Instruction::Srl {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("srl r1 a2 r3"),
            Instruction::Srl {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("srl r1 7 a8"),
            Instruction::Srl {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("srl r1 a9 10"),
            Instruction::Srl {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Sra
        assert_eq!(
            parse("sra r1 r2 r3"),
            Instruction::Sra {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sra r1 1 2"),
            Instruction::Sra {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("sra r1 a1 a2"),
            Instruction::Sra {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("sra r1 r2 3"),
            Instruction::Sra {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sra r1 4 r5"),
            Instruction::Sra {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("sra r1 r2 a3"),
            Instruction::Sra {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("sra r1 a2 r3"),
            Instruction::Sra {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sra r1 7 a8"),
            Instruction::Sra {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("sra r1 a9 10"),
            Instruction::Sra {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );
    }

    #[test]
    fn test_bit_field() {
        // Ext
        assert_eq!(
            parse("ext r1 r2 r3 r4"),
            Instruction::Ext {
                dest: Operand::Register(1),
                source: register(2),
                start: register(3),
                length: register(4)
            }
        );
        assert_eq!(
            parse("ext r1 1 2 3"),
            Instruction::Ext {
                dest: Operand::Register(1),
                source: immediate(1.0),
                start: immediate(2.0),
                length: immediate(3.0)
            }
        );
        assert_eq!(
            parse("ext r1 a1 a2 a3"),
            Instruction::Ext {
                dest: Operand::Register(1),
                source: alias("a1"),
                start: alias("a2"),
                length: alias("a3")
            }
        );
        assert_eq!(
            parse("ext r1 r2 3 a4"),
            Instruction::Ext {
                dest: Operand::Register(1),
                source: register(2),
                start: immediate(3.0),
                length: alias("a4")
            }
        );
        assert_eq!(
            parse("ext r1 5 a6 r7"),
            Instruction::Ext {
                dest: Operand::Register(1),
                source: immediate(5.0),
                start: alias("a6"),
                length: register(7)
            }
        );
        assert_eq!(
            parse("ext r1 a8 r9 10"),
            Instruction::Ext {
                dest: Operand::Register(1),
                source: alias("a8"),
                start: register(9),
                length: immediate(10.0)
            }
        );

        // Ins - format: ins dest start length value
        assert_eq!(
            parse("ins r1 r2 r3 r4"),
            Instruction::Ins {
                dest: Operand::Register(1),
                start: register(2),
                length: register(3),
                value: register(4)
            }
        );
        assert_eq!(
            parse("ins r1 1 2 3"),
            Instruction::Ins {
                dest: Operand::Register(1),
                start: immediate(1.0),
                length: immediate(2.0),
                value: immediate(3.0)
            }
        );
        assert_eq!(
            parse("ins r1 a1 a2 a3"),
            Instruction::Ins {
                dest: Operand::Register(1),
                start: alias("a1"),
                length: alias("a2"),
                value: alias("a3")
            }
        );
        assert_eq!(
            parse("ins r1 r2 3 a4"),
            Instruction::Ins {
                dest: Operand::Register(1),
                start: register(2),
                length: immediate(3.0),
                value: alias("a4")
            }
        );
        assert_eq!(
            parse("ins r1 5 a6 r7"),
            Instruction::Ins {
                dest: Operand::Register(1),
                start: immediate(5.0),
                length: alias("a6"),
                value: register(7)
            }
        );
        assert_eq!(
            parse("ins r1 a8 r9 10"),
            Instruction::Ins {
                dest: Operand::Register(1),
                start: alias("a8"),
                length: register(9),
                value: immediate(10.0)
            }
        );
    }

    #[test]
    fn test_comparison_set() {
        // Slt
        assert_eq!(
            parse("slt r1 r2 r3"),
            Instruction::Slt {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("slt r1 1 2"),
            Instruction::Slt {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("slt r1 a1 a2"),
            Instruction::Slt {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("slt r1 r2 3"),
            Instruction::Slt {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("slt r1 4 r5"),
            Instruction::Slt {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("slt r1 r2 a3"),
            Instruction::Slt {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("slt r1 a2 r3"),
            Instruction::Slt {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("slt r1 7 a8"),
            Instruction::Slt {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("slt r1 a9 10"),
            Instruction::Slt {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Sgt
        assert_eq!(
            parse("sgt r1 r2 r3"),
            Instruction::Sgt {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sgt r1 1 2"),
            Instruction::Sgt {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("sgt r1 a1 a2"),
            Instruction::Sgt {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("sgt r1 r2 3"),
            Instruction::Sgt {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sgt r1 4 r5"),
            Instruction::Sgt {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("sgt r1 r2 a3"),
            Instruction::Sgt {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("sgt r1 a2 r3"),
            Instruction::Sgt {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sgt r1 7 a8"),
            Instruction::Sgt {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("sgt r1 a9 10"),
            Instruction::Sgt {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Sle
        assert_eq!(
            parse("sle r1 r2 r3"),
            Instruction::Sle {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sle r1 1 2"),
            Instruction::Sle {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("sle r1 a1 a2"),
            Instruction::Sle {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("sle r1 r2 3"),
            Instruction::Sle {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sle r1 4 r5"),
            Instruction::Sle {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("sle r1 r2 a3"),
            Instruction::Sle {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("sle r1 a2 r3"),
            Instruction::Sle {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sle r1 7 a8"),
            Instruction::Sle {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("sle r1 a9 10"),
            Instruction::Sle {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Sge
        assert_eq!(
            parse("sge r1 r2 r3"),
            Instruction::Sge {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sge r1 1 2"),
            Instruction::Sge {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("sge r1 a1 a2"),
            Instruction::Sge {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("sge r1 r2 3"),
            Instruction::Sge {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sge r1 4 r5"),
            Instruction::Sge {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("sge r1 r2 a3"),
            Instruction::Sge {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("sge r1 a2 r3"),
            Instruction::Sge {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sge r1 7 a8"),
            Instruction::Sge {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("sge r1 a9 10"),
            Instruction::Sge {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Seq
        assert_eq!(
            parse("seq r1 r2 r3"),
            Instruction::Seq {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("seq r1 1 2"),
            Instruction::Seq {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("seq r1 a1 a2"),
            Instruction::Seq {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("seq r1 r2 3"),
            Instruction::Seq {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("seq r1 4 r5"),
            Instruction::Seq {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("seq r1 r2 a3"),
            Instruction::Seq {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("seq r1 a2 r3"),
            Instruction::Seq {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("seq r1 7 a8"),
            Instruction::Seq {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("seq r1 a9 10"),
            Instruction::Seq {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Sne
        assert_eq!(
            parse("sne r1 r2 r3"),
            Instruction::Sne {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sne r1 1 2"),
            Instruction::Sne {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0)
            }
        );
        assert_eq!(
            parse("sne r1 a1 a2"),
            Instruction::Sne {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("sne r1 r2 3"),
            Instruction::Sne {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sne r1 4 r5"),
            Instruction::Sne {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("sne r1 r2 a3"),
            Instruction::Sne {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("sne r1 a2 r3"),
            Instruction::Sne {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sne r1 7 a8"),
            Instruction::Sne {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("sne r1 a9 10"),
            Instruction::Sne {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Sltz
        assert_eq!(
            parse("sltz r1 r2"),
            Instruction::Sltz {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("sltz r1 -5"),
            Instruction::Sltz {
                dest: Operand::Register(1),
                arg: immediate(-5.0)
            }
        );
        assert_eq!(
            parse("sltz r1 a2"),
            Instruction::Sltz {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Sgtz
        assert_eq!(
            parse("sgtz r1 r2"),
            Instruction::Sgtz {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("sgtz r1 5"),
            Instruction::Sgtz {
                dest: Operand::Register(1),
                arg: immediate(5.0)
            }
        );
        assert_eq!(
            parse("sgtz r1 a2"),
            Instruction::Sgtz {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Slez
        assert_eq!(
            parse("slez r1 r2"),
            Instruction::Slez {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("slez r1 0"),
            Instruction::Slez {
                dest: Operand::Register(1),
                arg: immediate(0.0)
            }
        );
        assert_eq!(
            parse("slez r1 a2"),
            Instruction::Slez {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Sgez
        assert_eq!(
            parse("sgez r1 r2"),
            Instruction::Sgez {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("sgez r1 0"),
            Instruction::Sgez {
                dest: Operand::Register(1),
                arg: immediate(0.0)
            }
        );
        assert_eq!(
            parse("sgez r1 a2"),
            Instruction::Sgez {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Seqz
        assert_eq!(
            parse("seqz r1 r2"),
            Instruction::Seqz {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("seqz r1 0"),
            Instruction::Seqz {
                dest: Operand::Register(1),
                arg: immediate(0.0)
            }
        );
        assert_eq!(
            parse("seqz r1 a2"),
            Instruction::Seqz {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Snez
        assert_eq!(
            parse("snez r1 r2"),
            Instruction::Snez {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("snez r1 1"),
            Instruction::Snez {
                dest: Operand::Register(1),
                arg: immediate(1.0)
            }
        );
        assert_eq!(
            parse("snez r1 a2"),
            Instruction::Snez {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Snan
        assert_eq!(
            parse("snan r1 r2"),
            Instruction::Snan {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("snan r1 0"),
            Instruction::Snan {
                dest: Operand::Register(1),
                arg: immediate(0.0)
            }
        );
        assert_eq!(
            parse("snan r1 a2"),
            Instruction::Snan {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );

        // Snanz
        assert_eq!(
            parse("snanz r1 r2"),
            Instruction::Snanz {
                dest: Operand::Register(1),
                arg: register(2)
            }
        );
        assert_eq!(
            parse("snanz r1 1"),
            Instruction::Snanz {
                dest: Operand::Register(1),
                arg: immediate(1.0)
            }
        );
        assert_eq!(
            parse("snanz r1 a2"),
            Instruction::Snanz {
                dest: Operand::Register(1),
                arg: alias("a2")
            }
        );
    }

    #[test]
    fn test_approximate_comparison() {
        // Sap
        assert_eq!(
            parse("sap r1 r2 r3 r4"),
            Instruction::Sap {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3),
                arg3: register(4)
            }
        );
        assert_eq!(
            parse("sap r1 1 2 0.1"),
            Instruction::Sap {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                arg3: immediate(0.1)
            }
        );
        assert_eq!(
            parse("sap r1 a1 a2 a3"),
            Instruction::Sap {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2"),
                arg3: alias("a3")
            }
        );
        assert_eq!(
            parse("sap r1 r2 3 a4"),
            Instruction::Sap {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0),
                arg3: alias("a4")
            }
        );
        assert_eq!(
            parse("sap r1 5 a6 r7"),
            Instruction::Sap {
                dest: Operand::Register(1),
                arg1: immediate(5.0),
                arg2: alias("a6"),
                arg3: register(7)
            }
        );
        assert_eq!(
            parse("sap r1 a8 r9 10"),
            Instruction::Sap {
                dest: Operand::Register(1),
                arg1: alias("a8"),
                arg2: register(9),
                arg3: immediate(10.0)
            }
        );

        // Sna
        assert_eq!(
            parse("sna r1 r2 r3 r4"),
            Instruction::Sna {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3),
                arg3: register(4)
            }
        );
        assert_eq!(
            parse("sna r1 1 2 0.1"),
            Instruction::Sna {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                arg3: immediate(0.1)
            }
        );
        assert_eq!(
            parse("sna r1 a1 a2 a3"),
            Instruction::Sna {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2"),
                arg3: alias("a3")
            }
        );
        assert_eq!(
            parse("sna r1 r2 3 a4"),
            Instruction::Sna {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0),
                arg3: alias("a4")
            }
        );
        assert_eq!(
            parse("sna r1 5 a6 r7"),
            Instruction::Sna {
                dest: Operand::Register(1),
                arg1: immediate(5.0),
                arg2: alias("a6"),
                arg3: register(7)
            }
        );
        assert_eq!(
            parse("sna r1 a8 r9 10"),
            Instruction::Sna {
                dest: Operand::Register(1),
                arg1: alias("a8"),
                arg2: register(9),
                arg3: immediate(10.0)
            }
        );

        // Sapz
        assert_eq!(
            parse("sapz r1 r2 r3"),
            Instruction::Sapz {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sapz r1 0 0.1"),
            Instruction::Sapz {
                dest: Operand::Register(1),
                arg1: immediate(0.0),
                arg2: immediate(0.1)
            }
        );
        assert_eq!(
            parse("sapz r1 a1 a2"),
            Instruction::Sapz {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("sapz r1 r2 3"),
            Instruction::Sapz {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sapz r1 4 r5"),
            Instruction::Sapz {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("sapz r1 r2 a3"),
            Instruction::Sapz {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("sapz r1 a2 r3"),
            Instruction::Sapz {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("sapz r1 7 a8"),
            Instruction::Sapz {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("sapz r1 a9 10"),
            Instruction::Sapz {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );

        // Snaz
        assert_eq!(
            parse("snaz r1 r2 r3"),
            Instruction::Snaz {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("snaz r1 1 0.1"),
            Instruction::Snaz {
                dest: Operand::Register(1),
                arg1: immediate(1.0),
                arg2: immediate(0.1)
            }
        );
        assert_eq!(
            parse("snaz r1 a1 a2"),
            Instruction::Snaz {
                dest: Operand::Register(1),
                arg1: alias("a1"),
                arg2: alias("a2")
            }
        );
        assert_eq!(
            parse("snaz r1 r2 3"),
            Instruction::Snaz {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: immediate(3.0)
            }
        );
        assert_eq!(
            parse("snaz r1 4 r5"),
            Instruction::Snaz {
                dest: Operand::Register(1),
                arg1: immediate(4.0),
                arg2: register(5)
            }
        );
        assert_eq!(
            parse("snaz r1 r2 a3"),
            Instruction::Snaz {
                dest: Operand::Register(1),
                arg1: register(2),
                arg2: alias("a3")
            }
        );
        assert_eq!(
            parse("snaz r1 a2 r3"),
            Instruction::Snaz {
                dest: Operand::Register(1),
                arg1: alias("a2"),
                arg2: register(3)
            }
        );
        assert_eq!(
            parse("snaz r1 7 a8"),
            Instruction::Snaz {
                dest: Operand::Register(1),
                arg1: immediate(7.0),
                arg2: alias("a8")
            }
        );
        assert_eq!(
            parse("snaz r1 a9 10"),
            Instruction::Snaz {
                dest: Operand::Register(1),
                arg1: alias("a9"),
                arg2: immediate(10.0)
            }
        );
    }

    #[test]
    fn test_device_state_detection() {
        // Sdse
        assert_eq!(
            parse("sdse r1 r2"),
            Instruction::Sdse {
                dest: Operand::Register(1),
                device: register(2)
            }
        );
        assert_eq!(
            parse("sdse r1 5"),
            Instruction::Sdse {
                dest: Operand::Register(1),
                device: immediate(5.0)
            }
        );
        assert_eq!(
            parse("sdse r1 a2"),
            Instruction::Sdse {
                dest: Operand::Register(1),
                device: alias("a2")
            }
        );

        // Sdns
        assert_eq!(
            parse("sdns r1 r2"),
            Instruction::Sdns {
                dest: Operand::Register(1),
                device: register(2)
            }
        );
        assert_eq!(
            parse("sdns r1 5"),
            Instruction::Sdns {
                dest: Operand::Register(1),
                device: immediate(5.0)
            }
        );
        assert_eq!(
            parse("sdns r1 a2"),
            Instruction::Sdns {
                dest: Operand::Register(1),
                device: alias("a2")
            }
        );
    }

    #[test]
    fn test_branch_absolute() {
        // Beq
        assert_eq!(
            parse("beq r1 r2 r3"),
            Instruction::Beq {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("beq 1 2 3"),
            Instruction::Beq {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("beq a1 a2 a3"),
            Instruction::Beq {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("beq r1 2 a3"),
            Instruction::Beq {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("beq 1 a2 r3"),
            Instruction::Beq {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("beq a1 r2 3"),
            Instruction::Beq {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Bne
        assert_eq!(
            parse("bne r1 r2 r3"),
            Instruction::Bne {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bne 1 2 3"),
            Instruction::Bne {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bne a1 a2 a3"),
            Instruction::Bne {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bne r1 2 a3"),
            Instruction::Bne {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bne 1 a2 r3"),
            Instruction::Bne {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bne a1 r2 3"),
            Instruction::Bne {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Blt
        assert_eq!(
            parse("blt r1 r2 r3"),
            Instruction::Blt {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("blt 1 2 3"),
            Instruction::Blt {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("blt a1 a2 a3"),
            Instruction::Blt {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("blt r1 2 a3"),
            Instruction::Blt {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("blt 1 a2 r3"),
            Instruction::Blt {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("blt a1 r2 3"),
            Instruction::Blt {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Bgt
        assert_eq!(
            parse("bgt r1 r2 r3"),
            Instruction::Bgt {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bgt 1 2 3"),
            Instruction::Bgt {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bgt a1 a2 a3"),
            Instruction::Bgt {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bgt r1 2 a3"),
            Instruction::Bgt {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bgt 1 a2 r3"),
            Instruction::Bgt {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bgt a1 r2 3"),
            Instruction::Bgt {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Ble
        assert_eq!(
            parse("ble r1 r2 r3"),
            Instruction::Ble {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("ble 1 2 3"),
            Instruction::Ble {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("ble a1 a2 a3"),
            Instruction::Ble {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("ble r1 2 a3"),
            Instruction::Ble {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("ble 1 a2 r3"),
            Instruction::Ble {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("ble a1 r2 3"),
            Instruction::Ble {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Bge
        assert_eq!(
            parse("bge r1 r2 r3"),
            Instruction::Bge {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bge 1 2 3"),
            Instruction::Bge {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bge a1 a2 a3"),
            Instruction::Bge {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bge r1 2 a3"),
            Instruction::Bge {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bge 1 a2 r3"),
            Instruction::Bge {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bge a1 r2 3"),
            Instruction::Bge {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Beqz
        assert_eq!(
            parse("beqz r1 r2"),
            Instruction::Beqz {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("beqz 0 5"),
            Instruction::Beqz {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("beqz a1 a2"),
            Instruction::Beqz {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("beqz r1 5"),
            Instruction::Beqz {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("beqz 0 a2"),
            Instruction::Beqz {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("beqz a1 r2"),
            Instruction::Beqz {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Bnez
        assert_eq!(
            parse("bnez r1 r2"),
            Instruction::Bnez {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bnez 1 5"),
            Instruction::Bnez {
                arg: immediate(1.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bnez a1 a2"),
            Instruction::Bnez {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bnez r1 5"),
            Instruction::Bnez {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bnez 1 a2"),
            Instruction::Bnez {
                arg: immediate(1.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bnez a1 r2"),
            Instruction::Bnez {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Bltz
        assert_eq!(
            parse("bltz r1 r2"),
            Instruction::Bltz {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bltz -1 5"),
            Instruction::Bltz {
                arg: immediate(-1.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bltz a1 a2"),
            Instruction::Bltz {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bltz r1 5"),
            Instruction::Bltz {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bltz -1 a2"),
            Instruction::Bltz {
                arg: immediate(-1.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bltz a1 r2"),
            Instruction::Bltz {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Bgez
        assert_eq!(
            parse("bgez r1 r2"),
            Instruction::Bgez {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bgez 0 5"),
            Instruction::Bgez {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bgez a1 a2"),
            Instruction::Bgez {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bgez r1 5"),
            Instruction::Bgez {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bgez 0 a2"),
            Instruction::Bgez {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bgez a1 r2"),
            Instruction::Bgez {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Blez
        assert_eq!(
            parse("blez r1 r2"),
            Instruction::Blez {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("blez 0 5"),
            Instruction::Blez {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("blez a1 a2"),
            Instruction::Blez {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("blez r1 5"),
            Instruction::Blez {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("blez 0 a2"),
            Instruction::Blez {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("blez a1 r2"),
            Instruction::Blez {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Bgtz
        assert_eq!(
            parse("bgtz r1 r2"),
            Instruction::Bgtz {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bgtz 1 5"),
            Instruction::Bgtz {
                arg: immediate(1.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bgtz a1 a2"),
            Instruction::Bgtz {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bgtz r1 5"),
            Instruction::Bgtz {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bgtz 1 a2"),
            Instruction::Bgtz {
                arg: immediate(1.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bgtz a1 r2"),
            Instruction::Bgtz {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Bnan
        assert_eq!(
            parse("bnan r1 r2"),
            Instruction::Bnan {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bnan 0 5"),
            Instruction::Bnan {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bnan a1 a2"),
            Instruction::Bnan {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bnan r1 5"),
            Instruction::Bnan {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bnan 0 a2"),
            Instruction::Bnan {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bnan a1 r2"),
            Instruction::Bnan {
                arg: alias("a1"),
                line: register(2)
            }
        );
    }

    #[test]
    fn test_branch_relative() {
        // Breq
        assert_eq!(
            parse("breq r1 r2 r3"),
            Instruction::Breq {
                arg1: register(1),
                arg2: register(2),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("breq 1 2 3"),
            Instruction::Breq {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                offset: immediate(3.0)
            }
        );
        assert_eq!(
            parse("breq a1 a2 a3"),
            Instruction::Breq {
                arg1: alias("a1"),
                arg2: alias("a2"),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("breq r1 2 a3"),
            Instruction::Breq {
                arg1: register(1),
                arg2: immediate(2.0),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("breq 1 a2 r3"),
            Instruction::Breq {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("breq a1 r2 3"),
            Instruction::Breq {
                arg1: alias("a1"),
                arg2: register(2),
                offset: immediate(3.0)
            }
        );

        // Brne
        assert_eq!(
            parse("brne r1 r2 r3"),
            Instruction::Brne {
                arg1: register(1),
                arg2: register(2),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brne 1 2 3"),
            Instruction::Brne {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                offset: immediate(3.0)
            }
        );
        assert_eq!(
            parse("brne a1 a2 a3"),
            Instruction::Brne {
                arg1: alias("a1"),
                arg2: alias("a2"),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brne r1 2 a3"),
            Instruction::Brne {
                arg1: register(1),
                arg2: immediate(2.0),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brne 1 a2 r3"),
            Instruction::Brne {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brne a1 r2 3"),
            Instruction::Brne {
                arg1: alias("a1"),
                arg2: register(2),
                offset: immediate(3.0)
            }
        );

        // Brlt
        assert_eq!(
            parse("brlt r1 r2 r3"),
            Instruction::Brlt {
                arg1: register(1),
                arg2: register(2),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brlt 1 2 3"),
            Instruction::Brlt {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                offset: immediate(3.0)
            }
        );
        assert_eq!(
            parse("brlt a1 a2 a3"),
            Instruction::Brlt {
                arg1: alias("a1"),
                arg2: alias("a2"),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brlt r1 2 a3"),
            Instruction::Brlt {
                arg1: register(1),
                arg2: immediate(2.0),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brlt 1 a2 r3"),
            Instruction::Brlt {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brlt a1 r2 3"),
            Instruction::Brlt {
                arg1: alias("a1"),
                arg2: register(2),
                offset: immediate(3.0)
            }
        );

        // Brgt
        assert_eq!(
            parse("brgt r1 r2 r3"),
            Instruction::Brgt {
                arg1: register(1),
                arg2: register(2),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brgt 1 2 3"),
            Instruction::Brgt {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                offset: immediate(3.0)
            }
        );
        assert_eq!(
            parse("brgt a1 a2 a3"),
            Instruction::Brgt {
                arg1: alias("a1"),
                arg2: alias("a2"),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brgt r1 2 a3"),
            Instruction::Brgt {
                arg1: register(1),
                arg2: immediate(2.0),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brgt 1 a2 r3"),
            Instruction::Brgt {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brgt a1 r2 3"),
            Instruction::Brgt {
                arg1: alias("a1"),
                arg2: register(2),
                offset: immediate(3.0)
            }
        );

        // Brle
        assert_eq!(
            parse("brle r1 r2 r3"),
            Instruction::Brle {
                arg1: register(1),
                arg2: register(2),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brle 1 2 3"),
            Instruction::Brle {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                offset: immediate(3.0)
            }
        );
        assert_eq!(
            parse("brle a1 a2 a3"),
            Instruction::Brle {
                arg1: alias("a1"),
                arg2: alias("a2"),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brle r1 2 a3"),
            Instruction::Brle {
                arg1: register(1),
                arg2: immediate(2.0),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brle 1 a2 r3"),
            Instruction::Brle {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brle a1 r2 3"),
            Instruction::Brle {
                arg1: alias("a1"),
                arg2: register(2),
                offset: immediate(3.0)
            }
        );

        // Brge
        assert_eq!(
            parse("brge r1 r2 r3"),
            Instruction::Brge {
                arg1: register(1),
                arg2: register(2),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brge 1 2 3"),
            Instruction::Brge {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                offset: immediate(3.0)
            }
        );
        assert_eq!(
            parse("brge a1 a2 a3"),
            Instruction::Brge {
                arg1: alias("a1"),
                arg2: alias("a2"),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brge r1 2 a3"),
            Instruction::Brge {
                arg1: register(1),
                arg2: immediate(2.0),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brge 1 a2 r3"),
            Instruction::Brge {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brge a1 r2 3"),
            Instruction::Brge {
                arg1: alias("a1"),
                arg2: register(2),
                offset: immediate(3.0)
            }
        );

        // Breqz
        assert_eq!(
            parse("breqz r1 r2"),
            Instruction::Breqz {
                arg: register(1),
                offset: register(2)
            }
        );
        assert_eq!(
            parse("breqz 0 5"),
            Instruction::Breqz {
                arg: immediate(0.0),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("breqz a1 a2"),
            Instruction::Breqz {
                arg: alias("a1"),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("breqz r1 5"),
            Instruction::Breqz {
                arg: register(1),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("breqz 0 a2"),
            Instruction::Breqz {
                arg: immediate(0.0),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("breqz a1 r2"),
            Instruction::Breqz {
                arg: alias("a1"),
                offset: register(2)
            }
        );

        // Brnez
        assert_eq!(
            parse("brnez r1 r2"),
            Instruction::Brnez {
                arg: register(1),
                offset: register(2)
            }
        );
        assert_eq!(
            parse("brnez 1 5"),
            Instruction::Brnez {
                arg: immediate(1.0),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brnez a1 a2"),
            Instruction::Brnez {
                arg: alias("a1"),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brnez r1 5"),
            Instruction::Brnez {
                arg: register(1),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brnez 1 a2"),
            Instruction::Brnez {
                arg: immediate(1.0),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brnez a1 r2"),
            Instruction::Brnez {
                arg: alias("a1"),
                offset: register(2)
            }
        );

        // Brltz
        assert_eq!(
            parse("brltz r1 r2"),
            Instruction::Brltz {
                arg: register(1),
                offset: register(2)
            }
        );
        assert_eq!(
            parse("brltz -1 5"),
            Instruction::Brltz {
                arg: immediate(-1.0),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brltz a1 a2"),
            Instruction::Brltz {
                arg: alias("a1"),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brltz r1 5"),
            Instruction::Brltz {
                arg: register(1),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brltz -1 a2"),
            Instruction::Brltz {
                arg: immediate(-1.0),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brltz a1 r2"),
            Instruction::Brltz {
                arg: alias("a1"),
                offset: register(2)
            }
        );

        // Brgez
        assert_eq!(
            parse("brgez r1 r2"),
            Instruction::Brgez {
                arg: register(1),
                offset: register(2)
            }
        );
        assert_eq!(
            parse("brgez 0 5"),
            Instruction::Brgez {
                arg: immediate(0.0),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brgez a1 a2"),
            Instruction::Brgez {
                arg: alias("a1"),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brgez r1 5"),
            Instruction::Brgez {
                arg: register(1),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brgez 0 a2"),
            Instruction::Brgez {
                arg: immediate(0.0),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brgez a1 r2"),
            Instruction::Brgez {
                arg: alias("a1"),
                offset: register(2)
            }
        );

        // Brlez
        assert_eq!(
            parse("brlez r1 r2"),
            Instruction::Brlez {
                arg: register(1),
                offset: register(2)
            }
        );
        assert_eq!(
            parse("brlez 0 5"),
            Instruction::Brlez {
                arg: immediate(0.0),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brlez a1 a2"),
            Instruction::Brlez {
                arg: alias("a1"),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brlez r1 5"),
            Instruction::Brlez {
                arg: register(1),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brlez 0 a2"),
            Instruction::Brlez {
                arg: immediate(0.0),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brlez a1 r2"),
            Instruction::Brlez {
                arg: alias("a1"),
                offset: register(2)
            }
        );

        // Brgtz
        assert_eq!(
            parse("brgtz r1 r2"),
            Instruction::Brgtz {
                arg: register(1),
                offset: register(2)
            }
        );
        assert_eq!(
            parse("brgtz 1 5"),
            Instruction::Brgtz {
                arg: immediate(1.0),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brgtz a1 a2"),
            Instruction::Brgtz {
                arg: alias("a1"),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brgtz r1 5"),
            Instruction::Brgtz {
                arg: register(1),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brgtz 1 a2"),
            Instruction::Brgtz {
                arg: immediate(1.0),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brgtz a1 r2"),
            Instruction::Brgtz {
                arg: alias("a1"),
                offset: register(2)
            }
        );

        // Brnan
        assert_eq!(
            parse("brnan r1 r2"),
            Instruction::Brnan {
                arg: register(1),
                offset: register(2)
            }
        );
        assert_eq!(
            parse("brnan 0 5"),
            Instruction::Brnan {
                arg: immediate(0.0),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brnan a1 a2"),
            Instruction::Brnan {
                arg: alias("a1"),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brnan r1 5"),
            Instruction::Brnan {
                arg: register(1),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brnan 0 a2"),
            Instruction::Brnan {
                arg: immediate(0.0),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brnan a1 r2"),
            Instruction::Brnan {
                arg: alias("a1"),
                offset: register(2)
            }
        );
    }

    #[test]
    fn test_branch_and_link() {
        // Beqal
        assert_eq!(
            parse("beqal r1 r2 r3"),
            Instruction::Beqal {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("beqal 1 2 3"),
            Instruction::Beqal {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("beqal a1 a2 a3"),
            Instruction::Beqal {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("beqal r1 2 a3"),
            Instruction::Beqal {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("beqal 1 a2 r3"),
            Instruction::Beqal {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("beqal a1 r2 3"),
            Instruction::Beqal {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Bneal
        assert_eq!(
            parse("bneal r1 r2 r3"),
            Instruction::Bneal {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bneal 1 2 3"),
            Instruction::Bneal {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bneal a1 a2 a3"),
            Instruction::Bneal {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bneal r1 2 a3"),
            Instruction::Bneal {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bneal 1 a2 r3"),
            Instruction::Bneal {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bneal a1 r2 3"),
            Instruction::Bneal {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Bltal
        assert_eq!(
            parse("bltal r1 r2 r3"),
            Instruction::Bltal {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bltal 1 2 3"),
            Instruction::Bltal {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bltal a1 a2 a3"),
            Instruction::Bltal {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bltal r1 2 a3"),
            Instruction::Bltal {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bltal 1 a2 r3"),
            Instruction::Bltal {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bltal a1 r2 3"),
            Instruction::Bltal {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Bgtal
        assert_eq!(
            parse("bgtal r1 r2 r3"),
            Instruction::Bgtal {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bgtal 1 2 3"),
            Instruction::Bgtal {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bgtal a1 a2 a3"),
            Instruction::Bgtal {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bgtal r1 2 a3"),
            Instruction::Bgtal {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bgtal 1 a2 r3"),
            Instruction::Bgtal {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bgtal a1 r2 3"),
            Instruction::Bgtal {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Bleal
        assert_eq!(
            parse("bleal r1 r2 r3"),
            Instruction::Bleal {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bleal 1 2 3"),
            Instruction::Bleal {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bleal a1 a2 a3"),
            Instruction::Bleal {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bleal r1 2 a3"),
            Instruction::Bleal {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bleal 1 a2 r3"),
            Instruction::Bleal {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bleal a1 r2 3"),
            Instruction::Bleal {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Bgeal
        assert_eq!(
            parse("bgeal r1 r2 r3"),
            Instruction::Bgeal {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bgeal 1 2 3"),
            Instruction::Bgeal {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bgeal a1 a2 a3"),
            Instruction::Bgeal {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bgeal r1 2 a3"),
            Instruction::Bgeal {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bgeal 1 a2 r3"),
            Instruction::Bgeal {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bgeal a1 r2 3"),
            Instruction::Bgeal {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Beqzal
        assert_eq!(
            parse("beqzal r1 r2"),
            Instruction::Beqzal {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("beqzal 0 5"),
            Instruction::Beqzal {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("beqzal a1 a2"),
            Instruction::Beqzal {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("beqzal r1 5"),
            Instruction::Beqzal {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("beqzal 0 a2"),
            Instruction::Beqzal {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("beqzal a1 r2"),
            Instruction::Beqzal {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Bnezal
        assert_eq!(
            parse("bnezal r1 r2"),
            Instruction::Bnezal {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bnezal 0 5"),
            Instruction::Bnezal {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bnezal a1 a2"),
            Instruction::Bnezal {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bnezal r1 5"),
            Instruction::Bnezal {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bnezal 0 a2"),
            Instruction::Bnezal {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bnezal a1 r2"),
            Instruction::Bnezal {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Bltzal
        assert_eq!(
            parse("bltzal r1 r2"),
            Instruction::Bltzal {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bltzal 0 5"),
            Instruction::Bltzal {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bltzal a1 a2"),
            Instruction::Bltzal {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bltzal r1 5"),
            Instruction::Bltzal {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bltzal 0 a2"),
            Instruction::Bltzal {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bltzal a1 r2"),
            Instruction::Bltzal {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Bgezal
        assert_eq!(
            parse("bgezal r1 r2"),
            Instruction::Bgezal {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bgezal 0 5"),
            Instruction::Bgezal {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bgezal a1 a2"),
            Instruction::Bgezal {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bgezal r1 5"),
            Instruction::Bgezal {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bgezal 0 a2"),
            Instruction::Bgezal {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bgezal a1 r2"),
            Instruction::Bgezal {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Blezal
        assert_eq!(
            parse("blezal r1 r2"),
            Instruction::Blezal {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("blezal 0 5"),
            Instruction::Blezal {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("blezal a1 a2"),
            Instruction::Blezal {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("blezal r1 5"),
            Instruction::Blezal {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("blezal 0 a2"),
            Instruction::Blezal {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("blezal a1 r2"),
            Instruction::Blezal {
                arg: alias("a1"),
                line: register(2)
            }
        );

        // Bgtzal
        assert_eq!(
            parse("bgtzal r1 r2"),
            Instruction::Bgtzal {
                arg: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bgtzal 0 5"),
            Instruction::Bgtzal {
                arg: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bgtzal a1 a2"),
            Instruction::Bgtzal {
                arg: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bgtzal r1 5"),
            Instruction::Bgtzal {
                arg: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bgtzal 0 a2"),
            Instruction::Bgtzal {
                arg: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bgtzal a1 r2"),
            Instruction::Bgtzal {
                arg: alias("a1"),
                line: register(2)
            }
        );
    }

    #[test]
    fn test_approximate_branches() {
        // Bap
        assert_eq!(
            parse("bap r1 r2 r3 r4"),
            Instruction::Bap {
                arg1: register(1),
                arg2: register(2),
                arg3: register(3),
                line: register(4)
            }
        );
        assert_eq!(
            parse("bap 1 2 3 4"),
            Instruction::Bap {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                arg3: immediate(3.0),
                line: immediate(4.0)
            }
        );
        assert_eq!(
            parse("bap a1 a2 a3 a4"),
            Instruction::Bap {
                arg1: alias("a1"),
                arg2: alias("a2"),
                arg3: alias("a3"),
                line: alias("a4")
            }
        );
        assert_eq!(
            parse("bap r1 2 a3 r4"),
            Instruction::Bap {
                arg1: register(1),
                arg2: immediate(2.0),
                arg3: alias("a3"),
                line: register(4)
            }
        );
        assert_eq!(
            parse("bap 1 a2 r3 4"),
            Instruction::Bap {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                arg3: register(3),
                line: immediate(4.0)
            }
        );
        assert_eq!(
            parse("bap a1 r2 3 a4"),
            Instruction::Bap {
                arg1: alias("a1"),
                arg2: register(2),
                arg3: immediate(3.0),
                line: alias("a4")
            }
        );

        // Bna
        assert_eq!(
            parse("bna r1 r2 r3 r4"),
            Instruction::Bna {
                arg1: register(1),
                arg2: register(2),
                arg3: register(3),
                line: register(4)
            }
        );
        assert_eq!(
            parse("bna 1 2 3 4"),
            Instruction::Bna {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                arg3: immediate(3.0),
                line: immediate(4.0)
            }
        );
        assert_eq!(
            parse("bna a1 a2 a3 a4"),
            Instruction::Bna {
                arg1: alias("a1"),
                arg2: alias("a2"),
                arg3: alias("a3"),
                line: alias("a4")
            }
        );
        assert_eq!(
            parse("bna r1 2 a3 r4"),
            Instruction::Bna {
                arg1: register(1),
                arg2: immediate(2.0),
                arg3: alias("a3"),
                line: register(4)
            }
        );
        assert_eq!(
            parse("bna 1 a2 r3 4"),
            Instruction::Bna {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                arg3: register(3),
                line: immediate(4.0)
            }
        );
        assert_eq!(
            parse("bna a1 r2 3 a4"),
            Instruction::Bna {
                arg1: alias("a1"),
                arg2: register(2),
                arg3: immediate(3.0),
                line: alias("a4")
            }
        );

        // Brap
        assert_eq!(
            parse("brap r1 r2 r3 r4"),
            Instruction::Brap {
                arg1: register(1),
                arg2: register(2),
                arg3: register(3),
                offset: register(4)
            }
        );
        assert_eq!(
            parse("brap 1 2 3 4"),
            Instruction::Brap {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                arg3: immediate(3.0),
                offset: immediate(4.0)
            }
        );
        assert_eq!(
            parse("brap a1 a2 a3 a4"),
            Instruction::Brap {
                arg1: alias("a1"),
                arg2: alias("a2"),
                arg3: alias("a3"),
                offset: alias("a4")
            }
        );
        assert_eq!(
            parse("brap r1 2 a3 r4"),
            Instruction::Brap {
                arg1: register(1),
                arg2: immediate(2.0),
                arg3: alias("a3"),
                offset: register(4)
            }
        );
        assert_eq!(
            parse("brap 1 a2 r3 4"),
            Instruction::Brap {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                arg3: register(3),
                offset: immediate(4.0)
            }
        );
        assert_eq!(
            parse("brap a1 r2 3 a4"),
            Instruction::Brap {
                arg1: alias("a1"),
                arg2: register(2),
                arg3: immediate(3.0),
                offset: alias("a4")
            }
        );

        // Brna
        assert_eq!(
            parse("brna r1 r2 r3 r4"),
            Instruction::Brna {
                arg1: register(1),
                arg2: register(2),
                arg3: register(3),
                offset: register(4)
            }
        );
        assert_eq!(
            parse("brna 1 2 3 4"),
            Instruction::Brna {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                arg3: immediate(3.0),
                offset: immediate(4.0)
            }
        );
        assert_eq!(
            parse("brna a1 a2 a3 a4"),
            Instruction::Brna {
                arg1: alias("a1"),
                arg2: alias("a2"),
                arg3: alias("a3"),
                offset: alias("a4")
            }
        );
        assert_eq!(
            parse("brna r1 2 a3 r4"),
            Instruction::Brna {
                arg1: register(1),
                arg2: immediate(2.0),
                arg3: alias("a3"),
                offset: register(4)
            }
        );
        assert_eq!(
            parse("brna 1 a2 r3 4"),
            Instruction::Brna {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                arg3: register(3),
                offset: immediate(4.0)
            }
        );
        assert_eq!(
            parse("brna a1 r2 3 a4"),
            Instruction::Brna {
                arg1: alias("a1"),
                arg2: register(2),
                arg3: immediate(3.0),
                offset: alias("a4")
            }
        );

        // Bapz
        assert_eq!(
            parse("bapz r1 r2 r3"),
            Instruction::Bapz {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bapz 1 2 3"),
            Instruction::Bapz {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bapz a1 a2 a3"),
            Instruction::Bapz {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bapz r1 2 a3"),
            Instruction::Bapz {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bapz 1 a2 r3"),
            Instruction::Bapz {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bapz a1 r2 3"),
            Instruction::Bapz {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Bnaz
        assert_eq!(
            parse("bnaz r1 r2 r3"),
            Instruction::Bnaz {
                arg1: register(1),
                arg2: register(2),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bnaz 1 2 3"),
            Instruction::Bnaz {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                line: immediate(3.0)
            }
        );
        assert_eq!(
            parse("bnaz a1 a2 a3"),
            Instruction::Bnaz {
                arg1: alias("a1"),
                arg2: alias("a2"),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bnaz r1 2 a3"),
            Instruction::Bnaz {
                arg1: register(1),
                arg2: immediate(2.0),
                line: alias("a3")
            }
        );
        assert_eq!(
            parse("bnaz 1 a2 r3"),
            Instruction::Bnaz {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                line: register(3)
            }
        );
        assert_eq!(
            parse("bnaz a1 r2 3"),
            Instruction::Bnaz {
                arg1: alias("a1"),
                arg2: register(2),
                line: immediate(3.0)
            }
        );

        // Brapz
        assert_eq!(
            parse("brapz r1 r2 r3"),
            Instruction::Brapz {
                arg1: register(1),
                arg2: register(2),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brapz 1 2 3"),
            Instruction::Brapz {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                offset: immediate(3.0)
            }
        );
        assert_eq!(
            parse("brapz a1 a2 a3"),
            Instruction::Brapz {
                arg1: alias("a1"),
                arg2: alias("a2"),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brapz r1 2 a3"),
            Instruction::Brapz {
                arg1: register(1),
                arg2: immediate(2.0),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brapz 1 a2 r3"),
            Instruction::Brapz {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brapz a1 r2 3"),
            Instruction::Brapz {
                arg1: alias("a1"),
                arg2: register(2),
                offset: immediate(3.0)
            }
        );

        // Brnaz
        assert_eq!(
            parse("brnaz r1 r2 r3"),
            Instruction::Brnaz {
                arg1: register(1),
                arg2: register(2),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brnaz 1 2 3"),
            Instruction::Brnaz {
                arg1: immediate(1.0),
                arg2: immediate(2.0),
                offset: immediate(3.0)
            }
        );
        assert_eq!(
            parse("brnaz a1 a2 a3"),
            Instruction::Brnaz {
                arg1: alias("a1"),
                arg2: alias("a2"),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brnaz r1 2 a3"),
            Instruction::Brnaz {
                arg1: register(1),
                arg2: immediate(2.0),
                offset: alias("a3")
            }
        );
        assert_eq!(
            parse("brnaz 1 a2 r3"),
            Instruction::Brnaz {
                arg1: immediate(1.0),
                arg2: alias("a2"),
                offset: register(3)
            }
        );
        assert_eq!(
            parse("brnaz a1 r2 3"),
            Instruction::Brnaz {
                arg1: alias("a1"),
                arg2: register(2),
                offset: immediate(3.0)
            }
        );
    }

    #[test]
    fn test_device_state_branches() {
        // Bdse
        assert_eq!(
            parse("bdse r1 r2"),
            Instruction::Bdse {
                device: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bdse 0 5"),
            Instruction::Bdse {
                device: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bdse a1 a2"),
            Instruction::Bdse {
                device: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bdse r1 5"),
            Instruction::Bdse {
                device: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bdse 0 a2"),
            Instruction::Bdse {
                device: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bdse a1 r2"),
            Instruction::Bdse {
                device: alias("a1"),
                line: register(2)
            }
        );

        // Bdns
        assert_eq!(
            parse("bdns r1 r2"),
            Instruction::Bdns {
                device: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bdns 0 5"),
            Instruction::Bdns {
                device: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bdns a1 a2"),
            Instruction::Bdns {
                device: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bdns r1 5"),
            Instruction::Bdns {
                device: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bdns 0 a2"),
            Instruction::Bdns {
                device: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bdns a1 r2"),
            Instruction::Bdns {
                device: alias("a1"),
                line: register(2)
            }
        );

        // Brdse
        assert_eq!(
            parse("brdse r1 r2"),
            Instruction::Brdse {
                device: register(1),
                offset: register(2)
            }
        );
        assert_eq!(
            parse("brdse 0 5"),
            Instruction::Brdse {
                device: immediate(0.0),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brdse a1 a2"),
            Instruction::Brdse {
                device: alias("a1"),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brdse r1 5"),
            Instruction::Brdse {
                device: register(1),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brdse 0 a2"),
            Instruction::Brdse {
                device: immediate(0.0),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brdse a1 r2"),
            Instruction::Brdse {
                device: alias("a1"),
                offset: register(2)
            }
        );

        // Brdns
        assert_eq!(
            parse("brdns r1 r2"),
            Instruction::Brdns {
                device: register(1),
                offset: register(2)
            }
        );
        assert_eq!(
            parse("brdns 0 5"),
            Instruction::Brdns {
                device: immediate(0.0),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brdns a1 a2"),
            Instruction::Brdns {
                device: alias("a1"),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brdns r1 5"),
            Instruction::Brdns {
                device: register(1),
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("brdns 0 a2"),
            Instruction::Brdns {
                device: immediate(0.0),
                offset: alias("a2")
            }
        );
        assert_eq!(
            parse("brdns a1 r2"),
            Instruction::Brdns {
                device: alias("a1"),
                offset: register(2)
            }
        );

        // Bdseal
        assert_eq!(
            parse("bdseal r1 r2"),
            Instruction::Bdseal {
                device: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bdseal 0 5"),
            Instruction::Bdseal {
                device: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bdseal a1 a2"),
            Instruction::Bdseal {
                device: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bdseal r1 5"),
            Instruction::Bdseal {
                device: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bdseal 0 a2"),
            Instruction::Bdseal {
                device: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bdseal a1 r2"),
            Instruction::Bdseal {
                device: alias("a1"),
                line: register(2)
            }
        );

        // Bdnsal
        assert_eq!(
            parse("bdnsal r1 r2"),
            Instruction::Bdnsal {
                device: register(1),
                line: register(2)
            }
        );
        assert_eq!(
            parse("bdnsal 0 5"),
            Instruction::Bdnsal {
                device: immediate(0.0),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bdnsal a1 a2"),
            Instruction::Bdnsal {
                device: alias("a1"),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bdnsal r1 5"),
            Instruction::Bdnsal {
                device: register(1),
                line: immediate(5.0)
            }
        );
        assert_eq!(
            parse("bdnsal 0 a2"),
            Instruction::Bdnsal {
                device: immediate(0.0),
                line: alias("a2")
            }
        );
        assert_eq!(
            parse("bdnsal a1 r2"),
            Instruction::Bdnsal {
                device: alias("a1"),
                line: register(2)
            }
        );
    }

    #[test]
    fn test_jump() {
        // J
        assert_eq!(parse("j r1"), Instruction::J { line: register(1) });
        assert_eq!(
            parse("j 10"),
            Instruction::J {
                line: immediate(10.0)
            }
        );
        assert_eq!(parse("j a1"), Instruction::J { line: alias("a1") });

        // Jr
        assert_eq!(
            parse("jr r1"),
            Instruction::Jr {
                offset: register(1)
            }
        );
        assert_eq!(
            parse("jr 5"),
            Instruction::Jr {
                offset: immediate(5.0)
            }
        );
        assert_eq!(
            parse("jr a1"),
            Instruction::Jr {
                offset: alias("a1")
            }
        );

        // Jal
        assert_eq!(parse("jal r1"), Instruction::Jal { line: register(1) });
        assert_eq!(
            parse("jal 10"),
            Instruction::Jal {
                line: immediate(10.0)
            }
        );
        assert_eq!(parse("jal a1"), Instruction::Jal { line: alias("a1") });
    }

    #[test]
    fn test_stack() {
        // Push
        assert_eq!(parse("push r1"), Instruction::Push { arg: register(1) });
        assert_eq!(
            parse("push 42"),
            Instruction::Push {
                arg: immediate(42.0)
            }
        );
        assert_eq!(parse("push a1"), Instruction::Push { arg: alias("a1") });

        // Pop
        assert_eq!(
            parse("pop r1"),
            Instruction::Pop {
                dest: Operand::Register(1)
            }
        );

        // Peek
        assert_eq!(
            parse("peek r1"),
            Instruction::Peek {
                dest: Operand::Register(1)
            }
        );

        // Poke
        assert_eq!(
            parse("poke r1 r2"),
            Instruction::Poke {
                index: register(1),
                value: register(2)
            }
        );
        assert_eq!(
            parse("poke 1 2"),
            Instruction::Poke {
                index: immediate(1.0),
                value: immediate(2.0)
            }
        );
        assert_eq!(
            parse("poke a1 a2"),
            Instruction::Poke {
                index: alias("a1"),
                value: alias("a2")
            }
        );
        assert_eq!(
            parse("poke r1 2"),
            Instruction::Poke {
                index: register(1),
                value: immediate(2.0)
            }
        );
        assert_eq!(
            parse("poke 1 a2"),
            Instruction::Poke {
                index: immediate(1.0),
                value: alias("a2")
            }
        );
        assert_eq!(
            parse("poke a1 r2"),
            Instruction::Poke {
                index: alias("a1"),
                value: register(2)
            }
        );
    }

    #[test]
    fn test_device_io() {
        // L
        assert_eq!(
            parse("l r1 r2 r3"),
            Instruction::L {
                dest: Operand::Register(1),
                device: register(2),
                logic_type: register(3)
            }
        );
        assert_eq!(
            parse("l r1 2 3"),
            Instruction::L {
                dest: Operand::Register(1),
                device: immediate(2.0),
                logic_type: immediate(3.0)
            }
        );
        assert_eq!(
            parse("l r1 a2 a3"),
            Instruction::L {
                dest: Operand::Register(1),
                device: alias("a2"),
                logic_type: alias("a3")
            }
        );
        assert_eq!(
            parse("l r1 r2 3"),
            Instruction::L {
                dest: Operand::Register(1),
                device: register(2),
                logic_type: immediate(3.0)
            }
        );
        assert_eq!(
            parse("l r1 2 a3"),
            Instruction::L {
                dest: Operand::Register(1),
                device: immediate(2.0),
                logic_type: alias("a3")
            }
        );
        assert_eq!(
            parse("l r1 a2 r3"),
            Instruction::L {
                dest: Operand::Register(1),
                device: alias("a2"),
                logic_type: register(3)
            }
        );
        assert_eq!(
            parse("l r0 db Setting"),
            Instruction::L {
                dest: Operand::Register(0),
                device: alias("db"),
                logic_type: immediate(12.0)
            }
        );
        assert_eq!(
            parse("l r0 db Horizontal"),
            Instruction::L {
                dest: Operand::Register(0),
                device: alias("db"),
                logic_type: immediate(20.0)
            }
        );
        assert_eq!(
            parse("l r0 db Vertical"),
            Instruction::L {
                dest: Operand::Register(0),
                device: alias("db"),
                logic_type: immediate(21.0)
            }
        );

        // S
        assert_eq!(
            parse("s r1 r2 r3"),
            Instruction::S {
                device: register(1),
                logic_type: register(2),
                value: register(3)
            }
        );
        assert_eq!(
            parse("s 1 2 3"),
            Instruction::S {
                device: immediate(1.0),
                logic_type: immediate(2.0),
                value: immediate(3.0)
            }
        );
        assert_eq!(
            parse("s a1 a2 a3"),
            Instruction::S {
                device: alias("a1"),
                logic_type: alias("a2"),
                value: alias("a3")
            }
        );
        assert_eq!(
            parse("s r1 2 a3"),
            Instruction::S {
                device: register(1),
                logic_type: immediate(2.0),
                value: alias("a3")
            }
        );
        assert_eq!(
            parse("s 1 a2 r3"),
            Instruction::S {
                device: immediate(1.0),
                logic_type: alias("a2"),
                value: register(3)
            }
        );
        assert_eq!(
            parse("s a1 r2 3"),
            Instruction::S {
                device: alias("a1"),
                logic_type: register(2),
                value: immediate(3.0)
            }
        );
        assert_eq!(
            parse("s db Setting 42"),
            Instruction::S {
                device: alias("db"),
                logic_type: immediate(12.0),
                value: immediate(42.0)
            }
        );
        assert_eq!(
            parse("s db Horizontal 100"),
            Instruction::S {
                device: alias("db"),
                logic_type: immediate(20.0),
                value: immediate(100.0)
            }
        );
        assert_eq!(
            parse("s d0 Vertical 50"),
            Instruction::S {
                device: Operand::DevicePin(0),
                logic_type: immediate(21.0),
                value: immediate(50.0)
            }
        );

        // Ls
        assert_eq!(
            parse("ls r1 r2 r3 r4"),
            Instruction::Ls {
                dest: Operand::Register(1),
                device: register(2),
                slot_index: register(3),
                slot_logic_type: register(4)
            }
        );
        assert_eq!(
            parse("ls r1 2 3 4"),
            Instruction::Ls {
                dest: Operand::Register(1),
                device: immediate(2.0),
                slot_index: immediate(3.0),
                slot_logic_type: immediate(4.0)
            }
        );
        assert_eq!(
            parse("ls r1 a2 a3 a4"),
            Instruction::Ls {
                dest: Operand::Register(1),
                device: alias("a2"),
                slot_index: alias("a3"),
                slot_logic_type: alias("a4")
            }
        );
        assert_eq!(
            parse("ls r1 r2 3 a4"),
            Instruction::Ls {
                dest: Operand::Register(1),
                device: register(2),
                slot_index: immediate(3.0),
                slot_logic_type: alias("a4")
            }
        );
        assert_eq!(
            parse("ls r1 5 a6 r7"),
            Instruction::Ls {
                dest: Operand::Register(1),
                device: immediate(5.0),
                slot_index: alias("a6"),
                slot_logic_type: register(7)
            }
        );
        assert_eq!(
            parse("ls r1 a8 r9 10"),
            Instruction::Ls {
                dest: Operand::Register(1),
                device: alias("a8"),
                slot_index: register(9),
                slot_logic_type: immediate(10.0)
            }
        );

        // Ss
        assert_eq!(
            parse("ss r1 r2 r3 r4"),
            Instruction::Ss {
                device: register(1),
                slot_index: register(2),
                slot_logic_type: register(3),
                value: register(4)
            }
        );
        assert_eq!(
            parse("ss 1 2 3 4"),
            Instruction::Ss {
                device: immediate(1.0),
                slot_index: immediate(2.0),
                slot_logic_type: immediate(3.0),
                value: immediate(4.0)
            }
        );
        assert_eq!(
            parse("ss a1 a2 a3 a4"),
            Instruction::Ss {
                device: alias("a1"),
                slot_index: alias("a2"),
                slot_logic_type: alias("a3"),
                value: alias("a4")
            }
        );
        assert_eq!(
            parse("ss r1 2 a3 r4"),
            Instruction::Ss {
                device: register(1),
                slot_index: immediate(2.0),
                slot_logic_type: alias("a3"),
                value: register(4)
            }
        );
        assert_eq!(
            parse("ss 1 a2 r3 4"),
            Instruction::Ss {
                device: immediate(1.0),
                slot_index: alias("a2"),
                slot_logic_type: register(3),
                value: immediate(4.0)
            }
        );
        assert_eq!(
            parse("ss a1 r2 3 a4"),
            Instruction::Ss {
                device: alias("a1"),
                slot_index: register(2),
                slot_logic_type: immediate(3.0),
                value: alias("a4")
            }
        );

        // Lr
        assert_eq!(
            parse("lr r1 r2 r3 r4"),
            Instruction::Lr {
                dest: Operand::Register(1),
                device: register(2),
                reagent_mode: register(3),
                reagent: register(4)
            }
        );
        assert_eq!(
            parse("lr r1 2 3 4"),
            Instruction::Lr {
                dest: Operand::Register(1),
                device: immediate(2.0),
                reagent_mode: immediate(3.0),
                reagent: immediate(4.0)
            }
        );
        assert_eq!(
            parse("lr r1 a2 a3 a4"),
            Instruction::Lr {
                dest: Operand::Register(1),
                device: alias("a2"),
                reagent_mode: alias("a3"),
                reagent: alias("a4")
            }
        );
        assert_eq!(
            parse("lr r1 r2 3 a4"),
            Instruction::Lr {
                dest: Operand::Register(1),
                device: register(2),
                reagent_mode: immediate(3.0),
                reagent: alias("a4")
            }
        );
        assert_eq!(
            parse("lr r1 5 a6 r7"),
            Instruction::Lr {
                dest: Operand::Register(1),
                device: immediate(5.0),
                reagent_mode: alias("a6"),
                reagent: register(7)
            }
        );
        assert_eq!(
            parse("lr r1 a8 r9 10"),
            Instruction::Lr {
                dest: Operand::Register(1),
                device: alias("a8"),
                reagent_mode: register(9),
                reagent: immediate(10.0)
            }
        );
    }

    #[test]
    fn test_id_based_device_access() {
        // Ld
        assert_eq!(
            parse("ld r1 r2 r3"),
            Instruction::Ld {
                dest: Operand::Register(1),
                id: register(2),
                logic_type: register(3)
            }
        );
        assert_eq!(
            parse("ld r1 2 3"),
            Instruction::Ld {
                dest: Operand::Register(1),
                id: immediate(2.0),
                logic_type: immediate(3.0)
            }
        );
        assert_eq!(
            parse("ld r1 a2 a3"),
            Instruction::Ld {
                dest: Operand::Register(1),
                id: alias("a2"),
                logic_type: alias("a3")
            }
        );
        assert_eq!(
            parse("ld r1 r2 3"),
            Instruction::Ld {
                dest: Operand::Register(1),
                id: register(2),
                logic_type: immediate(3.0)
            }
        );
        assert_eq!(
            parse("ld r1 2 a3"),
            Instruction::Ld {
                dest: Operand::Register(1),
                id: immediate(2.0),
                logic_type: alias("a3")
            }
        );
        assert_eq!(
            parse("ld r1 a2 r3"),
            Instruction::Ld {
                dest: Operand::Register(1),
                id: alias("a2"),
                logic_type: register(3)
            }
        );
        assert_eq!(
            parse("ld r0 123 Setting"),
            Instruction::Ld {
                dest: Operand::Register(0),
                id: immediate(123.0),
                logic_type: immediate(12.0)
            }
        );
        assert_eq!(
            parse("ld r0 r5 Horizontal"),
            Instruction::Ld {
                dest: Operand::Register(0),
                id: register(5),
                logic_type: immediate(20.0)
            }
        );

        // Sd
        assert_eq!(
            parse("sd r1 r2 r3"),
            Instruction::Sd {
                id: register(1),
                logic_type: register(2),
                value: register(3)
            }
        );
        assert_eq!(
            parse("sd 1 2 3"),
            Instruction::Sd {
                id: immediate(1.0),
                logic_type: immediate(2.0),
                value: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sd a1 a2 a3"),
            Instruction::Sd {
                id: alias("a1"),
                logic_type: alias("a2"),
                value: alias("a3")
            }
        );
        assert_eq!(
            parse("sd r1 2 a3"),
            Instruction::Sd {
                id: register(1),
                logic_type: immediate(2.0),
                value: alias("a3")
            }
        );
        assert_eq!(
            parse("sd 1 a2 r3"),
            Instruction::Sd {
                id: immediate(1.0),
                logic_type: alias("a2"),
                value: register(3)
            }
        );
        assert_eq!(
            parse("sd a1 r2 3"),
            Instruction::Sd {
                id: alias("a1"),
                logic_type: register(2),
                value: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sd 456 Setting 99"),
            Instruction::Sd {
                id: immediate(456.0),
                logic_type: immediate(12.0),
                value: immediate(99.0)
            }
        );
        assert_eq!(
            parse("sd r2 Vertical 75"),
            Instruction::Sd {
                id: register(2),
                logic_type: immediate(21.0),
                value: immediate(75.0)
            }
        );
    }

    #[test]
    fn test_batch_device_access() {
        // Lb
        assert_eq!(
            parse("lb r1 r2 r3 r4"),
            Instruction::Lb {
                dest: Operand::Register(1),
                device_hash: register(2),
                logic_type: register(3),
                batch_mode: register(4)
            }
        );
        assert_eq!(
            parse("lb r1 2 3 4"),
            Instruction::Lb {
                dest: Operand::Register(1),
                device_hash: immediate(2.0),
                logic_type: immediate(3.0),
                batch_mode: immediate(4.0)
            }
        );
        assert_eq!(
            parse("lb r1 a2 a3 a4"),
            Instruction::Lb {
                dest: Operand::Register(1),
                device_hash: alias("a2"),
                logic_type: alias("a3"),
                batch_mode: alias("a4")
            }
        );
        assert_eq!(
            parse("lb r1 r2 3 a4"),
            Instruction::Lb {
                dest: Operand::Register(1),
                device_hash: register(2),
                logic_type: immediate(3.0),
                batch_mode: alias("a4")
            }
        );
        assert_eq!(
            parse("lb r1 5 a6 r7"),
            Instruction::Lb {
                dest: Operand::Register(1),
                device_hash: immediate(5.0),
                logic_type: alias("a6"),
                batch_mode: register(7)
            }
        );
        assert_eq!(
            parse("lb r1 a8 r9 10"),
            Instruction::Lb {
                dest: Operand::Register(1),
                device_hash: alias("a8"),
                logic_type: register(9),
                batch_mode: immediate(10.0)
            }
        );
        assert_eq!(
            parse("lb r0 -1234567890 Setting Sum"),
            Instruction::Lb {
                dest: Operand::Register(0),
                device_hash: immediate(-1234567890.0),
                logic_type: immediate(12.0),
                batch_mode: immediate(1.0)
            }
        );
        assert_eq!(
            parse("lb r1 r5 Horizontal Average"),
            Instruction::Lb {
                dest: Operand::Register(1),
                device_hash: register(5),
                logic_type: immediate(20.0),
                batch_mode: immediate(0.0)
            }
        );

        // Sb
        assert_eq!(
            parse("sb r1 r2 r3"),
            Instruction::Sb {
                device_hash: register(1),
                logic_type: register(2),
                value: register(3)
            }
        );
        assert_eq!(
            parse("sb 1 2 3"),
            Instruction::Sb {
                device_hash: immediate(1.0),
                logic_type: immediate(2.0),
                value: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sb a1 a2 a3"),
            Instruction::Sb {
                device_hash: alias("a1"),
                logic_type: alias("a2"),
                value: alias("a3")
            }
        );
        assert_eq!(
            parse("sb r1 2 a3"),
            Instruction::Sb {
                device_hash: register(1),
                logic_type: immediate(2.0),
                value: alias("a3")
            }
        );
        assert_eq!(
            parse("sb 1 a2 r3"),
            Instruction::Sb {
                device_hash: immediate(1.0),
                logic_type: alias("a2"),
                value: register(3)
            }
        );
        assert_eq!(
            parse("sb a1 r2 3"),
            Instruction::Sb {
                device_hash: alias("a1"),
                logic_type: register(2),
                value: immediate(3.0)
            }
        );
        assert_eq!(
            parse("sb -999 Setting 15"),
            Instruction::Sb {
                device_hash: immediate(-999.0),
                logic_type: immediate(12.0),
                value: immediate(15.0)
            }
        );
        assert_eq!(
            parse("sb r3 Vertical 88"),
            Instruction::Sb {
                device_hash: register(3),
                logic_type: immediate(21.0),
                value: immediate(88.0)
            }
        );

        // Lbn
        assert_eq!(
            parse("lbn r1 r2 r3 r4 r5"),
            Instruction::Lbn {
                dest: Operand::Register(1),
                device_hash: register(2),
                name_hash: register(3),
                logic_type: register(4),
                batch_mode: register(5)
            }
        );
        assert_eq!(
            parse("lbn r1 2 3 4 5"),
            Instruction::Lbn {
                dest: Operand::Register(1),
                device_hash: immediate(2.0),
                name_hash: immediate(3.0),
                logic_type: immediate(4.0),
                batch_mode: immediate(5.0)
            }
        );
        assert_eq!(
            parse("lbn r1 a2 a3 a4 a5"),
            Instruction::Lbn {
                dest: Operand::Register(1),
                device_hash: alias("a2"),
                name_hash: alias("a3"),
                logic_type: alias("a4"),
                batch_mode: alias("a5")
            }
        );
        assert_eq!(
            parse("lbn r1 r2 3 a4 r5"),
            Instruction::Lbn {
                dest: Operand::Register(1),
                device_hash: register(2),
                name_hash: immediate(3.0),
                logic_type: alias("a4"),
                batch_mode: register(5)
            }
        );
        assert_eq!(
            parse("lbn r1 6 a7 r8 9"),
            Instruction::Lbn {
                dest: Operand::Register(1),
                device_hash: immediate(6.0),
                name_hash: alias("a7"),
                logic_type: register(8),
                batch_mode: immediate(9.0)
            }
        );
        assert_eq!(
            parse("lbn r1 a10 r11 12 a13"),
            Instruction::Lbn {
                dest: Operand::Register(1),
                device_hash: alias("a10"),
                name_hash: register(11),
                logic_type: immediate(12.0),
                batch_mode: alias("a13")
            }
        );
        assert_eq!(
            parse("lbn r0 100 200 Setting Maximum"),
            Instruction::Lbn {
                dest: Operand::Register(0),
                device_hash: immediate(100.0),
                name_hash: immediate(200.0),
                logic_type: immediate(12.0),
                batch_mode: immediate(3.0)
            }
        );

        // Sbn
        assert_eq!(
            parse("sbn r1 r2 r3 r4"),
            Instruction::Sbn {
                device_hash: register(1),
                name_hash: register(2),
                logic_type: register(3),
                value: register(4)
            }
        );
        assert_eq!(
            parse("sbn 1 2 3 4"),
            Instruction::Sbn {
                device_hash: immediate(1.0),
                name_hash: immediate(2.0),
                logic_type: immediate(3.0),
                value: immediate(4.0)
            }
        );
        assert_eq!(
            parse("sbn a1 a2 a3 a4"),
            Instruction::Sbn {
                device_hash: alias("a1"),
                name_hash: alias("a2"),
                logic_type: alias("a3"),
                value: alias("a4")
            }
        );
        assert_eq!(
            parse("sbn r1 2 a3 r4"),
            Instruction::Sbn {
                device_hash: register(1),
                name_hash: immediate(2.0),
                logic_type: alias("a3"),
                value: register(4)
            }
        );
        assert_eq!(
            parse("sbn 1 a2 r3 4"),
            Instruction::Sbn {
                device_hash: immediate(1.0),
                name_hash: alias("a2"),
                logic_type: register(3),
                value: immediate(4.0)
            }
        );
        assert_eq!(
            parse("sbn a1 r2 3 a4"),
            Instruction::Sbn {
                device_hash: alias("a1"),
                name_hash: register(2),
                logic_type: immediate(3.0),
                value: alias("a4")
            }
        );
        assert_eq!(
            parse("sbn 500 600 Horizontal 25"),
            Instruction::Sbn {
                device_hash: immediate(500.0),
                name_hash: immediate(600.0),
                logic_type: immediate(20.0),
                value: immediate(25.0)
            }
        );

        // Lbs
        assert_eq!(
            parse("lbs r1 r2 r3 r4 r5"),
            Instruction::Lbs {
                dest: Operand::Register(1),
                device_hash: register(2),
                slot_index: register(3),
                slot_logic_type: register(4),
                batch_mode: register(5)
            }
        );
        assert_eq!(
            parse("lbs r1 2 3 4 5"),
            Instruction::Lbs {
                dest: Operand::Register(1),
                device_hash: immediate(2.0),
                slot_index: immediate(3.0),
                slot_logic_type: immediate(4.0),
                batch_mode: immediate(5.0)
            }
        );
        assert_eq!(
            parse("lbs r1 a2 a3 a4 a5"),
            Instruction::Lbs {
                dest: Operand::Register(1),
                device_hash: alias("a2"),
                slot_index: alias("a3"),
                slot_logic_type: alias("a4"),
                batch_mode: alias("a5")
            }
        );
        assert_eq!(
            parse("lbs r1 r2 3 a4 r5"),
            Instruction::Lbs {
                dest: Operand::Register(1),
                device_hash: register(2),
                slot_index: immediate(3.0),
                slot_logic_type: alias("a4"),
                batch_mode: register(5)
            }
        );
        assert_eq!(
            parse("lbs r1 6 a7 r8 9"),
            Instruction::Lbs {
                dest: Operand::Register(1),
                device_hash: immediate(6.0),
                slot_index: alias("a7"),
                slot_logic_type: register(8),
                batch_mode: immediate(9.0)
            }
        );
        assert_eq!(
            parse("lbs r1 a10 r11 12 a13"),
            Instruction::Lbs {
                dest: Operand::Register(1),
                device_hash: alias("a10"),
                slot_index: register(11),
                slot_logic_type: immediate(12.0),
                batch_mode: alias("a13")
            }
        );

        // Sbs
        assert_eq!(
            parse("sbs r1 r2 r3 r4"),
            Instruction::Sbs {
                device_hash: register(1),
                slot_index: register(2),
                slot_logic_type: register(3),
                value: register(4)
            }
        );
        assert_eq!(
            parse("sbs 1 2 3 4"),
            Instruction::Sbs {
                device_hash: immediate(1.0),
                slot_index: immediate(2.0),
                slot_logic_type: immediate(3.0),
                value: immediate(4.0)
            }
        );
        assert_eq!(
            parse("sbs a1 a2 a3 a4"),
            Instruction::Sbs {
                device_hash: alias("a1"),
                slot_index: alias("a2"),
                slot_logic_type: alias("a3"),
                value: alias("a4")
            }
        );
        assert_eq!(
            parse("sbs r1 2 a3 r4"),
            Instruction::Sbs {
                device_hash: register(1),
                slot_index: immediate(2.0),
                slot_logic_type: alias("a3"),
                value: register(4)
            }
        );
        assert_eq!(
            parse("sbs 1 a2 r3 4"),
            Instruction::Sbs {
                device_hash: immediate(1.0),
                slot_index: alias("a2"),
                slot_logic_type: register(3),
                value: immediate(4.0)
            }
        );
        assert_eq!(
            parse("sbs a1 r2 3 a4"),
            Instruction::Sbs {
                device_hash: alias("a1"),
                slot_index: register(2),
                slot_logic_type: immediate(3.0),
                value: alias("a4")
            }
        );

        // Lbns
        assert_eq!(
            parse("lbns r1 r2 r3 r4 r5 r6"),
            Instruction::Lbns {
                dest: Operand::Register(1),
                device_hash: register(2),
                name_hash: register(3),
                slot_index: register(4),
                slot_logic_type: register(5),
                batch_mode: register(6)
            }
        );
        assert_eq!(
            parse("lbns r1 2 3 4 5 6"),
            Instruction::Lbns {
                dest: Operand::Register(1),
                device_hash: immediate(2.0),
                name_hash: immediate(3.0),
                slot_index: immediate(4.0),
                slot_logic_type: immediate(5.0),
                batch_mode: immediate(6.0)
            }
        );
        assert_eq!(
            parse("lbns r1 a2 a3 a4 a5 a6"),
            Instruction::Lbns {
                dest: Operand::Register(1),
                device_hash: alias("a2"),
                name_hash: alias("a3"),
                slot_index: alias("a4"),
                slot_logic_type: alias("a5"),
                batch_mode: alias("a6")
            }
        );
        assert_eq!(
            parse("lbns r1 r2 3 a4 r5 6"),
            Instruction::Lbns {
                dest: Operand::Register(1),
                device_hash: register(2),
                name_hash: immediate(3.0),
                slot_index: alias("a4"),
                slot_logic_type: register(5),
                batch_mode: immediate(6.0)
            }
        );
        assert_eq!(
            parse("lbns r1 7 a8 r9 10 a11"),
            Instruction::Lbns {
                dest: Operand::Register(1),
                device_hash: immediate(7.0),
                name_hash: alias("a8"),
                slot_index: register(9),
                slot_logic_type: immediate(10.0),
                batch_mode: alias("a11")
            }
        );
        assert_eq!(
            parse("lbns r1 a12 r13 14 a15 r0"),
            Instruction::Lbns {
                dest: Operand::Register(1),
                device_hash: alias("a12"),
                name_hash: register(13),
                slot_index: immediate(14.0),
                slot_logic_type: alias("a15"),
                batch_mode: register(0)
            }
        );
    }

    #[test]
    fn test_memory_access() {
        // Get
        assert_eq!(
            parse("get r1 r2 r3"),
            Instruction::Get {
                dest: Operand::Register(1),
                device: register(2),
                stack_index: register(3)
            }
        );
        assert_eq!(
            parse("get r1 2 3"),
            Instruction::Get {
                dest: Operand::Register(1),
                device: immediate(2.0),
                stack_index: immediate(3.0)
            }
        );
        assert_eq!(
            parse("get r1 a2 a3"),
            Instruction::Get {
                dest: Operand::Register(1),
                device: alias("a2"),
                stack_index: alias("a3")
            }
        );
        assert_eq!(
            parse("get r1 r2 3"),
            Instruction::Get {
                dest: Operand::Register(1),
                device: register(2),
                stack_index: immediate(3.0)
            }
        );
        assert_eq!(
            parse("get r1 2 a3"),
            Instruction::Get {
                dest: Operand::Register(1),
                device: immediate(2.0),
                stack_index: alias("a3")
            }
        );
        assert_eq!(
            parse("get r1 a2 r3"),
            Instruction::Get {
                dest: Operand::Register(1),
                device: alias("a2"),
                stack_index: register(3)
            }
        );

        // Put
        assert_eq!(
            parse("put r1 r2 r3"),
            Instruction::Put {
                value: register(1),
                device: register(2),
                stack_index: register(3)
            }
        );
        assert_eq!(
            parse("put 1 2 3"),
            Instruction::Put {
                value: immediate(1.0),
                device: immediate(2.0),
                stack_index: immediate(3.0)
            }
        );
        assert_eq!(
            parse("put a1 a2 a3"),
            Instruction::Put {
                value: alias("a1"),
                device: alias("a2"),
                stack_index: alias("a3")
            }
        );
        assert_eq!(
            parse("put r1 2 a3"),
            Instruction::Put {
                value: register(1),
                device: immediate(2.0),
                stack_index: alias("a3")
            }
        );
        assert_eq!(
            parse("put 1 a2 r3"),
            Instruction::Put {
                value: immediate(1.0),
                device: alias("a2"),
                stack_index: register(3)
            }
        );
        assert_eq!(
            parse("put a1 r2 3"),
            Instruction::Put {
                value: alias("a1"),
                device: register(2),
                stack_index: immediate(3.0)
            }
        );

        // Getd
        assert_eq!(
            parse("getd r1 r2 r3"),
            Instruction::Getd {
                dest: Operand::Register(1),
                id: register(2),
                stack_index: register(3)
            }
        );
        assert_eq!(
            parse("getd r1 2 3"),
            Instruction::Getd {
                dest: Operand::Register(1),
                id: immediate(2.0),
                stack_index: immediate(3.0)
            }
        );
        assert_eq!(
            parse("getd r1 a2 a3"),
            Instruction::Getd {
                dest: Operand::Register(1),
                id: alias("a2"),
                stack_index: alias("a3")
            }
        );
        assert_eq!(
            parse("getd r1 r2 3"),
            Instruction::Getd {
                dest: Operand::Register(1),
                id: register(2),
                stack_index: immediate(3.0)
            }
        );
        assert_eq!(
            parse("getd r1 2 a3"),
            Instruction::Getd {
                dest: Operand::Register(1),
                id: immediate(2.0),
                stack_index: alias("a3")
            }
        );
        assert_eq!(
            parse("getd r1 a2 r3"),
            Instruction::Getd {
                dest: Operand::Register(1),
                id: alias("a2"),
                stack_index: register(3)
            }
        );

        // Putd
        assert_eq!(
            parse("putd r1 r2 r3"),
            Instruction::Putd {
                value: register(1),
                id: register(2),
                stack_index: register(3)
            }
        );
        assert_eq!(
            parse("putd 1 2 3"),
            Instruction::Putd {
                value: immediate(1.0),
                id: immediate(2.0),
                stack_index: immediate(3.0)
            }
        );
        assert_eq!(
            parse("putd a1 a2 a3"),
            Instruction::Putd {
                value: alias("a1"),
                id: alias("a2"),
                stack_index: alias("a3")
            }
        );
        assert_eq!(
            parse("putd r1 2 a3"),
            Instruction::Putd {
                value: register(1),
                id: immediate(2.0),
                stack_index: alias("a3")
            }
        );
        assert_eq!(
            parse("putd 1 a2 r3"),
            Instruction::Putd {
                value: immediate(1.0),
                id: alias("a2"),
                stack_index: register(3)
            }
        );
        assert_eq!(
            parse("putd a1 r2 3"),
            Instruction::Putd {
                value: alias("a1"),
                id: register(2),
                stack_index: immediate(3.0)
            }
        );
    }

    #[test]
    fn test_special() {
        // Yield
        assert_eq!(parse("yield"), Instruction::Yield);

        // Sleep
        assert_eq!(
            parse("sleep r1"),
            Instruction::Sleep {
                duration: register(1)
            }
        );
        assert_eq!(
            parse("sleep 42"),
            Instruction::Sleep {
                duration: immediate(42.0)
            }
        );
        assert_eq!(
            parse("sleep a1"),
            Instruction::Sleep {
                duration: alias("a1")
            }
        );

        // Hcf
        assert_eq!(parse("hcf"), Instruction::Hcf);

        // Select
        assert_eq!(
            parse("select r1 r2 r3 r4"),
            Instruction::Select {
                dest: Operand::Register(1),
                cond: register(2),
                arg1: register(3),
                arg2: register(4)
            }
        );
        assert_eq!(
            parse("select r1 2 3 4"),
            Instruction::Select {
                dest: Operand::Register(1),
                cond: immediate(2.0),
                arg1: immediate(3.0),
                arg2: immediate(4.0)
            }
        );
        assert_eq!(
            parse("select r1 a2 a3 a4"),
            Instruction::Select {
                dest: Operand::Register(1),
                cond: alias("a2"),
                arg1: alias("a3"),
                arg2: alias("a4")
            }
        );
        assert_eq!(
            parse("select r1 r2 3 a4"),
            Instruction::Select {
                dest: Operand::Register(1),
                cond: register(2),
                arg1: immediate(3.0),
                arg2: alias("a4")
            }
        );
        assert_eq!(
            parse("select r1 5 a6 r7"),
            Instruction::Select {
                dest: Operand::Register(1),
                cond: immediate(5.0),
                arg1: alias("a6"),
                arg2: register(7)
            }
        );
        assert_eq!(
            parse("select r1 a8 r9 10"),
            Instruction::Select {
                dest: Operand::Register(1),
                cond: alias("a8"),
                arg1: register(9),
                arg2: immediate(10.0)
            }
        );

        // Clr
        assert_eq!(
            parse("clr r1"),
            Instruction::Clr {
                device: register(1)
            }
        );
        assert_eq!(
            parse("clr 5"),
            Instruction::Clr {
                device: immediate(5.0)
            }
        );
        assert_eq!(
            parse("clr a1"),
            Instruction::Clr {
                device: alias("a1")
            }
        );

        // Clrd
        assert_eq!(parse("clrd r1"), Instruction::Clrd { id: register(1) });
        assert_eq!(parse("clrd 5"), Instruction::Clrd { id: immediate(5.0) });
        assert_eq!(parse("clrd a1"), Instruction::Clrd { id: alias("a1") });

        // Rmap
        assert_eq!(
            parse("rmap r1 r2 r3"),
            Instruction::Rmap {
                dest: Operand::Register(1),
                device: register(2),
                reagent_hash: register(3)
            }
        );
        assert_eq!(
            parse("rmap r1 2 3"),
            Instruction::Rmap {
                dest: Operand::Register(1),
                device: immediate(2.0),
                reagent_hash: immediate(3.0)
            }
        );
        assert_eq!(
            parse("rmap r1 a2 a3"),
            Instruction::Rmap {
                dest: Operand::Register(1),
                device: alias("a2"),
                reagent_hash: alias("a3")
            }
        );
        assert_eq!(
            parse("rmap r1 r2 3"),
            Instruction::Rmap {
                dest: Operand::Register(1),
                device: register(2),
                reagent_hash: immediate(3.0)
            }
        );
        assert_eq!(
            parse("rmap r1 2 a3"),
            Instruction::Rmap {
                dest: Operand::Register(1),
                device: immediate(2.0),
                reagent_hash: alias("a3")
            }
        );
        assert_eq!(
            parse("rmap r1 a2 r3"),
            Instruction::Rmap {
                dest: Operand::Register(1),
                device: alias("a2"),
                reagent_hash: register(3)
            }
        );
    }
}
