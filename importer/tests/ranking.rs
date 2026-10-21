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
    input
}

fn history(code: &str, september_close: i32) -> Value {
    json!({
        "instrument": {"code": code},
        "source": "yahoo_finance",
        "fetched_at": "2026-09-20T17:00:00Z",
        "candles": [
            {"date": "2025-08-29", "close": 100, "adjusted_close": 999},
            {"date": "2025-09-30", "close": september_close, "adjusted_close": 1},
            {"date": "2026-09-18", "close": 100},
            // This invalid close must be ignored because date == as_of.
            {"date": AS_OF, "close": 0}
        ]
    })
}

fn write_json(root: &Path, relative: &str, input: &Value) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, serde_json::to_vec(input).unwrap()).unwrap();
    path
}

fn write_pair(root: &Path, code: &str, fundamentals: &Value, history: &Value) {
    write_json(
        root,
        &format!("reports/{code}/fundamentals.json"),
        fundamentals,
    );
    write_json(root, &format!("prices/{code}/history.json"), history);
}

fn run(directory: &Path, as_of: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .args(["rank", "--as-of", as_of])
        .arg(directory)
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
    assert!(output.stdout.starts_with(b"[\n  {\n"));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn combines_two_families_with_fixed_weights_and_sorts_scores_then_codes() {
    let directory = tempfile::tempdir().unwrap();
    for code in ["KGHM", "KETY"] {
        write_pair(
            directory.path(),
            code,
            &fundamentals(code),
            &history(code, 110),
        );
    }
    let mut negative = fundamentals("PGE");
    negative["fundamentals"]["net_income_parent"] = json!(-100);
    negative["fundamentals"]["operating_profit"] = json!(-100);
    write_pair(directory.path(), "PGE", &negative, &history("PGE", 90));
    for (code, equity, close) in [("PZU", 220, 110), ("PKOBP", 200, 100)] {
        let mut financial = fundamentals(code);
        financial["fundamentals"]["equity_parent"] = json!(equity);
        financial["fundamentals"]["assets"] = json!(2000);
        write_pair(directory.path(), code, &financial, &history(code, close));
    }

    let ranked = rows(&run(directory.path(), AS_OF));
    assert_eq!(ranked.len(), 5);
    for (row, (code, sector, fundamental, technical, total)) in ranked.iter().zip([
        ("KETY", "Materials", 3, 1, 10),
        ("KGHM", "Materials", 3, 1, 10),
        ("PZU", "Financials", 1, 1, 4),
        ("PKOBP", "Financials", 0, 0, 0),
        ("PGE", "Utilities", -3, -1, -10),
    ]) {
        assert_eq!(
            row,
            &json!({
                "code": code,
                "gics_sector": sector,
                "fundamental_score": fundamental,
                "technical_score": technical,
                "sentiment_score": null,
                "fundamental_weight": 3,
                "technical_weight": 1,
                "sentiment_weight": 0,
                "final_score": total
            })
        );
    }
}

#[test]
fn missing_family_never_becomes_a_partial_sum_even_with_zero_sentiment_weight() {
    let directory = tempfile::tempdir().unwrap();
    let mut no_season = history("KGHM", 110);
    no_season["candles"] = json!([{"date": "2026-09-18", "close": 100}]);
    write_pair(directory.path(), "KGHM", &fundamentals("KGHM"), &no_season);

    let mut no_points = fundamentals("PGE");
    no_points["fundamentals"] = json!({
        "report_period": "I-II kw. 2026", "currency": "PLN",
        "unit_multiplier": 1000, "consolidated": true
    });
    write_pair(directory.path(), "PGE", &no_points, &history("PGE", 110));

    no_points["instrument"]["code"] = json!("PZU");
    no_season["instrument"]["code"] = json!("PZU");
    write_pair(directory.path(), "PZU", &no_points, &no_season);
    let mut negative = fundamentals("KETY");
    negative["fundamentals"]["net_income_parent"] = json!(-100);
    negative["fundamentals"]["operating_profit"] = json!(-100);
    write_pair(directory.path(), "KETY", &negative, &history("KETY", 90));

    let ranked = rows(&run(directory.path(), AS_OF));
    assert_eq!(ranked.len(), 4);
    assert_eq!(ranked[0]["code"], "KETY");
    assert_eq!(ranked[0]["final_score"], -10);
    assert_eq!(ranked[1]["code"], "KGHM");
    assert_eq!(ranked[1]["fundamental_score"], 3);
    assert!(ranked[1]["technical_score"].is_null());
    assert_eq!(ranked[2]["code"], "PGE");
    assert!(ranked[2]["fundamental_score"].is_null());
    assert_eq!(ranked[2]["technical_score"], 1);
    assert_eq!(ranked[3]["code"], "PZU");
    assert!(ranked[3]["fundamental_score"].is_null());
    assert!(ranked[3]["technical_score"].is_null());
    for row in &ranked[1..] {
        assert!(row["final_score"].is_null());
        assert!(row["sentiment_score"].is_null());
        assert_eq!(row["sentiment_weight"], 0);
    }
}

#[test]
fn selects_latest_fundamentals_and_history_independently_of_path_order() {
    for (old, new) in [("a-old", "z-new"), ("z-old", "a-new")] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let old_fundamentals = write_json(
            root,
            &format!("{old}/fundamentals.json"),
            &fundamentals("KGHM"),
        );
        let old_history = write_json(root, &format!("{old}/history.json"), &history("KGHM", 110));
        let mut latest_fundamentals = fundamentals("KGHM");
        latest_fundamentals["fetched_at"] = json!("2026-10-02T18:30:00+01:00");
        latest_fundamentals["fundamentals"]["net_income_parent"] = json!(-100);
        latest_fundamentals["fundamentals"]["operating_profit"] = json!(-100);
        write_json(
            root,
            &format!("{new}/report/fundamentals.json"),
            &latest_fundamentals,
        );
        let mut latest_history = history("KGHM", 90);
        latest_history["fetched_at"] = json!("2026-09-20T20:00:00+02:00");
        write_json(root, &format!("{new}/prices/history.json"), &latest_history);

        let output = run(root, AS_OF);
        let ranked = rows(&output);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0]["fundamental_score"], -3);
        assert_eq!(ranked[0]["technical_score"], -1);
        assert_eq!(ranked[0]["final_score"], -10);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(&old_fundamentals.display().to_string()));
        assert!(stderr.contains(&old_history.display().to_string()));
    }
}

