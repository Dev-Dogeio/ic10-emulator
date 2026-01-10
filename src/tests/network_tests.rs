//! Unit tests for networks
#[cfg(test)]
mod tests {
    use crate::devices::SimulationDeviceSettings;
    use crate::devices::property_descriptor::{PropertyDescriptor, PropertyRegistry};
    use crate::error::SimulationError;
    use crate::parser::string_to_hash;
    use crate::types::{OptShared, OptWeakShared, shared};
    use crate::{BatchMode, Device, LogicType, SimulationResult};
    use crate::{CableNetwork, devices::ICHousing};
    use std::cell::Cell;
    use std::sync::OnceLock;

    /// Test device for network testing
    #[derive(Debug)]
    struct TestNetworkDevice {
        id: i32,
        prefab_hash: i32,
        name_hash: i32,
        setting: Cell<f64>,
    }

    impl TestNetworkDevice {
        fn new(id: i32, prefab_hash: i32, name_hash: i32) -> Self {
            Self {
                id,
                prefab_hash,
                name_hash,
                setting: Cell::new(0.0),
            }
        }
    }

    impl Device for TestNetworkDevice {
        fn get_id(&self) -> i32 {
            self.id
        }

        fn get_prefab_hash(&self) -> i32 {
            self.prefab_hash
        }

        fn get_name_hash(&self) -> i32 {
            self.name_hash
        }

        fn get_name(&self) -> &str {
            "Test"
        }
        fn get_network(&self) -> OptShared<CableNetwork> {
            None
        }
        fn set_network(&mut self, _network: OptWeakShared<CableNetwork>) {}

        fn rename(&mut self, _name: &str) {}

        fn can_read(&self, logic_type: LogicType) -> bool {
            matches!(logic_type, LogicType::Setting)
        }

        fn can_write(&self, logic_type: LogicType) -> bool {
            matches!(logic_type, LogicType::Setting)
        }

        fn read(&self, logic_type: LogicType) -> SimulationResult<f64> {
            match logic_type {
                LogicType::Setting => Ok(self.setting.get()),
                _ => Err(SimulationError::RuntimeError {
                    message: "Unsupported logic type".to_string(),
                    line: 0,
                }),
            }
        }

        fn write(&self, logic_type: LogicType, value: f64) -> SimulationResult<()> {
            match logic_type {
                LogicType::Setting => {
                    self.setting.set(value);
                    Ok(())
                }
                _ => Err(SimulationError::RuntimeError {
                    message: "Unsupported logic type".to_string(),
                    line: 0,
                }),
            }
        }

        fn supported_types(&self) -> Vec<LogicType> {
            vec![LogicType::Setting]
        }

        fn properties() -> &'static PropertyRegistry<Self>
        where
            Self: Sized,
        {
            static REG: OnceLock<PropertyRegistry<TestNetworkDevice>> = OnceLock::new();
            REG.get_or_init(|| {
                const DESCRIPTORS: &[PropertyDescriptor<TestNetworkDevice>] =
                    &[PropertyDescriptor::read_write(
                        LogicType::Setting,
                        |device, _| Ok(device.setting.get()),
                        |device, _, value| {
                            device.setting.set(value);
                            Ok(())
                        },
                    )];
                PropertyRegistry::new(DESCRIPTORS)
            })
        }

        fn display_name_static() -> &'static str
        where
            Self: Sized,
        {
            "TestNetworkDevice"
        }
    }

    #[test]
    fn test_add_and_get_device() {
        let network = CableNetwork::new();
        let device = shared(TestNetworkDevice::new(1, 100, 200));

        network
            .borrow_mut()
            .add_device(device.clone(), network.clone());

        assert!(network.borrow().device_exists(1));
        assert_eq!(network.borrow().device_count(), 1);

        {
            let net = network.borrow();
            let retrieved = net.get_device(1).unwrap();
            assert_eq!(retrieved.get_id(), 1);
            assert_eq!(retrieved.get_prefab_hash(), 100);
        }
    }

    #[test]
    fn test_remove_device() {
        let network = CableNetwork::new();
        let device = shared(TestNetworkDevice::new(1, 100, 200));

        network.borrow_mut().add_device(device, network.clone());
        assert!(network.borrow().device_exists(1));

        network.borrow_mut().remove_device(1);
        assert!(!network.borrow().device_exists(1));
        assert_eq!(network.borrow().device_count(), 0);
    }

