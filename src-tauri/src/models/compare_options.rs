use serde::{Deserialize, Serialize};

use super::parse_options::ParseOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonMode {
    KeyBased,
    RowByRow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareOptions {
    pub file_a_path: String,
    pub file_b_path: String,

    pub file_a_parse_options: ParseOptions,
    pub file_b_parse_options: ParseOptions,

    pub comparison_mode: ComparisonMode,

    pub key_columns: Vec<String>,
    pub excluded_columns: Vec<String>,

    pub trim_whitespace: bool,
    pub ignore_case: bool,

    /// Unified absolute tolerance applied to numeric value cells. `None` disables
    /// numeric tolerance (strict string comparison). Key columns are never affected.
    #[serde(default)]
    pub numeric_tolerance: Option<f64>,
}
