//! IC10 source code preprocessing and parsing

use crate::error::{SimulationError, SimulationResult};
use crc::{CRC_32_ISO_HDLC, Crc};
use regex::Regex;

/// Preprocess IC10 source (handle defines, strings, hex/bin literals).
pub fn preprocess(source: &str) -> SimulationResult<String> {
    let comment_re = Regex::new(r"#.*$").unwrap();
    let str_re = Regex::new(r#"STR\("([^"]+)"\)"#).unwrap();
    let hash_str_re = Regex::new(r#"HASH\("([^"]+)"\)"#).unwrap();
    let bin_re = Regex::new(r"%([01_]+)").unwrap();
    let hex_re = Regex::new(r"\$([A-Fa-f0-9_]+)").unwrap();

    let mut result = Vec::new();
    for line in source.lines() {
        let line = comment_re.replace(line, "");
        let mut line = line.to_string();

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

        line = hash_str_re
            .replace_all(&line, |caps: &regex::Captures| {
                let text = &caps[1];
                format!("{}", string_to_hash(text))
            })
            .to_string();

        line = bin_re
            .replace_all(&line, |caps: &regex::Captures| {
                match parse_binary_str(&caps[1]) {
                    Some(val) => format!("{val}"),
                    None => "<ERR:InvalidProcessBinary>".to_string(),
                }
            })
            .to_string();

        line = hex_re
            .replace_all(&line, |caps: &regex::Captures| {
                match parse_hex_str(&caps[1]) {
                    Some(val) => format!("{val}"),
                    None => "<ERR:InvalidPreprocessHex>".to_string(),
                }
            })
            .to_string();

        result.push(line.trim_end().to_string());
    }
    Ok(result.join("\n"))
}

/// Pack an ASCII string (<=6 chars) into a 48-bit integer.
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

/// Compute a CRC32-based hash compatible with Unity's hashing.
pub const fn string_to_hash(text: &str) -> i32 {
    const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);
    let checksum = CRC32.checksum(text.as_bytes());
    checksum as i32
}

/// Parse a binary literal string (no leading `%`).
pub fn parse_binary_str(bin_str: &str) -> Option<i64> {
    let clean = bin_str.replace('_', "");
    if clean.is_empty() {
        return None;
    }
    i64::from_str_radix(&clean, 2).ok()
}

/// Parse a hexadecimal literal string (no leading `$`).
pub fn parse_hex_str(hex_str: &str) -> Option<i64> {
    let clean = hex_str.replace('_', "");
    if clean.is_empty() {
        return None;
    }
    i64::from_str_radix(&clean, 16).ok()
}

/// Parse a hexadecimal literal like `$FF` into a numeric value.
pub fn parse_hex(input: &str) -> SimulationResult<i64> {
    let hex_str = input.trim_start_matches('$');
    parse_hex_str(hex_str).ok_or_else(|| SimulationError::IC10ParseError {
        line: 0,
        message: format!("Invalid hexadecimal literal: {input}"),
    })
}

/// Parse a binary literal like `%1010` into a numeric value.
pub fn parse_binary(input: &str) -> SimulationResult<i64> {
    let bin_str = input.trim_start_matches('%');
    parse_binary_str(bin_str).ok_or_else(|| SimulationError::IC10ParseError {
        line: 0,
        message: format!("Invalid binary literal: {input}"),
    })
}