    #[test]
    fn test_get_devices_by_prefab() {
        let network = CableNetwork::new();

        // Add 3 devices with same prefab hash
        for i in 1..=3 {
            let device = shared(TestNetworkDevice::new(i, 100, 200));
            network.borrow_mut().add_device(device, network.clone());
        }

        // Add 2 devices with different prefab hash
        for i in 4..=5 {
            let device = shared(TestNetworkDevice::new(i, 999, 200));
            network.borrow_mut().add_device(device, network.clone());
        }

        let prefab_100_devices = network.borrow().get_devices_by_prefab(100);
        assert_eq!(prefab_100_devices.len(), 3);

        let prefab_999_devices = network.borrow().get_devices_by_prefab(999);
        assert_eq!(prefab_999_devices.len(), 2);

        let prefab_empty = network.borrow().get_devices_by_prefab(888);
        assert!(prefab_empty.is_empty());
    }

    #[test]
    fn test_get_devices_by_name() {
        let network = CableNetwork::new();

        // Add devices with different name hashes
        network
            .borrow_mut()
            .add_device(shared(TestNetworkDevice::new(1, 100, 200)), network.clone());
        network
            .borrow_mut()
            .add_device(shared(TestNetworkDevice::new(2, 100, 200)), network.clone());
        network
            .borrow_mut()
            .add_device(shared(TestNetworkDevice::new(3, 100, 300)), network.clone());

        let name_200_devices = network.borrow().get_devices_by_name(200);
        assert_eq!(name_200_devices.len(), 2);

        let name_300_devices = network.borrow().get_devices_by_name(300);
        assert_eq!(name_300_devices.len(), 1);
    }

