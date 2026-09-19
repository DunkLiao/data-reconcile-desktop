use serde::{Deserialize, Serialize};

use super::difference::{Difference, DuplicateKeyInfo};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompareResult {
    pub identical: bool,

    pub rows_a: u64,
    pub rows_b: u64,

    pub key_columns: Vec<String>,
    pub compared_columns: Vec<String>,
    pub excluded_columns: Vec<String>,
    pub column_a_only: Vec<String>,
    pub column_b_only: Vec<String>,

    pub matched_records: u64,
    pub same_records: u64,
    pub different_records: u64,

    pub a_only_records: u64,
    pub b_only_records: u64,

    pub duplicate_keys_a: u64,
    pub duplicate_keys_b: u64,

    pub empty_keys_a: u64,
    pub empty_keys_b: u64,
    pub incomplete_keys_a: u64,
    pub incomplete_keys_b: u64,

    pub different_cells: u64,

    pub differences: Vec<Difference>,
    pub duplicate_details_a: Vec<DuplicateKeyInfo>,
    pub duplicate_details_b: Vec<DuplicateKeyInfo>,

    pub file_a_name: String,
    pub file_b_name: String,
    pub file_a_encoding: String,
    pub file_b_encoding: String,
    pub file_a_delimiter: String,
    pub file_b_delimiter: String,
    pub compare_mode: String,
    pub compare_time: String,
    pub cancelled: bool,
}
