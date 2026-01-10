use crate::{
    Filter, Item,
    atmospherics::GasType,
    devices::{
        AirConditioner, AtmosphericDevice, Device, DeviceAtmosphericNetworkType, Filtration,
        LogicType, SimulationDeviceSettings,
    },
    items::{FilterSize, SimulationItemSettings},
    networks::AtmosphericNetwork,
    types::{Shared, shared},
};

#[test]
fn filtration_respects_mode() {
    let filtration = Filtration::new(SimulationDeviceSettings {
        id: Some(1),
        ..SimulationDeviceSettings::default()
    });

    let input = AtmosphericNetwork::new(10.0);
    let filtered = AtmosphericNetwork::new(10.0);
    let waste = AtmosphericNetwork::new(10.0);

    // add some oxygen to input so filtration has something to do
    input.borrow_mut().add_gas(GasType::Oxygen, 10.0, 300.0);

    // attach networks
    {
        let mut f = filtration.borrow_mut();
        f.set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(input.clone()))
            .unwrap();
        f.set_atmospheric_network(DeviceAtmosphericNetworkType::Output, Some(filtered.clone()))
            .unwrap();
        f.set_atmospheric_network(DeviceAtmosphericNetworkType::Output2, Some(waste.clone()))
            .unwrap();
    }

    // For testing we will directly construct and call through a new Filtration instance
    let f = Filtration::new(SimulationDeviceSettings {
        id: Some(2),
        ..SimulationDeviceSettings::default()
    });
    let input_rc = AtmosphericNetwork::new(10.0);
    input_rc.borrow_mut().add_gas(GasType::Oxygen, 10.0, 300.0);
    let filtered_rc = AtmosphericNetwork::new(10.0);
    let waste_rc = AtmosphericNetwork::new(10.0);

    // insert physical filter (slot 0) for Oxygen
    {
        let mut f_borrow = f.borrow_mut();
        let slot = f_borrow.get_slot_mut(0).unwrap();
        let mut filter_item = Filter::new(SimulationItemSettings {
            id: Some(3),
            ..SimulationItemSettings::default()
        });
        filter_item.set_gas_type(GasType::Oxygen);
        filter_item.set_size(FilterSize::Small);
        filter_item.set_quantity(1);
        let filter: Shared<dyn Item> = shared(filter_item);
        let _ = slot.try_insert(filter);
    }
    f.borrow_mut()
        .set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(input_rc.clone()))
        .unwrap();
    f.borrow_mut()
        .set_atmospheric_network(
            DeviceAtmosphericNetworkType::Output,
            Some(filtered_rc.clone()),
        )
        .unwrap();
    f.borrow_mut()
        .set_atmospheric_network(
            DeviceAtmosphericNetworkType::Output2,
            Some(waste_rc.clone()),
        )
        .unwrap();
    f.borrow_mut().write(LogicType::Mode, 1.0).unwrap();

    // Ensure default Mode is enabled; Run update and expect filtered has gas
    f.borrow().update(0).unwrap();
    assert!(
        filtered_rc.borrow().total_moles() > 0.0,
        "Filtered should have moles when Mode=1.0"
    );

    // New test: filtration should remove both liquid and gas forms when configured for either
    {
        let f3 = Filtration::new(SimulationDeviceSettings {
            id: Some(10),
            ..SimulationDeviceSettings::default()
        });

        let input3 = AtmosphericNetwork::new(10.0);
        // Add both steam (gas) and liquid water
        input3.borrow_mut().add_gas(GasType::Steam, 50.0, 300.0);
        input3.borrow_mut().add_gas(GasType::Water, 50.0, 300.0);
        let filtered3 = AtmosphericNetwork::new(10.0);
        let waste3 = AtmosphericNetwork::new(10.0);

        // insert physical filter (slot 0) set to Water (liquid form)
        {
            let mut f3_borrow = f3.borrow_mut();
            let slot = f3_borrow.get_slot_mut(0).unwrap();
            let mut filter_item = Filter::new(SimulationItemSettings {
                id: Some(11),
                ..SimulationItemSettings::default()
            });
            filter_item.set_gas_type(GasType::Water);
            filter_item.set_size(FilterSize::Small);
            filter_item.set_quantity(1);
            let filter: Shared<dyn Item> = shared(filter_item);
            let _ = slot.try_insert(filter);
        }

        f3.borrow_mut()
            .set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(input3.clone()))
            .unwrap();
        f3.borrow_mut()
            .set_atmospheric_network(
                DeviceAtmosphericNetworkType::Output,
                Some(filtered3.clone()),
            )
            .unwrap();
        f3.borrow_mut()
            .set_atmospheric_network(DeviceAtmosphericNetworkType::Output2, Some(waste3.clone()))
            .unwrap();

        f3.borrow_mut().write(LogicType::Mode, 1.0).unwrap();
        f3.borrow().update(0).unwrap();

        // Both steam and water should have been filtered
        assert!(
            filtered3.borrow().get_moles(GasType::Steam) > 0.0,
            "Filtered should contain Steam"
        );
        assert!(
            filtered3.borrow().get_moles(GasType::Water) > 0.0,
            "Filtered should contain Water"
        );
    }

    // Reset networks
    let input2 = AtmosphericNetwork::new(10.0);
    input2.borrow_mut().add_gas(GasType::Oxygen, 10.0, 300.0);
    let input2_rc = input2.clone();
    let filtered2_rc = AtmosphericNetwork::new(10.0);
    let waste2_rc = AtmosphericNetwork::new(10.0);

    let f2 = Filtration::new(SimulationDeviceSettings {
        id: Some(1),
        ..SimulationDeviceSettings::default()
    });
    // insert physical filter (slot 0) for Oxygen
    {
        let mut f2_borrow = f2.borrow_mut();
        let slot = f2_borrow.get_slot_mut(0).unwrap();
        let mut filter_item = Filter::new(SimulationItemSettings {
            id: Some(2),
            ..SimulationItemSettings::default()
        });
        filter_item.set_gas_type(GasType::Oxygen);
        filter_item.set_size(FilterSize::Small);
        filter_item.set_quantity(1);
        let filter: Shared<dyn Item> = shared(filter_item);
        let _ = slot.try_insert(filter);
    }
    f2.borrow_mut()
        .set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(input2_rc.clone()))
        .unwrap();
    f2.borrow_mut()
        .set_atmospheric_network(
            DeviceAtmosphericNetworkType::Output,
            Some(filtered2_rc.clone()),
        )
        .unwrap();
    f2.borrow_mut()
        .set_atmospheric_network(
            DeviceAtmosphericNetworkType::Output2,
            Some(waste2_rc.clone()),
        )
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
    let ac = AirConditioner::new(SimulationDeviceSettings {
        id: Some(1),
        ..SimulationDeviceSettings::default()
    });

    let input_rc = AtmosphericNetwork::new(100.0);
    let output_rc = AtmosphericNetwork::new(100.0);
    let waste_rc = AtmosphericNetwork::new(100.0);

    // configure networks and set a goal temperature
    input_rc.borrow_mut().add_gas(GasType::Oxygen, 10.0, 260.0);
    output_rc.borrow_mut().add_gas(GasType::Oxygen, 10.0, 260.0);
    waste_rc.borrow_mut().add_gas(GasType::Oxygen, 10.0, 200.0);

    ac.borrow_mut()
        .set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(input_rc.clone()))
        .unwrap();
    ac.borrow_mut()
        .set_atmospheric_network(
            DeviceAtmosphericNetworkType::Output,
            Some(output_rc.clone()),
        )
        .unwrap();
    ac.borrow_mut()
        .set_atmospheric_network(
            DeviceAtmosphericNetworkType::Output2,
            Some(waste_rc.clone()),
        )
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
