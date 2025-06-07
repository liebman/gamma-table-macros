use gamma_correction_proc::gamma_table;

// Test basic functionality (gamma encoding by default)
gamma_table! {
    name: TEST_GAMMA_ENCODING_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256,
    max_value: 255
}

// Test gamma decoding/correction
gamma_table! {
    name: TEST_GAMMA_DECODING_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256,
    max_value: 255,
    decoding: true
}

// Test different data types
gamma_table! {
    name: TEST_U16_TABLE,
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
    assert_eq!(TEST_U16_TABLE[255], 65535);

    // Check that u16 table uses full range
    assert!(TEST_U16_TABLE[128] > 255); // Should exceed u8 range
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
