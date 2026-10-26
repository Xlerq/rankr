use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::{Value, json};

const AS_OF: &str = "2026-09-19";

fn fundamentals(code: &str) -> Value {
    let mut input: Value =
        serde_json::from_str(include_str!("fixtures/fundamental_company.json")).unwrap();
    input["instrument"]["code"] = json!(code);
    input["instrument"]["isin"] = json!(format!("fixture-{code}"));
    // Two positive margins; all three remaining ratios are neutral.
    input["fundamentals"]["operating_cash_flow"] = json!(100);
    input
}

fn history(code: &str) -> Value {
    json!({
        "instrument": {"code": code, "isin": format!("fixture-{code}")},
        "fetched_at": "2026-10-02T17:00:00Z",
        "candles": [
            {"date": "2025-08-29", "close": 100, "adjusted_close": 1},
            {"date": "2025-09-30", "close": 103, "adjusted_close": 999},
            {"date": "2026-09-18", "close": 100},
            {"date": AS_OF, "close": 0}
        ]
    })
}

fn shorts(code: &str, older: bool) -> Value {
    let older_fixture = include_str!("fixtures/knf_snapshot_older.json");
    let newer_fixture = include_str!("fixtures/knf_snapshot_newer.json");
    let mut input: Value =
        serde_json::from_str(if older { older_fixture } else { newer_fixture }).unwrap();
    input["code"] = json!(code);
    input["isin"] = json!(format!("fixture-{code}"));
    // Equal disclosed exposure gives a real sentiment score of zero.
    let baseline: Value = serde_json::from_str(older_fixture).unwrap();
    input["positions"] = baseline["positions"].clone();
    input
}

fn write_file(root: &Path, relative: &str, bytes: &[u8]) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, bytes).unwrap();
    path
}

fn write_json(root: &Path, relative: &str, input: &Value) -> PathBuf {
    write_file(root, relative, &serde_json::to_vec(input).unwrap())
}

fn write_company(root: &Path, code: &str) {
    write_json(
        root,
        &format!("{code}/fundamentals.json"),
        &fundamentals(code),
    );
    write_json(root, &format!("{code}/history.json"), &history(code));
    write_json(
        root,
        &format!("{code}/old/shorts.json"),
        &shorts(code, true),
    );
    write_json(
        root,
        &format!("{code}/new/shorts.json"),
        &shorts(code, false),
    );
}

fn run(root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .args(["rank", "--as-of", AS_OF])
        .arg(root)
        .env("HTTP_PROXY", "http://127.0.0.1:1")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .env("ALL_PROXY", "http://127.0.0.1:1")
        .output()
        .unwrap()
}

fn rows(output: &Output) -> Vec<Value> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.starts_with(b"[\n"));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn sums_two_plus_one_plus_zero_as_three_and_emits_only_the_six_requested_fields() {
    let directory = tempfile::tempdir().unwrap();
    write_company(directory.path(), "KGHM");
    let output = run(directory.path());
    assert_eq!(
        rows(&output),
        [json!({
            "code": "KGHM",
            "gics_sector": "Materials",
            "fundamental_score": 2,
            "technical_score": 1,
            "sentiment_score": 0,
            "final_score": 3
        })]
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Ranked 1 companies; 0 with null final_score")
    );
    assert!(!directory.path().join("data").exists());
}

#[test]
fn sentiment_changes_the_final_sum_in_both_directions_with_unit_weight() {
    for (percent, sentiment, total) in [("0.8", -1, 2), ("0.6", 1, 4)] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        write_company(root, "KGHM");
        let mut newer = shorts("KGHM", false);
        newer["positions"][1]["net_short_position_percent"] = json!(percent);
        write_json(root, "KGHM/new/shorts.json", &newer);
        let result = rows(&run(root));
        assert_eq!(result[0]["sentiment_score"], sentiment);
        assert_eq!(result[0]["final_score"], total);
    }
}

