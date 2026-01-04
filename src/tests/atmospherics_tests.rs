#[cfg(test)]
mod tests {
    use crate::atmospherics::{
        GasMixture, GasType, Mole, calculate_moles, calculate_pressure, celsius_to_kelvin,
        kelvin_to_celsius,
    };

    #[test]
    fn test_empty_mixture() {
        let mixture = GasMixture::new(1000.0);
        assert!(mixture.is_empty());
        assert_eq!(mixture.total_moles(), 0.0);
    }

    #[test]
    fn test_add_gas() {
        let mut mixture = GasMixture::new(1000.0);
        mixture.add_gas(GasType::Oxygen, 10.0, 300.0);

        assert!((mixture.get_moles(GasType::Oxygen) - 10.0).abs() < 0.0001);
        assert!((mixture.temperature() - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_pressure_calculation() {
        let mut mixture = GasMixture::new(1000.0);
        mixture.add_gas(GasType::Nitrogen, 10.0, 300.0);

        let pressure = mixture.pressure();
        // P = nRT/V = 10 * 8.3144 * 300 / 1000 = 24.94 kPa
        assert!(pressure > 0.0);
    }

    #[test]
    fn test_partial_pressure() {
        let mut mixture = GasMixture::new(1000.0);
        mixture.add_gas(GasType::Oxygen, 5.0, 300.0);
        mixture.add_gas(GasType::Nitrogen, 15.0, 300.0);

        let total_pressure = mixture.pressure();
        let o2_pp = mixture.partial_pressure(GasType::Oxygen);
        let n2_pp = mixture.partial_pressure(GasType::Nitrogen);

        // Partial pressures should sum to total
        assert!((o2_pp + n2_pp - total_pressure).abs() < 0.01);

        // O2 should be 25% of total pressure (5/20)
        assert!((o2_pp / total_pressure - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_gas_transfer() {
        let mut source = GasMixture::new(1000.0);
        source.add_gas(GasType::Oxygen, 10.0, 300.0);

        let mut target = GasMixture::new(1000.0);

        let transferred = source.transfer_ratio_to(&mut target, 0.5);

        assert!((transferred - 5.0).abs() < 0.0001);
        assert!((source.get_moles(GasType::Oxygen) - 5.0).abs() < 0.0001);
        assert!((target.get_moles(GasType::Oxygen) - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_temperature_mixing() {
        let mut mixture = GasMixture::new(1000.0);
        mixture.add_gas(GasType::Nitrogen, 10.0, 200.0);
        mixture.add_gas(GasType::Nitrogen, 10.0, 400.0);

        // Temperature should be ~300K (average weighted by heat capacity)
        let temp = mixture.temperature();
        assert!((temp - 300.0).abs() < 1.0);
    }

    #[test]
    fn test_equalization() {
        let mut high = GasMixture::new(1000.0);
        high.add_gas(GasType::Oxygen, 20.0, 300.0);

        let mut low = GasMixture::new(1000.0);
        low.add_gas(GasType::Oxygen, 10.0, 300.0);

        high.equalize_with(&mut low);

        // Both should have equal pressure now
        let diff = (high.pressure() - low.pressure()).abs();
        assert!(diff < 0.1);
    }

    #[test]
    fn test_temperature_mixing_different_gases() {
        let mut mixture = GasMixture::new(1000.0);
        mixture.add_gas(GasType::Oxygen, 5.0, 200.0);
        mixture.add_gas(GasType::Nitrogen, 10.0, 400.0);

        // Calculate expected equilibrium temperature
        let oxygen_capacity = 5.0 * GasType::Oxygen.specific_heat();
        let nitrogen_capacity = 10.0 * GasType::Nitrogen.specific_heat();
        let total_capacity = oxygen_capacity + nitrogen_capacity;
        let total_energy = (oxygen_capacity * 200.0) + (nitrogen_capacity * 400.0);
        let expected_temp = total_energy / total_capacity;

        // Check if the temperature matches the expected value
        let temp = mixture.temperature();
        assert!(
            (temp - expected_temp).abs() < 1.0,
            "Expected {:.2}, got {:.2}",
            expected_temp,
            temp
        );
    }

    #[test]
    fn test_temperature_conversion() {
        assert!((celsius_to_kelvin(0.0) - 273.15).abs() < 0.01);
        assert!((celsius_to_kelvin(100.0) - 373.15).abs() < 0.01);
        assert!((kelvin_to_celsius(273.15) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_ideal_gas_law_consistency() {
        let moles = 10.0;
        let temp = 300.0;
        let volume = 100.0;

        let pressure = calculate_pressure(moles, temp, volume);
        let calculated_moles = calculate_moles(pressure, volume, temp);

        assert!((moles - calculated_moles).abs() < 0.0001);
    }

    #[test]
    fn test_mole_creation() {
        let mole = Mole::new(GasType::Oxygen, 10.0, 300.0);
        assert_eq!(mole.gas_type(), GasType::Oxygen);
        assert!((mole.quantity() - 10.0).abs() < 0.0001);
        assert!((mole.temperature() - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_zero_mole() {
        let mole = Mole::zero(GasType::Nitrogen);
        assert!(mole.is_empty());
        assert_eq!(mole.quantity(), 0.0);
    }

    #[test]
    fn test_energy_calculation() {
        let mole = Mole::new(GasType::Oxygen, 1.0, 300.0);
        // E = n * Cv * T = 1 * 21.1 * 300 = 6330 J
        let expected_energy = 1.0 * GasType::Oxygen.specific_heat() * 300.0;
        assert!((mole.energy() - expected_energy).abs() < 0.01);
    }

    #[test]
    fn test_add_moles() {
        let mut mole1 = Mole::new(GasType::Nitrogen, 5.0, 300.0);
        let mole2 = Mole::new(GasType::Nitrogen, 5.0, 400.0);

        mole1.add(&mole2);

        assert!((mole1.quantity() - 10.0).abs() < 0.0001);
        assert!(mole1.temperature() == 350.0);

        mole1.add(&Mole::new(GasType::Nitrogen, 5.0, 400.0));
        assert!(mole1.temperature() == (10.0 * 350.0 + 5.0 * 400.0) / 15.0);
    }

    #[test]
    fn test_remove_moles() {
        let mut mole = Mole::new(GasType::Oxygen, 10.0, 300.0);
        let removed = mole.remove(3.0);

        assert!((mole.quantity() - 7.0).abs() < 0.0001);
        assert!((removed.quantity() - 3.0).abs() < 0.0001);
        assert!((mole.temperature() - 300.0).abs() < 0.01);
        assert!((removed.temperature() - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_temperature_consistency() {
        let mut mole = Mole::new(GasType::Water, 5.0, 400.0);
        mole.set_temperature(500.0);
        assert!((mole.temperature() - 500.0).abs() < 0.01);
    }
}
