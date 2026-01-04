use crate::error::{IC10Error, IC10Result};
use crc::{CRC_32_ISO_HDLC, Crc};
use regex::Regex;

/// Preprocess source code (handle defines, includes, etc.)
pub fn preprocess(source: &str) -> IC10Result<String> {
    // 1. Remove comments (everything after #)
    let comment_re = Regex::new(r"#.*$").unwrap();
    // 2. String packing: STR("text") - packs ASCII into double (max 6 chars)
    // C# regex: STR\("([^"]+)"\) - matches at least 1 non-quote char
    let str_re = Regex::new(r#"STR\("([^"]+)"\)"#).unwrap();
    // 3. Hash preprocessing: HASH("text") - Unity StringToHash (no length limit)
    // C# regex: HASH\("([^"]+)"\) - matches at least 1 non-quote char
    let hash_str_re = Regex::new(r#"HASH\("([^"]+)"\)"#).unwrap();
    // 4. Binary: %1010_1111
    let bin_re = Regex::new(r"%([01_]+)").unwrap();
    // 5. Hex: $1A_2B
    let hex_re = Regex::new(r"\$([A-Fa-f0-9_]+)").unwrap();

    let mut result = Vec::new();
    for line in source.lines() {
        let line = comment_re.replace(line, "");
        let mut line = line.to_string();

        // String packing: STR("text")
        line = str_re
            .replace_all(&line, |caps: &regex::Captures| {
                let text = &caps[1];
                match pack_ascii6(text) {
                    Some(num) => format!("{}", num as f64),
                    None if text.len() > 6 => "<ERR:InvalidStringLength>".to_string(),
                    None if text.chars().any(|c| c as u32 > 0x7F) => {
                        "<ERR:InvalidStringNonAscii>".to_string()
                    }
                    None => "<ERR:InvalidStringNull>".to_string(),
                }
            })
            .to_string();

        // Hash preprocessing: HASH("text")
        line = hash_str_re
            .replace_all(&line, |caps: &regex::Captures| {
                let text = &caps[1];
                format!("{}", string_to_hash(text))
            })
            .to_string();

        // Binary: %1010_1111
        line = bin_re
            .replace_all(&line, |caps: &regex::Captures| {
                match parse_binary_str(&caps[1]) {
                    Some(val) => format!("{val}"),
                    None => "<ERR:InvalidProcessBinary>".to_string(),
                }
            })
            .to_string();

        // Hex: $1A_2B
        line = hex_re
            .replace_all(&line, |caps: &regex::Captures| {
                match parse_hex_str(&caps[1]) {
                    Some(val) => format!("{val}"),
                    None => "<ERR:InvalidPreprocessHex>".to_string(),
                }
            })
            .to_string();

        // Normalize whitespace
        result.push(line.trim_end().to_string());
    }
    Ok(result.join("\n"))
}

/// Pack an ASCII string into a 64-bit integer (max 6 chars)
/// C# algorithm: num = num << 8 | (byte)ch - packs left-to-right (big-endian)
/// Returns None if string is empty, too long, or contains non-ASCII
pub fn pack_ascii6(text: &str) -> Option<i64> {
    if text.is_empty() || text.len() > 6 {
        return None;
    }
    let mut num: i64 = 0;
    for c in text.chars() {
        if c as u32 > 0x7F {
            return None;
        }
        num = (num << 8) | (c as u8 as i64);
    }
    Some(num)
}

/// Unity Animator.StringToHash algorithm (CRC32, UTF-8, case-sensitive)
pub const fn string_to_hash(text: &str) -> i32 {
    const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let checksum = CRC32.checksum(text.as_bytes());
    checksum as i32
}

/// Parse a binary string (without % prefix) into i64
/// Supports underscores as separators
pub fn parse_binary_str(bin_str: &str) -> Option<i64> {
    let clean = bin_str.replace('_', "");
    if clean.is_empty() {
        return None;
    }
    i64::from_str_radix(&clean, 2).ok()
}

/// Parse a hex string (without $ prefix) into i64
/// Supports underscores as separators
pub fn parse_hex_str(hex_str: &str) -> Option<i64> {
    let clean = hex_str.replace('_', "");
    if clean.is_empty() {
        return None;
    }
    i64::from_str_radix(&clean, 16).ok()
}

/// Parse hexadecimal literal (e.g., $FF, $1A_2B)
pub fn parse_hex(input: &str) -> IC10Result<i64> {
    let hex_str = input.trim_start_matches('$');
    parse_hex_str(hex_str).ok_or_else(|| IC10Error::ParseError {
        line: 0,
        message: format!("Invalid hexadecimal literal: {input}"),
    })
}

/// Parse binary literal (e.g., %1010, %1010_1111)
pub fn parse_binary(input: &str) -> IC10Result<i64> {
    let bin_str = input.trim_start_matches('%');
    parse_binary_str(bin_str).ok_or_else(|| IC10Error::ParseError {
        line: 0,
        message: format!("Invalid binary literal: {input}"),
    })
}
