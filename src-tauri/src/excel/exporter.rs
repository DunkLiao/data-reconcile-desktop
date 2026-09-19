use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook};
use std::collections::BTreeMap;

use crate::error::AppError;
use crate::models::compare_result::CompareResult;

fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xD9E1F2))
        .set_align(FormatAlign::Center)
        .set_border(rust_xlsxwriter::FormatBorder::Thin)
}

fn text_format() -> Format {
    Format::new().set_border(rust_xlsxwriter::FormatBorder::Thin)
}

fn ensure_parent(path: &str) -> Result<(), AppError> {
    if let Some(parent) = std::path::Path::new(path).parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|_| AppError::ExportError)?;
        }
    }
    Ok(())
}

pub fn export_excel(result: &CompareResult, path: &str) -> Result<(), AppError> {
    ensure_parent(path)?;

    let mut workbook = Workbook::new();

    write_summary(&mut workbook, result)?;
    write_differences(&mut workbook, result)?;
    write_column_differences(&mut workbook, result)?;
    write_excluded_columns(&mut workbook, result)?;
    write_duplicate_keys(&mut workbook, result)?;

    workbook.save(path).map_err(|e| {
        log::error!("Excel save failed: {e}");
        AppError::ExportError
    })?;

    Ok(())
}

fn write_summary(workbook: &mut Workbook, result: &CompareResult) -> Result<(), AppError> {
    let ws = workbook
        .add_worksheet()
        .set_name("Summary")
        .map_err(|_| AppError::ExportError)?;

    let title_fmt = Format::new().set_bold().set_font_size(14);
    let item_fmt = Format::new().set_bold();
    let label_fmt = text_format();

    ws.write_string_with_format(0, 0, "CSV Compare 結果摘要", &title_fmt)
        .map_err(|_| AppError::ExportError)?;

    let rows: Vec<(&str, String)> = vec![
        ("File A", result.file_a_name.clone()),
        ("File A Encoding", result.file_a_encoding.clone()),
        ("File A Delimiter", result.file_a_delimiter.clone()),
        ("File B", result.file_b_name.clone()),
        ("File B Encoding", result.file_b_encoding.clone()),
        ("File B Delimiter", result.file_b_delimiter.clone()),
        ("Compare Mode", result.compare_mode.clone()),
        (
            "Key Columns",
            if result.key_columns.is_empty() {
                "—".to_string()
            } else {
                result.key_columns.join(" / ")
            },
        ),
        ("Rows A", format_number(result.rows_a)),
        ("Rows B", format_number(result.rows_b)),
        ("Matched Records", format_number(result.matched_records)),
        ("Same Records", format_number(result.same_records)),
        ("Different Records", format_number(result.different_records)),
        ("A Only", format_number(result.a_only_records)),
        ("B Only", format_number(result.b_only_records)),
        ("Different Cells", format_number(result.different_cells)),
        ("Duplicate Keys A", format_number(result.duplicate_keys_a)),
        ("Duplicate Keys B", format_number(result.duplicate_keys_b)),
        ("Empty Keys A", format_number(result.empty_keys_a)),
        ("Empty Keys B", format_number(result.empty_keys_b)),
        ("Incomplete Keys A", format_number(result.incomplete_keys_a)),
        ("Incomplete Keys B", format_number(result.incomplete_keys_b)),
        (
            "Result",
            if result.identical {
                "IDENTICAL".to_string()
            } else {
                "DIFFERENT".to_string()
            },
        ),
        ("Compare Time", result.compare_time.clone()),
    ];

    for (r, (k, v)) in (2_u32..).zip(rows) {
        ws.write_string_with_format(r, 0, k, &item_fmt)
            .map_err(|_| AppError::ExportError)?;
        ws.write_string_with_format(r, 1, &v, &label_fmt)
            .map_err(|_| AppError::ExportError)?;
    }

    ws.set_column_width(0, 24)
        .map_err(|_| AppError::ExportError)?;
    ws.set_column_width(1, 80)
        .map_err(|_| AppError::ExportError)?;
    Ok(())
}

