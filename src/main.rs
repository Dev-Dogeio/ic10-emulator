use ic10_emulator::{DaylightSensor, Device, LogicType, ProgrammableChip, devices::LogicMemory};
use std::{cell::RefCell, rc::Rc, thread::sleep, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a network
    let (chip, housing, network) = ProgrammableChip::new_with_network();

    let sensor = Rc::new(RefCell::new(DaylightSensor::new(None)));
    network
        .borrow_mut()
        .add_device(sensor.clone(), network.clone());

    let memory = Rc::new(RefCell::new(LogicMemory::new()));
    network
        .borrow_mut()
        .add_device(memory.clone(), network.clone());

    let program = format!(
        r#"define Sensor {}
define Memory {}

define LIGHT_TICKS 1800

alias clock r14
alias vertical r15

# Setup
move clock LIGHT_TICKS
jal updateclock
sd Memory Setting 0

# Find entry point
# If sun is rising, go to day detection, otherwise go to night detection
ld vertical Sensor Vertical
yield
jal issunrising
bnez r0 main
j waitdayendloop

# Main program
main:
  ld vertical Sensor Vertical
  # Wait for day detection !issunrising
  waitdayloop:
    yield
    jal issunrising
    seqz r0 r0
  beqz r0 waitdayloop

  # Stage: Wait for light ticks
  sd Memory Setting 1
  waitticksloop:
    yield
    sub clock clock 1
    jal updateclock
  bnez clock waitticksloop

  # Reset the clock
  move clock LIGHT_TICKS
  jal updateclock

  sd Memory Setting 0

  # Stage: Wait for sun to start rising
  ld vertical Sensor Vertical
  waitsunrisingloop:
    yield
    jal issunrising 
  beqz r0 waitsunrisingloop
j main

# in: vertical (previous tick vertical)
# out: r0 (isSunRising), vertical (current tick vertical)
issunrising:
  ld r1 Sensor Vertical
  slt r0 r1 vertical
  move vertical r1
j ra

# updates the display based on the current clock variable
# in: clock
updateclock:
  div r0 clock 2
  ceil r0 r0
j ra"#,
        sensor.borrow().get_id(),
        memory.borrow().get_id()
    );

    // Load the program
    chip.borrow_mut().load_program(program.as_str())?;

    let mut tick: u64 = 0;

    // Run the simulation until the script is done
    while !(chip.borrow().is_script_over()) {
        sensor.borrow_mut().update(tick);
        let chip_ref = housing.borrow_mut().update(tick).unwrap();
        let steps = chip_ref.borrow_mut().run(128)?;

        let is_light_on = memory.borrow().read(LogicType::Setting)? == 1.0f64;

        println!(
            "Tick {} ({} steps): Growlight Status: ({}), Sensor Vertical: ({:.2})",
            tick,
            steps,
            if is_light_on { "On" } else { "Off" },
            sensor.borrow().read(LogicType::Vertical)?
        );

        tick += 1; // Increment the tick
        sleep(Duration::from_millis(10));
    }

    chip.borrow().print_debug_info();

    Ok(())
}
