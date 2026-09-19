use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValue {
    pub column: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DifferenceType {
    ValueChanged,
    AOnly,
    BOnly,
    ColumnAOnly,
    ColumnBOnly,
    DuplicateKeyA,
    DuplicateKeyB,
    EmptyKeyA,
    EmptyKeyB,
    IncompleteKeyA,
    IncompleteKeyB,
}

impl DifferenceType {
    pub fn label(&self) -> &'static str {
        match self {
            DifferenceType::ValueChanged => "VALUE_CHANGED",
            DifferenceType::AOnly => "A_ONLY",
            DifferenceType::BOnly => "B_ONLY",
            DifferenceType::ColumnAOnly => "COLUMN_A_ONLY",
            DifferenceType::ColumnBOnly => "COLUMN_B_ONLY",
            DifferenceType::DuplicateKeyA => "DUPLICATE_KEY_A",
            DifferenceType::DuplicateKeyB => "DUPLICATE_KEY_B",
            DifferenceType::EmptyKeyA => "EMPTY_KEY_A",
            DifferenceType::EmptyKeyB => "EMPTY_KEY_B",
            DifferenceType::IncompleteKeyA => "INCOMPLETE_KEY_A",
            DifferenceType::IncompleteKeyB => "INCOMPLETE_KEY_B",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    pub key_values: Vec<KeyValue>,

    pub row_a: Option<u64>,
    pub row_b: Option<u64>,

    pub column_name: Option<String>,

    pub value_a: Option<String>,
    pub value_b: Option<String>,

    pub difference_type: DifferenceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateKeyInfo {
    pub source: String,
    pub key_values: Vec<KeyValue>,
    pub count: u64,
    pub rows: Vec<u64>,
}
