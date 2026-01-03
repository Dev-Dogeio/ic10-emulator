#[cfg(test)]
mod tests {
    use crate::parser::*;

    // ==================== pack_ascii6 tests ====================

    #[test]
    fn test_pack_ascii6_single_char() {
        // Single char: just the ASCII value
        assert_eq!(pack_ascii6("A"), Some(65));
        assert_eq!(pack_ascii6("Z"), Some(90));
        assert_eq!(pack_ascii6("a"), Some(97));
        assert_eq!(pack_ascii6("0"), Some(48));
        assert_eq!(pack_ascii6(" "), Some(32));
    }

    #[test]
    fn test_pack_ascii6_two_chars() {
        // Two chars: first << 8 | second
        // "AB" = A(65) << 8 | B(66) = 0x4142 = 16706
        assert_eq!(pack_ascii6("AB"), Some(0x4142));
        assert_eq!(pack_ascii6("Hi"), Some(0x4869)); // H=72, i=105
    }

    #[test]
    fn test_pack_ascii6_three_chars() {
        // "ABC" = 0x414243 = 4276803
        assert_eq!(pack_ascii6("ABC"), Some(0x414243));
    }

    #[test]
    fn test_pack_ascii6_max_length() {
        // 6 chars is the maximum
        // "ABCDEF" = 0x414243444546 = 71752519618886
        assert_eq!(pack_ascii6("ABCDEF"), Some(0x414243444546));
        assert_eq!(pack_ascii6("Hello!"), Some(0x48656C6C6F21));
    }

    #[test]
    fn test_pack_ascii6_too_long() {
        // More than 6 chars should fail
        assert_eq!(pack_ascii6("ABCDEFG"), None);
        assert_eq!(pack_ascii6("HelloWorld"), None);
    }

    #[test]
    fn test_pack_ascii6_empty() {
        // Empty string should fail
        assert_eq!(pack_ascii6(""), None);
    }

    #[test]
    fn test_pack_ascii6_non_ascii() {
        // Non-ASCII chars should fail
        assert_eq!(pack_ascii6("é"), None);
        assert_eq!(pack_ascii6("日本"), None);
        assert_eq!(pack_ascii6("ABC™"), None);
        assert_eq!(pack_ascii6("\u{80}"), None); // First non-ASCII
    }

    #[test]
    fn test_pack_ascii6_special_chars() {
        // Special ASCII chars should work
        assert_eq!(pack_ascii6("!@#$%^"), Some(0x21402324255E));
        assert_eq!(pack_ascii6("\t"), Some(9)); // Tab
        assert_eq!(pack_ascii6("\n"), Some(10)); // Newline
        assert_eq!(pack_ascii6("\x7F"), Some(127)); // DEL (last ASCII)
    }

    // ==================== string_to_hash tests ====================
    // Uses CRC32 (ISO HDLC) algorithm, matching Unity's Animator.StringToHash

    #[test]
    fn test_string_to_hash_empty() {
        assert_eq!(string_to_hash(""), 0);
    }

    #[test]
    fn test_string_to_hash_single_char() {
        // CRC32 hash values for single characters
        assert_eq!(string_to_hash("A"), -740712821);
        assert_eq!(string_to_hash("a"), -390611389);
        assert_eq!(string_to_hash("0"), -186917087);
    }

    #[test]
    fn test_string_to_hash_two_chars() {
        // CRC32 hash for "AB"
        assert_eq!(string_to_hash("AB"), 812207111);
    }

    #[test]
    fn test_string_to_hash_three_chars() {
        // CRC32 hash for "ABC"
        assert_eq!(string_to_hash("ABC"), -1551695032);
    }

    #[test]
    fn test_string_to_hash_test() {
        // CRC32 hash for "Test"
        assert_eq!(string_to_hash("Test"), 2018365746);
    }

    #[test]
    fn test_string_to_hash_long_string() {
        // Should handle arbitrarily long strings
        let result = string_to_hash("ThisIsAVeryLongStringThatExceedsSixCharacters");
        // Just verify it doesn't panic and returns a value
        assert!(result != 0);
    }

    #[test]
    fn test_string_to_hash_unicode() {
        // CRC32 hash for "日" (UTF-8 encoded: E6 97 A5)
        assert_eq!(string_to_hash("日"), -896385350);
    }

