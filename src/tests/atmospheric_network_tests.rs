#[cfg(test)]
mod tests {
    use crate::{atmospherics::GasType, networks::AtmosphericNetwork};

    #[test]
    fn test_new_network() {
        let network = AtmosphericNetwork::new(1000.0);
        assert_eq!(network.device_count(), 0);
        assert!(network.is_empty());
    }

    #[test]
    fn test_add_remove_device() {
        let mut network = AtmosphericNetwork::new(1000.0);

        // Add device
        assert!(network.add_device(100));
        assert_eq!(network.device_count(), 1);

        // Try to add same device again
        assert!(!network.add_device(100));
        assert_eq!(network.device_count(), 1);

        // Add another device
        assert!(network.add_device(101));
        assert_eq!(network.device_count(), 2);

        // Remove device
        assert!(network.remove_device(100));
        assert_eq!(network.device_count(), 1);
    }

    #[test]
    fn test_gas_operations() {
        let mut network = AtmosphericNetwork::new(1000.0);

        // Add gas
        network.add_gas(GasType::Oxygen, 10.0, 300.0);
        assert!(!network.is_empty());
        assert!((network.total_moles() - 10.0).abs() < 0.001);

        // Remove gas
        let removed = network.remove_gas(GasType::Oxygen, 5.0);
        assert!((removed - 5.0).abs() < 0.001);
        assert!((network.total_moles() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_merge_networks() {
        let mut network1 = AtmosphericNetwork::new(1000.0);
        let mut network2 = AtmosphericNetwork::new(500.0);

        network1.add_device(100);
        network2.add_device(200);

        network1.add_gas(GasType::Oxygen, 10.0, 300.0);
        network2.add_gas(GasType::Nitrogen, 5.0, 400.0);

        network1.merge_network(&mut network2);

        assert_eq!(network1.device_count(), 2);
        assert_eq!(network1.total_volume(), 1000.0);
        assert!((network1.total_moles() - 15.0).abs() < 0.001);
        assert_eq!(network2.device_count(), 0);
        assert!(network2.is_empty());
    }

    #[test]
    fn test_transfer_between_networks() {
        let mut network1 = AtmosphericNetwork::new(1000.0);
        let mut network2 = AtmosphericNetwork::new(1000.0);

        network1.add_gas(GasType::Oxygen, 20.0, 300.0);

        network1.transfer_to(&mut network2, 10.0);

        assert!((network1.total_moles() - 10.0).abs() < 0.001);
        assert!((network2.total_moles() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_equalization() {
        let mut network1 = AtmosphericNetwork::new(1000.0);
        let mut network2 = AtmosphericNetwork::new(1000.0);

        network1.add_gas(GasType::Oxygen, 20.0, 300.0);
        network2.add_gas(GasType::Oxygen, 10.0, 300.0);

        network1.equalize_with(&mut network2);

        // Pressures should be equal
        let diff = (network1.pressure() - network2.pressure()).abs();
        assert!(diff < 0.1);
    }
}
