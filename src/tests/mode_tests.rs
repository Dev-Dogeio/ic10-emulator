use crate::{
    Filter,
    atmospherics::GasType,
    devices::{
        AirConditioner, AtmosphericDevice, Device, FilterConnectionType, Filtration, LogicType,
    },
    items::FilterSize,
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
    input.add_gas(GasType::Oxygen, 10.0, 300.0);

    // attach networks
    {
        let mut f = filtration.borrow_mut();
        f.set_atmospheric_network(FilterConnectionType::Input, Some(shared(input)))
            .unwrap();
        f.set_atmospheric_network(FilterConnectionType::Output, Some(shared(filtered)))
            .unwrap();
        f.set_atmospheric_network(FilterConnectionType::Output2, Some(shared(waste)))
            .unwrap();
    }

    // For testing we will directly construct and call through a new Filtration instance
    let f = Filtration::new(None);
    let input_rc = shared(AtmosphericNetwork::new(10.0));
    input_rc.borrow_mut().add_gas(GasType::Oxygen, 10.0, 300.0);
    let filtered_rc = shared(AtmosphericNetwork::new(10.0));
    let waste_rc = shared(AtmosphericNetwork::new(10.0));

    // insert physical filter (slot 0) for Oxygen
    {
        let mut f_borrow = f.borrow_mut();
        let slot = f_borrow.get_slot_mut(0).unwrap();
        let _ = slot.try_insert(Box::new(Filter::new(
            1.0,
            GasType::Oxygen,
            FilterSize::Small,
        )));
    }
    f.borrow_mut()
        .set_atmospheric_network(FilterConnectionType::Input, Some(input_rc.clone()))
        .unwrap();
    f.borrow_mut()
        .set_atmospheric_network(FilterConnectionType::Output, Some(filtered_rc.clone()))
        .unwrap();
    f.borrow_mut()
        .set_atmospheric_network(FilterConnectionType::Output2, Some(waste_rc.clone()))
        .unwrap();
    f.borrow_mut().write(LogicType::Mode, 1.0).unwrap();

    // Ensure default Mode is enabled; Run update and expect filtered has gas
    f.borrow().update(0).unwrap();
    assert!(
        filtered_rc.borrow().total_moles() > 0.0,
        "Filtered should have moles when Mode=1.0"
    );

    // Reset networks
    let mut input2 = AtmosphericNetwork::new(10.0);
    input2.add_gas(GasType::Oxygen, 10.0, 300.0);
    let input2_rc = shared(input2);
    let filtered2_rc = shared(AtmosphericNetwork::new(10.0));
    let waste2_rc = shared(AtmosphericNetwork::new(10.0));

    let f2 = Filtration::new(None);
    // insert physical filter (slot 0) for Oxygen
    {
        let mut f2_borrow = f2.borrow_mut();
        let slot = f2_borrow.get_slot_mut(0).unwrap();
        let _ = slot.try_insert(Box::new(Filter::new(
            1.0,
            GasType::Oxygen,
            FilterSize::Small,
        )));
    }
    f2.borrow_mut()
        .set_atmospheric_network(FilterConnectionType::Input, Some(input2_rc.clone()))
        .unwrap();
    f2.borrow_mut()
        .set_atmospheric_network(FilterConnectionType::Output, Some(filtered2_rc.clone()))
        .unwrap();
    f2.borrow_mut()
        .set_atmospheric_network(FilterConnectionType::Output2, Some(waste2_rc.clone()))
        .unwrap();

    // Set Mode to 0.0 and ensure update doesn't move gas
    f2.borrow_mut().write(LogicType::Mode, 0.0).unwrap();
    f2.borrow_mut().update(0).unwrap();
    assert!(
        filtered2_rc.borrow().total_moles() == 0.0,
        "Filtered should have no moles when Mode=0.0"
    );
}

#[test]
fn airconditioner_respects_mode() {
    let ac = AirConditioner::new(None);

    let input_rc = shared(AtmosphericNetwork::new(100.0));
    let output_rc = shared(AtmosphericNetwork::new(100.0));
    let waste_rc = shared(AtmosphericNetwork::new(100.0));

    // configure networks and set a goal temperature
    input_rc.borrow_mut().add_gas(GasType::Oxygen, 10.0, 260.0);
    output_rc.borrow_mut().add_gas(GasType::Oxygen, 10.0, 260.0);
    waste_rc.borrow_mut().add_gas(GasType::Oxygen, 10.0, 200.0);

    ac.borrow_mut()
        .set_atmospheric_network(FilterConnectionType::Input, Some(input_rc.clone()))
        .unwrap();
    ac.borrow_mut()
        .set_atmospheric_network(FilterConnectionType::Output, Some(output_rc.clone()))
        .unwrap();
    ac.borrow_mut()
        .set_atmospheric_network(FilterConnectionType::Output2, Some(waste_rc.clone()))
        .unwrap();

    // set goal temperature so AC will want to run
    ac.borrow().write(LogicType::Mode, 1.0).unwrap();
    ac.borrow().write(LogicType::Setting, 400.0).unwrap();

    // default Mode is 1.0 -> should run
    ac.borrow().update(0).unwrap();
    assert!(
        ac.borrow().processed_moles_last_tick() > 0.0,
        "AC should process moles when Mode=1.0"
    );

    // set Mode to 0.0 -> should not run
    ac.borrow().write(LogicType::Mode, 0.0).unwrap();
    ac.borrow().update(0).unwrap();
    assert_eq!(
        ac.borrow().processed_moles_last_tick(),
        0.0,
        "AC should not process moles when Mode=0.0"
    );
}
