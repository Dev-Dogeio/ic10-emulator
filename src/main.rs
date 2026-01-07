use std::{error::Error, thread::sleep, time::Duration};

use ic10_emulator::{
    CableNetwork, Device, ItemIntegratedCircuit10, LogicType, SimulationManager,
    atmospherics::{GasType, celsius_to_kelvin},
    devices::{
        AirConditioner, AtmosphericDevice, DeviceAtmosphericNetworkType, Filtration, ICHostDevice,
        ICHousing, VolumePump,
    },
    items::{Filter, FilterSize},
    networks::AtmosphericNetwork,
    parser::{preprocess, string_to_hash},
    types::shared,
};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Running Phase Change test...");
    phase_change_test()?;
    sleep(Duration::from_secs(1));

    // println!("Running Phase Change test 2...");
    // phase_change_test_2()?;
    // sleep(Duration::from_secs(1));

    // println!("Running Elmo AC test...");
    // elmo_ac_test()?;
    // sleep(Duration::from_secs(1));

    // println!("Running AC device test...");
    // ac_device_test()?;
    // sleep(Duration::from_secs(1));

    // println!("Running Filtration device test...");
    // filtration_device_test()?;
    // sleep(Duration::from_secs(1));

    // println!("Running IC program test...");
    // ic_program_test()?;
    // sleep(Duration::from_secs(1));

    Ok(())
}
#[allow(dead_code)]
fn phase_change_test() -> Result<(), Box<dyn Error>> {
    // Atmospheric phase change test
    SimulationManager::reset_global();

    let cable_network = CableNetwork::new();

    let input_network = AtmosphericNetwork::new(10.0);
    let output_network = AtmosphericNetwork::new(10.0);

    let pump = VolumePump::new(None);
    cable_network
        .borrow_mut()
        .add_device(pump.clone(), cable_network.clone());
    pump.borrow_mut().set_atmospheric_network(
        DeviceAtmosphericNetworkType::Input,
        Some(input_network.clone()),
    )?;
    pump.borrow_mut().set_atmospheric_network(
        DeviceAtmosphericNetworkType::Output,
        Some(output_network.clone()),
    )?;
    pump.borrow().write(LogicType::Setting, 1.0)?;
    pump.borrow().write(LogicType::On, 1.0)?;

    input_network
        .borrow_mut()
        .add_gas(GasType::Water, 1.0, celsius_to_kelvin(30.0));

    println!("Initial state:\n{}", input_network.borrow().mixture());

    let mut ticks = 1;
    const MAX_TICKS: u64 = 1000;

    loop {
        let changes = SimulationManager::update_global(ticks);

        println!(
            "\nAfter tick #{ticks} (phase changes: {changes}):\n Input: {}\nOutput: {}",
            input_network.borrow().mixture(),
            output_network.borrow().mixture()
        );

        if changes == 0 {
            println!("Stable state reached after tick #{ticks}.");
            break;
        }

        ticks += 1;
        if ticks > MAX_TICKS {
            println!("Reached max ticks ({MAX_TICKS}) without stabilizing.");
            break;
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn phase_change_test_2() -> Result<(), Box<dyn Error>> {
    // Atmospheric phase change test
    SimulationManager::reset_global();

    let network = AtmosphericNetwork::new(10.0);

    network
        .borrow_mut()
        .add_gas(GasType::Water, 1.0, celsius_to_kelvin(30.0));

    println!("Initial state:\n{}", network.borrow().mixture());

    let mut ticks = 1;
    const MAX_TICKS: u64 = 1000;

    loop {
        let changes = SimulationManager::update_global(ticks);

        println!(
            "\nAfter tick #{ticks} (phase changes: {changes}):\n{}",
            network.borrow().mixture()
        );

        if changes == 0 {
            println!("Stable state reached after tick #{ticks}.");
            break;
        }

        ticks += 1;
        if ticks > MAX_TICKS {
            println!("Reached max ticks ({MAX_TICKS}) without stabilizing.");
            break;
        }
    }

    network.borrow_mut().set_volume(20.0);

    loop {
        let changes = SimulationManager::update_global(ticks);

        println!(
            "\nAfter tick #{ticks} (phase changes: {changes}):\n{}",
            network.borrow().mixture()
        );

        if changes == 0 {
            println!("Stable state reached after tick #{ticks}.");
            break;
        }

        ticks += 1;
        if ticks > MAX_TICKS {
            println!("Reached max ticks ({MAX_TICKS}) without stabilizing.");
            break;
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn elmo_ac_test() -> Result<(), Box<dyn Error>> {
    // Elmo AC Setup test
    SimulationManager::reset_global();

    let tank = AtmosphericNetwork::new(780.0); // Gas tank / pump input / ac waste
    let input = AtmosphericNetwork::new(10.0); // AC input
    let vent = AtmosphericNetwork::new(1130.0); // AC hot gas output

    let ac = AirConditioner::new(None);
    ac.borrow_mut()
        .set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(input.clone()))?;
    ac.borrow_mut()
        .set_atmospheric_network(DeviceAtmosphericNetworkType::Output, Some(vent.clone()))?;
    ac.borrow_mut()
        .set_atmospheric_network(DeviceAtmosphericNetworkType::Output2, Some(tank.clone()))?;

    let pump = VolumePump::new(None);
    pump.borrow_mut()
        .set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(tank.clone()))?;
    pump.borrow_mut()
        .set_atmospheric_network(DeviceAtmosphericNetworkType::Output, Some(input.clone()))?;

    let chip = shared(ItemIntegratedCircuit10::new());
    ac.borrow_mut().set_chip(chip.clone());

    let network = CableNetwork::new();

    network.borrow_mut().add_device(ac.clone(), network.clone());
    network
        .borrow_mut()
        .add_device(pump.clone(), network.clone());

    tank.borrow_mut().add_gas(GasType::Volatiles, 500.0, 315.15); // Fill tank with hot gas

    let program = format!(
        r#"
    define AC {}
    define PUMP {}

    define TARGET_TEMP 293.15
    define TARGET_INPUT_PRESSURE 111
    define AC_INPUT_VOLUME 10

    # Initialize
    sd AC Mode 1
    sd AC Setting 999

    # Main loop
    main:
        yield
        ld r0 AC TemperatureOutput2
        push r0
        ble r0 TARGET_TEMP stop

        # Calculate pump setting for target temperature
        # V = TARGET_INPUT_PRESSURE * AC_INPUT_VOLUME / AC.PressureOutput2
        mul r0 TARGET_INPUT_PRESSURE AC_INPUT_VOLUME
        ld r1 AC PressureOutput2
        div r0 r0 r1

        # Turn on AC and set pump
        sd AC Mode 1
        sd PUMP Setting r0
        sd PUMP On 1
        j main
    stop:
        sd PUMP On 0
        sd AC Mode 0"#,
        ac.borrow().get_id(),
        pump.borrow().get_id(),
    );

    chip.borrow_mut().load_program(program.as_str())?;

    println!("Elmo AC Test Program:\n{}\n", program);

    println!("Initial state:");
    println!(" Tank: {}", tank.borrow().mixture());
    println!(" Input: {}", input.borrow().mixture());
    println!(" Vent: {}", vent.borrow().mixture());

    let mut ticks = 0;

    while ticks < 2 || ac.borrow().read(LogicType::Mode)? == 1.0 {
        // Time the simulation
        let _start = std::time::Instant::now();
        SimulationManager::update_global(ticks);
        let duration = _start.elapsed();

        println!(
            "\nAfter tick #{} ({:.2} ms):",
            ticks,
            duration.as_micros() as f64 / 1000.0
        );
        println!(" Tank: {}", tank.borrow().mixture());
        println!(" Input: {}", input.borrow().mixture());
        println!(" Vent: {}", vent.borrow().mixture());

        ticks += 1;
    }

    chip.borrow().print_debug_info();

    Ok(())
}

#[allow(dead_code)]
fn ac_device_test() -> Result<(), Box<dyn Error>> {
    // AC device test
    SimulationManager::reset_global();

    let input = AtmosphericNetwork::new(120.0);
    let waste = AtmosphericNetwork::new(60.0);

    let airconditioner = AirConditioner::new(None);
    {
        let mut ac = airconditioner.borrow_mut();
        ac.set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(input.clone()))?;
        ac.set_atmospheric_network(DeviceAtmosphericNetworkType::Output, Some(input.clone()))?;
        ac.set_atmospheric_network(DeviceAtmosphericNetworkType::Output2, Some(waste.clone()))?;
        ac.write(LogicType::Setting, celsius_to_kelvin(20.0))?;
        ac.write(LogicType::Mode, 1.0)?;
    }

    input
        .borrow_mut()
        .add_gas(GasType::Oxygen, 100.0, celsius_to_kelvin(15.0));
    waste
        .borrow_mut()
        .add_gas(GasType::Oxygen, 100.0, celsius_to_kelvin(15.0));

    println!(
        "Initial state:\n Input: {}\n Output: {}\n Waste: {}",
        input.borrow().mixture(),
        input.borrow().mixture(),
        waste.borrow().mixture()
    );

    let mut ticks = 1;

    while ticks <= 38 {
        SimulationManager::update_global(ticks);

        println!(
            "\nAfter tick #{ticks}:\n Input: {}\n Output: {}\n Waste: {}",
            input.borrow().mixture(),
            input.borrow().mixture(),
            waste.borrow().mixture()
        );
        ticks += 1;
    }

    Ok(())
}

#[allow(dead_code)]
fn filtration_device_test() -> Result<(), Box<dyn Error>> {
    // Filtration device test
    SimulationManager::reset_global();

    let network = CableNetwork::new();

    let input = AtmosphericNetwork::new(10.0);
    let filtered = AtmosphericNetwork::new(20.0);
    let waste = AtmosphericNetwork::new(10.0);

    let filtration = Filtration::new(None);
    {
        let mut f = filtration.borrow_mut();
        // Insert a filter item into slot 0
        let slot = f.get_slot_mut(0).unwrap();
        slot.try_insert(Box::new(Filter::new(
            100.0,
            GasType::Oxygen,
            FilterSize::Small,
        )))
        .unwrap();

        f.set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(input.clone()))?;
        f.set_atmospheric_network(DeviceAtmosphericNetworkType::Output, Some(filtered.clone()))?;
        f.set_atmospheric_network(DeviceAtmosphericNetworkType::Output2, Some(waste.clone()))?;
        f.write(LogicType::Mode, 1.0)?;
    }

    input.borrow_mut().add_gas(GasType::Oxygen, 63.0, 273.15);
    input
        .borrow_mut()
        .add_gas(GasType::CarbonDioxide, 4.0, 273.15);

    network
        .borrow_mut()
        .add_device(filtration.clone(), network.clone());

    println!(
        "Initial state:\n Input: {}\n Filtered: {}\n Waste: {}",
        input.borrow().mixture(),
        filtered.borrow().mixture(),
        waste.borrow().mixture()
    );

    let mut ticks = 1;

    while !input.borrow().is_empty() {
        // Run the filtration until input network is empty
        SimulationManager::update_global(ticks);

        println!(
            "\nAfter filtration #{ticks}:\n Input: {}\n Filtered: {}\n Waste: {}",
            input.borrow().mixture(),
            filtered.borrow().mixture(),
            waste.borrow().mixture()
        );
        ticks += 1;
    }

    Ok(())
}

#[allow(dead_code)]
fn ic_program_test() -> Result<(), Box<dyn Error>> {
    // IC program test
    SimulationManager::reset_global();

    // Create a network
    let network = CableNetwork::new();
    let chip = shared(ItemIntegratedCircuit10::new());
    let housing = ICHousing::new(None);

    housing.borrow_mut().set_chip(chip.clone());
    network
        .borrow_mut()
        .add_device(housing.clone(), network.clone());

    let program = r#"define PREFIX HASH("Named Device Prefix ") # Generated by PyTrapIC v0.2.2.dev3+gb1dad33d5
define FIRST_NUMBER 0
define LAST_NUMBER 5
# s db Mode 6
# s db Setting STR("CALC..")
poke 496 0
poke 497 $1DB71064
poke 498 $3B6E20C8
poke 499 $26D930AC
poke 500 $76DC4190
poke 501 $6B6B51F4
poke 502 $4DB26158
poke 503 $5005713C
poke 504 $EDB88320
poke 505 $F00F9344
poke 506 $D6D6A3E8
poke 507 $CB61B38C
poke 508 $9B64C2B0
poke 509 $86D3D2D4
poke 510 $A00AE278
poke 511 $BDBDF21C
move r0 FIRST_NUMBER
brgt r0 LAST_NUMBER 40
move r2 PREFIX
move r3 r0
brge r2 0 2
add r2 r2 $100000000
xor r2 r2 $FFFFFFFF
move r4 1
move r5 1
mul r6 10 r5
brgt r6 r3 4
add r4 r4 1
mul r5 r5 10
jr -4
brle r4 0 20
sub r4 r4 1
div r8 r3 r5
floor r6 r8
mul r8 r6 r5
sub r3 r3 r8
div r5 r5 10
add r6 r6 48
xor r2 r2 r6
and r8 r2 15
add r7 496 r8
get r8 db r7
srl r9 r2 4
xor r2 r8 r9
and r9 r2 15
add r7 496 r9
get r9 db r7
srl r8 r2 4
xor r2 r9 r8
jr -19
xor r2 r2 $FFFFFFFF
brlt r2 $80000000 2
sub r2 r2 $100000000
move r1 r2
push r1
add r0 r0 1
jr -39
s db Setting STR("DONE")"#.to_string();

    // Load the program
    chip.borrow_mut().load_program(program.as_str())?;

    let processed = preprocess(program.as_str());
    print!("Program:\n{}\n\n", processed?);

    let mut ticks: u64 = 0;

    // Run the simulation until the script is done
    while !(chip.borrow().is_halted() || housing.borrow().read(LogicType::On)? == 0.0) {
        SimulationManager::update_global(ticks);
        let steps = housing.borrow().get_last_executed_instructions();

        println!("Tick {} ({} steps)", ticks, steps);

        ticks += 1; // Increment the ticks
        sleep(Duration::from_millis(10));
    }

    chip.borrow().print_debug_info();

    // Print the hash of `Named Device Prefix {i}` of i = 0..=5
    for i in 0..=5 {
        let name = format!("Named Device Prefix {}", i);
        let hash = string_to_hash(&name);
        println!("Name: {name} Hash: {hash}");
    }

    Ok(())
}
