use std::collections::{HashMap, HashSet};
use std::sync::atomic::AtomicBool;

use crate::error::AppError;
use crate::models::compare_options::CompareOptions;
use crate::models::compare_result::CompareResult;
use crate::models::composite_key::CompositeKey;
use crate::models::difference::{Difference, DifferenceType};
use crate::parser::decoder::open_delimited_reader;
use crate::parser::delimited_parser::check_duplicate_headers;
use crate::parser::reader::file_name;

use super::difference::{
    build_key_values, column_differences, format_mode, normalize, ColumnLayout,
};
use super::duplicate_detector::{key_status_diff, DuplicateTracker};
use super::key_builder::{settings_from, validate_options, KeyBuilder};
use super::util::{check_cancel, now_string, ProgressFn};

pub fn compare_key_based(
    opts: &CompareOptions,
    cancel: &AtomicBool,
    progress: &ProgressFn,
) -> Result<CompareResult, AppError> {
    check_cancel(cancel)?;
    let (reader_a, enc_a, delim_a) = open_delimited_reader(
        &opts.file_a_path,
        opts.file_a_parse_options.encoding,
        opts.file_a_parse_options.delimiter,
        true,
    )?;
    let layout_a = ColumnLayout::new(reader_a.headers().to_vec());
    check_duplicate_headers(&layout_a.headers)?;

    let key_builder = KeyBuilder::new(opts.key_columns.clone(), &layout_a);

    // ---- Load File A into a key index (streaming) ----
    let mut records: HashMap<CompositeKey, (u64, Vec<String>)> = HashMap::new();
    let mut duplicate_records_a: HashMap<CompositeKey, Vec<(u64, Vec<String>)>> = HashMap::new();
    let mut dup_a = DuplicateTracker::new();
    let mut rows_a: u64 = 0;
    let mut empty_keys_a: u64 = 0;
    let mut incomplete_keys_a: u64 = 0;
    let mut differences: Vec<Difference> = Vec::new();

    let mut reader_a = reader_a;
    let mut a_processed: u64 = 0;
    while let Some((row, fields)) = reader_a.next_record()? {
        check_cancel(cancel)?;
        rows_a += 1;
        a_processed += 1;
        if a_processed.is_multiple_of(5000) {
            progress(a_processed, None, "讀取 File A");
        }

        let (key, status) = key_builder.build(&fields, &layout_a);
        match status {
            super::difference::KeyStatus::Empty => {
                empty_keys_a += 1;
                let kv = build_key_values(key_builder.columns(), &layout_a, &fields);
                if let Some(d) = key_status_diff(status, true, &kv, row) {
                    differences.push(d);
                }
                continue;
            }
            super::difference::KeyStatus::Incomplete => {
                incomplete_keys_a += 1;
                let kv = build_key_values(key_builder.columns(), &layout_a, &fields);
                if let Some(d) = key_status_diff(status, true, &kv, row) {
                    differences.push(d);
                }
                continue;
            }
            super::difference::KeyStatus::Valid => {}
        }

        duplicate_records_a
            .entry(key.clone())
            .or_default()
            .push((row, fields.clone()));
        if dup_a.record(&key, row) {
            continue;
        }
        records.insert(key, (row, fields));
    }

    // ---- First File B pass: discover duplicate keys before comparison ----
    let (mut reader_b_scan, enc_b, delim_b) = open_delimited_reader(
        &opts.file_b_path,
        opts.file_b_parse_options.encoding,
        opts.file_b_parse_options.delimiter,
        true,
    )?;
    let layout_b = ColumnLayout::new(reader_b_scan.headers().to_vec());
    check_duplicate_headers(&layout_b.headers)?;
    validate_options(opts, &layout_a, &layout_b, &key_builder)?;

    let settings = settings_from(opts);
    let compared_columns: Vec<String> = layout_a
        .common_with(&layout_b)
        .into_iter()
        .filter(|c| !opts.excluded_columns.iter().any(|e| e == c))
        .collect();

    let col_diffs = column_differences(&layout_a, &layout_b, &opts.excluded_columns);
    let column_a_only: Vec<String> = col_diffs
        .iter()
        .filter(|d| d.difference_type == DifferenceType::ColumnAOnly)
        .filter_map(|d| d.column_name.clone())
        .collect();
    let column_b_only: Vec<String> = col_diffs
        .iter()
        .filter(|d| d.difference_type == DifferenceType::ColumnBOnly)
        .filter_map(|d| d.column_name.clone())
        .collect();
    differences.extend(col_diffs);

    let mut dup_b = DuplicateTracker::new();
    let mut duplicate_records_b: HashMap<CompositeKey, Vec<(u64, Vec<String>)>> = HashMap::new();
    let mut rows_b: u64 = 0;
    while let Some((row, fields)) = reader_b_scan.next_record()? {
        check_cancel(cancel)?;
        rows_b += 1;
        let (key, status) = key_builder.build(&fields, &layout_b);
        if status == super::difference::KeyStatus::Valid {
            duplicate_records_b
                .entry(key.clone())
                .or_default()
                .push((row, fields.clone()));
            dup_b.record(&key, row);
        }
        if rows_b.is_multiple_of(5000) {
            progress(rows_b, None, "掃描 File B 重複 Key");
        }
    }
    check_cancel(cancel)?;

    // ---- Second File B pass: compare keys known to be unique ----
    let (reader_b, _, _) = open_delimited_reader(
        &opts.file_b_path,
        opts.file_b_parse_options.encoding,
        opts.file_b_parse_options.delimiter,
        true,
    )?;
    let mut matched_keys: HashSet<CompositeKey> = HashSet::new();
    let mut empty_keys_b: u64 = 0;
    let mut incomplete_keys_b: u64 = 0;
    let mut matched_records: u64 = 0;
    let mut same_records: u64 = 0;
    let mut different_records: u64 = 0;
    let mut different_cells: u64 = 0;
    let mut b_only_records: u64 = 0;

    let mut reader_b = reader_b;
    let mut b_processed: u64 = 0;
    while let Some((row, fields)) = reader_b.next_record()? {
        check_cancel(cancel)?;
        b_processed += 1;
        if b_processed.is_multiple_of(5000) {
            progress(b_processed, None, "比對 File B");
        }

        let (key, status) = key_builder.build(&fields, &layout_b);
        match status {
            super::difference::KeyStatus::Empty => {
                empty_keys_b += 1;
                let kv = build_key_values(key_builder.columns(), &layout_b, &fields);
                if let Some(d) = key_status_diff(status, false, &kv, row) {
                    differences.push(d);
                }
                continue;
            }
            super::difference::KeyStatus::Incomplete => {
                incomplete_keys_b += 1;
                let kv = build_key_values(key_builder.columns(), &layout_b, &fields);
                if let Some(d) = key_status_diff(status, false, &kv, row) {
                    differences.push(d);
                }
                continue;
            }
            super::difference::KeyStatus::Valid => {}
        }

        if dup_b.is_duplicate(&key) {
            continue;
        }

        if dup_a.is_duplicate(&key) {
            // This key cannot be uniquely paired (File A has duplicates).
            continue;
        }

        if let Some((a_row, a_fields)) = records.get(&key) {
            matched_records += 1;
            let mut changed: Vec<(String, String, String)> = Vec::new();
            for col in &compared_columns {
                let a_val = layout_a.get(col, a_fields).cloned().unwrap_or_default();
                let b_val = layout_b.get(col, &fields).cloned().unwrap_or_default();
                if normalize(&a_val, settings) != normalize(&b_val, settings) {
                    changed.push((col.clone(), a_val, b_val));
                }
            }

            if changed.is_empty() {
                same_records += 1;
            } else {
                different_records += 1;
                different_cells += changed.len() as u64;
                let kv = build_key_values(key_builder.columns(), &layout_a, a_fields);
                for (col, a_val, b_val) in changed {
                    differences.push(Difference {
                        key_values: kv.clone(),
                        row_a: Some(*a_row),
                        row_b: Some(row),
                        column_name: Some(col),
                        value_a: Some(a_val),
                        value_b: Some(b_val),
                        difference_type: DifferenceType::ValueChanged,
                    });
                }
            }
            matched_keys.insert(key);
        } else {
            b_only_records += 1;
            let kv = build_key_values(key_builder.columns(), &layout_b, &fields);
            differences.push(Difference {
                key_values: kv,
                row_a: None,
                row_b: Some(row),
                column_name: None,
                value_a: None,
                value_b: Some(fields.join("\t")),
                difference_type: DifferenceType::BOnly,
            });
        }
    }

    // A duplicate key is ambiguous when it exists on only one side. When both
    // files contain the same duplicate key, however, compare corresponding
    // occurrences so changed cells are not silently omitted from the result.
    for (key, a_rows) in &duplicate_records_a {
        if !dup_a.is_duplicate(key) {
            continue;
        }
        let Some(b_rows) = duplicate_records_b.get(key) else {
            continue;
        };
        if !dup_b.is_duplicate(key) {
            continue;
        }
        for ((a_row, a_fields), (b_row, b_fields)) in a_rows.iter().zip(b_rows) {
            matched_records += 1;
            let mut changed: Vec<(String, String, String)> = Vec::new();
            for col in &compared_columns {
                let a_val = layout_a.get(col, a_fields).cloned().unwrap_or_default();
                let b_val = layout_b.get(col, b_fields).cloned().unwrap_or_default();
                if normalize(&a_val, settings) != normalize(&b_val, settings) {
                    changed.push((col.clone(), a_val, b_val));
                }
            }

            if changed.is_empty() {
                same_records += 1;
            } else {
                different_records += 1;
                different_cells += changed.len() as u64;
                let kv = build_key_values(key_builder.columns(), &layout_a, a_fields);
                for (col, a_val, b_val) in changed {
                    differences.push(Difference {
                        key_values: kv.clone(),
                        row_a: Some(*a_row),
                        row_b: Some(*b_row),
                        column_name: Some(col),
                        value_a: Some(a_val),
                        value_b: Some(b_val),
                        difference_type: DifferenceType::ValueChanged,
                    });
                }
            }
        }
    }

    // ---- A Only ----
    let mut a_only_records: u64 = 0;
    for (key, (a_row, a_fields)) in records.iter() {
        if dup_a.is_duplicate(key) || dup_b.is_duplicate(key) || matched_keys.contains(key) {
            continue;
        }
        a_only_records += 1;
        let kv = build_key_values(key_builder.columns(), &layout_a, a_fields);
        differences.push(Difference {
            key_values: kv,
            row_a: Some(*a_row),
            row_b: None,
            column_name: None,
            value_a: Some(a_fields.join("\t")),
            value_b: None,
            difference_type: DifferenceType::AOnly,
        });
    }

    check_cancel(cancel)?;
    let key_cols = key_builder.columns().to_vec();
    let mut result = CompareResult {
        identical: false,
        rows_a,
        rows_b,
        key_columns: key_cols.clone(),
        compared_columns,
        excluded_columns: opts.excluded_columns.clone(),
        column_a_only,
        column_b_only,
        matched_records,
        same_records,
        different_records,
        a_only_records,
        b_only_records,
        duplicate_keys_a: dup_a.duplicate_keys().len() as u64,
        duplicate_keys_b: dup_b.duplicate_keys().len() as u64,
        empty_keys_a,
        empty_keys_b,
        incomplete_keys_a,
        incomplete_keys_b,
        different_cells,
        differences,
        duplicate_details_a: dup_a.to_infos("File A", &key_cols),
        duplicate_details_b: dup_b.to_infos("File B", &key_cols),
        file_a_name: file_name(&opts.file_a_path),
        file_b_name: file_name(&opts.file_b_path),
        file_a_encoding: enc_a,
        file_b_encoding: enc_b,
        file_a_delimiter: delim_a,
        file_b_delimiter: delim_b,
        compare_mode: format_mode(opts.comparison_mode).to_string(),
        compare_time: now_string(),
        cancelled: false,
    };
    result.identical = super::summary::is_identical(&result);
    check_cancel(cancel)?;
    Ok(result)
}
