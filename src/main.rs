use ic10_emulator::{
    Device, LogicType,
    atmospherics::{GasType, celsius_to_kelvin},
    devices::{AirConditioner, AtmosphericDevice, FilterConnectionType},
    networks::AtmosphericNetwork,
    types::shared,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // AC device test
    let input = shared(AtmosphericNetwork::new(120.0));
    let waste = shared(AtmosphericNetwork::new(60.0));

    let airconditioner = shared(AirConditioner::new(None));
    {
        let mut ac = airconditioner.borrow_mut();
        ac.set_atmospheric_network(FilterConnectionType::Input, Some(input.clone()))?;
        ac.set_atmospheric_network(FilterConnectionType::Output, Some(input.clone()))?;
        ac.set_atmospheric_network(FilterConnectionType::Output2, Some(waste.clone()))?;
        ac.write(LogicType::Setting, celsius_to_kelvin(20.0))?;
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

    let mut n = 1;

    while n <= 38 {
        airconditioner.borrow().update(0)?;

        println!(
            "\nAfter tick #{n}:\n Input: {}\n Output: {}\n Waste: {}",
            input.borrow().mixture(),
            input.borrow().mixture(),
            waste.borrow().mixture()
        );
        n += 1;
    }

    //     // Filtration device test
    //     let input = shared(AtmosphericNetwork::new(10.0));
    //     let filtered = shared(AtmosphericNetwork::new(20.0));
    //     let waste = shared(AtmosphericNetwork::new(10.0));

    //     let filtration = shared(Filtration::new(None));
    //     {
    //         let mut f = filtration.borrow_mut();
    //         f.add_filter(GasType::Oxygen)?;
    //         f.set_atmospheric_network(FilterConnectionType::Input, Some(input.clone()))?;
    //         f.set_atmospheric_network(FilterConnectionType::Output, Some(filtered.clone()))?;
    //         f.set_atmospheric_network(FilterConnectionType::Output2, Some(waste.clone()))?;
    //     }

    //     input.borrow_mut().add_gas(GasType::Oxygen, 63.0, 273.15);
    //     input
    //         .borrow_mut()
    //         .add_gas(GasType::CarbonDioxide, 4.0, 273.15);

    //     println!(
    //         "Initial state:\n Input: {}\n Filtered: {}\n Waste: {}",
    //         input.borrow().mixture(),
    //         filtered.borrow().mixture(),
    //         waste.borrow().mixture()
    //     );

    //     let mut n = 1;

    //     while !input.borrow().is_empty() {
    //         // Run the filtration until input network is empty
    //         filtration.borrow().update(0)?;

    //         println!(
    //             "\nAfter filtration #{n}:\n Input: {}\n Filtered: {}\n Waste: {}",
    //             input.borrow().mixture(),
    //             filtered.borrow().mixture(),
    //             waste.borrow().mixture()
    //         );
    //         n += 1;
    //     }

    //     // IC program test

    //     // Create a network
    //     let (chip, housing, network) = ProgrammableChip::new_with_network();

    //     // normal devices used by the example
    //     let sensor = shared(DaylightSensor::new(None));
    //     let memory_aio = shared(LogicMemory::new(None));
    //     let memory_hours = shared(LogicMemory::new(None));
    //     let memory_minutes = shared(LogicMemory::new(None));
    //     let memory_seconds = shared(LogicMemory::new(None));

    //     {
    //         let mut n = network.borrow_mut();
    //         n.add_device(sensor.clone(), network.clone());
    //         n.add_device(memory_aio.clone(), network.clone());
    //         n.add_device(memory_hours.clone(), network.clone());
    //         n.add_device(memory_minutes.clone(), network.clone());
    //         n.add_device(memory_seconds.clone(), network.clone());
    //     }

    //     {
    //         let mut h = housing.borrow_mut();
    //         h.set_device_pin(0, Some(sensor.borrow().get_id()));
    //         h.set_device_pin(1, Some(memory_aio.borrow().get_id()));
    //         h.set_device_pin(2, Some(memory_hours.borrow().get_id()));
    //         h.set_device_pin(3, Some(memory_minutes.borrow().get_id()));
    //         h.set_device_pin(4, Some(memory_seconds.borrow().get_id()));
    //     }

    //     let program = r#"# 24-hour clock | immrsv

    // alias DaySens d0
    // alias AIO d1
    // alias Hours d2
    // alias Minutes d3
    // alias Seconds d4

    // alias String r6
    // alias IsEdgeDetected r7
    // alias Calibrating r8
    // alias AdjustedTicks r9
    // alias ThisVertical r10
    // alias LastVertical r11
    // alias IsSunRising r12
    // alias WasSunRising r13
    // alias LastTicksPerDay r14
    // alias Ticks r15

    // init:
    // move Ticks 0
    // move LastTicksPerDay 2400
    // move Calibrating 4 #day cycles required to calibrate

    // l LastVertical DaySens Vertical
    // yield
    // l ThisVertical DaySens Vertical
    // slt WasSunRising ThisVertical LastVertical #Is Sun Now Rising
    // move LastVertical ThisVertical

    // main:
    // yield
    // add Ticks Ticks 1

    // # Reset At Midday/Midnight
    // l ThisVertical DaySens Vertical
    // slt IsSunRising ThisVertical LastVertical #Is Sun Now Rising
    // sne IsEdgeDetected IsSunRising WasSunRising #Is this first tick of sun rising/setting
    // s db Setting WasSunRising
    // breqz IsEdgeDetected 2 #If not first tick, skip tick reset
    // jal reset
    // move LastVertical ThisVertical
    // move WasSunRising IsSunRising

    // # Scale Ticks from Yesterday's TPD
    // div AdjustedTicks Ticks LastTicksPerDay
    // mul AdjustedTicks AdjustedTicks 2400

    // # "Expected" Total Ticks per day = 20min * 60sec * 2tps = 2400
    // move String 0
    // div r0 AdjustedTicks 100
    // trunc r0 r0
    // s Hours Setting r0
    // jal append

    // mod r0 AdjustedTicks 100 #percentage fraction of hour
    // mul r0 r0 36 #convert to seconds within the hour
    // mod r1 r0 60 #extract remainder seconds
    // trunc r1 r1
    // div r0 r0 60 #extract whole minutes
    // trunc r0 r0

    // s Minutes Setting r0
    // jal append
    // s Seconds Setting r1
    // move r0 r1
    // jal append
    // s AIO Setting String

    // j main

    // reset:
    // select r0 WasSunRising LastTicksPerDay Ticks
    // move LastTicksPerDay r0
    // div r0 LastTicksPerDay 2 #half-day of ticks from yesterday
    // select r0 WasSunRising r0 0
    // move Ticks r0
    // beqz Calibrating ra
    // sub Calibrating Calibrating 1
    // bnez Calibrating ra
    // j ra

    // append:
    // div r5 r0 10
    // trunc r5 r5
    // mod r4 r0 10
    // sll String String 8
    // add String String STR("0")
    // add String String r5
    // sll String String 8
    // add String String STR("0")
    // add String String r4
    // j ra"#;

    //     // Load the program
    //     chip.borrow_mut().load_program(program)?;

    //     let processed = preprocess(program);
    //     print!("Program:\n{}\n\n", processed?);

    //     let mut tick: u64 = 0;

    //     // Run the simulation until the script is done
    //     while !(chip.borrow().is_halted() || chip.borrow().get_housing().read(LogicType::On)? == 0.0) {
    //         network.borrow().update(tick);
    //         let steps = housing.borrow().get_last_executed_instructions();

    //         let aio_text = packed_number_to_text(memory_aio.borrow().read(LogicType::Setting)? as u64);
    //         let formatted_aio = if aio_text.len() >= 6 {
    //             format!(
    //                 "{:02}:{:02}:{:02}",
    //                 &aio_text[0..2].parse::<u8>().unwrap_or(0),
    //                 &aio_text[2..4].parse::<u8>().unwrap_or(0),
    //                 &aio_text[4..6].parse::<u8>().unwrap_or(0)
    //             )
    //         } else {
    //             aio_text
    //         };

    //         println!(
    //             "Tick {} ({} steps): AIO: ({}), Hours: ({:02}), Minutes: ({:02}), Seconds: ({:02})",
    //             tick,
    //             steps,
    //             formatted_aio,
    //             memory_hours.borrow().read(LogicType::Setting)? as u64,
    //             memory_minutes.borrow().read(LogicType::Setting)? as u64,
    //             memory_seconds.borrow().read(LogicType::Setting)? as u64
    //         );

    //         tick += 1; // Increment the ticks
    //         sleep(Duration::from_millis(10));
    //     }

    //     chip.borrow().print_debug_info();

    Ok(())
}
