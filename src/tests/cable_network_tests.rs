#[cfg(test)]
mod tests {
    use crate::Device;
    use crate::IC10Result;
    use crate::cable_network::{BatchMode, CableNetwork};
    use crate::devices::LogicType;
    use crate::error::IC10Error;
    use crate::types::{OptShared, shared};
    use std::cell::{Cell, RefCell};

    /// Test device for cable network testing
    #[derive(Debug)]
    struct MockDevice {
        id: i32,
        prefab_hash: i32,
        name_hash: Cell<i32>,
        name: RefCell<String>,
        setting: Cell<f64>,
        horizontal: Cell<f64>,
        vertical: Cell<f64>,
        network: RefCell<OptShared<CableNetwork>>,
    }

    impl MockDevice {
        fn new(id: i32, prefab_hash: i32, name_hash: i32) -> Self {
            Self {
                id,
                prefab_hash,
                name_hash: Cell::new(name_hash),
                name: RefCell::new(String::from("MockDevice")),
                setting: Cell::new(0.0),
                horizontal: Cell::new(0.0),
                vertical: Cell::new(0.0),
                network: RefCell::new(None),
            }
        }

        fn with_values(
            id: i32,
            prefab_hash: i32,
            name_hash: i32,
            setting: f64,
            horizontal: f64,
            vertical: f64,
        ) -> Self {
            Self {
                id,
                prefab_hash,
                name_hash: Cell::new(name_hash),
                name: RefCell::new(String::from("MockDevice")),
                setting: Cell::new(setting),
                horizontal: Cell::new(horizontal),
                vertical: Cell::new(vertical),
                network: RefCell::new(None),
            }
        }
    }

    impl Device for MockDevice {
        fn get_id(&self) -> i32 {
            self.id
        }

        fn get_prefab_hash(&self) -> i32 {
            self.prefab_hash
        }

        fn get_name_hash(&self) -> i32 {
            self.name_hash.get()
        }

        fn get_name(&self) -> &str {
            // This is safe because we're returning a reference to the borrowed string
            // that lives as long as the borrow of self
            unsafe { &*self.name.as_ptr() }
        }

        fn get_network(&self) -> OptShared<CableNetwork> {
            self.network.borrow().clone()
        }

        fn set_network(&mut self, network: OptShared<CableNetwork>) {
            *self.network.borrow_mut() = network;
        }

        fn set_name(&mut self, name: &str) {
            *self.name.borrow_mut() = name.to_string();
        }

        fn can_read(&self, logic_type: LogicType) -> bool {
            matches!(
                logic_type,
                LogicType::Setting | LogicType::Horizontal | LogicType::Vertical
            )
        }

        fn can_write(&self, logic_type: LogicType) -> bool {
            matches!(
                logic_type,
                LogicType::Setting | LogicType::Horizontal | LogicType::Vertical
            )
        }

        fn read(&self, logic_type: LogicType) -> IC10Result<f64> {
            match logic_type {
                LogicType::Setting => Ok(self.setting.get()),
                LogicType::Horizontal => Ok(self.horizontal.get()),
                LogicType::Vertical => Ok(self.vertical.get()),
                _ => Err(IC10Error::RuntimeError {
                    message: format!("Cannot read {:?}", logic_type),
                    line: 0,
                }),
            }
        }

        fn write(&self, logic_type: LogicType, value: f64) -> IC10Result<()> {
            match logic_type {
                LogicType::Setting => {
                    self.setting.set(value);
                    Ok(())
                }
                LogicType::Horizontal => {
                    self.horizontal.set(value);
                    Ok(())
                }
                LogicType::Vertical => {
                    self.vertical.set(value);
                    Ok(())
                }
                _ => Err(IC10Error::RuntimeError {
                    message: format!("Cannot write {:?}", logic_type),
                    line: 0,
                }),
            }
        }
    }

    // ==================== Basic Device Management Tests ====================

    #[test]
    fn test_new_network_is_empty() {
        let network = CableNetwork::new();
        assert_eq!(network.device_count(), 0);
        assert!(network.all_device_ids().is_empty());
    }

