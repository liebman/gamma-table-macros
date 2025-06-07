use gamma_table_macros::gamma_table;

// This should fail to compile because size 1 is less than the minimum of 3
gamma_table! {
    name: INVALID_SIZE_ONE_TABLE,
    entry_type: u8,
    gamma: 2.2,
    size: 1
}

fn main() {} 