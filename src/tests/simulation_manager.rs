//! Unit tests for the simulation manager
#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        Filter, SimulationManager,
        devices::{
            AirConditioner, AtmosphericDevice, DaylightSensor, Device,
            DeviceAtmosphericNetworkType, Filtration, ICHostDevice, ICHousing, LogicMemory,
            SimulationDeviceSettings, SlotHostDevice, VolumePump,
        },
        items::{FilterSize, ItemIntegratedCircuit10, SimulationItemSettings},
        networks::AtmosphericNetwork,
        types::shared,
    };

    #[test]
    fn test_reset_drops_devices_and_networks() {
        // Start from a clean state
        let mut manager = SimulationManager::new();

        let cn = manager.create_cable_network();

        // Create one of each device
        let ac = AirConditioner::new(SimulationDeviceSettings {
            id: Some(1),
            ..SimulationDeviceSettings::default()
        });
        let ac_weak = Rc::downgrade(&ac);

        let fil = Filtration::new(SimulationDeviceSettings {
            id: Some(2),
            ..SimulationDeviceSettings::default()
        });
        let fil_weak = Rc::downgrade(&fil);

        let pump = VolumePump::new(SimulationDeviceSettings {
            id: Some(3),
            ..SimulationDeviceSettings::default()
        });
        let pump_weak = Rc::downgrade(&pump);

        let housing = ICHousing::new(SimulationDeviceSettings {
            id: Some(4),
            ..SimulationDeviceSettings::default()
        });
        let housing_weak = Rc::downgrade(&housing);

        let lm = LogicMemory::new(SimulationDeviceSettings {
            id: Some(5),
            ..SimulationDeviceSettings::default()
        });
        let lm_weak = Rc::downgrade(&lm);

        let ds = DaylightSensor::new(SimulationDeviceSettings {
            id: Some(6),
            ..SimulationDeviceSettings::default()
        });
        let ds_weak = Rc::downgrade(&ds);

        // Add all devices to the cable network
        cn.borrow_mut().add_device(ac.clone(), cn.clone()).unwrap();
        cn.borrow_mut().add_device(fil.clone(), cn.clone()).unwrap();
        cn.borrow_mut()
            .add_device(pump.clone(), cn.clone())
            .unwrap();
        cn.borrow_mut()
            .add_device(housing.clone(), cn.clone())
            .unwrap();
        cn.borrow_mut().add_device(lm.clone(), cn.clone()).unwrap();
        cn.borrow_mut().add_device(ds.clone(), cn.clone()).unwrap();

        // Insert an IC chip into each IC host device (AirConditioner, Filtration, ICHousing)
        let chip_ac = shared(ItemIntegratedCircuit10::new(SimulationItemSettings {
            id: Some(7),
            ..SimulationItemSettings::default()
        }));
        let chip_ac_weak = Rc::downgrade(&chip_ac);
        ac.borrow().set_chip(chip_ac.clone()).unwrap();
        drop(chip_ac);

        let chip_fil = shared(ItemIntegratedCircuit10::new(SimulationItemSettings {
            id: Some(8),
            ..SimulationItemSettings::default()
        }));
        let chip_fil_weak = Rc::downgrade(&chip_fil);
        fil.borrow().set_chip(chip_fil.clone()).unwrap();
        drop(chip_fil);

        let chip_housing = shared(ItemIntegratedCircuit10::new(SimulationItemSettings {
            id: Some(9),
            ..SimulationItemSettings::default()
        }));
        let chip_housing_weak = Rc::downgrade(&chip_housing);
        housing.borrow().set_chip(chip_housing.clone()).unwrap();
        drop(chip_housing);

        // Create and attach atmospheric networks to atmospheric devices
        let an_ac_in = AtmosphericNetwork::new(40.0);
        let an_ac_out = AtmosphericNetwork::new(40.0);
        let an_ac_waste = AtmosphericNetwork::new(40.0);

        ac.borrow_mut()
            .set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(an_ac_in.clone()))
            .unwrap();
        ac.borrow_mut()
            .set_atmospheric_network(
                DeviceAtmosphericNetworkType::Output,
                Some(an_ac_out.clone()),
            )
            .unwrap();
        ac.borrow_mut()
            .set_atmospheric_network(
                DeviceAtmosphericNetworkType::Output2,
                Some(an_ac_waste.clone()),
            )
            .unwrap();

        let an_fil_in = AtmosphericNetwork::new(20.0);
        let an_fil_out = AtmosphericNetwork::new(20.0);
        let an_fil_waste = AtmosphericNetwork::new(20.0);

        fil.borrow_mut()
            .set_atmospheric_network(DeviceAtmosphericNetworkType::Input, Some(an_fil_in.clone()))
            .unwrap();
        fil.borrow_mut()
            .set_atmospheric_network(
                DeviceAtmosphericNetworkType::Output,
                Some(an_fil_out.clone()),
            )
            .unwrap();
        fil.borrow_mut()
            .set_atmospheric_network(
                DeviceAtmosphericNetworkType::Output2,
                Some(an_fil_waste.clone()),
            )
            .unwrap();

        let an_pump_in = AtmosphericNetwork::new(10.0);
        let an_pump_out = AtmosphericNetwork::new(10.0);
        pump.borrow_mut()
            .set_atmospheric_network(
                DeviceAtmosphericNetworkType::Input,
                Some(an_pump_in.clone()),
            )
            .unwrap();
        pump.borrow_mut()
            .set_atmospheric_network(
                DeviceAtmosphericNetworkType::Output,
                Some(an_pump_out.clone()),
            )
            .unwrap();

        // Keep a weak to one representative atmospheric network
        let an_rep_weak = Rc::downgrade(&an_ac_in);

        // Also keep a weak to the AirConditioner's internal network
        let ac_internal = ac
            .borrow()
            .get_atmospheric_network(DeviceAtmosphericNetworkType::Internal)
            .unwrap();
        let ac_internal_weak = Rc::downgrade(&ac_internal);

        // Drop local strong refs so only global manager holds references
        drop(ac);
        drop(fil);
        drop(pump);
        drop(housing);
        drop(lm);
        drop(ds);

        // Drop local networks too so manager must be the only owner
        drop(an_ac_in);
        drop(an_ac_out);
        drop(an_ac_waste);
        drop(an_fil_in);
        drop(an_fil_out);
        drop(an_fil_waste);
        drop(an_pump_in);
        drop(an_pump_out);
        drop(ac_internal);

        // Also drop our local handle to the cable network
        drop(cn);

        // Reset everything
        manager.reset();

        // All devices and networks should be fully dropped
        assert!(
            ac_weak.upgrade().is_none(),
            "AirConditioner should be dropped"
        );
        assert!(fil_weak.upgrade().is_none(), "Filtration should be dropped");
        assert!(
            pump_weak.upgrade().is_none(),
            "VolumePump should be dropped"
        );
        assert!(
            housing_weak.upgrade().is_none(),
            "ICHousing should be dropped"
        );
        assert!(lm_weak.upgrade().is_none(), "LogicMemory should be dropped");
        assert!(
            ds_weak.upgrade().is_none(),
            "DaylightSensor should be dropped"
        );

        assert!(
            an_rep_weak.upgrade().is_none(),
            "Representative atmospheric network should be dropped"
        );
        assert!(
            ac_internal_weak.upgrade().is_none(),
            "AirConditioner internal atmospheric network should be dropped"
        );

        // Chips inserted into hosts should be dropped as well
        assert!(
            chip_ac_weak.upgrade().is_none(),
            "Chip in AirConditioner should be dropped"
        );
        assert!(
            chip_fil_weak.upgrade().is_none(),
            "Chip in Filtration should be dropped"
        );
        assert!(
            chip_housing_weak.upgrade().is_none(),
            "Chip in ICHousing should be dropped"
        );
    }

    #[test]
    fn test_simulation_manager_device_item_enumeration() {
        let mut manager = SimulationManager::new();

        let cn = manager.create_cable_network();

        // Create Filtration device and register it on the network
        let fil = Filtration::new(SimulationDeviceSettings {
            id: Some(1),
            ..SimulationDeviceSettings::default()
        });
        cn.borrow_mut().add_device(fil.clone(), cn.clone()).unwrap();

        // Insert an Oxygen Large filter into slot 0
        let mut f = Filter::new(SimulationItemSettings {
            id: Some(2),
            ..SimulationItemSettings::default()
        });
        f.set_gas_type(crate::atmospherics::GasType::Oxygen);
        f.set_size(FilterSize::Large);
        f.set_quantity(42);
        fil.borrow_mut().try_insert_item(0, shared(f)).unwrap();

        let out = format!("{}", manager);
        assert!(
            out.contains("Items:"),
            "Output should include item enumeration: {}",
            out
        );
        assert!(
            out.contains("Oxygen Filter (Large)"),
            "Should include filter display name: {}",
            out
        );
        assert!(out.contains("slot 0"), "Should include slot index: {}", out);
    }

    #[test]
    fn test_simulation_settings_internal_network_used_by_devices() {
        let an_internal = AtmosphericNetwork::new(50.0);
        let settings = SimulationDeviceSettings {
            ticks_per_day: Some(2400.0),
            max_instructions_per_tick: Some(128),
            name: None,
            id: Some(1),
            internal_atmospheric_network: Some(an_internal.clone()),
        };
        let ac = AirConditioner::new(settings);
        let ac_internal = ac
            .borrow()
            .get_atmospheric_network(DeviceAtmosphericNetworkType::Internal)
            .unwrap();
        assert!(Rc::ptr_eq(&an_internal, &ac_internal));
    }

    #[test]
    fn test_manager_creates_and_registers_internal_atmo_network() {
        let mut manager = SimulationManager::new();
        assert_eq!(manager.all_atmospheric_networks().len(), 0);

        let d = manager
            .create_device(AirConditioner::PREFAB_HASH, None)
            .expect("Device creation failed");

        let device = d.borrow();

        let atmospheric_device = device
            .as_atmospheric_device()
            .expect("Device should be atmospheric device");

        let internal_net = atmospheric_device
            .get_atmospheric_network(DeviceAtmosphericNetworkType::Internal)
            .expect("Expected internal network created");

        let id = internal_net.borrow().get_id().expect("id should be set");

        // The network should have an id assigned and be discoverable via the manager
        let found = manager
            .get_atmospheric_network_by_id(id)
            .expect("manager should have registered the network");
        assert!(Rc::ptr_eq(&found, &internal_net));
        assert_eq!(manager.all_atmospheric_networks().len(), 1);
    }

    #[test]
    fn test_simulation_settings_id_used_by_all_devices() {
        // Create distinct settings for each device with explicit ids
        // Use negative IDs to avoid clashes with other tests that allocate global IDs concurrently
        let ac_settings = SimulationDeviceSettings {
            ticks_per_day: Some(2400.0),
            max_instructions_per_tick: Some(128),
            name: None,
            id: Some(-1100),
            internal_atmospheric_network: None,
        };
        let fil_settings = SimulationDeviceSettings {
            ticks_per_day: Some(2400.0),
            max_instructions_per_tick: Some(128),
            name: None,
            id: Some(-1101),
            internal_atmospheric_network: None,
        };
        let pump_settings = SimulationDeviceSettings {
            ticks_per_day: Some(2400.0),
            max_instructions_per_tick: Some(128),
            name: None,
            id: Some(-1102),
            internal_atmospheric_network: None,
        };
        let housing_settings = SimulationDeviceSettings {
            ticks_per_day: Some(2400.0),
            max_instructions_per_tick: Some(128),
            name: None,
            id: Some(-1103),
            internal_atmospheric_network: None,
        };
        let ds_settings = SimulationDeviceSettings {
            ticks_per_day: Some(2400.0),
            max_instructions_per_tick: Some(128),
            name: None,
            id: Some(-1104),
            internal_atmospheric_network: None,
        };
        let lm_settings = SimulationDeviceSettings {
            ticks_per_day: Some(2400.0),
            max_instructions_per_tick: Some(128),
            name: None,
            id: Some(-1105),
            internal_atmospheric_network: None,
        };

        let ac = AirConditioner::new(ac_settings);
        let fil = Filtration::new(fil_settings);
        let pump = VolumePump::new(pump_settings);
        let housing = ICHousing::new(housing_settings);
        let ds = DaylightSensor::new(ds_settings);
        let lm = LogicMemory::new(lm_settings);

        assert_eq!(ac.borrow().get_id(), -1100);
        assert_eq!(fil.borrow().get_id(), -1101);
        assert_eq!(pump.borrow().get_id(), -1102);
        assert_eq!(housing.borrow().get_id(), -1103);
        assert_eq!(ds.borrow().get_id(), -1104);
        assert_eq!(lm.borrow().get_id(), -1105);
    }
}
