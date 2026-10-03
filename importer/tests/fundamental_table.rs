use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use rust_decimal::Decimal;
use serde_json::{Value, json};

fn company_fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/fundamental_company.json")).unwrap()
}

fn bank_fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/fundamental_bank.json")).unwrap()
}

fn write_file(root: &Path, relative: &str, body: &[u8]) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

fn write_snapshot(root: &Path, relative: &str, snapshot: &Value) -> PathBuf {
    write_file(root, relative, &serde_json::to_vec(snapshot).unwrap())
}

fn run_table(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .arg("fundamental-table")
        .arg(directory)
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .env("ALL_PROXY", "http://127.0.0.1:1")
        .output()
        .unwrap()
}

fn rows(result: &Output) -> Vec<Value> {
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}

fn assert_ratio(value: &Value, expected: &str) {
    assert_eq!(
        Decimal::from_str_exact(value.as_str().unwrap()).unwrap(),
        Decimal::from_str_exact(expected).unwrap()
    );
}

#[test]
fn recursively_selects_latest_snapshots_and_keeps_a_bank_with_null_ratios() {
    for (new_directory, old_directory) in [("a-new", "z-old"), ("z-new", "a-old")] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let mut old = company_fixture();
        old["fundamentals"]["net_income_parent"] = json!("250.00");
        old["fundamentals"]["report_period"] = json!("2025");
        old["fundamentals"]["currency"] = json!("EUR");
        old["fundamentals"]["unit_multiplier"] = json!(1);
        old["fundamentals"]["consolidated"] = json!(false);
        let old_path = write_snapshot(root, &format!("{old_directory}/fundamentals.json"), &old);
        let mut latest = company_fixture();
        latest["fetched_at"] = json!("2026-10-02T18:30:00+01:00");
        // Do not recover a missing value from an older report or use provider ratios.
        latest["fundamentals"]["operating_profit"] = Value::Null;
        latest["fundamentals"]["reported_roe"] = json!("unused provider ratio");
        latest["fundamentals"]["reported_roa"] = json!("unused provider ratio");
        let latest_path =
            write_snapshot(root, &format!("{new_directory}/fundamentals.json"), &latest);
        let bank_path = write_snapshot(root, "nested/bank/fundamentals.json", &bank_fixture());
        let result = run_table(root);
        let rows = rows(&result);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["code"], "KGHM");
        assert_eq!(rows[1]["code"], "PKOBP");
        assert_eq!(rows[0]["report_period"], "I-II kw. 2026");
        assert_eq!(rows[0]["currency"], "PLN");
        assert_eq!(rows[0]["consolidated"], true);
        assert_eq!(rows[0]["unit_multiplier"], 1000);
        assert_eq!(rows[0]["source_file"], json!(latest_path));
        assert_ratio(&rows[0]["net_margin"], "0.1");
        assert!(rows[0]["operating_margin"].is_null());
        assert_ratio(&rows[0]["equity_ratio"], "0.4");
        assert_ratio(&rows[0]["debt_to_equity"], "0.75");
        assert_ratio(&rows[0]["ocf_to_net_income"], "1.5");
        assert_eq!(rows[1]["source_file"], json!(bank_path));
        assert_eq!(rows[1]["consolidated"], false);
        assert!(rows[1]["net_margin"].is_null());
        assert!(rows[1]["operating_margin"].is_null());
        assert!(rows[1]["debt_to_equity"].is_null());
        assert_ratio(&rows[1]["equity_ratio"], "0.1");
        assert_ratio(&rows[1]["ocf_to_net_income"], "-0.5");
        for row in &rows {
            let fields: Vec<_> = row
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect();
            assert_eq!(
                fields,
                [
                    "code",
                    "consolidated",
                    "currency",
                    "debt_to_equity",
                    "equity_ratio",
                    "net_margin",
                    "ocf_to_net_income",
                    "operating_margin",
                    "report_period",
                    "source_file",
                    "unit_multiplier"
                ]
            );
        }
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains(&old_path.display().to_string()));
        assert!(stderr.contains("duplicate snapshot for KGHM"));
        assert!(stderr.contains("Fundamental ratios for 2 companies; skipped 1 paths"));
        assert!(stderr.contains(
            "Null ratios: net_margin=1, operating_margin=2, equity_ratio=0, debt_to_equity=1, ocf_to_net_income=0"
        ));
        assert!(!root.join("data").exists());
    }
}