    #[test]
    fn test_batch_read_average() {
        let network = CableNetwork::new();

        // Add devices with different settings
        for (i, val) in [10.0, 20.0, 30.0].iter().enumerate() {
            let device = shared(TestNetworkDevice::new(i as i32 + 1, 100, 200));
            device.borrow_mut().setting.set(*val);
            network.borrow_mut().add_device(device, network.clone());
        }

        let avg = network
            .borrow()
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Average)
            .unwrap();
        assert_eq!(avg, 20.0); // (10 + 20 + 30) / 3
    }

    #[test]
    fn test_batch_read_sum() {
        let network = CableNetwork::new();

        for (i, val) in [10.0, 20.0, 30.0].iter().enumerate() {
            let device = shared(TestNetworkDevice::new(i as i32 + 1, 100, 200));
            device.borrow_mut().setting.set(*val);
            network.borrow_mut().add_device(device, network.clone());
        }

        let sum = network
            .borrow()
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Sum)
            .unwrap();
        assert_eq!(sum, 60.0);
    }

    #[test]
    fn test_batch_read_min_max() {
        let network = CableNetwork::new();

        for (i, val) in [10.0, 5.0, 30.0].iter().enumerate() {
            let device = shared(TestNetworkDevice::new(i as i32 + 1, 100, 200));
            device.borrow_mut().setting.set(*val);
            network.borrow_mut().add_device(device, network.clone());
        }

        let min = network
            .borrow()
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Minimum)
            .unwrap();
        assert_eq!(min, 5.0);

        let max = network
            .borrow()
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Maximum)
            .unwrap();
        assert_eq!(max, 30.0);
    }

    #[test]
    fn test_batch_write() {
        let network = CableNetwork::new();

        // Add 3 devices with same prefab hash
        let devices: Vec<_> = (1..=3)
            .map(|i| shared(TestNetworkDevice::new(i, 100, 200)))
            .collect();

        for device in &devices {
            network
                .borrow_mut()
                .add_device(device.clone(), network.clone());
        }

        // Batch write to all devices
        let count = network
            .borrow()
            .batch_write_by_prefab(100, LogicType::Setting, 42.0)
            .unwrap();
        assert_eq!(count, 3);

        // Verify all devices were updated
        for device in &devices {
            assert_eq!(device.borrow().setting.get(), 42.0);
        }
    }

    #[test]
    fn test_batch_read_by_name() {
        let network = CableNetwork::new();

        // Add devices with different name hashes
        let device1 = shared(TestNetworkDevice::new(1, 100, 200));
        device1.borrow_mut().setting.set(10.0);
        network.borrow_mut().add_device(device1, network.clone());

        let device2 = shared(TestNetworkDevice::new(2, 100, 200));
        device2.borrow_mut().setting.set(20.0);
        network.borrow_mut().add_device(device2, network.clone());

        let device3 = shared(TestNetworkDevice::new(3, 100, 300));
        device3.borrow_mut().setting.set(100.0);
        network.borrow_mut().add_device(device3, network.clone());
        // Read only devices with prefab 100 AND name 200
        let avg = network
            .borrow()
            .batch_read_by_name(100, 200, LogicType::Setting, BatchMode::Average)
            .unwrap();
        assert_eq!(avg, 15.0); // (10 + 20) / 2, excludes device3
    }

    #[test]
    fn test_batch_write_by_name() {
        let network = CableNetwork::new();

        let device1 = shared(TestNetworkDevice::new(1, 100, 200));
        network
            .borrow_mut()
            .add_device(device1.clone(), network.clone());

        let device2 = shared(TestNetworkDevice::new(2, 100, 200));
        network
            .borrow_mut()
            .add_device(device2.clone(), network.clone());

        let device3 = shared(TestNetworkDevice::new(3, 100, 300));
        network
            .borrow_mut()
            .add_device(device3.clone(), network.clone());

        // Write only to devices with prefab 100 AND name 200
        let count = network
            .borrow()
            .batch_write_by_name(100, 200, LogicType::Setting, 99.0)
            .unwrap();
        assert_eq!(count, 2);

        // Verify only matching devices were updated
        assert_eq!(device1.borrow().setting.get(), 99.0);
        assert_eq!(device2.borrow().setting.get(), 99.0);
        assert_eq!(device3.borrow().setting.get(), 0.0); // unchanged
    }

    #[test]
    fn test_batch_mode_from_value() {
        assert_eq!(BatchMode::from_value(0.0), Some(BatchMode::Average));
        assert_eq!(BatchMode::from_value(1.0), Some(BatchMode::Sum));
        assert_eq!(BatchMode::from_value(2.0), Some(BatchMode::Minimum));
        assert_eq!(BatchMode::from_value(3.0), Some(BatchMode::Maximum));
        assert_eq!(BatchMode::from_value(4.0), None);
    }

    #[test]
    fn test_empty_batch_returns_zero() {
        let network = CableNetwork::new();

        let result = network
            .borrow()
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Average)
            .unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_housing_on_network() {
        let network = CableNetwork::new();
        let housing = ICHousing::new(SimulationDeviceSettings {
            id: Some(1),
            ..SimulationDeviceSettings::default()
        });

        network
            .borrow_mut()
            .add_device(housing.clone(), network.clone());

        assert_eq!(network.borrow().device_count(), 1);
        assert!(network.borrow().device_exists(housing.borrow().get_id()));
    }

    #[test]
    fn test_device_rename_updates_network() {
        let network = CableNetwork::new();
        let housing = ICHousing::new(SimulationDeviceSettings {
            id: Some(1),
            ..SimulationDeviceSettings::default()
        });
        let device_id = housing.borrow().get_id();

        // Add device to network
        network
            .borrow_mut()
            .add_device(housing.clone(), network.clone());

        // Get initial name hash
        let old_name_hash = housing.borrow().get_name_hash();

        // Verify device can be found by old name hash
        let devices_by_old_name = network.borrow().get_devices_by_name(old_name_hash);
        assert_eq!(devices_by_old_name.len(), 1);
        assert_eq!(devices_by_old_name[0], device_id);

        // Rename the device
        housing.borrow_mut().rename("NewName");

        // Get new name hash
        let new_name_hash = string_to_hash("NewName");
        assert_eq!(housing.borrow().get_name_hash(), new_name_hash);

        // Verify device can be found by new name hash
        let devices_by_new_name = network.borrow().get_devices_by_name(new_name_hash);
        assert_eq!(devices_by_new_name.len(), 1);
        assert_eq!(devices_by_new_name[0], device_id);

        // Verify device cannot be found by old name hash
        let devices_by_old_name = network.borrow().get_devices_by_name(old_name_hash);
        assert_eq!(devices_by_old_name.len(), 0);
    }
}
