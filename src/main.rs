use ic10_emulator::{Device, LogicType, ProgrammableChip, devices::LogicMemory};
use std::{cell::RefCell, rc::Rc, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a network
    let (chip, housing, network) = ProgrammableChip::new_with_network();

    // Setup 10 memory devices
    for i in 0..=9 {
        let memory_device = Rc::new(RefCell::new(LogicMemory::new()));
        network
            .borrow_mut()
            .add_device(memory_device.clone(), network.clone());
        // Initialize memory with index values
        memory_device
            .borrow_mut()
            .write(LogicType::Setting, i as f64)?;
        assert_eq!(memory_device.borrow().get_id(), i as i32 + 2);
    }

    let program = r#"define MEM_HASH HASH("StructureLogicMemory")
alias accumulator r0
alias temp r1

push 3
push 4
push 5
push 6
push 7
push 8
push 9
push 10
push 11

readloop:
yield
pop temp
ld temp temp Setting
add accumulator accumulator temp
bnez sp readloop

lb r1 MEM_HASH Setting Minimum
lb r2 MEM_HASH Setting Average
lb r3 MEM_HASH Setting Sum
lb r4 MEM_HASH Setting Maximum"#;

    // Load the program
    chip.borrow_mut().load_program(program)?;

    let mut tick: u64 = 0;

    // Run the simulation until the script is done
    while !(chip.borrow().is_script_over()) {
        let chip_ref = housing.borrow_mut().update(tick).unwrap();
        let steps = chip_ref.borrow_mut().run(128)?;

        let accumulator = chip.borrow().get_register(0)?;

        println!(
            "Tick {} ({} steps): accumulator: {}",
            tick, steps, accumulator
        );

        tick += 1; // Increment the tick
        sleep(Duration::from_millis(100));
    }

    chip.borrow().print_debug_info();

    Ok(())
}