#[test]
fn a_missing_family_yields_null_instead_of_a_partial_sum() {
    for missing in ["sentiment_score", "technical_score", "fundamental_score"] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        write_company(root, "KGHM");
        match missing {
            "sentiment_score" => {
                fs::remove_file(root.join("KGHM/old/shorts.json")).unwrap();
                fs::remove_file(root.join("KGHM/new/shorts.json")).unwrap();
            }
            "technical_score" => {
                let mut input = history("KGHM");
                input["candles"] = json!([{"date": "2026-09-18", "close": 100}]);
                write_json(root, "KGHM/history.json", &input);
            }
            "fundamental_score" => {
                let mut input = fundamentals("KGHM");
                for field in [
                    "revenue",
                    "net_income_parent",
                    "operating_profit",
                    "equity_parent",
                    "assets",
                    "long_term_liabilities",
                    "short_term_liabilities",
                    "operating_cash_flow",
                ] {
                    input["fundamentals"][field] = Value::Null;
                }
                write_json(root, "KGHM/fundamentals.json", &input);
            }
            _ => unreachable!(),
        }
        let output = run(root);
        let result = rows(&output);
        assert_eq!(result.len(), 1);
        assert!(result[0][missing].is_null(), "{missing}");
        assert!(result[0]["final_score"].is_null(), "{missing}");
        for (field, expected) in [
            ("fundamental_score", 2),
            ("technical_score", 1),
            ("sentiment_score", 0),
        ] {
            if field != missing {
                assert_eq!(result[0][field], expected, "{missing}: {field}");
            }
        }
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("Ranked 1 companies; 1 with null final_score")
        );
    }
}

#[test]
fn a_single_shorts_snapshot_or_a_gap_of_only_29_days_has_null_sentiment() {
    for single_snapshot in [true, false] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        write_company(root, "KGHM");
        if single_snapshot {
            fs::remove_file(root.join("KGHM/old/shorts.json")).unwrap();
        } else {
            let mut input = shorts("KGHM", true);
            input["fetched_at"] = json!("2026-09-02T00:00:00Z");
            write_json(root, "KGHM/old/shorts.json", &input);
        }
        let result = rows(&run(root));
        assert_eq!(result[0]["fundamental_score"], 2);
        assert_eq!(result[0]["technical_score"], 1);
        assert!(result[0]["sentiment_score"].is_null());
        assert!(result[0]["final_score"].is_null());
    }
}

#[test]
fn chooses_the_latest_shorts_and_latest_eligible_older_snapshot_of_the_same_isin() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_company(root, "KGHM");
    let mut extra = shorts("KGHM", true);
    // Much older observations would give -1 instead of the expected 0.
    extra["fetched_at"] = json!("2026-08-01T00:00:00Z");
    extra["positions"][1]["net_short_position_percent"] = json!(0.5);
    write_json(root, "a-very-old/shorts.json", &extra);
    extra["fetched_at"] = json!("2026-09-02T00:00:00Z");
    write_json(root, "a-too-recent/shorts.json", &extra);
    extra["fetched_at"] = json!("2026-09-01T23:59:00Z");
    extra["isin"] = json!("different-isin");
    write_json(root, "a-wrong-isin/shorts.json", &extra);
    // Equal timestamps retain the first lexical path, as in the other tables.
    extra["isin"] = json!("fixture-KGHM");
    extra["fetched_at"] = json!("2026-09-01T23:30:00Z");
    write_json(root, "z-duplicate-old/shorts.json", &extra);
    extra["fetched_at"] = json!("2026-10-01T00:30:00Z");
    write_json(root, "z-duplicate-new/shorts.json", &extra);
    let result = rows(&run(root));
    assert_eq!(result[0]["sentiment_score"], 0);
    assert_eq!(result[0]["final_score"], 3);
}

#[test]
fn does_not_fall_back_to_a_more_favorable_pair_when_selected_shorts_are_incomparable() {
    for invalid_newer in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        write_company(root, "KGHM");
        if invalid_newer {
            let mut newer = shorts("KGHM", false);
            newer["positions"] = json!([]);
            write_json(root, "KGHM/new/shorts.json", &newer);
            let mut very_old = shorts("KGHM", true);
            very_old["fetched_at"] = json!("2026-08-01T00:00:00Z");
            write_json(root, "earlier/shorts.json", &very_old);
        } else {
            let mut latest_older = shorts("KGHM", true);
            latest_older["fetched_at"] = json!("2026-09-01T23:59:00Z");
            latest_older["positions"][0]["holder"] = json!("Different Holder");
            write_json(root, "latest-older/shorts.json", &latest_older);
        }
        let result = rows(&run(root));
        assert!(result[0]["sentiment_score"].is_null());
        assert!(result[0]["final_score"].is_null());
    }
}

