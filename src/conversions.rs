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

/// Convert a double to a long integer for bitwise operations.
///
/// This matches the C# implementation:
/// ```csharp
/// public static long DoubleToLong(double d, bool signed)
/// {
///     long num = (long) (d % 9.00719925474099E+15);
///     if (!signed)
///         num &= 18014398509481983L;  // 0x3FFFFFFFFFFFFF
///     return num;
/// }
/// ```
///
/// # Arguments
/// * `d` - The double value to convert
/// * `signed` - Whether to preserve the sign (true) or mask to unsigned (false)
///
/// # Returns
/// A 64-bit signed integer suitable for bitwise operations
pub fn double_to_long(d: f64, signed: bool) -> i64 {
    // Handle special cases
    if d.is_nan() || d.is_infinite() {
        return 0;
    }

    // Wrap via modulo to fit within representable range
    let num = (d % MODULO_VALUE) as i64;

    if signed { num } else { num & UNSIGNED_MASK }
}

/// Convert a long integer back to a double after bitwise operations.
///
/// This handles IC10's special 53-bit integer representation where bit 53
/// is used as a sign indicator.
///
/// This matches the C# implementation:
/// ```csharp
/// public static double LongToDouble(long l)
/// {
///     int num = (l & 9007199254740992L) != 0L ? 1 : 0;  // Check bit 53
///     l &= 9007199254740991L;  // Mask to 53 bits
///     if (num != 0)
///         l |= -9007199254740992L;  // Sign extend from bit 53
///     return (double) l;
/// }
/// ```
///
/// # Arguments
/// * `l` - The long value to convert
///
/// # Returns
/// A double representing the value
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

/// Helper function to convert a packed number into a text string.
/// Each byte in the packed number represents an ASCII character.
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
