//! Unit tests for the simulation manager
#[cfg(test)]
mod tests {
    use crate::{
        SimulationManager,
        devices::{
            AirConditioner, AtmosphericDevice, DaylightSensor, DeviceAtmosphericNetworkType,
            Filtration, ICHostDevice, ICHousing, LogicMemory, VolumePump,
        },
        items::ItemIntegratedCircuit10,
        networks::{AtmosphericNetwork, CableNetwork},
        types::shared,
    };

    #[test]
    fn test_reset_drops_devices_and_networks() {
        // Start from a clean state
        SimulationManager::reset_global();

        let cn = CableNetwork::new();

        // Create one of each device
        let ac = AirConditioner::new(None);
        let ac_weak = std::rc::Rc::downgrade(&ac);

        let fil = Filtration::new(None);
        let fil_weak = std::rc::Rc::downgrade(&fil);

        let pump = VolumePump::new(None);
        let pump_weak = std::rc::Rc::downgrade(&pump);

        let housing = ICHousing::new(None);
        let housing_weak = std::rc::Rc::downgrade(&housing);

        let lm = LogicMemory::new(None);
        let lm_weak = std::rc::Rc::downgrade(&lm);

        let ds = DaylightSensor::new(None);
        let ds_weak = std::rc::Rc::downgrade(&ds);

        // Add all devices to the cable network
        cn.borrow_mut().add_device(ac.clone(), cn.clone());
        cn.borrow_mut().add_device(fil.clone(), cn.clone());
        cn.borrow_mut().add_device(pump.clone(), cn.clone());
        cn.borrow_mut().add_device(housing.clone(), cn.clone());
        cn.borrow_mut().add_device(lm.clone(), cn.clone());
        cn.borrow_mut().add_device(ds.clone(), cn.clone());

        // Insert an IC chip into each IC host device (AirConditioner, Filtration, ICHousing)
        let chip_ac = shared(ItemIntegratedCircuit10::new());
        let chip_ac_weak = std::rc::Rc::downgrade(&chip_ac);
        ac.borrow_mut().set_chip(chip_ac.clone());
        drop(chip_ac);

        let chip_fil = shared(ItemIntegratedCircuit10::new());
        let chip_fil_weak = std::rc::Rc::downgrade(&chip_fil);
        fil.borrow_mut().set_chip(chip_fil.clone());
        drop(chip_fil);

        let chip_housing = shared(ItemIntegratedCircuit10::new());
        let chip_housing_weak = std::rc::Rc::downgrade(&chip_housing);
        housing.borrow_mut().set_chip(chip_housing.clone());
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
        let an_rep_weak = std::rc::Rc::downgrade(&an_ac_in);

        // Also keep a weak to the AirConditioner's internal network
        let ac_internal = ac
            .borrow()
            .get_atmospheric_network(DeviceAtmosphericNetworkType::Internal)
            .unwrap();
        let ac_internal_weak = std::rc::Rc::downgrade(&ac_internal);

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
        SimulationManager::reset_global();

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
}
