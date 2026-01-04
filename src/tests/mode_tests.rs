use crate::{
    devices::{
        AirConditioner, AtmosphericDevice, Device, FilterConnectionType, Filtration, LogicType,
    },
    networks::AtmosphericNetwork,
    types::shared,
};

#[test]
fn filtration_respects_mode() {
    let filtration = Filtration::new(None);

    let mut input = AtmosphericNetwork::new(10.0);
    let filtered = AtmosphericNetwork::new(10.0);
    let waste = AtmosphericNetwork::new(10.0);

    // add some oxygen to input so filtration has something to do
    input.add_gas(crate::atmospherics::GasType::Oxygen, 10.0, 300.0);

    // attach networks
    let shared_filtration = shared(filtration);
    {
        let mut f = shared_filtration.borrow_mut();
        f.set_atmospheric_network(FilterConnectionType::Input, Some(shared(input)))
            .unwrap();
        f.set_atmospheric_network(FilterConnectionType::Output, Some(shared(filtered)))
            .unwrap();
        f.set_atmospheric_network(FilterConnectionType::Output2, Some(shared(waste)))
            .unwrap();
    }

    // For testing we will directly construct and call through a new Filtration instance
    let mut f = Filtration::new(None);
    let input_rc = shared(AtmosphericNetwork::new(10.0));
    input_rc
        .borrow_mut()
        .add_gas(crate::atmospherics::GasType::Oxygen, 10.0, 300.0);
    let filtered_rc = shared(AtmosphericNetwork::new(10.0));
    let waste_rc = shared(AtmosphericNetwork::new(10.0));

    f.set_filters(vec![crate::atmospherics::GasType::Oxygen])
        .unwrap();
    f.set_atmospheric_network(
        crate::devices::FilterConnectionType::Input,
        Some(input_rc.clone()),
    )
    .unwrap();
    f.set_atmospheric_network(
        crate::devices::FilterConnectionType::Output,
        Some(filtered_rc.clone()),
    )
    .unwrap();
    f.set_atmospheric_network(
        crate::devices::FilterConnectionType::Output2,
        Some(waste_rc.clone()),
    )
    .unwrap();

    // Ensure default Mode is enabled; Run update and expect filtered has gas
    f.update(0).unwrap();
    assert!(
        filtered_rc.borrow().total_moles() > 0.0,
        "Filtered should have moles when Mode=1.0"
    );

    // Reset networks
    let mut input2 = AtmosphericNetwork::new(10.0);
    input2.add_gas(crate::atmospherics::GasType::Oxygen, 10.0, 300.0);
    let input2_rc = shared(input2);
    let filtered2_rc = shared(AtmosphericNetwork::new(10.0));
    let waste2_rc = shared(AtmosphericNetwork::new(10.0));

    let mut f2 = Filtration::new(None);
    f2.set_filters(vec![crate::atmospherics::GasType::Oxygen])
        .unwrap();
    f2.set_atmospheric_network(
        crate::devices::FilterConnectionType::Input,
        Some(input2_rc.clone()),
    )
    .unwrap();
    f2.set_atmospheric_network(
        crate::devices::FilterConnectionType::Output,
        Some(filtered2_rc.clone()),
    )
    .unwrap();
    f2.set_atmospheric_network(
        crate::devices::FilterConnectionType::Output2,
        Some(waste2_rc.clone()),
    )
    .unwrap();

    // Set Mode to 0.0 and ensure update doesn't move gas
    f2.write(LogicType::Mode, 0.0).unwrap();
    f2.update(0).unwrap();
    assert!(
        filtered2_rc.borrow().total_moles() == 0.0,
        "Filtered should have no moles when Mode=0.0"
    );
}

#[test]
fn airconditioner_respects_mode() {
    let mut ac = AirConditioner::new(None);

    let input_rc = shared(AtmosphericNetwork::new(100.0));
    let output_rc = shared(AtmosphericNetwork::new(100.0));
    let waste_rc = shared(AtmosphericNetwork::new(100.0));

    // configure networks and set a goal temperature
    input_rc
        .borrow_mut()
        .add_gas(crate::atmospherics::GasType::Oxygen, 10.0, 260.0);
    output_rc
        .borrow_mut()
        .add_gas(crate::atmospherics::GasType::Oxygen, 10.0, 260.0);
    waste_rc
        .borrow_mut()
        .add_gas(crate::atmospherics::GasType::Oxygen, 10.0, 200.0);

    ac.set_atmospheric_network(
        crate::devices::FilterConnectionType::Input,
        Some(input_rc.clone()),
    )
    .unwrap();
    ac.set_atmospheric_network(
        crate::devices::FilterConnectionType::Output,
        Some(output_rc.clone()),
    )
    .unwrap();
    ac.set_atmospheric_network(
        crate::devices::FilterConnectionType::Output2,
        Some(waste_rc.clone()),
    )
    .unwrap();

    // set goal temperature so AC will want to run
    ac.write(LogicType::Setting, 400.0).unwrap();

    // default Mode is 1.0 -> should run
    ac.update(0).unwrap();
    assert!(
        ac.processed_moles_last_tick() > 0.0,
        "AC should process moles when Mode=1.0"
    );

    // set Mode to 0.0 -> should not run
    ac.write(LogicType::Mode, 0.0).unwrap();
    ac.update(0).unwrap();
    assert_eq!(
        ac.processed_moles_last_tick(),
        0.0,
        "AC should not process moles when Mode=0.0"
    );
}
