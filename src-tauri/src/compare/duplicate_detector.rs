use std::collections::HashMap;

use crate::models::composite_key::CompositeKey;
use crate::models::difference::{Difference, DuplicateKeyInfo, KeyValue};

use super::difference::KeyStatus;

/// Tracks duplicate keys per file during a scan.
pub struct DuplicateTracker {
    first: HashMap<CompositeKey, u64>,
    duplicates: HashMap<CompositeKey, (u64, Vec<u64>)>,
}

impl DuplicateTracker {
    pub fn new() -> Self {
        DuplicateTracker {
            first: HashMap::new(),
            duplicates: HashMap::new(),
        }
    }

    /// Record a key occurrence. Returns true if this occurrence is a duplicate.
    pub fn record(&mut self, key: &CompositeKey, row: u64) -> bool {
        if let Some(&first_row) = self.first.get(key) {
            let entry = self
                .duplicates
                .entry(key.clone())
                .or_insert_with(|| (1, vec![first_row]));
            entry.0 += 1;
            entry.1.push(row);
            true
        } else {
            self.first.insert(key.clone(), row);
            false
        }
    }

    pub fn is_duplicate(&self, key: &CompositeKey) -> bool {
        self.duplicates.contains_key(key)
    }

    pub fn duplicate_keys(&self) -> &HashMap<CompositeKey, (u64, Vec<u64>)> {
        &self.duplicates
    }

    pub fn to_infos(&self, source: &str, key_columns: &[String]) -> Vec<DuplicateKeyInfo> {
        self.duplicates
            .iter()
            .map(|(key, (count, rows))| {
                let key_values: Vec<KeyValue> = key_columns
                    .iter()
                    .zip(key.0.iter())
                    .map(|(col, val)| KeyValue {
                        column: col.clone(),
                        value: val.clone(),
                    })
                    .collect();
                DuplicateKeyInfo {
                    source: source.to_string(),
                    key_values,
                    count: *count,
                    rows: rows.clone(),
                }
            })
            .collect()
    }
}

impl Default for DuplicateTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Classify a key and report the matching difference type.
pub fn key_status_diff(
    status: KeyStatus,
    is_a: bool,
    key_values: &[KeyValue],
    row: u64,
) -> Option<Difference> {
    let difference_type = match (is_a, status) {
        (true, KeyStatus::Empty) => crate::models::difference::DifferenceType::EmptyKeyA,
        (true, KeyStatus::Incomplete) => crate::models::difference::DifferenceType::IncompleteKeyA,
        (false, KeyStatus::Empty) => crate::models::difference::DifferenceType::EmptyKeyB,
        (false, KeyStatus::Incomplete) => crate::models::difference::DifferenceType::IncompleteKeyB,
        (_, KeyStatus::Valid) => return None,
    };
    Some(Difference {
        key_values: key_values.to_vec(),
        row_a: if is_a { Some(row) } else { None },
        row_b: if is_a { None } else { Some(row) },
        column_name: None,
        value_a: None,
        value_b: None,
        difference_type,
    })
}