#[test]
fn reuses_latest_fundamental_and_history_selection_instead_of_lexical_order() {
    for (old_directory, new_directory) in [("a-old", "z-new"), ("z-old", "a-new")] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        write_company(root, "KGHM");
        fs::remove_file(root.join("KGHM/fundamentals.json")).unwrap();
        fs::remove_file(root.join("KGHM/history.json")).unwrap();
        let mut old = fundamentals("KGHM");
        old["fundamentals"]["net_income_parent"] = json!(-100);
        old["fundamentals"]["operating_profit"] = json!(-125);
        let old_fundamentals =
            write_json(root, &format!("{old_directory}/fundamentals.json"), &old);
        let mut newer = fundamentals("KGHM");
        newer["fetched_at"] = json!("2026-10-02T18:30:00+01:00");
        write_json(root, &format!("{new_directory}/fundamentals.json"), &newer);
        let mut old = history("KGHM");
        old["candles"][1]["close"] = json!(97);
        let old_history = write_json(root, &format!("{old_directory}/history.json"), &old);
        let mut newer = history("KGHM");
        newer["fetched_at"] = json!("2026-10-02T18:30:00+01:00");
        write_json(root, &format!("{new_directory}/history.json"), &newer);
        let output = run(root);
        assert_eq!(rows(&output)[0]["final_score"], 3);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&old_fundamentals.display().to_string()));
        assert!(stderr.contains(&old_history.display().to_string()));
    }
}

#[test]
fn sorts_descending_then_by_code_with_null_after_negative_scores() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    for code in ["PGE", "KGHM", "ALLEGRO", "KETY"] {
        write_company(root, code);
    }
    fs::remove_file(root.join("ALLEGRO/old/shorts.json")).unwrap();
    let mut input = fundamentals("PGE");
    input["fundamentals"]["net_income_parent"] = json!(-100);
    input["fundamentals"]["operating_profit"] = json!(-125);
    input["fundamentals"]["operating_cash_flow"] = json!(0);
    write_json(root, "PGE/fundamentals.json", &input);
    let mut input = history("PGE");
    input["candles"][1]["close"] = json!(97);
    write_json(root, "PGE/history.json", &input);
    let output = run(root);
    let result = rows(&output);
    let codes: Vec<_> = result
        .iter()
        .map(|row| row["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["KETY", "KGHM", "PGE", "ALLEGRO"]);
    assert_eq!(result[2]["final_score"], -3);
    assert!(result[3]["final_score"].is_null());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Ranked 4 companies; 1 with null final_score")
    );
}

#[test]
fn keeps_known_companies_with_missing_snapshot_files_as_incomplete_rows() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_json(
        root,
        "fundamental-only/fundamentals.json",
        &fundamentals("KGHM"),
    );
    write_json(root, "technical-only/history.json", &history("KETY"));
    write_json(root, "shorts-only/old/shorts.json", &shorts("PGE", true));
    write_json(root, "shorts-only/new/shorts.json", &shorts("PGE", false));
    let output = run(root);
    let result = rows(&output);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0]["code"], "KETY");
    assert_eq!(result[0]["technical_score"], 1);
    assert!(result[0]["fundamental_score"].is_null());
    assert_eq!(result[1]["code"], "KGHM");
    assert_eq!(result[1]["fundamental_score"], 2);
    assert!(result[1]["technical_score"].is_null());
    assert_eq!(result[2]["code"], "PGE");
    assert_eq!(result[2]["sentiment_score"], 0);
    for row in result {
        assert!(row["final_score"].is_null());
    }
}

#[test]
fn bad_json_and_missing_identifiers_are_reported_without_aborting_other_companies() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_company(root, "KGHM");
    let mut skipped = vec![
        write_file(root, "bad-fundamentals/fundamentals.json", b"{"),
        write_file(root, "bad-history/history.json", b"{"),
        write_file(root, "bad-shorts/shorts.json", b"{"),
    ];
    for field in ["code", "isin"] {
        let mut input = shorts("KGHM", false);
        input[field] = json!(" ");
        skipped.push(write_json(root, &format!("no-{field}/shorts.json"), &input));
    }
    let output = run(root);
    assert_eq!(rows(&output)[0]["final_score"], 3);
    let stderr = String::from_utf8_lossy(&output.stderr);
    for path in skipped {
        assert!(stderr.contains(&path.display().to_string()));
    }
}

#[test]
fn zero_known_companies_is_an_error_with_empty_stdout() {
    for input in ["empty", "invalid", "unknown"] {
        let directory = tempfile::tempdir().unwrap();
        match input {
            "invalid" => {
                write_file(directory.path(), "bad/shorts.json", b"{");
            }
            "unknown" => {
                write_company(directory.path(), "UNKNOWN");
            }
            _ => {}
        }
        let output = run(directory.path());
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("no valid companies found"));
    }
}
