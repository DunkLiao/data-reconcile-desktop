use std::sync::atomic::AtomicBool;

use crate::error::AppError;
use crate::models::compare_options::CompareOptions;
use crate::models::compare_result::CompareResult;
use crate::models::difference::DifferenceType;
use crate::parser::decoder::open_delimited_reader;
use crate::parser::delimited_parser::check_duplicate_headers;
use crate::parser::reader::file_name;

use super::difference::{column_differences, format_mode, values_equal, ColumnLayout};
use super::key_builder::settings_from;
use super::util::{check_cancel, now_string, ProgressFn};

pub fn compare_row_by_row(
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

    let (reader_b, enc_b, delim_b) = open_delimited_reader(
        &opts.file_b_path,
        opts.file_b_parse_options.encoding,
        opts.file_b_parse_options.delimiter,
        true,
    )?;
    let layout_b = ColumnLayout::new(reader_b.headers().to_vec());
    check_duplicate_headers(&layout_b.headers)?;

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

    let mut differences = col_diffs;
    let mut rows_a: u64 = 0;
    let mut rows_b: u64 = 0;
    let mut matched_records: u64 = 0;
    let mut same_records: u64 = 0;
    let mut different_records: u64 = 0;
    let mut different_cells: u64 = 0;
    let mut a_only_records: u64 = 0;
    let mut b_only_records: u64 = 0;

    let mut reader_a = reader_a;
    let mut reader_b = reader_b;
    let mut processed: u64 = 0;

    loop {
        check_cancel(cancel)?;
        processed += 1;
        if processed.is_multiple_of(2000) {
            progress(processed, None, "依資料列順序比對");
        }

        let a = reader_a.next_record()?;
        let b = reader_b.next_record()?;
        match (a, b) {
            (Some((ra, fa)), Some((rb, fb))) => {
                rows_a += 1;
                rows_b += 1;
                matched_records += 1;

                let mut changed: Vec<(String, String, String)> = Vec::new();
                for col in &compared_columns {
                    let av = layout_a.get(col, &fa).cloned().unwrap_or_default();
                    let bv = layout_b.get(col, &fb).cloned().unwrap_or_default();
                    if !values_equal(&av, &bv, settings) {
                        changed.push((col.clone(), av, bv));
                    }
                }
                if changed.is_empty() {
                    same_records += 1;
                } else {
                    different_records += 1;
                    different_cells += changed.len() as u64;
                    for (col, av, bv) in changed {
                        differences.push(crate::models::difference::Difference {
                            key_values: vec![],
                            row_a: Some(ra),
                            row_b: Some(rb),
                            column_name: Some(col),
                            value_a: Some(av),
                            value_b: Some(bv),
                            difference_type: DifferenceType::ValueChanged,
                        });
                    }
                }
            }
            (Some((ra, fa)), None) => {
                rows_a += 1;
                a_only_records += 1;
                differences.push(crate::models::difference::Difference {
                    key_values: vec![],
                    row_a: Some(ra),
                    row_b: None,
                    column_name: None,
                    value_a: Some(fa.join("\t")),
                    value_b: None,
                    difference_type: DifferenceType::AOnly,
                });
            }
            (None, Some((rb, fb))) => {
                rows_b += 1;
                b_only_records += 1;
                differences.push(crate::models::difference::Difference {
                    key_values: vec![],
                    row_a: None,
                    row_b: Some(rb),
                    column_name: None,
                    value_a: None,
                    value_b: Some(fb.join("\t")),
                    difference_type: DifferenceType::BOnly,
                });
            }
            (None, None) => break,
        }
    }

    check_cancel(cancel)?;
    let mut result = CompareResult {
        identical: false,
        rows_a,
        rows_b,
        key_columns: vec![],
        compared_columns,
        excluded_columns: opts.excluded_columns.clone(),
        column_a_only,
        column_b_only,
        matched_records,
        same_records,
        different_records,
        a_only_records,
        b_only_records,
        duplicate_keys_a: 0,
        duplicate_keys_b: 0,
        empty_keys_a: 0,
        empty_keys_b: 0,
        incomplete_keys_a: 0,
        incomplete_keys_b: 0,
        different_cells,
        differences,
        duplicate_details_a: vec![],
        duplicate_details_b: vec![],
        file_a_name: file_name(&opts.file_a_path),
        file_b_name: file_name(&opts.file_b_path),
        file_a_encoding: enc_a,
        file_b_encoding: enc_b,
        file_a_delimiter: delim_a,
        file_b_delimiter: delim_b,
        compare_mode: format_mode(opts.comparison_mode).to_string(),
        compare_time: now_string(),
        cancelled: false,
        numeric_tolerance: opts.numeric_tolerance,
    };
    result.identical = super::summary::is_identical(&result);
    check_cancel(cancel)?;
    Ok(result)
}
