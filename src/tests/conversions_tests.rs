//! Unit tests for numeric conversions
#[cfg(test)]
mod tests {
    use crate::conversions::{BIT_53, UNSIGNED_MASK, double_to_long, long_to_double};

    #[test]
    fn test_double_to_long_positive() {
        // Simple positive values
        assert_eq!(double_to_long(42.0, true), 42);
        assert_eq!(double_to_long(42.0, false), 42);
        assert_eq!(double_to_long(1234567.0, true), 1234567);
    }

    #[test]
    fn test_double_to_long_negative() {
        // Negative values - signed should preserve sign
        assert_eq!(double_to_long(-42.0, true), -42);
        // Unsigned should mask
        let result = double_to_long(-42.0, false);
        assert!(result >= 0 || result & UNSIGNED_MASK == result);
    }

    #[test]
    fn test_long_to_double_positive() {
        // Small positive values should round-trip
        assert_eq!(long_to_double(42), 42.0);
        assert_eq!(long_to_double(0), 0.0);
        assert_eq!(long_to_double(1234567), 1234567.0);
    }

    #[test]
    fn test_long_to_double_bit53_set() {
        // When bit 53 is set, the value should be sign-extended
        let val_with_bit53 = BIT_53; // Just bit 53 set
        let result = long_to_double(val_with_bit53);
        // Should be negative due to sign extension
        assert!(result < 0.0);
    }

    #[test]
    fn test_round_trip_positive() {
        // Positive integers should round-trip correctly
        for &val in &[0.0, 1.0, 42.0, 1000.0, 65535.0, 1000000.0] {
            let as_long = double_to_long(val, true);
            let back = long_to_double(as_long);
            assert_eq!(back, val, "Round trip failed for {val}");
        }
    }

    #[test]
    fn test_bitwise_and() {
        // Test that AND works correctly with conversions
        let a = 0xFF00;
        let b = 0x0FFF;
        let a_long = double_to_long(a as f64, true);
        let b_long = double_to_long(b as f64, true);
        let result = long_to_double(a_long & b_long);
        assert_eq!(result, (a & b) as f64);
    }

    #[test]
    fn test_bitwise_or() {
        let a = 0xFF00;
        let b = 0x00FF;
        let a_long = double_to_long(a as f64, true);
        let b_long = double_to_long(b as f64, true);
        let result = long_to_double(a_long | b_long);
        assert_eq!(result, (a | b) as f64);
    }

    #[test]
    fn test_bitwise_not() {
        // NOT of 0 should give all 1s in the 53-bit range
        let val = double_to_long(0.0, true);
        let notted = !val;
        let result = long_to_double(notted);
        // Result should be -1 due to sign extension
        assert_eq!(result, -1.0);
    }

    #[test]
    fn test_special_values() {
        // NaN and infinity should convert to 0
        assert_eq!(double_to_long(f64::NAN, true), 0);
        assert_eq!(double_to_long(f64::INFINITY, true), 0);
        assert_eq!(double_to_long(f64::NEG_INFINITY, true), 0);
    }
}
