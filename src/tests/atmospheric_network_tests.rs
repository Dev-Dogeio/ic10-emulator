//! Unit tests for atmospheric networks
#[cfg(test)]
mod tests {
    use crate::{atmospherics::GasType, networks::AtmosphericNetwork};

    #[test]
    fn test_gas_operations() {
        let network = AtmosphericNetwork::new(1000.0);

        // Add gas
        network.borrow_mut().add_gas(GasType::Oxygen, 10.0, 300.0);
        assert!(!network.borrow().is_empty());
        assert!((network.borrow().total_moles() - 10.0).abs() < 0.001);

        // Remove gas
        let removed = network.borrow_mut().remove_gas(GasType::Oxygen, 5.0);
        assert!((removed - 5.0).abs() < 0.001);
        assert!((network.borrow().total_moles() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_transfer_between_networks() {
        let network1 = AtmosphericNetwork::new(1000.0);
        let network2 = AtmosphericNetwork::new(1000.0);

        network1.borrow_mut().add_gas(GasType::Oxygen, 20.0, 300.0);

        network1
            .borrow_mut()
            .transfer_to(&mut network2.borrow_mut(), 10.0);

        assert!((network1.borrow().total_moles() - 10.0).abs() < 0.001);
        assert!((network2.borrow().total_moles() - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_equalization() {
        let network1 = AtmosphericNetwork::new(1000.0);
        let network2 = AtmosphericNetwork::new(1000.0);

        network1.borrow_mut().add_gas(GasType::Oxygen, 20.0, 300.0);
        network2.borrow_mut().add_gas(GasType::Oxygen, 10.0, 300.0);

        network1
            .borrow_mut()
            .equalize_with(&mut network2.borrow_mut());

        // Pressures should be equal
        let diff = (network1.borrow().pressure() - network2.borrow().pressure()).abs();
        assert!(diff < 0.1);
    }
}
