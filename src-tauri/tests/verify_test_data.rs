use std::sync::atomic::AtomicBool;

use csv_compare_lib::compare::key_comparator::compare_key_based;
use csv_compare_lib::models::compare_options::{CompareOptions, ComparisonMode};
use csv_compare_lib::models::difference::DifferenceType;
use csv_compare_lib::models::parse_options::{DelimiterOption, EncodingOption, ParseOptions};

fn td(name: &str) -> String {
    std::path::Path::new("..")
        .join("test-data")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

#[allow(clippy::too_many_arguments)]
fn opts(
    a: &str,
    b: &str,
    a_enc: EncodingOption,
    a_delim: DelimiterOption,
    b_enc: EncodingOption,
    b_delim: DelimiterOption,
    keys: Vec<&str>,
    excluded: Vec<&str>,
) -> CompareOptions {
    CompareOptions {
        file_a_path: td(a),
        file_b_path: td(b),
        file_a_parse_options: ParseOptions {
            encoding: a_enc,
            delimiter: a_delim,
        },
        file_b_parse_options: ParseOptions {
            encoding: b_enc,
            delimiter: b_delim,
        },
        comparison_mode: ComparisonMode::KeyBased,
        key_columns: keys.into_iter().map(String::from).collect(),
        excluded_columns: excluded.into_iter().map(String::from).collect(),
        trim_whitespace: false,
        ignore_case: false,
    }
}

fn run(name: &str, o: CompareOptions) -> csv_compare_lib::models::compare_result::CompareResult {
    let cancel = AtomicBool::new(false);
    let progress = |_p: u64, _t: Option<u64>, _s: &str| {};
    match compare_key_based(&o, &cancel, &progress) {
        Ok(r) => r,
        Err(e) => panic!("{name} error: {e}"),
    }
}

#[test]
fn verify_scenario_01_same() {
    let r = run(
        "s01",
        opts(
            "01_same_A.csv",
            "01_same_B.csv",
            EncodingOption::Auto,
            DelimiterOption::Auto,
            EncodingOption::Auto,
            DelimiterOption::Auto,
            vec!["客戶編號", "交易日期", "交易序號"],
            vec![],
        ),
    );
    assert!(r.identical, "s01 should be identical");
    assert_eq!(r.same_records, 3);
    assert_eq!(r.file_a_delimiter, "Comma (,)");
    assert_eq!(r.file_b_delimiter, "Pipe (|)");
    println!("s01 OK identical={} same={}", r.identical, r.same_records);
}

#[test]
fn verify_scenario_02_change() {
    let r = run(
        "s02",
        opts(
            "02_change_A.csv",
            "02_change_B.csv",
            EncodingOption::Auto,
            DelimiterOption::Comma,
            EncodingOption::Auto,
            DelimiterOption::Auto,
            vec!["客戶編號", "交易日期", "交易序號"],
            vec!["更新時間"],
        ),
    );
    assert!(
        r.file_a_encoding == "CP950" || r.file_a_encoding == "Big5",
        "A should be detected as CP950/Big5, got {}",
        r.file_a_encoding
    );
    assert_eq!(r.file_b_delimiter, "Semicolon (;)");
    assert_eq!(r.same_records, 3);
    assert_eq!(r.different_records, 1);
    assert_eq!(r.different_cells, 1);
    let vc = r
        .differences
        .iter()
        .find(|d| d.difference_type == DifferenceType::ValueChanged)
        .unwrap();
    assert_eq!(vc.column_name.as_deref(), Some("金額"));
    assert_eq!(vc.value_a.as_deref(), Some("2000"));
    assert_eq!(vc.value_b.as_deref(), Some("2500"));
    println!(
        "s02 OK A_encoding={} B_delim={} different={}",
        r.file_a_encoding, r.file_b_delimiter, r.different_records
    );
}

#[test]
fn verify_scenario_03_duplicate() {
    let r = run(
        "s03",
        opts(
            "03_dup_A.csv",
            "03_dup_B.csv",
            EncodingOption::Auto,
            DelimiterOption::Auto,
            EncodingOption::Auto,
            DelimiterOption::Auto,
            vec!["ID"],
            vec![],
        ),
    );
    assert_eq!(r.duplicate_keys_a, 1);
    assert_eq!(r.empty_keys_a, 1);
    // Key 001 is duplicated in A, so it cannot be paired with B's 001 (spec 32).
    assert_eq!(r.same_records, 2);
    assert!(!r.identical);
    println!(
        "s03 OK dupA={} emptyA={} same={}",
        r.duplicate_keys_a, r.empty_keys_a, r.same_records
    );
}

#[test]
fn verify_scenario_04_only() {
    let r = run(
        "s04",
        opts(
            "04_only_A.csv",
            "04_only_B.csv",
            EncodingOption::Auto,
            DelimiterOption::Auto,
            EncodingOption::Auto,
            DelimiterOption::Auto,
            vec!["ID"],
            vec![],
        ),
    );
    assert_eq!(r.same_records, 2);
    assert_eq!(r.a_only_records, 3);
    assert_eq!(r.b_only_records, 1);
    assert_eq!(r.column_a_only, vec!["金額"]);
    assert_eq!(r.column_b_only, vec!["日期"]);
    println!(
        "s04 OK a_only={} b_only={} colA={:?} colB={:?}",
        r.a_only_records, r.b_only_records, r.column_a_only, r.column_b_only
    );
}

#[test]
fn verify_scenario_05_quoted() {
    let r = run(
        "s05",
        opts(
            "05_quoted_A.csv",
            "05_custom_B.csv",
            EncodingOption::Auto,
            DelimiterOption::Comma,
            EncodingOption::Utf16Le,
            DelimiterOption::Custom('^'),
            vec!["編號"],
            vec![],
        ),
    );
    assert!(r.identical, "s05 should be identical");
    assert_eq!(r.same_records, 3);
    assert_eq!(r.file_b_encoding, "UTF-16 LE");
    println!(
        "s05 OK identical={} same={} B_enc={}",
        r.identical, r.same_records, r.file_b_encoding
    );
}
