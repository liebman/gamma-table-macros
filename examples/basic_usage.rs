use gamma_table_macros::gamma_table;

// Generate a gamma encoding table (default behavior)
gamma_table! {
    name: GAMMA_ENCODING_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256
}

// Generate a gamma correction/decoding table (traditional gamma correction)
gamma_table! {
    name: GAMMA_DECODING_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256,
    decoding: true
}

// Generate a high-resolution 16-bit table
gamma_table! {
    name: GAMMA_TABLE_16BIT,
    entry_type: u16,
    gamma: 1.8,
    size: 65536
}

// Generate a brightness-limited table for LED control
gamma_table! {
    name: LED_GAMMA_TABLE,
    entry_type: u8,
    gamma: 2.5,
    size: 256,
    max_value: 128  // Limit to 50% brightness
}

fn main() {
    println!("=== Gamma Encoding vs Decoding Comparison ===");
    println!("Gamma Encoding Table (input^gamma):");
    for (i, &value) in GAMMA_ENCODING_TABLE.iter().take(16).enumerate() {
        println!("  Input: {:3} -> Output: {:3}", i, value);
    }

    println!("\nGamma Decoding Table (input^(1/gamma) - traditional gamma correction):");
    for (i, &value) in GAMMA_DECODING_TABLE.iter().take(16).enumerate() {
        println!("  Input: {:3} -> Output: {:3}", i, value);
    }

    println!("\n=== LED Gamma Table (brightness limited) ===");
    for (i, &value) in LED_GAMMA_TABLE.iter().take(16).enumerate() {
        println!(
            "  Input: {:3} -> Output: {:3} (max possible: 128)",
            i, value
        );
    }

    // Demonstrate the difference between encoding and decoding
    let test_input = 128u8; // 50% input
    let encoded_value = GAMMA_ENCODING_TABLE[test_input as usize];
    let decoded_value = GAMMA_DECODING_TABLE[test_input as usize];

    println!("\n=== Gamma Processing Comparison at 50% Input ===");
    println!("Input value: {} (50%)", test_input);
    println!("Gamma encoding (input^2.2): {} (darker)", encoded_value);
    println!(
        "Gamma decoding (input^(1/2.2)): {} (brighter)",
        decoded_value
    );
    println!("Linear (no gamma): {} (reference)", test_input);
}