#[test]
fn bad_files_are_reported_without_aborting_valid_companies() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_snapshot(root, "valid/fundamentals.json", &company_fixture());
    let mut skipped = vec![write_file(root, "broken/fundamentals.json", b"{")];
    let mut missing = company_fixture();
    missing.as_object_mut().unwrap().remove("instrument");
    skipped.push(write_snapshot(root, "missing/fundamentals.json", &missing));
    missing["instrument"] = Value::Null;
    skipped.push(write_snapshot(
        root,
        "null-instrument/fundamentals.json",
        &missing,
    ));
    missing["instrument"] = json!({"code": "  "});
    skipped.push(write_snapshot(
        root,
        "blank-code/fundamentals.json",
        &missing,
    ));
    missing["instrument"] = json!({"name": "No code"});
    skipped.push(write_snapshot(root, "no-code/fundamentals.json", &missing));
    let mut bad_amount = company_fixture();
    bad_amount["instrument"]["code"] = json!("BADAMOUNT");
    bad_amount["fundamentals"]["revenue"] = json!("not a decimal");
    skipped.push(write_snapshot(
        root,
        "bad-amount/fundamentals.json",
        &bad_amount,
    ));
    let mut overflow = company_fixture();
    overflow["instrument"]["code"] = json!("OVERFLOW");
    overflow["fundamentals"]["assets"] = json!(Decimal::MAX.to_string());
    skipped.push(write_snapshot(
        root,
        "overflow/fundamentals.json",
        &overflow,
    ));
    write_file(root, "valid/raw.json", b"not JSON");
    write_file(root, "valid/history.json", b"not JSON");
    let result = run_table(root);
    let rows = rows(&result);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["code"], "KGHM");
    assert_ratio(&rows[0]["net_margin"], "0.1");
    let stderr = String::from_utf8_lossy(&result.stderr);
    for path in &skipped {
        assert!(stderr.contains(&path.display().to_string()), "{stderr}");
    }
    assert!(stderr.contains("invalid fundamentals JSON"));
    assert!(stderr.contains("missing company code"));
    assert!(stderr.contains("exceeds Decimal precision/range"));
    assert!(stderr.contains("Fundamental ratios for 1 companies; skipped 7 paths"));
    assert!(stderr.contains(
        "Null ratios: net_margin=0, operating_margin=0, equity_ratio=0, debt_to_equity=0, ocf_to_net_income=0"
    ));
    assert!(!stderr.contains("raw.json"));
    assert!(!stderr.contains("history.json"));
}

#[test]
fn zero_companies_produces_an_error_and_empty_stdout() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let empty = run_table(root);
    assert!(!empty.status.success());
    assert!(empty.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&empty.stderr);
    assert!(stderr.contains("Fundamental ratios for 0 companies"));
    assert!(stderr.contains("no valid companies found"));

    let bad = write_file(root, "bad/fundamentals.json", b"not JSON");
    let invalid = run_table(root);
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&invalid.stderr);
    assert!(stderr.contains(&bad.display().to_string()));
    assert!(stderr.contains("no valid companies found"));

    for path in [root.join("does-not-exist"), bad] {
        let invalid_directory = run_table(&path);
        assert!(!invalid_directory.status.success());
        assert!(invalid_directory.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&invalid_directory.stderr)
                .contains("reading fundamentals under")
        );
    }
}
