use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde::Deserialize;
use serde_json::{Value, json};

fn snapshot(code: &str) -> Value {
    let mut input: Value =
        serde_json::from_str(include_str!("fixtures/fundamental_company.json")).unwrap();
    input["instrument"]["code"] = json!(code);
    input
}

fn write_file(root: &Path, relative: &str, body: &[u8]) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, body).unwrap();
    path
}

fn write_snapshot(root: &Path, relative: &str, input: &Value) -> PathBuf {
    write_file(root, relative, &serde_json::to_vec(input).unwrap())
}

fn run(command: &str, directory: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .arg(command)
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
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn gics_map_matches_all_twenty_explicit_assignments() {
    #[derive(Deserialize)]
    struct Entry {
        gpw_code: String,
        sector: String,
        gics_sector_code: u16,
        gics_sector: String,
        gics_industry_group_code: u16,
        gics_industry_group: String,
    }
    let mut reader =
        csv::Reader::from_reader(include_str!("../../data/raw/wig20_symbols.csv").as_bytes());
    let entries: Vec<Entry> = reader.deserialize().collect::<Result<_, _>>().unwrap();
    assert_eq!(entries.len(), 20);
    for (codes, sector_code, sector, group_code, group) in [
        (vec!["PKNORLEN"], 10, "Energy", 1010, "Energy"),
        (vec!["KGHM", "KETY"], 15, "Materials", 1510, "Materials"),
        (vec!["BUDIMEX"], 20, "Industrials", 2010, "Capital Goods"),
        (
            vec!["ALLEGRO", "LPP", "PEPCO", "MODIVO"],
            25,
            "Consumer Discretionary",
            2550,
            "Consumer Discretionary Distribution & Retail",
        ),
        (
            vec!["DINOPL", "ZABKA"],
            30,
            "Consumer Staples",
            3010,
            "Consumer Staples Distribution & Retail",
        ),
        (
            vec!["PKOBP", "PEKAO", "ERSTEPL", "MBANK", "ALIOR"],
            40,
            "Financials",
            4010,
            "Banks",
        ),
        (vec!["KRUK"], 40, "Financials", 4020, "Financial Services"),
        (vec!["PZU"], 40, "Financials", 4030, "Insurance"),
        (
            vec!["CDPROJEKT"],
            50,
            "Communication Services",
            5020,
            "Media & Entertainment",
        ),
        (vec!["PGE", "TAURONPE"], 55, "Utilities", 5510, "Utilities"),
    ] {
        for code in codes {
            let entry = entries.iter().find(|entry| entry.gpw_code == code).unwrap();
            assert_eq!(entry.gics_sector_code, sector_code, "{code}");
            assert_eq!(entry.gics_sector, sector, "{code}");
            assert_eq!(entry.gics_industry_group_code, group_code, "{code}");
            assert_eq!(entry.gics_industry_group, group, "{code}");
            assert!(!entry.sector.is_empty());
        }
    }
}

#[test]
fn kghm_pko_pzu_and_kruk_use_their_templates_and_raw_table_is_unchanged() {
    let directory = tempfile::tempdir().unwrap();
    for code in ["KGHM", "PKOBP", "PZU", "KRUK"] {
        let mut input = snapshot(code);
        // All companies have identical figures, including nonzero margins.
        input["fundamentals"]["equity_parent"] = json!("110");
        write_snapshot(
            directory.path(),
            &format!("{code}/fundamentals.json"),
            &input,
        );
    }
    let result = run("fundamental-points", directory.path());
    let scored = rows(&result);
    assert_eq!(scored.len(), 4);
    for row in &scored {
        assert_eq!(row.as_object().unwrap().len(), 15);
        assert!(row["equity_ratio"].is_number());
        if row["code"] == "KGHM" {
            assert_eq!(row["gics_sector"], "Materials");
            assert_eq!(row["equity_ratio_points"], -1);
            assert_eq!(row["net_margin_points"], 1);
            assert_eq!(row["fundamental_score"], 1);
        } else {
            assert_eq!(row["gics_sector"], "Financials");
            assert_eq!(row["equity_ratio_points"], 1);
            assert_eq!(row["fundamental_score"], 1);
            for field in [
                "net_margin",
                "operating_margin",
                "debt_to_equity",
                "ocf_to_net_income",
            ] {
                assert!(row[field].is_null(), "{field}");
                assert!(row[format!("{field}_points")].is_null(), "{field}");
            }
        }
    }
    assert_eq!(
        scored
            .iter()
            .map(|row| row["code"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["KGHM", "KRUK", "PKOBP", "PZU"]
    );
    assert_eq!(scored[1]["gics_industry_group"], "Financial Services");
    assert_eq!(scored[2]["gics_industry_group"], "Banks");
    assert_eq!(scored[3]["gics_industry_group"], "Insurance");
    assert!(
        String::from_utf8_lossy(&result.stderr)
            .contains("Scored 4 companies; 0 with null fundamental_score")
    );

    for row in rows(&run("fundamental-table", directory.path())) {
        assert!(row["net_margin"].is_number());
        assert!(row.get("fundamental_score").is_none());
        assert!(row.get("net_margin_points").is_none());
    }
}

#[test]
fn latest_snapshot_wins_and_scores_sort_descending_with_null_last() {
    for (new_directory, old_directory) in [("a-new", "z-old"), ("z-new", "a-old")] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let old = write_snapshot(
            root,
            &format!("{old_directory}/fundamentals.json"),
            &snapshot("KGHM"),
        );
        let mut latest = snapshot("KGHM");
        latest["fetched_at"] = json!("2026-10-02T18:30:00+01:00");
        latest["fundamentals"]["net_income_parent"] = json!("-100");
        latest["fundamentals"]["operating_profit"] = json!("-100");
        let new = write_snapshot(root, &format!("{new_directory}/fundamentals.json"), &latest);
        for code in ["PGE", "KETY"] {
            write_snapshot(
                root,
                &format!("nested/{code}/fundamentals.json"),
                &snapshot(code),
            );
        }
        let mut missing = snapshot("ALIOR");
        missing["fundamentals"]["assets"] = json!(0);
        write_snapshot(root, "missing/fundamentals.json", &missing);
        let result = run("fundamental-points", root);
        let scored = rows(&result);
        assert_eq!(
            scored
                .iter()
                .map(|row| row["code"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["KETY", "PGE", "KGHM", "ALIOR"]
        );
        assert_eq!(scored[0]["fundamental_score"], 3);
        assert_eq!(scored[2]["fundamental_score"], -3);
        assert_eq!(scored[2]["source_file"], json!(new));
        assert!(scored[3]["equity_ratio_points"].is_null());
        assert!(scored[3]["fundamental_score"].is_null());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.contains(&old.display().to_string()));
        assert!(
            stderr.contains("Scored 4 companies; 1 with null fundamental_score; skipped 1 paths")
        );
    }
}

#[test]
fn missing_revenue_excludes_margin_points_and_financials_can_use_only_equity() {
    let directory = tempfile::tempdir().unwrap();
    let mut input = snapshot("KGHM");
    input["fundamentals"]["revenue"] = Value::Null;
    write_snapshot(directory.path(), "company/fundamentals.json", &input);
    let mut bank: Value =
        serde_json::from_str(include_str!("fixtures/fundamental_bank.json")).unwrap();
    bank["fundamentals"] = json!({
        "report_period": "I-II kw. 2026", "currency": "PLN", "consolidated": true,
        "unit_multiplier": 1000, "equity_parent": "200", "assets": "2000"
    });
    write_snapshot(directory.path(), "bank/fundamentals.json", &bank);
    let scored = rows(&run("fundamental-points", directory.path()));
    assert_eq!(scored[0]["code"], "KGHM");
    assert!(scored[0]["net_margin_points"].is_null());
    assert!(scored[0]["operating_margin_points"].is_null());
    assert_eq!(scored[0]["fundamental_score"], 1);
    assert_eq!(scored[1]["code"], "PKOBP");
    assert_eq!(scored[1]["equity_ratio_points"], 0);
    assert_eq!(scored[1]["fundamental_score"], 0);
}

#[test]
fn unknown_codes_and_bad_files_are_reported_without_aborting_other_companies() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_snapshot(root, "valid/fundamentals.json", &snapshot("KGHM"));
    let unknown = write_snapshot(root, "unknown/fundamentals.json", &snapshot("UNKNOWN"));
    let broken = write_file(root, "broken/fundamentals.json", b"{");
    let mut missing = snapshot("MISSING");
    missing.as_object_mut().unwrap().remove("instrument");
    let missing = write_snapshot(root, "missing/fundamentals.json", &missing);
    write_file(root, "valid/raw.json", b"not JSON");
    let result = run("fundamental-points", root);
    assert_eq!(rows(&result).len(), 1);
    let stderr = String::from_utf8_lossy(&result.stderr);
    for path in [unknown, broken, missing] {
        assert!(stderr.contains(&path.display().to_string()));
    }
    assert!(stderr.contains("missing GICS mapping for UNKNOWN"));
    assert!(stderr.contains("skipped 3 paths"));
}

#[test]
fn points_do_not_depend_on_other_companies() {
    let directory = tempfile::tempdir().unwrap();
    write_snapshot(
        directory.path(),
        "kghm/fundamentals.json",
        &snapshot("KGHM"),
    );
    let before = rows(&run("fundamental-points", directory.path())).remove(0);
    let mut better = snapshot("KETY");
    better["fundamentals"]["net_income_parent"] = json!("999");
    write_snapshot(directory.path(), "kety/fundamentals.json", &better);
    let after = rows(&run("fundamental-points", directory.path()));
    assert_eq!(
        after.iter().find(|row| row["code"] == "KGHM").unwrap(),
        &before
    );
}

#[test]
fn no_known_companies_is_an_error_with_empty_stdout() {
    let directory = tempfile::tempdir().unwrap();
    for add_unknown in [false, true] {
        if add_unknown {
            write_snapshot(
                directory.path(),
                "unknown/fundamentals.json",
                &snapshot("UNKNOWN"),
            );
        }
        let result = run("fundamental-points", directory.path());
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("no valid companies found"));
    }
}
