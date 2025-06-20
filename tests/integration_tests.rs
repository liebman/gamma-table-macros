use gamma_table_macros::gamma_table;

// Test basic functionality (gamma encoding by default)
gamma_table! {
    name: TEST_GAMMA_ENCODING_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256
}

// Test gamma decoding/correction
gamma_table! {
    name: TEST_GAMMA_DECODING_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256,
    decoding: true
}

// Test different data types
gamma_table! {
    name: TEST_U16_TABLE,
    entry_type: u16,
    gamma: 1.8,
    size: 256
}

// Test u16 with full range
gamma_table! {
    name: TEST_U16_FULL_RANGE_TABLE,
    entry_type: u16,
    gamma: 1.8,
    size: 256,
    max_value: 65535
}

// Test brightness limiting
gamma_table! {
    name: TEST_LIMITED_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256,
    max_value: 128
}

// Test minimum valid size
gamma_table! {
    name: TEST_MINIMUM_SIZE_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 3
}

#[test]
fn test_encoding_table_properties() {
    // Test gamma encoding table properties
    assert_eq!(TEST_GAMMA_ENCODING_TABLE.len(), 256);
    assert_eq!(TEST_GAMMA_ENCODING_TABLE[0], 0);
    assert_eq!(TEST_GAMMA_ENCODING_TABLE[255], 255);

    // Values should be monotonically increasing
    for i in 1..TEST_GAMMA_ENCODING_TABLE.len() {
        assert!(TEST_GAMMA_ENCODING_TABLE[i] >= TEST_GAMMA_ENCODING_TABLE[i - 1]);
    }
}

#[test]
fn test_decoding_table_properties() {
    // Test gamma decoding table properties
    assert_eq!(TEST_GAMMA_DECODING_TABLE.len(), 256);
    assert_eq!(TEST_GAMMA_DECODING_TABLE[0], 0);
    assert_eq!(TEST_GAMMA_DECODING_TABLE[255], 255);

    // Values should be monotonically increasing
    for i in 1..TEST_GAMMA_DECODING_TABLE.len() {
        assert!(TEST_GAMMA_DECODING_TABLE[i] >= TEST_GAMMA_DECODING_TABLE[i - 1]);
    }
}

#[test]
fn test_different_data_types() {
    assert_eq!(TEST_U16_TABLE.len(), 256);
    assert_eq!(TEST_U16_TABLE[0], 0);
    assert_eq!(TEST_U16_TABLE[255], 255);

    // Check that u16 table with default max_value
    assert!(TEST_U16_TABLE[128] <= 255); // Will be <= 255 since max_value defaults to size-1 = 255

    // Test u16 table with explicit full range
    assert_eq!(TEST_U16_FULL_RANGE_TABLE[0], 0);
    assert_eq!(TEST_U16_FULL_RANGE_TABLE[255], 65535);
    assert!(TEST_U16_FULL_RANGE_TABLE[128] > 255); // Should exceed u8 range with explicit max_value
}

#[test]
fn test_brightness_limiting() {
    assert_eq!(TEST_LIMITED_TABLE.len(), 256);
    assert_eq!(TEST_LIMITED_TABLE[0], 0);
    assert_eq!(TEST_LIMITED_TABLE[255], 128); // Limited to max_value

    // All values should be <= 128
    for &value in &TEST_LIMITED_TABLE {
        assert!(value <= 128);
    }
}

#[test]
fn test_encoding_vs_decoding_difference() {
    // Compare encoding vs decoding at 50% input
    let input_50_percent = 128usize;
    let encoding_50_percent = TEST_GAMMA_ENCODING_TABLE[input_50_percent];
    let decoding_50_percent = TEST_GAMMA_DECODING_TABLE[input_50_percent];

    // Encoding and decoding should produce different results for mid-values
    assert_ne!(encoding_50_percent, decoding_50_percent);

    // Both should have same endpoints
    assert_eq!(TEST_GAMMA_ENCODING_TABLE[0], TEST_GAMMA_DECODING_TABLE[0]); // Both 0
    assert_eq!(
        TEST_GAMMA_ENCODING_TABLE[255],
        TEST_GAMMA_DECODING_TABLE[255]
    ); // Both 255

    // For gamma 2.2:
    // - Encoding (input^2.2) makes mid-tones darker
    // - Decoding (input^(1/2.2)) makes mid-tones brighter
    let linear_50_percent = 128u8;
    assert!(encoding_50_percent < linear_50_percent); // Encoding makes darker
    assert!(decoding_50_percent > linear_50_percent); // Decoding makes brighter
}

#[test]
fn test_default_max_value() {
    // Test that when max_value is not specified, it defaults to size-1
    assert_eq!(TEST_GAMMA_ENCODING_TABLE[255], 255); // size 256, max should be 255
    assert_eq!(TEST_U16_TABLE[255], 255); // size 256, max should be 255 (not related to u16 range)
}

#[test]
fn test_minimum_size_table() {
    // Test that a table with size 3 (minimum) works correctly
    assert_eq!(TEST_MINIMUM_SIZE_TABLE.len(), 3);
    assert_eq!(TEST_MINIMUM_SIZE_TABLE[0], 0); // Always 0 at start
    assert_eq!(TEST_MINIMUM_SIZE_TABLE[2], 2); // max_value defaults to size-1 = 2

    // Values should be monotonically increasing
    assert!(TEST_MINIMUM_SIZE_TABLE[1] >= TEST_MINIMUM_SIZE_TABLE[0]);
    assert!(TEST_MINIMUM_SIZE_TABLE[2] >= TEST_MINIMUM_SIZE_TABLE[1]);
}

#[test]
fn test_compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
