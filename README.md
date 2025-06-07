# Gamma Correction Proc Macro

A Rust procedural macro crate for generating compile-time gamma lookup tables with support for both gamma encoding and gamma correction/decoding.

## Overview

This crate provides a proc macro that generates efficient lookup tables for gamma processing at compile time. By default, it uses **gamma encoding** (input^gamma). It also supports **gamma correction/decoding** (input^(1/gamma)) when the `decoding` parameter is set to true. This is particularly useful for graphics applications, LED control, and any scenario where you need fast gamma processing without runtime computation.

## Features

- **Compile-time generation**: Tables are computed at compile time, resulting in zero runtime overhead
- **Dual gamma modes**: Gamma encoding (default) and gamma correction/decoding
- **Flexible parameters**: Configurable gamma values, table sizes, entry types, and brightness limits
- **Step quantization**: Support for reduced step counts for memory-constrained applications
- **Multiple data types**: Support for u8, u16, u32, and u64 entry types
- **Brightness limiting**: Optional max_value parameter to cap output brightness

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
gamma-correction-proc = "0.1.0"
```

### Gamma Encoding Example (Default)

```rust
use gamma_correction_proc::gamma_table;

// Generate a gamma encoding table (default behavior)
gamma_table! {
    name: GAMMA_ENCODED_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256,
    max_value: 255
}

fn main() {
    let input = 128u8;  // 50% brightness
    let encoded = GAMMA_ENCODED_TABLE[input as usize];
    println!("Input: {}, Gamma encoded: {}", input, encoded);  // Will be darker
}
```

### Gamma Correction/Decoding Example

```rust
use gamma_correction_proc::gamma_table;

// Generate a gamma correction/decoding table
gamma_table! {
    name: GAMMA_DECODED_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256,
    max_value: 255,
    decoding: true
}

fn main() {
    let input = 128u8;  // 50% brightness
    let corrected = GAMMA_DECODED_TABLE[input as usize];
    println!("Input: {}, Gamma corrected: {}", input, corrected);  // Will be brighter
}
```

### Step Quantization Example

For memory-constrained applications, you can use step quantization:

```rust
gamma_table! {
    name: STEPPED_GAMMA_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256,
    steps: 8,        // Only 8 discrete output levels
    max_value: 255
}
```

With 8 steps and 256 entries, this creates step boundaries at approximately:

- Step 0: inputs 0-18 → output 0
- Step 1: inputs 19-54 → output ~15
- Step 2: inputs 55-91 → output ~45
- ...and so on

### LED Control Example

For LED control where you want to limit maximum brightness:

```rust
gamma_table! {
    name: LED_GAMMA_TABLE,
    entry_type: u8,
    gamma: 2.5,
    size: 256,
    max_value: 128  // Limit to 50% max brightness
}
```

## Parameters

- **`name`** (required): The name of the const table to be generated
- **`entry_type`** (required): The unsigned integer type for each entry (`u8`, `u16`, `u32`, `u64`)
- **`gamma`** (required): The gamma value (positive float)
- **`size`** (required): Number of table entries
- **`steps`** (optional): Number of discrete output levels (defaults to `size` for smooth gradients)
- **`max_value`** (required): Maximum output value to limit brightness
- **`decoding`** (optional): Use gamma correction/decoding instead of encoding (defaults to `false`)

## Mathematics

### Gamma Encoding (Default)

```c
output = (input / max_input)^gamma * max_value
```

### Gamma Correction/Decoding (`decoding: true`)

```c
output = (input / max_input)^(1/gamma) * max_value
```

### Step Quantization

When using fewer steps than the table size:

1. Input values are mapped to step indices
2. Step values are normalized (0.0 to 1.0)
3. Gamma processing is applied to step values
4. Results are scaled to the output range

## Performance

Since tables are generated at compile time, runtime performance is simply a single array lookup operation - O(1) with no floating-point computation needed.

## Examples

Run the examples to see the macro in action:

```bash
cargo run --example basic_usage
```

## Testing

Run the test suite:

```bash
cargo test
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