fn write_differences(workbook: &mut Workbook, result: &CompareResult) -> Result<(), AppError> {
    let ws = workbook
        .add_worksheet()
        .set_name("Differences")
        .map_err(|_| AppError::ExportError)?;

    let header: Vec<String> = result
        .key_columns
        .iter()
        .cloned()
        .chain([
            "Row A".to_string(),
            "Row B".to_string(),
            "Column".to_string(),
            "File A Value".to_string(),
            "File B Value".to_string(),
            "Type".to_string(),
        ])
        .collect();

    for (i, h) in header.iter().enumerate() {
        ws.write_string_with_format(0, i as u16, h, &header_format())
            .map_err(|_| AppError::ExportError)?;
    }

    let key_cols = result.key_columns.len();
    let mut r: u32 = 1;
    for diff in &result.differences {
        // key values split into their own columns
        let mut key_by_col: BTreeMap<&str, &String> = BTreeMap::new();
        for kv in &diff.key_values {
            key_by_col.insert(kv.column.as_str(), &kv.value);
        }
        let mut c: u16 = 0;
        for col in &result.key_columns {
            let val = key_by_col
                .get(col.as_str())
                .map(|s| s.as_str())
                .unwrap_or("");
            ws.write_string_with_format(r, c, val, &text_format())
                .map_err(|_| AppError::ExportError)?;
            c += 1;
        }

        let row_a = diff.row_a.map(|v| v.to_string()).unwrap_or_default();
        let row_b = diff.row_b.map(|v| v.to_string()).unwrap_or_default();
        let column = diff.column_name.clone().unwrap_or_default();
        let val_a = diff.value_a.clone().unwrap_or_default();
        let val_b = diff.value_b.clone().unwrap_or_default();
        let ty = diff.difference_type.label().to_string();

        let mut rest: Vec<(&str, &String)> = Vec::new();
        let row_a_s = row_a;
        let row_b_s = row_b;
        rest.push(("", &row_a_s));
        rest.push(("", &row_b_s));
        rest.push(("", &column));
        rest.push(("", &val_a));
        rest.push(("", &val_b));
        rest.push(("", &ty));

        for (_, v) in rest {
            ws.write_string_with_format(r, c, v.as_str(), &text_format())
                .map_err(|_| AppError::ExportError)?;
            c += 1;
        }
        r += 1;
    }

    if result.differences.is_empty() {
        ws.write_string_with_format(1, 0, "No differences found.", &text_format())
            .map_err(|_| AppError::ExportError)?;
        r = 2;
    }

    for (i, h) in header.iter().enumerate() {
        ws.set_column_width(
            i as u16,
            if h == "File A Value" || h == "File B Value" {
                40
            } else {
                16
            },
        )
        .map_err(|_| AppError::ExportError)?;
    }
    ws.set_freeze_panes(1, 0)
        .map_err(|_| AppError::ExportError)?;
    ws.autofilter(0, 0, r.saturating_sub(1), (key_cols + 6 - 1) as u16)
        .map_err(|_| AppError::ExportError)?;
    Ok(())
}

fn write_column_differences(
    workbook: &mut Workbook,
    result: &CompareResult,
) -> Result<(), AppError> {
    let ws = workbook
        .add_worksheet()
        .set_name("Column_Differences")
        .map_err(|_| AppError::ExportError)?;

    let header = ["Column", "File A", "File B", "Status"];
    for (i, h) in header.iter().enumerate() {
        ws.write_string_with_format(0, i as u16, *h, &header_format())
            .map_err(|_| AppError::ExportError)?;
    }

    let mut rows: Vec<(String, &'static str, &'static str)> = Vec::new();
    for col in &result.compared_columns {
        rows.push((col.clone(), "Yes", "Yes"));
    }
    for col in &result.column_a_only {
        rows.push((col.clone(), "Yes", "No"));
    }
    for col in &result.column_b_only {
        rows.push((col.clone(), "No", "Yes"));
    }
    for col in &result.excluded_columns {
        rows.push((col.clone(), "-", "-"));
    }

    let mut r: u32 = 1;
    for (col, in_a, in_b) in rows {
        ws.write_string_with_format(r, 0, &col, &text_format())
            .map_err(|_| AppError::ExportError)?;
        ws.write_string_with_format(r, 1, in_a, &text_format())
            .map_err(|_| AppError::ExportError)?;
        ws.write_string_with_format(r, 2, in_b, &text_format())
            .map_err(|_| AppError::ExportError)?;
        let status = if result.excluded_columns.iter().any(|e| e == &col) {
            "EXCLUDED"
        } else if result.column_a_only.iter().any(|c| c == &col) {
            "A_ONLY"
        } else if result.column_b_only.iter().any(|c| c == &col) {
            "B_ONLY"
        } else {
            "SAME"
        };
        ws.write_string_with_format(r, 3, status, &text_format())
            .map_err(|_| AppError::ExportError)?;
        r += 1;
    }

    for i in 0..4 {
        ws.set_column_width(i, 20)
            .map_err(|_| AppError::ExportError)?;
    }
    ws.set_freeze_panes(1, 0)
        .map_err(|_| AppError::ExportError)?;
    ws.autofilter(0, 0, r.saturating_sub(1), 3)
        .map_err(|_| AppError::ExportError)?;
    Ok(())
}

