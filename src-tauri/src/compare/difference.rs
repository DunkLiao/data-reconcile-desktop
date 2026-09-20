use std::collections::HashMap;

use crate::models::compare_options::ComparisonMode;
use crate::models::difference::{Difference, DifferenceType, KeyValue};

#[derive(Debug, Clone, Copy)]
pub struct CompareSettings {
    pub trim_whitespace: bool,
    pub ignore_case: bool,
    /// `None` disables numeric tolerance entirely.
    pub numeric_tolerance: Option<f64>,
}

pub fn normalize(value: &str, settings: CompareSettings) -> String {
    let mut s = value.to_string();
    if settings.trim_whitespace {
        s = s.trim().to_string();
    }
    if settings.ignore_case {
        s = s.to_lowercase();
    }
    s
}

/// Parse a value as a finite number, accepting an optional thousands separator
/// (comma) and surrounding whitespace. Returns `None` for anything else.
pub fn parse_numeric(value: &str) -> Option<f64> {
    let cleaned: String = value.trim().chars().filter(|c| *c != ',').collect();
    if cleaned.is_empty() {
        return None;
    }
    match cleaned.parse::<f64>() {
        Ok(n) if n.is_finite() => Some(n),
        _ => None,
    }
}

/// Decide whether two raw cell values are considered equal.
///
/// String equality (after optional normalization) always wins. When a numeric
/// tolerance is configured and both sides parse as finite numbers, an absolute
/// difference within the tolerance is treated as equal. Non-numeric pairs fall
/// back to string comparison.
pub fn values_equal(a: &str, b: &str, settings: CompareSettings) -> bool {
    let na = normalize(a, settings);
    let nb = normalize(b, settings);
    if na == nb {
        return true;
    }
    if let Some(tolerance) = settings.numeric_tolerance {
        if let (Some(x), Some(y)) = (parse_numeric(&na), parse_numeric(&nb)) {
            return (x - y).abs() <= tolerance;
        }
    }
    false
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    Valid,
    Empty,
    Incomplete,
}

pub fn classify_key(values: &[String]) -> KeyStatus {
    if values.is_empty() {
        return KeyStatus::Empty;
    }
    let empty_count = values.iter().filter(|v| v.is_empty()).count();
    if empty_count == values.len() {
        KeyStatus::Empty
    } else if empty_count > 0 {
        KeyStatus::Incomplete
    } else {
        KeyStatus::Valid
    }
}

pub struct ColumnLayout {
    pub headers: Vec<String>,
    pub index_of: HashMap<String, usize>,
}

impl ColumnLayout {
    pub fn new(headers: Vec<String>) -> Self {
        let index_of = headers
            .iter()
            .enumerate()
            .map(|(i, h)| (h.clone(), i))
            .collect();
        ColumnLayout { headers, index_of }
    }

    pub fn get<'a>(&self, column: &str, record: &'a [String]) -> Option<&'a String> {
        self.index_of.get(column).and_then(move |&i| record.get(i))
    }

    /// Common columns between this layout and another, ordered by this layout.
    pub fn common_with(&self, other: &ColumnLayout) -> Vec<String> {
        self.headers
            .iter()
            .filter(|h| other.index_of.contains_key(*h))
            .cloned()
            .collect()
    }
}

/// Compute header-level differences (COLUMN_A_ONLY / COLUMN_B_ONLY).
pub fn column_differences(
    a: &ColumnLayout,
    b: &ColumnLayout,
    excluded: &[String],
) -> Vec<Difference> {
    let is_excluded = |c: &str| excluded.iter().any(|e| e == c);

    let a_only: Vec<String> = a
        .headers
        .iter()
        .filter(|h| !b.index_of.contains_key(*h) && !is_excluded(h))
        .cloned()
        .collect();
    let b_only: Vec<String> = b
        .headers
        .iter()
        .filter(|h| !a.index_of.contains_key(*h) && !is_excluded(h))
        .cloned()
        .collect();

    let mut out = Vec::new();
    for c in a_only {
        out.push(Difference {
            key_values: vec![],
            row_a: None,
            row_b: None,
            column_name: Some(c),
            value_a: None,
            value_b: None,
            difference_type: DifferenceType::ColumnAOnly,
        });
    }
    for c in b_only {
        out.push(Difference {
            key_values: vec![],
            row_a: None,
            row_b: None,
            column_name: Some(c),
            value_a: None,
            value_b: None,
            difference_type: DifferenceType::ColumnBOnly,
        });
    }
    out
}

pub fn build_key_values(
    key_columns: &[String],
    layout: &ColumnLayout,
    record: &[String],
) -> Vec<KeyValue> {
    key_columns
        .iter()
        .map(|col| KeyValue {
            column: col.clone(),
            value: layout.get(col, record).cloned().unwrap_or_default(),
        })
        .collect()
}

pub fn format_mode(mode: ComparisonMode) -> &'static str {
    match mode {
        ComparisonMode::KeyBased => "KEY_BASED",
        ComparisonMode::RowByRow => "ROW_BY_ROW",
    }
}
