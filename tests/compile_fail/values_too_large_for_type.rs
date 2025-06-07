use gamma_table_macros::gamma_table;

// This should fail to compile because:
// - entry_type: u8 (max value 255)
// - size: 1024 
// - default max_value would be 1023 (size-1)
// - u8 cannot hold values larger than 255
gamma_table! {
    name: OVERFLOW_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 1024
}

fn main() {} 