    #[test]
    fn test_string_to_hash_wrapping() {
        // Should use wrapping arithmetic (no overflow panic)
        let result = string_to_hash("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
        // Just verify it doesn't panic
        let _ = result;
    }

    // ==================== parse_binary_str tests ====================

    #[test]
    fn test_parse_binary_str_simple() {
        assert_eq!(parse_binary_str("0"), Some(0));
        assert_eq!(parse_binary_str("1"), Some(1));
        assert_eq!(parse_binary_str("10"), Some(2));
        assert_eq!(parse_binary_str("1010"), Some(10));
        assert_eq!(parse_binary_str("11111111"), Some(255));
    }

    #[test]
    fn test_parse_binary_str_with_underscores() {
        assert_eq!(parse_binary_str("1010_1010"), Some(0xAA));
        assert_eq!(parse_binary_str("1111_0000"), Some(0xF0));
        assert_eq!(parse_binary_str("1_0_1_0"), Some(10));
        assert_eq!(parse_binary_str("____1____"), Some(1));
    }

    #[test]
    fn test_parse_binary_str_empty() {
        assert_eq!(parse_binary_str(""), None);
        assert_eq!(parse_binary_str("____"), None); // Only underscores
    }

    #[test]
    fn test_parse_binary_str_large() {
        // 64-bit max
        assert_eq!(
            parse_binary_str("111111111111111111111111111111111111111111111111111111111111111"),
            Some(i64::MAX)
        );
    }

    // ==================== parse_hex_str tests ====================

    #[test]
    fn test_parse_hex_str_simple() {
        assert_eq!(parse_hex_str("0"), Some(0));
        assert_eq!(parse_hex_str("F"), Some(15));
        assert_eq!(parse_hex_str("f"), Some(15));
        assert_eq!(parse_hex_str("FF"), Some(255));
        assert_eq!(parse_hex_str("ff"), Some(255));
        assert_eq!(parse_hex_str("10"), Some(16));
        assert_eq!(parse_hex_str("DEAD"), Some(0xDEAD));
        assert_eq!(parse_hex_str("BEEF"), Some(0xBEEF));
    }

    #[test]
    fn test_parse_hex_str_with_underscores() {
        assert_eq!(parse_hex_str("DE_AD"), Some(0xDEAD));
        assert_eq!(parse_hex_str("1A_2B_3C"), Some(0x1A2B3C));
        assert_eq!(parse_hex_str("____FF____"), Some(255));
    }

    #[test]
    fn test_parse_hex_str_empty() {
        assert_eq!(parse_hex_str(""), None);
        assert_eq!(parse_hex_str("____"), None);
    }

    #[test]
    fn test_parse_hex_str_mixed_case() {
        assert_eq!(parse_hex_str("DeAdBeEf"), Some(0xDEADBEEF));
    }

    // ==================== parse_hex / parse_binary tests ====================

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_hex("$FF").unwrap(), 255);
        assert_eq!(parse_hex("$1A_2B").unwrap(), 0x1A2B);
        assert_eq!(parse_hex("$10").unwrap(), 16);
        assert_eq!(parse_hex("$0").unwrap(), 0);
    }

    #[test]
    fn test_parse_hex_error() {
        assert!(parse_hex("$").is_err());
        assert!(parse_hex("$___").is_err());
        assert!(parse_hex("$GG").is_err());
    }

    #[test]
    fn test_parse_binary() {
        assert_eq!(parse_binary("%1010").unwrap(), 10);
        assert_eq!(parse_binary("%1111_0000").unwrap(), 0b11110000);
        assert_eq!(parse_binary("%1").unwrap(), 1);
        assert_eq!(parse_binary("%0").unwrap(), 0);
    }

    #[test]
    fn test_parse_binary_error() {
        assert!(parse_binary("%").is_err());
        assert!(parse_binary("%___").is_err());
        assert!(parse_binary("%2").is_err());
    }

    // ==================== preprocess integration tests ====================

    #[test]
    fn test_preprocess_str() {
        let input = r#"move r0 STR("Hello")"#;
        let result = preprocess(input).unwrap();
        // "Hello" = 0x48656C6C6F = 310939249775
        assert!(result.contains("310939249775"));
    }

    #[test]
    fn test_preprocess_str_too_long() {
        let input = r#"move r0 STR("TooLong!")"#;
        let result = preprocess(input).unwrap();
        assert!(result.contains("<ERR:InvalidStringLength>"));
    }

    #[test]
    fn test_preprocess_hash() {
        let input = r#"move r0 HASH("Test")"#;
        let result = preprocess(input).unwrap();
        assert!(result.contains("2018365746"));
    }

    #[test]
    fn test_preprocess_hash_long_string() {
        let input = r#"move r0 HASH("LongStringTest")"#;
        let result = preprocess(input).unwrap();
        assert!(result.contains("1680529357"));
    }

    #[test]
    fn test_preprocess_binary() {
        let input = "move r0 %1010";
        let result = preprocess(input).unwrap();
        assert!(result.contains("move r0 10"));
    }

    #[test]
    fn test_preprocess_hex() {
        let input = "move r0 $FF";
        let result = preprocess(input).unwrap();
        assert!(result.contains("move r0 255"));
    }

    #[test]
    fn test_preprocess_comment_removal() {
        let input = "move r0 1 # this is a comment";
        let result = preprocess(input).unwrap();
        assert_eq!(result, "move r0 1");
    }

    #[test]
    fn test_preprocess_comment_affects_str() {
        // STR("1#3") - comment removes #3")
        let input = r#"move r0 STR("1#3")"#;
        let result = preprocess(input).unwrap();
        // Should pass through as-is (no valid STR match after comment removal)
        assert!(result.contains(r#"STR("1"#));
    }

    #[test]
    fn test_preprocess_multiple_on_line() {
        let input = "add r0 $FF %1010";
        let result = preprocess(input).unwrap();
        assert!(result.contains("add r0 255 10"));
    }

    #[test]
    fn test_preprocess_str_vs_hash_different() {
        let result_str = preprocess(r#"move r0 STR("ABC")"#).unwrap();
        let result_hash = preprocess(r#"move r0 HASH("ABC")"#).unwrap();

        // STR: 0x414243 = 4276803
        assert!(result_str.contains("4276803"));
        // HASH: -1551695032
        assert!(result_hash.contains("-1551695032"));
    }

    #[test]
    fn test_preprocess_empty_lines_not_removed() {
        let input = "move r0 1\n\n# comment only\n\nmove r1 2";
        let result = preprocess(input).unwrap();
        assert_eq!(result, "move r0 1\n\n\n\nmove r1 2");
    }
}
