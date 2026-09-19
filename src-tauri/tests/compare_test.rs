use std::fs::File;
use std::io::Write;
use std::sync::atomic::AtomicBool;

use csv_compare_lib::compare::key_comparator::compare_key_based;
use csv_compare_lib::compare::row_comparator::compare_row_by_row;
use csv_compare_lib::compare::util::is_identical;
use csv_compare_lib::excel::exporter::export_excel;
use csv_compare_lib::models::compare_options::{CompareOptions, ComparisonMode};
use csv_compare_lib::models::difference::DifferenceType;
use csv_compare_lib::models::parse_options::{DelimiterOption, EncodingOption, ParseOptions};

fn tmp_path(name: &str) -> String {
    let dir = std::env::temp_dir().join("csv_compare_test");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name).to_string_lossy().into_owned()
}

fn write_utf8(path: &str, content: &str) {
    let mut f = File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

fn write_big5(path: &str, content: &str) {
    let (bytes, _, _) = encoding_rs::BIG5.encode(content);
    let mut f = File::create(path).unwrap();
    f.write_all(&bytes).unwrap();
}

// encoding_rs has no UTF-16 encoder (per the Encoding Standard its output is UTF-8),
// so we emit UTF-16 bytes manually for test fixtures.
fn encode_utf16(s: &str, little_endian: bool) -> Vec<u8> {
    let mut out = Vec::new();
    for unit in s.encode_utf16() {
        if little_endian {
            out.extend_from_slice(&unit.to_le_bytes());
        } else {
            out.extend_from_slice(&unit.to_be_bytes());
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn opts(
    a_path: &str,
    b_path: &str,
    a_enc: EncodingOption,
    b_enc: EncodingOption,
    a_delim: DelimiterOption,
    b_delim: DelimiterOption,
    mode: ComparisonMode,
    keys: Vec<&str>,
    excluded: Vec<&str>,
) -> CompareOptions {
    CompareOptions {
        file_a_path: a_path.to_string(),
        file_b_path: b_path.to_string(),
        file_a_parse_options: ParseOptions {
            encoding: a_enc,
            delimiter: a_delim,
        },
        file_b_parse_options: ParseOptions {
            encoding: b_enc,
            delimiter: b_delim,
        },
        comparison_mode: mode,
        key_columns: keys.into_iter().map(|s| s.to_string()).collect(),
        excluded_columns: excluded.into_iter().map(|s| s.to_string()).collect(),
        trim_whitespace: false,
        ignore_case: false,
    }
}

#[test]
fn test_integration_spec_case() {
    let a = tmp_path("spec_a.csv");
    let b = tmp_path("spec_b.txt");
    write_big5(
        &a,
        "\"客戶編號\",\"交易日期\",\"交易序號\",\"金額\",\"更新時間\"\n\
         \"001\",\"2026/09/18\",\"01\",\"1000\",\"08:00\"\n\
         \"001\",\"2026/09/18\",\"02\",\"2000\",\"08:00\"\n\
         \"002\",\"2026/09/18\",\"01\",\"3000\",\"08:00\"\n",
    );
    write_utf8(
        &b,
        "\"交易日期\"|\"交易序號\"|\"客戶編號\"|\"金額\"|\"更新時間\"\n\
         \"2026/09/18\"|\"01\"|\"002\"|\"3000\"|\"09:30\"\n\
         \"2026/09/18\"|\"01\"|\"001\"|\"1000\"|\"09:30\"\n\
         \"2026/09/18\"|\"02\"|\"001\"|\"2500\"|\"09:30\"\n",
    );

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Cp950,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Pipe,
        ComparisonMode::KeyBased,
        vec!["客戶編號", "交易日期", "交易序號"],
        vec!["更新時間"],
    );

    let result = compare_key_based(&options, &cancel, &progress).unwrap();

    assert_eq!(result.rows_a, 3);
    assert_eq!(result.rows_b, 3);
    assert_eq!(result.matched_records, 3);
    assert_eq!(result.same_records, 2);
    assert_eq!(result.different_records, 1);
    assert_eq!(result.a_only_records, 0);
    assert_eq!(result.b_only_records, 0);
    assert_eq!(result.different_cells, 1);
    assert_eq!(result.duplicate_keys_a, 0);
    assert_eq!(result.duplicate_keys_b, 0);
    assert!(!result.identical);
    assert_eq!(result.file_a_encoding, "CP950");
    assert_eq!(result.file_b_delimiter, "Pipe (|)");

    let vc: Vec<_> = result
        .differences
        .iter()
        .filter(|d| d.difference_type == DifferenceType::ValueChanged)
        .collect();
    assert_eq!(vc.len(), 1);
    assert_eq!(vc[0].column_name.as_deref(), Some("金額"));
    assert_eq!(vc[0].value_a.as_deref(), Some("2000"));
    assert_eq!(vc[0].value_b.as_deref(), Some("2500"));

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_a_only_and_b_only() {
    let a = tmp_path("ab_a.csv");
    let b = tmp_path("ab_b.csv");
    write_utf8(&a, "ID,Name\n001,Wang\n002,Chen\n003,Lin\n");
    write_utf8(&b, "ID,Name\n002,Chen\n004,Zhao\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.rows_a, 3);
    assert_eq!(result.rows_b, 2);
    assert_eq!(result.a_only_records, 2); // 001, 003
    assert_eq!(result.b_only_records, 1); // 004
    assert_eq!(result.same_records, 1); // 002
    assert_eq!(result.matched_records, 1);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_duplicate_keys() {
    let a = tmp_path("dup_a.csv");
    let b = tmp_path("dup_b.csv");
    write_utf8(&a, "ID,Name\n001,Wang\n001,Chen\n002,Lin\n");
    write_utf8(&b, "ID,Name\n001,Wang\n002,Lin\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.duplicate_keys_a, 1); // key 001 duplicated
    assert_eq!(result.duplicate_details_a.len(), 1);
    assert_eq!(result.duplicate_details_a[0].count, 2);
    assert_eq!(result.duplicate_details_a[0].rows, vec![1, 2]);
    assert!(!result.identical);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_file_b_duplicate_key_is_excluded_from_all_match_statistics() {
    let a = tmp_path("dup_b_same_a.csv");
    let b = tmp_path("dup_b_same_b.csv");
    write_utf8(&a, "ID,Name\n001,Wang\n");
    write_utf8(&b, "ID,Name\n001,Wang\n001,Wang\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.matched_records, 0);
    assert_eq!(result.same_records, 0);
    assert_eq!(result.different_records, 0);
    assert_eq!(result.a_only_records, 0);
    assert_eq!(result.b_only_records, 0);
    assert_eq!(result.duplicate_keys_b, 1);
    assert_eq!(result.duplicate_details_b[0].count, 2);
    assert_eq!(result.duplicate_details_b[0].rows, vec![1, 2]);
}

#[test]
fn test_file_b_duplicate_key_never_produces_value_changed_or_b_only() {
    let a = tmp_path("dup_b_changed_a.csv");
    let b = tmp_path("dup_b_changed_b.csv");
    write_utf8(&a, "ID,Name\n001,Wang\n");
    write_utf8(&b, "ID,Name\n001,Chen\n001,Lin\n002,Zhao\n002,Li\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.matched_records, 0);
    assert_eq!(result.different_records, 0);
    assert_eq!(result.a_only_records, 0);
    assert_eq!(result.b_only_records, 0);
    assert_eq!(result.duplicate_keys_b, 2);
    assert!(!result.differences.iter().any(|d| matches!(
        d.difference_type,
        DifferenceType::ValueChanged | DifferenceType::BOnly
    )));
}

#[test]
fn test_matching_duplicate_keys_still_compare_duplicate_rows() {
    let a = tmp_path("dup_both_changed_a.csv");
    let b = tmp_path("dup_both_changed_b.csv");
    write_utf8(&a, "ID,Name\n001,Wang\n001,Chen\n002,Lin\n");
    write_utf8(&b, "ID,Name\n001,Wang\n001,Chang\n002,Lin\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.matched_records, 3);
    assert_eq!(result.same_records, 2);
    assert_eq!(result.different_records, 1);
    assert_eq!(result.different_cells, 1);
    assert!(result.differences.iter().any(|d| {
        d.difference_type == DifferenceType::ValueChanged
            && d.row_a == Some(2)
            && d.row_b == Some(2)
            && d.column_name.as_deref() == Some("Name")
            && d.value_a.as_deref() == Some("Chen")
            && d.value_b.as_deref() == Some("Chang")
    }));

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_pre_cancelled_small_comparisons_stop_immediately() {
    let a = tmp_path("cancel_a.csv");
    let b = tmp_path("cancel_b.csv");
    write_utf8(&a, "ID,Name\n001,Wang\n");
    write_utf8(&b, "ID,Name\n001,Wang\n");
    let cancel = AtomicBool::new(true);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};

    for mode in [ComparisonMode::KeyBased, ComparisonMode::RowByRow] {
        let options = opts(
            &a,
            &b,
            EncodingOption::Utf8,
            EncodingOption::Utf8,
            DelimiterOption::Comma,
            DelimiterOption::Comma,
            mode,
            if mode == ComparisonMode::KeyBased {
                vec!["ID"]
            } else {
                vec![]
            },
            vec![],
        );
        let err = if mode == ComparisonMode::KeyBased {
            compare_key_based(&options, &cancel, &progress).unwrap_err()
        } else {
            compare_row_by_row(&options, &cancel, &progress).unwrap_err()
        };
        assert!(matches!(err, csv_compare_lib::error::AppError::Cancelled));
    }
}

#[test]
fn test_low_confidence_auto_encoding_is_blocked_but_manual_encoding_works() {
    let a = tmp_path("low_confidence_a.csv");
    let b = tmp_path("low_confidence_b.csv");
    let mut uncertain = b"ID,Name\n1,".to_vec();
    uncertain.push(0x80);
    uncertain.push(b'\n');
    std::fs::write(&a, &uncertain).unwrap();
    std::fs::write(&b, &uncertain).unwrap();
    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let auto = opts(
        &a,
        &b,
        EncodingOption::Auto,
        EncodingOption::Cp950,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    let err = compare_key_based(&auto, &cancel, &progress).unwrap_err();
    assert!(err.to_string().contains("Encoding"), "{err}");

    let manual = opts(
        &a,
        &b,
        EncodingOption::Cp950,
        EncodingOption::Cp950,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    assert_eq!(
        compare_key_based(&manual, &cancel, &progress)
            .unwrap()
            .same_records,
        1
    );
}

#[test]
fn test_empty_incomplete_keys() {
    let a = tmp_path("empty_a.csv");
    let b = tmp_path("empty_b.csv");
    write_utf8(&a, "ID,Name\n,Wang\n001,Chen\n");
    write_utf8(&b, "ID,Name\n001,Chen\n002,Lin\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.empty_keys_a, 1);
    assert_eq!(result.rows_a, 2);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_row_by_row() {
    let a = tmp_path("row_a.csv");
    let b = tmp_path("row_b.csv");
    write_utf8(
        &a,
        "ID,Name,Amount\n001,Wang,1000\n002,Chen,2000\n003,Lin,3000\n",
    );
    write_utf8(
        &b,
        "Amount,Name,ID\n3000,Lin,003\n2000,Chen,002\n1000,Wang,001\n",
    );

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::RowByRow,
        vec![],
        vec![],
    );

    let result = compare_row_by_row(&options, &cancel, &progress).unwrap();
    // Row-by-row pairs by position; B is reversed so only row 2 matches by name-mapping.
    assert_eq!(result.same_records, 1);
    assert_eq!(result.different_records, 2);
    assert!(!result.identical);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_value_changed_and_excluded() {
    let a = tmp_path("val_a.csv");
    let b = tmp_path("val_b.csv");
    write_utf8(&a, "ID,Amount,Stamp\n001,1000,08:00\n002,2000,08:00\n");
    write_utf8(&b, "ID,Amount,Stamp\n002,2000,09:00\n001,1200,09:00\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec!["Stamp"],
    );

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.same_records, 1); // 002, Stamp excluded
    assert_eq!(result.different_records, 1); // 001 Amount changed
    assert_eq!(result.different_cells, 1);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_column_order_and_quoted() {
    let a = tmp_path("q_a.csv");
    let b = tmp_path("q_b.csv");
    write_utf8(
        &a,
        "\"ID\",\"Name\",\"Address\"\n\"001\",\"Wang\",\"台北市,中正區\"\n\"002\",\"Chen\",\"高雄市\"\n",
    );
    write_utf8(
        &b,
        "\"Address\"|\"Name\"|\"ID\"\n\"高雄市\"|\"Chen\"|\"002\"\n\"台北市,中正區\"|\"Wang\"|\"001\"\n",
    );

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Pipe,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.same_records, 2);
    assert!(result.identical);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_excel_export() {
    let a = tmp_path("ex_a.csv");
    let b = tmp_path("ex_b.csv");
    write_utf8(&a, "ID,Name\n001,Wang\n002,Chen\n");
    write_utf8(&b, "ID,Name\n001,Wang\n002,Chun\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    let result = compare_key_based(&options, &cancel, &progress).unwrap();

    let out = tmp_path("result.xlsx");
    export_excel(&result, &out).unwrap();
    assert!(std::path::Path::new(&out).exists());
    let meta = std::fs::metadata(&out).unwrap();
    assert!(meta.len() > 1000);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn test_is_identical_helper() {
    let mut r = csv_compare_lib::models::compare_result::CompareResult::default();
    assert!(is_identical(&r));
    r.a_only_records = 1;
    assert!(!is_identical(&r));
}

#[test]
fn test_invalid_record_count() {
    let a = tmp_path("ir_a.csv");
    let b = tmp_path("ir_b.csv");
    write_utf8(&a, "ID,Name\n001,Wang\n002\n"); // 1 field but header has 2
    write_utf8(&b, "ID,Name\n001,Wang\n002,Chen\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    let err = compare_key_based(&options, &cancel, &progress).unwrap_err();
    assert!(err.to_string().contains("檔案格式異常"), "{err}");

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_duplicate_header_error() {
    let a = tmp_path("dh_a.csv");
    let b = tmp_path("dh_b.csv");
    write_utf8(&a, "ID,Name,ID\n001,Wang,9\n");
    write_utf8(&b, "ID,Name\n001,Wang\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    let err = compare_key_based(&options, &cancel, &progress).unwrap_err();
    assert!(err.to_string().contains("重複欄位名稱"), "{err}");

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_missing_key_column_error() {
    let a = tmp_path("mk_a.csv");
    let b = tmp_path("mk_b.csv");
    write_utf8(&a, "ID,Name\n001,Wang\n");
    write_utf8(&b, "ID,Name\n001,Wang\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["不存在欄位"],
        vec![],
    );
    let err = compare_key_based(&options, &cancel, &progress).unwrap_err();
    assert!(err.to_string().contains("Key 欄位不存在"), "{err}");

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_trim_and_ignore_case() {
    let a = tmp_path("tc_a.csv");
    let b = tmp_path("tc_b.csv");
    write_utf8(&a, "ID,Name\n001,  Wang  \n002,ABC\n");
    write_utf8(&b, "ID,Name\n001,Wang\n002,abc\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let mut options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    options.trim_whitespace = true;
    options.ignore_case = true;

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.same_records, 2);
    assert!(result.identical);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_key_strict_string() {
    let a = tmp_path("ks_a.csv");
    let b = tmp_path("ks_b.csv");
    // 001 vs 1 must NOT match; ABC vs abc must NOT match as keys.
    write_utf8(&a, "ID,Name\n001,Wang\nABC,Chen\n");
    write_utf8(&b, "ID,Name\n1,Wang\nabc,Chen\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let mut options = opts(
        &a,
        &b,
        EncodingOption::Utf8,
        EncodingOption::Utf8,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    options.ignore_case = true; // keys must still match strictly

    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.same_records, 0);
    assert_eq!(result.a_only_records, 2);
    assert_eq!(result.b_only_records, 2);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_utf16_files() {
    let a = tmp_path("u16_a.csv");
    let b = tmp_path("u16_b.csv");
    std::fs::write(&a, encode_utf16("ID,Name\n001,Wang\n", true)).unwrap();
    std::fs::write(&b, encode_utf16("Name,ID\nWang,001\n", false)).unwrap();

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Utf16Le,
        EncodingOption::Utf16Be,
        DelimiterOption::Comma,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.same_records, 1);
    assert!(result.identical);

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_utf8_bom() {
    let a = tmp_path("bom_a.csv");
    let b = tmp_path("bom_b.csv");
    let mut content = vec![0xEF, 0xBB, 0xBF];
    content.extend_from_slice(b"ID,Name\n001,Wang\n");
    std::fs::write(&a, &content).unwrap();
    write_utf8(&b, "ID,Name\n001,Wang\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Auto,
        EncodingOption::Utf8,
        DelimiterOption::Auto,
        DelimiterOption::Comma,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.same_records, 1);
    assert!(result.identical);
    assert_eq!(result.file_a_encoding, "UTF-8");

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

#[test]
fn test_auto_detection() {
    let a = tmp_path("auto_a.csv");
    let b = tmp_path("auto_b.txt");
    write_utf8(&a, "ID;Name;City\n001;Wang;台北\n");
    write_utf8(&b, "Name|ID|City\nWang|001|台北\n");

    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    let options = opts(
        &a,
        &b,
        EncodingOption::Auto,
        EncodingOption::Auto,
        DelimiterOption::Auto,
        DelimiterOption::Auto,
        ComparisonMode::KeyBased,
        vec!["ID"],
        vec![],
    );
    let result = compare_key_based(&options, &cancel, &progress).unwrap();
    assert_eq!(result.same_records, 1);
    assert!(result.identical);
    assert_eq!(result.file_a_delimiter, "Semicolon (;)");
    assert_eq!(result.file_b_delimiter, "Pipe (|)");

    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}