fn write_excluded_columns(workbook: &mut Workbook, result: &CompareResult) -> Result<(), AppError> {
    let ws = workbook
        .add_worksheet()
        .set_name("Excluded_Columns")
        .map_err(|_| AppError::ExportError)?;

    ws.write_string_with_format(0, 0, "Column", &header_format())
        .map_err(|_| AppError::ExportError)?;

    if result.excluded_columns.is_empty() {
        ws.write_string_with_format(1, 0, "No excluded columns.", &text_format())
            .map_err(|_| AppError::ExportError)?;
    } else {
        for (i, col) in result.excluded_columns.iter().enumerate() {
            ws.write_string_with_format(i as u32 + 1, 0, col, &text_format())
                .map_err(|_| AppError::ExportError)?;
        }
    }
    ws.set_column_width(0, 24)
        .map_err(|_| AppError::ExportError)?;
    Ok(())
}

fn write_duplicate_keys(workbook: &mut Workbook, result: &CompareResult) -> Result<(), AppError> {
    let ws = workbook
        .add_worksheet()
        .set_name("Duplicate_Keys")
        .map_err(|_| AppError::ExportError)?;

    let header: Vec<String> = ["Source".to_string()]
        .into_iter()
        .chain(result.key_columns.iter().cloned())
        .chain(["Count".to_string(), "Rows".to_string()])
        .collect();

    if result.duplicate_details_a.is_empty() && result.duplicate_details_b.is_empty() {
        ws.write_string_with_format(0, 0, "No duplicate keys found.", &text_format())
            .map_err(|_| AppError::ExportError)?;
        ws.set_column_width(0, 30)
            .map_err(|_| AppError::ExportError)?;
        return Ok(());
    }

    for (i, h) in header.iter().enumerate() {
        ws.write_string_with_format(0, i as u16, h, &header_format())
            .map_err(|_| AppError::ExportError)?;
    }

    let key_cols = result.key_columns.len();
    for (r, detail) in (1_u32..).zip(
        result
            .duplicate_details_a
            .iter()
            .chain(result.duplicate_details_b.iter()),
    ) {
        ws.write_string_with_format(r, 0, &detail.source, &text_format())
            .map_err(|_| AppError::ExportError)?;

        let mut by_col: BTreeMap<&str, &String> = BTreeMap::new();
        for kv in &detail.key_values {
            by_col.insert(kv.column.as_str(), &kv.value);
        }
        for (i, col) in result.key_columns.iter().enumerate() {
            let val = by_col.get(col.as_str()).map(|s| s.as_str()).unwrap_or("");
            ws.write_string_with_format(r, i as u16 + 1, val, &text_format())
                .map_err(|_| AppError::ExportError)?;
        }

        let count_col = key_cols + 1;
        let rows_col = key_cols + 2;
        ws.write_string_with_format(
            r,
            count_col as u16,
            format_number(detail.count),
            &text_format(),
        )
        .map_err(|_| AppError::ExportError)?;
        ws.write_string_with_format(
            r,
            rows_col as u16,
            detail
                .rows
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            &text_format(),
        )
        .map_err(|_| AppError::ExportError)?;
    }

    for (i, h) in header.iter().enumerate() {
        ws.set_column_width(i as u16, if h == "Rows" { 60 } else { 20 })
            .map_err(|_| AppError::ExportError)?;
    }
    ws.set_freeze_panes(1, 0)
        .map_err(|_| AppError::ExportError)?;
    Ok(())
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*c);
    }
    out
}
