# Gamma Table Macros

[![Crates.io](https://img.shields.io/crates/v/gamma-table-macros.svg)](https://crates.io/crates/gamma-table-macros)
[![Documentation](https://docs.rs/gamma-table-macros/badge.svg)](https://docs.rs/gamma-table-macros)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](README.md)
[![Coverage Status](https://coveralls.io/repos/github/liebman/gamma-table-macros/badge.svg?branch=main)](https://coveralls.io/github/liebman/gamma-table-macros?branch=main)

A Rust procedural macro crate for generating compile-time gamma lookup tables with support for both gamma encoding and gamma correction/decoding.

## Overview

This crate provides a proc macro that generates efficient lookup tables for gamma processing at compile time. By default, it uses **gamma encoding** (input^gamma). It also supports **gamma correction/decoding** (input^(1/gamma)) when the `decoding` parameter is set to true. This is particularly useful for graphics applications, LED control, and any scenario where you need fast gamma processing without runtime computation.

## Features

- **Compile-time generation**: Tables are computed at compile time, resulting in zero runtime overhead
- **Dual gamma modes**: Gamma encoding (default) and gamma correction/decoding
- **Flexible parameters**: Configurable gamma values, table sizes, entry types, and brightness limits
- **Multiple data types**: Support for u8, u16, u32, and u64 entry types
- **Brightness limiting**: Optional max_value parameter to cap output brightness

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
gamma-table-macros = "0.1.0"
```

### Gamma Encoding Example (Default)

```rust
use gamma_table_macros::gamma_table;

// Generate a gamma encoding table (default behavior)
gamma_table! {
    name: GAMMA_ENCODED_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256
}

fn main() {
    let input = 128u8;  // 50% brightness
    let encoded = GAMMA_ENCODED_TABLE[input as usize];
    println!("Input: {}, Gamma encoded: {}", input, encoded);  // Will be darker
}
```

### Gamma Correction/Decoding Example

```rust
use gamma_table_macros::gamma_table;

// Generate a gamma correction/decoding table
gamma_table! {
    name: GAMMA_DECODED_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 256,
    decoding: true
}

fn main() {
    let input = 128u8;  // 50% brightness
    let corrected = GAMMA_DECODED_TABLE[input as usize];
    println!("Input: {}, Gamma corrected: {}", input, corrected);  // Will be brighter
}
```

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
- **`size`** (required): Number of table entries (minimum 3)
- **`max_value`** (optional): Maximum output value to limit brightness (defaults to `size-1`)
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
