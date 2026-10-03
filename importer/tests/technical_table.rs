use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::{Value, json};

const AS_OF: &str = "2026-09-19";

fn history(code: &str, september_close: &str) -> Value {
    json!({
        "instrument": {"code": code, "name": code, "isin": "fixture-isin"},
        "source": "yahoo_finance",
        "symbol": format!("{code}.WA"),
        "currency": "PLN",
        "fetched_at": "2026-09-20T17:00:00Z",
        "candles": [
            {"date": "2025-08-29", "close": 100, "adjusted_close": "1"},
            {"date": "2025-09-30", "close": september_close, "adjusted_close": "999"},
            {"date": "2026-09-18", "close": "100", "adjusted_close": "1"},
            {"date": AS_OF, "close": 0}
        ],
        "skipped_candles": []
    })
}

fn write_file(root: &Path, relative: &str, body: &[u8]) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

fn write_history(root: &Path, relative: &str, history: &Value) -> PathBuf {
    write_file(root, relative, &serde_json::to_vec(history).unwrap())
}

fn run_table(directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .args(["technical-table", "--as-of", AS_OF])
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

#[test]
fn ranks_two_companies_using_the_latest_fetch_regardless_of_path_order() {
    for (new_directory, old_directory) in [("a-new", "z-old"), ("z-new", "a-old")] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let old = write_history(
            root,
            &format!("{old_directory}/history.json"),
            &history("KGHM", "103"),
        );
        let mut latest = history("KGHM", "97");
        // Compare actual instants, including timezone offsets, rather than strings.
        latest["fetched_at"] = json!("2026-09-20T18:30:00+01:00");
        let new = write_history(root, &format!("{new_directory}/history.json"), &latest);
        let bank = write_history(root, "nested/bank/history.json", &history("PKOBP", "101"));
        let result = run_table(root);
        let rows = rows(&result);
        assert_eq!(
            rows,
            vec![
                json!({
                    "code": "PKOBP", "as_of": AS_OF,
                    "seasonality_points": 0, "trend_points": 0, "technical_score": 0,
                    "r": "0.01", "years": 1, "source_file": bank
                }),
                json!({
                    "code": "KGHM", "as_of": AS_OF,
                    "seasonality_points": -1, "trend_points": 0, "technical_score": -1,
                    "r": "-0.03", "years": 1, "source_file": new
                })
            ]
        );
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains(&old.display().to_string()));
        assert!(stderr.contains("duplicate history for KGHM"));
        assert!(
            stderr.contains("Ranked 2 companies; 0 with null technical_score; skipped 1 paths")
        );
        assert!(!root.join("data").exists());
    }
}

#[test]
fn sorts_scores_descending_with_company_ties_and_null_last() {
    let directory = tempfile::tempdir().unwrap();
    for (code, close) in [
        ("ZZZ", "103"),
        ("AAA", "103"),
        ("SIDE", "101"),
        ("NEG", "97"),
    ] {
        write_history(
            directory.path(),
            &format!("{code}/history.json"),
            &history(code, close),
        );
    }
    let mut missing = history("000NULL", "103");
    missing["candles"] = json!([{"date": "2026-09-18", "close": "100"}]);
    write_history(directory.path(), "missing/history.json", &missing);
    let result = run_table(directory.path());
    let rows = rows(&result);
    let codes: Vec<_> = rows
        .iter()
        .map(|row| row["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["AAA", "ZZZ", "SIDE", "NEG", "000NULL"]);
    assert!(rows[4]["seasonality_points"].is_null());
    assert!(rows[4]["technical_score"].is_null());
    assert!(rows[4]["r"].is_null());
    assert_eq!(rows[4]["years"], 0);
    assert_eq!(rows[4]["trend_points"], 0);
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("Ranked 5 companies; 1 with null technical_score; skipped 0 paths")
    );
}

#[test]
fn bad_json_missing_companies_and_score_errors_do_not_abort_the_table() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_history(root, "valid/history.json", &history("KGHM", "103"));
    let mut skipped = vec![write_file(root, "broken/history.json", b"{")];
    let mut missing = history("MISSING", "103");
    missing.as_object_mut().unwrap().remove("instrument");
    skipped.push(write_history(root, "missing/history.json", &missing));
    missing["instrument"] = Value::Null;
    skipped.push(write_history(
        root,
        "null-instrument/history.json",
        &missing,
    ));
    missing["instrument"] = json!({"code": "  "});
    skipped.push(write_history(root, "blank-code/history.json", &missing));
    let mut invalid_date = history("BADTIME", "103");
    invalid_date["fetched_at"] = json!("not a timestamp");
    skipped.push(write_history(
        root,
        "bad-timestamp/history.json",
        &invalid_date,
    ));
    for (code, candles) in [
        (
            "DUPLICATE",
            json!([
                {"date": "2026-09-18", "close": "100"},
                {"date": "2026-09-18", "close": "101"}
            ]),
        ),
        ("ZERO", json!([{"date": "2026-09-18", "close": "0"}])),
        ("NEGATIVE", json!([{"date": "2026-09-18", "close": "-1"}])),
    ] {
        let mut invalid = history(code, "103");
        invalid["candles"] = candles;
        skipped.push(write_history(
            root,
            &format!("{code}/history.json"),
            &invalid,
        ));
    }
    // Discovery must ignore unrelated archive files.
    write_file(root, "valid/raw.json", b"not JSON");
    let result = run_table(root);
    let rows = rows(&result);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["code"], "KGHM");
    assert_eq!(rows[0]["technical_score"], 1);
    let stderr = String::from_utf8_lossy(&result.stderr);
    for path in &skipped {
        assert!(stderr.contains(&path.display().to_string()), "{stderr}");
    }
    assert!(stderr.contains("invalid history JSON"));
    assert!(stderr.contains("missing company code"));
    assert!(stderr.contains("duplicate daily close"));
    assert!(stderr.contains("close must be positive"));
    assert!(stderr.contains("Ranked 1 companies; 0 with null technical_score; skipped 8 paths"));
    assert!(!stderr.contains("raw.json"));
}

#[test]
fn invalid_latest_score_skips_the_company_without_using_its_older_history() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let old = write_history(root, "old/history.json", &history("KGHM", "103"));
    let mut invalid = history("KGHM", "103");
    invalid["fetched_at"] = json!("2026-09-21T12:00:00Z");
    invalid["candles"] = json!([{"date": "2026-09-18", "close": "0"}]);
    let new = write_history(root, "new/history.json", &invalid);
    write_history(root, "bank/history.json", &history("PKOBP", "103"));
    let result = run_table(root);
    let rows = rows(&result);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["code"], "PKOBP");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(stderr.contains(&old.display().to_string()));
    assert!(stderr.contains(&new.display().to_string()));
    assert!(stderr.contains("duplicate history for KGHM"));
    assert!(stderr.contains("close must be positive"));
    assert!(stderr.contains("Ranked 1 companies; 0 with null technical_score; skipped 2 paths"));
}

#[test]
fn empty_or_invalid_inputs_fail_without_emitting_json() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let empty = run_table(root);
    assert!(!empty.status.success());
    assert!(empty.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&empty.stderr);
    assert!(stderr.contains("Ranked 0 companies"));
    assert!(stderr.contains("no valid companies found"));

    let bad = write_file(root, "bad/history.json", b"not JSON");
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
            String::from_utf8_lossy(&invalid_directory.stderr).contains("reading histories under")
        );
    }
}
