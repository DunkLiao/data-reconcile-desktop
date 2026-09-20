use super::difference::{classify_key, ColumnLayout, CompareSettings, KeyStatus};
use crate::error::AppError;
use crate::models::compare_options::CompareOptions;
use crate::models::composite_key::CompositeKey;

/// Extract the composite key values from a record using the key column layout.
/// The key column order follows the layout's header order (File A header order).
pub struct KeyBuilder {
    requested: Vec<String>,
    columns: Vec<String>,
}

impl KeyBuilder {
    pub fn new(key_columns: Vec<String>, layout: &ColumnLayout) -> Self {
        // Preserve the layout (File A header) order for the composite key.
        let columns = layout
            .headers
            .iter()
            .filter(|h| key_columns.iter().any(|k| k == *h))
            .cloned()
            .collect();
        KeyBuilder {
            requested: key_columns,
            columns,
        }
    }

    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    pub fn requested(&self) -> &[String] {
        &self.requested
    }

    pub fn validate_present(&self, layout: &ColumnLayout) -> Result<(), AppError> {
        for col in &self.requested {
            if !layout.index_of.contains_key(col) {
                return Err(AppError::MissingKeyColumn(col.clone()));
            }
        }
        Ok(())
    }

    pub fn build(&self, record: &[String], layout: &ColumnLayout) -> (CompositeKey, KeyStatus) {
        let values: Vec<String> = self
            .columns
            .iter()
            .map(|col| layout.get(col, record).cloned().unwrap_or_default())
            .collect();
        let status = classify_key(&values);
        (CompositeKey(values), status)
    }
}

/// Validate the compare options against the actual file layouts.
pub fn validate_options(
    opts: &CompareOptions,
    layout_a: &ColumnLayout,
    layout_b: &ColumnLayout,
    key_builder: &KeyBuilder,
) -> Result<(), AppError> {
    key_builder.validate_present(layout_a)?;
    key_builder.validate_present(layout_b)?;

    // Key columns must exist in both files (common columns only).
    for col in key_builder.requested() {
        if !layout_a.index_of.contains_key(col) || !layout_b.index_of.contains_key(col) {
            return Err(AppError::MissingKeyColumn(col.clone()));
        }
    }

    // Key columns must not be excluded from comparison.
    for col in key_builder.requested() {
        if opts.excluded_columns.iter().any(|e| e == col) {
            return Err(AppError::Other(format!(
                "Key 欄位不得同時設為排除比較欄位：{col}"
            )));
        }
    }

    Ok(())
}

pub fn settings_from(opts: &CompareOptions) -> CompareSettings {
    CompareSettings {
        trim_whitespace: opts.trim_whitespace,
        ignore_case: opts.ignore_case,
        numeric_tolerance: opts.numeric_tolerance,
    }
}