    #[test]
    fn test_add_single_device() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());
        let device = shared(MockDevice::new(1, 100, 200));

        network.add_device(device.clone(), network_rc.clone());

        assert_eq!(network.device_count(), 1);
        assert!(network.device_exists(1));
        assert_eq!(network.all_device_ids(), vec![1]);
    }

    #[test]
    fn test_add_multiple_devices() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        for i in 1..=5 {
            let device = shared(MockDevice::new(i, 100, 200));
            network.add_device(device, network_rc.clone());
        }

        assert_eq!(network.device_count(), 5);
        assert_eq!(network.all_device_ids(), vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_device_ref_id_ordering() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices in non-sequential order
        let ids = vec![5, 2, 8, 1, 3];
        for &id in &ids {
            let device = shared(MockDevice::new(id, 100, 200));
            network.add_device(device, network_rc.clone());
        }

        // All device IDs should be in sorted order
        let all_ids = network.all_device_ids();
        assert_eq!(all_ids, vec![1, 2, 3, 5, 8]);
    }

    #[test]
    fn test_remove_device() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());
        let device = shared(MockDevice::new(1, 100, 200));

        network.add_device(device, network_rc);
        assert!(network.device_exists(1));

        let removed = network.remove_device(1);
        assert!(removed.is_some());
        assert!(!network.device_exists(1));
        assert_eq!(network.device_count(), 0);
    }

    #[test]
    fn test_remove_nonexistent_device() {
        let mut network = CableNetwork::new();
        let removed = network.remove_device(999);
        assert!(removed.is_none());
    }

    #[test]
    fn test_remove_device_from_indices() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with same prefab and name hash
        for i in 1..=3 {
            let device = shared(MockDevice::new(i, 100, 200));
            network.add_device(device, network_rc.clone());
        }

        assert_eq!(network.get_devices_by_prefab(100).len(), 3);
        assert_eq!(network.get_devices_by_name(200).len(), 3);

        // Remove one device
        network.remove_device(2);

        let prefab_devices = network.get_devices_by_prefab(100);
        assert_eq!(prefab_devices.len(), 2);
        assert_eq!(prefab_devices, vec![1, 3]);

        let name_devices = network.get_devices_by_name(200);
        assert_eq!(name_devices.len(), 2);
        assert_eq!(name_devices, vec![1, 3]);
    }

    #[test]
    fn test_clear_network() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        for i in 1..=5 {
            let device = shared(MockDevice::new(i, 100, 200));
            network.add_device(device, network_rc.clone());
        }

        assert_eq!(network.device_count(), 5);
        network.clear();
        assert_eq!(network.device_count(), 0);
        assert!(network.all_device_ids().is_empty());
    }

    // ==================== Device Index Tests ====================

    #[test]
    fn test_prefab_index_ordering() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with same prefab in random order
        let ids = vec![7, 2, 9, 4, 1];
        for &id in &ids {
            let device = shared(MockDevice::new(id, 100, 200));
            network.add_device(device, network_rc.clone());
        }

        let prefab_devices = network.get_devices_by_prefab(100);
        // Should be sorted by reference ID
        assert_eq!(prefab_devices, vec![1, 2, 4, 7, 9]);
    }

    #[test]
    fn test_name_index_ordering() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with same name hash in random order
        let ids = vec![6, 3, 8, 1, 5];
        for &id in &ids {
            let device = shared(MockDevice::new(id, 100, 200));
            network.add_device(device, network_rc.clone());
        }

        let name_devices = network.get_devices_by_name(200);
        // Should be sorted by reference ID
        assert_eq!(name_devices, vec![1, 3, 5, 6, 8]);
    }

    #[test]
    fn test_multiple_prefab_hashes() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with different prefab hashes
        network.add_device(shared(MockDevice::new(1, 100, 200)), network_rc.clone());
        network.add_device(shared(MockDevice::new(2, 100, 200)), network_rc.clone());
        network.add_device(shared(MockDevice::new(3, 200, 200)), network_rc.clone());
        network.add_device(shared(MockDevice::new(4, 200, 200)), network_rc.clone());
        network.add_device(shared(MockDevice::new(5, 300, 200)), network_rc.clone());

        assert_eq!(network.get_devices_by_prefab(100), vec![1, 2]);
        assert_eq!(network.get_devices_by_prefab(200), vec![3, 4]);
        assert_eq!(network.get_devices_by_prefab(300), vec![5]);
        assert!(network.get_devices_by_prefab(999).is_empty());
    }

    #[test]
    fn test_multiple_name_hashes() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with different name hashes
        network.add_device(shared(MockDevice::new(1, 100, 1000)), network_rc.clone());
        network.add_device(shared(MockDevice::new(2, 100, 1000)), network_rc.clone());
        network.add_device(shared(MockDevice::new(3, 100, 2000)), network_rc.clone());
        network.add_device(shared(MockDevice::new(4, 100, 2000)), network_rc.clone());
        network.add_device(shared(MockDevice::new(5, 100, 3000)), network_rc.clone());

        assert_eq!(network.get_devices_by_name(1000), vec![1, 2]);
        assert_eq!(network.get_devices_by_name(2000), vec![3, 4]);
        assert_eq!(network.get_devices_by_name(3000), vec![5]);
        assert!(network.get_devices_by_name(9999).is_empty());
    }

    #[test]
    fn test_count_devices_by_prefab() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        for i in 1..=5 {
            network.add_device(shared(MockDevice::new(i, 100, 200)), network_rc.clone());
        }

        assert_eq!(network.count_devices_by_prefab(100), 5);
        assert_eq!(network.count_devices_by_prefab(999), 0);
    }

    #[test]
    fn test_count_devices_by_name() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        for i in 1..=3 {
            network.add_device(shared(MockDevice::new(i, 100, 200)), network_rc.clone());
        }

        assert_eq!(network.count_devices_by_name(200), 3);
        assert_eq!(network.count_devices_by_name(999), 0);
    }

    #[test]
    fn test_update_device_name() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());
        let device = shared(MockDevice::new(1, 100, 200));

        network.add_device(device.clone(), network_rc);

        // Initially in name index 200
        assert_eq!(network.get_devices_by_name(200), vec![1]);
        assert!(network.get_devices_by_name(300).is_empty());

        // Update the name hash
        device.borrow().name_hash.set(300);
        network.update_device_name(1, 200, 300);

        // Should now be in name index 300
        assert!(network.get_devices_by_name(200).is_empty());
        assert_eq!(network.get_devices_by_name(300), vec![1]);
    }

    // ==================== Batch Read Tests ====================

    #[test]
    fn test_batch_read_average_by_prefab() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with values: 10, 20, 30, 40, 50
        for i in 1..=5 {
            let device = shared(MockDevice::with_values(
                i,
                100,
                200,
                (i * 10) as f64,
                0.0,
                0.0,
            ));
            network.add_device(device, network_rc.clone());
        }

        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Average)
            .unwrap();
        // Average of 10, 20, 30, 40, 50 = 30
        assert_eq!(result, 30.0);
    }

    #[test]
    fn test_batch_read_sum_by_prefab() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with values: 10, 20, 30
        for i in 1..=3 {
            let device = shared(MockDevice::with_values(
                i,
                100,
                200,
                (i * 10) as f64,
                0.0,
                0.0,
            ));
            network.add_device(device, network_rc.clone());
        }

        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Sum)
            .unwrap();
        // Sum of 10, 20, 30 = 60
        assert_eq!(result, 60.0);
    }

    #[test]
    fn test_batch_read_minimum_by_prefab() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with values: 50, 10, 30, 20, 40
        let values = vec![50.0, 10.0, 30.0, 20.0, 40.0];
        for (i, &val) in values.iter().enumerate() {
            let device = shared(MockDevice::with_values(
                (i + 1) as i32,
                100,
                200,
                val,
                0.0,
                0.0,
            ));
            network.add_device(device, network_rc.clone());
        }

        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Minimum)
            .unwrap();
        assert_eq!(result, 10.0);
    }

    #[test]
    fn test_batch_read_maximum_by_prefab() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with values: 50, 10, 30, 20, 40
        let values = vec![50.0, 10.0, 30.0, 20.0, 40.0];
        for (i, &val) in values.iter().enumerate() {
            let device = shared(MockDevice::with_values(
                (i + 1) as i32,
                100,
                200,
                val,
                0.0,
                0.0,
            ));
            network.add_device(device, network_rc.clone());
        }

        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Maximum)
            .unwrap();
        assert_eq!(result, 50.0);
    }

    #[test]
    fn test_batch_read_empty_returns_zero() {
        let network = CableNetwork::new();

        // No devices with prefab 999
        let result = network
            .batch_read_by_prefab(999, LogicType::Setting, BatchMode::Average)
            .unwrap();
        assert_eq!(result, 0.0);

        let result = network
            .batch_read_by_prefab(999, LogicType::Setting, BatchMode::Sum)
            .unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_batch_read_by_name() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with same prefab but different names
        network.add_device(
            shared(MockDevice::with_values(1, 100, 1000, 10.0, 0.0, 0.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(2, 100, 1000, 20.0, 0.0, 0.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(3, 100, 2000, 30.0, 0.0, 0.0)),
            network_rc.clone(),
        );

        // Read devices with prefab 100 and name 1000
        let result = network
            .batch_read_by_name(100, 1000, LogicType::Setting, BatchMode::Average)
            .unwrap();
        // Average of 10, 20 = 15
        assert_eq!(result, 15.0);

        // Read devices with prefab 100 and name 2000
        let result = network
            .batch_read_by_name(100, 2000, LogicType::Setting, BatchMode::Average)
            .unwrap();
        assert_eq!(result, 30.0);
    }

    #[test]
    fn test_batch_read_by_name_no_intersection() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with different prefab/name combinations
        network.add_device(shared(MockDevice::new(1, 100, 1000)), network_rc.clone());
        network.add_device(shared(MockDevice::new(2, 200, 2000)), network_rc.clone());

        // No devices with prefab 100 AND name 2000
        let result = network
            .batch_read_by_name(100, 2000, LogicType::Setting, BatchMode::Average)
            .unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_batch_read_different_logic_types() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with different property values
        network.add_device(
            shared(MockDevice::with_values(1, 100, 200, 10.0, 25.0, 100.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(2, 100, 200, 20.0, 35.0, 200.0)),
            network_rc.clone(),
        );

        // Test reading different logic types
        let setting_avg = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Average)
            .unwrap();
        assert_eq!(setting_avg, 15.0);

        let horizontal_avg = network
            .batch_read_by_prefab(100, LogicType::Horizontal, BatchMode::Average)
            .unwrap();
        assert_eq!(horizontal_avg, 30.0);

        let vertical_sum = network
            .batch_read_by_prefab(100, LogicType::Vertical, BatchMode::Sum)
            .unwrap();
        assert_eq!(vertical_sum, 300.0);
    }

    // ==================== Batch Write Tests ====================

    #[test]
    fn test_batch_write_by_prefab() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add 3 devices
        for i in 1..=3 {
            let device = shared(MockDevice::new(i, 100, 200));
            network.add_device(device, network_rc.clone());
        }

        // Write value 42 to all devices
        let write_count = network
            .batch_write_by_prefab(100, LogicType::Setting, 42.0)
            .unwrap();
        assert_eq!(write_count, 3);

        // Verify all devices have the new value
        for i in 1..=3 {
            let device = network.get_device(i).unwrap();
            assert_eq!(device.read(LogicType::Setting).unwrap(), 42.0);
        }
    }

    #[test]
    fn test_batch_write_by_name() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with different names
        network.add_device(shared(MockDevice::new(1, 100, 1000)), network_rc.clone());
        network.add_device(shared(MockDevice::new(2, 100, 1000)), network_rc.clone());
        network.add_device(shared(MockDevice::new(3, 100, 2000)), network_rc.clone());

        // Write to devices with prefab 100 and name 1000
        let write_count = network
            .batch_write_by_name(100, 1000, LogicType::Setting, 99.0)
            .unwrap();
        assert_eq!(write_count, 2);

        // Verify correct devices were written
        assert_eq!(
            network
                .get_device(1)
                .unwrap()
                .read(LogicType::Setting)
                .unwrap(),
            99.0
        );
        assert_eq!(
            network
                .get_device(2)
                .unwrap()
                .read(LogicType::Setting)
                .unwrap(),
            99.0
        );
        assert_eq!(
            network
                .get_device(3)
                .unwrap()
                .read(LogicType::Setting)
                .unwrap(),
            0.0
        );
    }

    #[test]
    fn test_batch_write_empty_returns_zero() {
        let network = CableNetwork::new();

        // No devices with prefab 999
        let write_count = network
            .batch_write_by_prefab(999, LogicType::Setting, 42.0)
            .unwrap();
        assert_eq!(write_count, 0);
    }

    #[test]
    fn test_batch_write_different_logic_types() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        for i in 1..=2 {
            let device = shared(MockDevice::new(i, 100, 200));
            network.add_device(device, network_rc.clone());
        }

        // Write to different logic types
        network
            .batch_write_by_prefab(100, LogicType::Setting, 10.0)
            .unwrap();
        network
            .batch_write_by_prefab(100, LogicType::Horizontal, 25.5)
            .unwrap();
        network
            .batch_write_by_prefab(100, LogicType::Vertical, 105.0)
            .unwrap();

        // Verify all values
        for i in 1..=2 {
            let device = network.get_device(i).unwrap();
            assert_eq!(device.read(LogicType::Setting).unwrap(), 10.0);
            assert_eq!(device.read(LogicType::Horizontal).unwrap(), 25.5);
            assert_eq!(device.read(LogicType::Vertical).unwrap(), 105.0);
        }
    }

    // ==================== BatchMode Tests ====================

    #[test]
    fn test_batch_mode_from_value() {
        assert_eq!(BatchMode::from_value(0.0), Some(BatchMode::Average));
        assert_eq!(BatchMode::from_value(1.0), Some(BatchMode::Sum));
        assert_eq!(BatchMode::from_value(2.0), Some(BatchMode::Minimum));
        assert_eq!(BatchMode::from_value(3.0), Some(BatchMode::Maximum));
        assert_eq!(BatchMode::from_value(4.0), None);
        assert_eq!(BatchMode::from_value(-1.0), None);
    }

    #[test]
    fn test_batch_mode_to_value() {
        assert_eq!(BatchMode::Average.to_value(), 0.0);
        assert_eq!(BatchMode::Sum.to_value(), 1.0);
        assert_eq!(BatchMode::Minimum.to_value(), 2.0);
        assert_eq!(BatchMode::Maximum.to_value(), 3.0);
    }

    #[test]
    fn test_batch_mode_from_name() {
        assert_eq!(BatchMode::from_name("Average"), Some(BatchMode::Average));
        assert_eq!(BatchMode::from_name("Sum"), Some(BatchMode::Sum));
        assert_eq!(BatchMode::from_name("Minimum"), Some(BatchMode::Minimum));
        assert_eq!(BatchMode::from_name("Maximum"), Some(BatchMode::Maximum));
        assert_eq!(BatchMode::from_name("Invalid"), None);
        assert_eq!(BatchMode::from_name("average"), None); // Case sensitive
    }

    #[test]
    fn test_batch_mode_aggregate_average() {
        let values = vec![10.0, 20.0, 30.0, 40.0];
        assert_eq!(BatchMode::Average.aggregate(&values), 25.0);
    }

    #[test]
    fn test_batch_mode_aggregate_sum() {
        let values = vec![10.0, 20.0, 30.0];
        assert_eq!(BatchMode::Sum.aggregate(&values), 60.0);
    }

    #[test]
    fn test_batch_mode_aggregate_minimum() {
        let values = vec![50.0, 10.0, 30.0, 20.0];
        assert_eq!(BatchMode::Minimum.aggregate(&values), 10.0);
    }

    #[test]
    fn test_batch_mode_aggregate_maximum() {
        let values = vec![50.0, 10.0, 30.0, 20.0];
        assert_eq!(BatchMode::Maximum.aggregate(&values), 50.0);
    }

    #[test]
    fn test_batch_mode_aggregate_empty() {
        let values: Vec<f64> = vec![];
        assert_eq!(BatchMode::Average.aggregate(&values), 0.0);
        assert_eq!(BatchMode::Sum.aggregate(&values), 0.0);
        assert_eq!(BatchMode::Minimum.aggregate(&values), 0.0);
        assert_eq!(BatchMode::Maximum.aggregate(&values), 0.0);
    }

    #[test]
    fn test_batch_mode_aggregate_single_value() {
        let values = vec![42.0];
        assert_eq!(BatchMode::Average.aggregate(&values), 42.0);
        assert_eq!(BatchMode::Sum.aggregate(&values), 42.0);
        assert_eq!(BatchMode::Minimum.aggregate(&values), 42.0);
        assert_eq!(BatchMode::Maximum.aggregate(&values), 42.0);
    }

    #[test]
    fn test_batch_mode_aggregate_negative_values() {
        let values = vec![-10.0, -5.0, 5.0, 10.0];
        assert_eq!(BatchMode::Average.aggregate(&values), 0.0);
        assert_eq!(BatchMode::Sum.aggregate(&values), 0.0);
        assert_eq!(BatchMode::Minimum.aggregate(&values), -10.0);
        assert_eq!(BatchMode::Maximum.aggregate(&values), 10.0);
    }

    // ==================== Edge Cases and Complex Scenarios ====================

    #[test]
    fn test_mixed_prefab_and_name_batch_operations() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Create a complex network with overlapping prefab and name hashes
        // Prefab 100, Name 1000: devices 1, 2
        // Prefab 100, Name 2000: devices 3, 4
        // Prefab 200, Name 1000: devices 5, 6
        network.add_device(
            shared(MockDevice::with_values(1, 100, 1000, 10.0, 0.0, 0.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(2, 100, 1000, 20.0, 0.0, 0.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(3, 100, 2000, 30.0, 0.0, 0.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(4, 100, 2000, 40.0, 0.0, 0.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(5, 200, 1000, 50.0, 0.0, 0.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(6, 200, 1000, 60.0, 0.0, 0.0)),
            network_rc.clone(),
        );

        // Test batch read by prefab
        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Average)
            .unwrap();
        assert_eq!(result, 25.0); // Average of 10, 20, 30, 40

        // Test batch read by name (requires both prefab and name match)
        let result = network
            .batch_read_by_name(100, 1000, LogicType::Setting, BatchMode::Sum)
            .unwrap();
        assert_eq!(result, 30.0); // Sum of 10, 20

        let result = network
            .batch_read_by_name(200, 1000, LogicType::Setting, BatchMode::Maximum)
            .unwrap();
        assert_eq!(result, 60.0); // Max of 50, 60
    }

    #[test]
    fn test_device_ordering_maintained_after_removals() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices: 1, 2, 3, 4, 5
        for i in 1..=5 {
            network.add_device(shared(MockDevice::new(i, 100, 200)), network_rc.clone());
        }

        // Remove device 3
        network.remove_device(3);

        let devices = network.get_devices_by_prefab(100);
        assert_eq!(devices, vec![1, 2, 4, 5]); // Still sorted

        // Add device 3 back
        network.add_device(shared(MockDevice::new(3, 100, 200)), network_rc.clone());

        let devices = network.get_devices_by_prefab(100);
        assert_eq!(devices, vec![1, 2, 3, 4, 5]); // Properly reinserted in sorted position
    }

    #[test]
    fn test_large_scale_batch_operations() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add 100 devices with sequential values
        for i in 1..=100 {
            let device = shared(MockDevice::with_values(i, 100, 200, i as f64, 0.0, 0.0));
            network.add_device(device, network_rc.clone());
        }

        // Test average: (1 + 2 + ... + 100) / 100 = 50.5
        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Average)
            .unwrap();
        assert_eq!(result, 50.5);

        // Test sum: (1 + 2 + ... + 100) = 5050
        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Sum)
            .unwrap();
        assert_eq!(result, 5050.0);

        // Test min/max
        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Minimum)
            .unwrap();
        assert_eq!(result, 1.0);

        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Maximum)
            .unwrap();
        assert_eq!(result, 100.0);

        // Batch write to all 100 devices
        let write_count = network
            .batch_write_by_prefab(100, LogicType::Setting, 777.0)
            .unwrap();
        assert_eq!(write_count, 100);

        // Verify all were written
        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Minimum)
            .unwrap();
        assert_eq!(result, 777.0);
    }

    #[test]
    fn test_get_device_borrow() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());
        let device = shared(MockDevice::new(1, 100, 200));

        network.add_device(device, network_rc);

        // Get borrow and modify directly
        {
            let device = network.get_device(1).unwrap();
            device.write(LogicType::Setting, 123.0).unwrap();
        }

        // Verify change
        let device = network.get_device(1).unwrap();
        assert_eq!(device.read(LogicType::Setting).unwrap(), 123.0);
    }

    #[test]
    fn test_network_with_duplicate_device_ids() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        let device1 = shared(MockDevice::with_values(1, 100, 200, 10.0, 0.0, 0.0));
        network.add_device(device1, network_rc.clone());

        // Try to add another device with same ID but different prefab
        let device2 = shared(MockDevice::with_values(1, 200, 300, 20.0, 0.0, 0.0));
        network.add_device(device2, network_rc.clone());

        // The second device should replace the first
        assert_eq!(network.device_count(), 1);
        let device = network.get_device(1).unwrap();
        assert_eq!(device.get_prefab_hash(), 200); // Second device's prefab
        assert_eq!(device.read(LogicType::Setting).unwrap(), 20.0); // Second device's value
    }

    #[test]
    fn test_batch_operations_with_floating_point_precision() {
        let mut network = CableNetwork::new();
        let network_rc = shared(network.clone());

        // Add devices with fractional values
        network.add_device(
            shared(MockDevice::with_values(1, 100, 200, 0.1, 0.0, 0.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(2, 100, 200, 0.2, 0.0, 0.0)),
            network_rc.clone(),
        );
        network.add_device(
            shared(MockDevice::with_values(3, 100, 200, 0.3, 0.0, 0.0)),
            network_rc.clone(),
        );

        let result = network
            .batch_read_by_prefab(100, LogicType::Setting, BatchMode::Sum)
            .unwrap();
        // Should handle floating point correctly
        assert!((result - 0.6).abs() < 1e-10);
    }
}
