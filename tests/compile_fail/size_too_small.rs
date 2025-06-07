use gamma_table_macros::gamma_table;

// This should fail to compile because size is less than 3
gamma_table! {
    name: INVALID_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 2
}

fn main() {} 