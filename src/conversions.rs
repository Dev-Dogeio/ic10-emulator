//! Numeric conversion functions for IC10
//!
//! These functions match the C# implementations in ProgrammableChip.cs
//! The IC10 uses 64-bit IEEE 754 doubles but operates on integers within
//! the 53-bit mantissa range for bitwise operations.

/// Mask for the 53-bit mantissa (2^53 - 1 = 9,007,199,254,740,991)
pub const MANTISSA_MASK: i64 = 0x1FFFFFFFFFFFFF; // 9007199254740991

/// Mask for bit 53 (sign indicator in IC10's convention)
pub const BIT_53: i64 = 0x20000000000000; // 9007199254740992

/// Mask for unsigned conversion (54 bits)
pub const UNSIGNED_MASK: i64 = 0x3FFFFFFFFFFFFF; // 18014398509481983

/// Modulo value for wrapping doubles (2^53 as float)
pub const MODULO_VALUE: f64 = 9.00719925474099E+15;

/// Convert a double to a 64-bit integer for bitwise ops.
///
/// Matches the original C# conversion behavior for IC10 values.
pub fn double_to_long(d: f64, signed: bool) -> i64 {
    // Handle special cases
    if d.is_nan() || d.is_infinite() {
        return 0;
    }

    // Wrap via modulo to fit within representable range
    let num = (d % MODULO_VALUE) as i64;

    if signed { num } else { num & UNSIGNED_MASK }
}

/// Convert a 53-bit-style integer back to `f64`.
///
/// Handles IC10's sign bit and mantissa masking semantics.
pub fn long_to_double(l: i64) -> f64 {
    // Check if bit 53 is set (sign indicator)
    let sign_bit_set = (l & BIT_53) != 0;

    // Mask to 53 bits
    let mut result = l & MANTISSA_MASK;

    // If bit 53 was set, sign-extend by setting all upper bits
    if sign_bit_set {
        // Sign extend: set bits 54-63 to 1
        // -9007199254740992L is 0xFFE0000000000000 in two's complement
        result |= !MANTISSA_MASK; // This sets all bits above bit 52 to 1
    }

    result as f64
}

/// Convert a packed 48-bit number into an ASCII string.
pub fn packed_number_to_text(packed: u64) -> String {
    let mut text = String::new();
    for i in (0..6).rev() {
        // Process 6 bytes (48 bits) from most significant to least significant
        let byte = ((packed >> (i * 8)) & 0xFF) as u8;
        if byte != 0 {
            // Ignore null bytes
            text.push(byte as char);
        }
    }
    text
}

/// Linear clamped interpolation between two values.
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

/// Format a float with up to `prec` decimal places, trimming trailing zeros
/// and removing the decimal point when not needed. This is useful for
/// printing where `3.0` should print as "3" and `3.50` -> "3.5".
pub fn fmt_trim(v: f64, prec: usize) -> String {
    let mut s = format!("{:.*}", prec, v);
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    s
}
