use std::{fs, path::Path};

use chrono::{DateTime, Duration, Utc};
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
            {"date": "2025-08-29", "close": 100},
            {"date": "2025-09-30", "close": 103},
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

fn write_json(root: &Path, relative: &str, input: &Value) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, serde_json::to_vec(input).unwrap()).unwrap();
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

fn rank(root: &Path) -> Vec<Value> {
    let date = chrono::NaiveDate::parse_from_str(AS_OF, "%Y-%m-%d").unwrap();
    let ranking = rankr::ranking::rank_directory(root, date).unwrap();
    serde_json::from_value(serde_json::to_value(ranking.rows).unwrap()).unwrap()
}

#[test]
fn sums_two_plus_one_plus_zero_as_three() {
    let directory = tempfile::tempdir().unwrap();
    write_company(directory.path(), "KGHM");
    let output = rank(directory.path());
    assert_eq!(
        output,
        [json!({
            "code": "KGHM",
            "gics_sector": "Materials",
            "fundamental_score": 2,
            "technical_score": 1,
            "sentiment_score": 0,
            "final_score": 3
        })]
    );
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
        let result = rank(root);
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
    }
}

#[test]
fn sentiment_uses_latest_snapshot_from_an_earlier_utc_day() {
    let newer_at: DateTime<Utc> = "2026-10-28T07:00:00Z".parse().unwrap();
    for (hours, expected) in [(1, None), (13, Some(-1)), (24, Some(-1)), (48, Some(-1))] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        write_company(root, "KGHM");
        let mut newer = shorts("KGHM", false);
        newer["fetched_at"] = json!(newer_at);
        newer["positions"][0]["net_short_position_percent"] = json!(0.8);
        let mut older = shorts("KGHM", true);
        older["fetched_at"] = json!(newer_at - Duration::hours(hours));
        write_json(root, "KGHM/new/shorts.json", &newer);
        write_json(root, "KGHM/old/shorts.json", &older);
        if expected.is_some() {
            let mut recent = newer.clone();
            recent["fetched_at"] = json!(newer_at - Duration::hours(1));
            write_json(root, "KGHM/recent/shorts.json", &recent);
            older["fetched_at"] = json!(newer_at - Duration::days(3));
            older["positions"][0]["net_short_position_percent"] = json!(5);
            write_json(root, "KGHM/earliest/shorts.json", &older);
        }
        let rows = rank(root);
        assert_eq!(rows[0]["sentiment_score"], json!(expected));
        assert_eq!(
            rows[0]["final_score"],
            json!(expected.map(|points| 3 + points))
        );
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
    let result = rank(root);
    let codes: Vec<_> = result
        .iter()
        .map(|row| row["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["KETY", "KGHM", "PGE", "ALLEGRO"]);
    assert_eq!(result[2]["final_score"], -3);
    assert!(result[3]["final_score"].is_null());
}