#[test]
fn passes_as_of_to_technical_and_keeps_paired_rows_with_null_scores() {
    let directory = tempfile::tempdir().unwrap();
    write_pair(
        directory.path(),
        "KGHM",
        &fundamentals("KGHM"),
        &history("KGHM", 110),
    );
    assert_eq!(rows(&run(directory.path(), AS_OF))[0]["final_score"], 10);

    let august = rows(&run(directory.path(), "2026-08-19"));
    assert_eq!(august.len(), 1);
    assert_eq!(august[0]["fundamental_score"], 3);
    assert!(august[0]["technical_score"].is_null());
    assert!(august[0]["final_score"].is_null());
}

#[test]
fn reports_unpaired_companies_and_bad_files_without_losing_valid_pairs() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_pair(root, "KGHM", &fundamentals("KGHM"), &history("KGHM", 110));
    let no_history = write_json(root, "kety/fundamentals.json", &fundamentals("KETY"));
    let no_fundamentals = write_json(root, "pge/history.json", &history("PGE", 110));
    write_pair(root, "PZU", &fundamentals("PZU"), &history("PZU", 110));
    let mut bad_latest = history("PZU", 110);
    bad_latest["fetched_at"] = json!("2026-10-03T12:00:00Z");
    bad_latest["candles"][0]["close"] = json!(0);
    let invalid = write_json(root, "pzu-latest/history.json", &bad_latest);
    let broken = write_json(root, "broken/fundamentals.json", &fundamentals("PGE"));
    fs::write(&broken, b"{").unwrap();

    let output = run(root, AS_OF);
    let ranked = rows(&output);
    assert_eq!(ranked.len(), 1);
    assert_eq!(ranked[0]["code"], "KGHM");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for path in [no_history, no_fundamentals, invalid, broken] {
        assert!(stderr.contains(&path.display().to_string()));
    }
    assert!(stderr.contains("KETY: missing usable history.json"));
    assert!(stderr.contains("PGE: missing usable fundamentals.json"));
    assert!(stderr.contains("PZU: missing usable history.json"));
    assert!(stderr.contains("close must be positive"));
}

#[test]
fn empty_directory_and_disjoint_companies_fail_with_empty_stdout() {
    let directory = tempfile::tempdir().unwrap();
    for add_unpaired in [false, true] {
        if add_unpaired {
            write_json(
                directory.path(),
                "kghm/fundamentals.json",
                &fundamentals("KGHM"),
            );
            write_json(directory.path(), "pge/history.json", &history("PGE", 110));
        }
        let output = run(directory.path(), AS_OF);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("no paired companies found"));
    }
}
