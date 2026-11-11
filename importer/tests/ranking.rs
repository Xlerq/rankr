use std::{fs, path::Path};

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
    input["fetched_at"] = json!(if older {
        "2026-09-11T12:00:00Z"
    } else {
        "2026-09-18T12:00:00Z"
    });
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
fn sentiment_uses_the_latest_observation_in_each_of_two_consecutive_iso_weeks() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_company(root, "KGHM");
    let mut newest = shorts("KGHM", false);
    newest["positions"][0]["net_short_position_percent"] = json!(0.8);
    write_json(root, "KGHM/new/shorts.json", &newest);
    for (name, date, total) in [
        ("within-week", "2026-09-17T12:00:00Z", 5),
        ("early-prior-week", "2026-09-08T12:00:00Z", 9),
        ("future", "2026-09-21T12:00:00Z", 0),
    ] {
        let mut other = newest.clone();
        other["fetched_at"] = json!(date);
        other["positions"][0]["net_short_position_percent"] = json!(total);
        write_json(root, &format!("KGHM/{name}/shorts.json"), &other);
    }
    let rows = rank(root);
    assert_eq!(rows[0]["sentiment_score"], -1);
    assert_eq!(rows[0]["final_score"], 2);
}

#[test]
fn sentiment_requires_the_current_and_immediately_preceding_week_including_iso_year_boundaries() {
    for (as_of, older_at, newer_at, expected) in [
        (
            "2021-01-06",
            "2021-01-01T12:00:00Z",
            "2021-01-05T12:00:00Z",
            Some(-1),
        ),
        (AS_OF, "2026-09-04T12:00:00Z", "2026-09-18T12:00:00Z", None),
        (AS_OF, "2026-09-11T12:00:00Z", "2026-09-12T12:00:00Z", None),
        (AS_OF, "2026-09-11T12:00:00Z", "2026-09-21T12:00:00Z", None),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let mut older = shorts("KGHM", true);
        older["fetched_at"] = json!(older_at);
        let mut newer = shorts("KGHM", false);
        newer["fetched_at"] = json!(newer_at);
        newer["positions"][0]["net_short_position_percent"] = json!(0.8);
        write_json(directory.path(), "old/shorts.json", &older);
        write_json(directory.path(), "new/shorts.json", &newer);
        let rows = rankr::ranking::rank_directory(directory.path(), as_of.parse().unwrap())
            .unwrap()
            .rows;
        assert_eq!(
            rows[0].sentiment_score, expected,
            "{as_of}: {older_at} -> {newer_at}"
        );
        assert!(rows[0].final_score.is_none());
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

#[test]
fn archives_ranked_scores_with_utc_metadata_and_preserves_null_and_zero() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_company(root, "KGHM");
    write_company(root, "ALLEGRO");
    fs::remove_file(root.join("ALLEGRO/old/shorts.json")).unwrap();
    let computed_at = "2026-09-19T23:30:00Z".parse().unwrap();
    let path = rankr::ranking::archive_snapshot(root, computed_at)
        .unwrap()
        .unwrap();
    let snapshot: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();

    assert_eq!(snapshot["computed_at"], json!(computed_at));
    assert_eq!(snapshot["as_of"], AS_OF);
    assert_eq!(snapshot["scoring_version"], rankr::ranking::SCORING_VERSION);
    assert_eq!(snapshot["rows"], json!(rank(root)));
    assert_eq!(snapshot["rows"][0]["final_score"], 3);
    assert_eq!(snapshot["rows"][0]["sentiment_score"], 0);
    assert!(snapshot["rows"][1]["sentiment_score"].is_null());
    assert!(snapshot["rows"][1]["final_score"].is_null());
}

#[test]
fn repeated_ranking_snapshots_never_overwrite_previous_scores() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_company(root, "KGHM");
    let computed_at = "2026-09-19T23:30:00Z".parse().unwrap();
    let first = rankr::ranking::archive_snapshot(root, computed_at)
        .unwrap()
        .unwrap();
    let original = fs::read(&first).unwrap();

    fs::remove_file(root.join("KGHM/old/shorts.json")).unwrap();
    // Even identical timestamps must preserve both calculations.
    let second = rankr::ranking::archive_snapshot(root, computed_at)
        .unwrap()
        .unwrap();
    assert_ne!(first, second);
    assert_eq!(fs::read(first).unwrap(), original);
    let snapshot: Value = serde_json::from_slice(&fs::read(second).unwrap()).unwrap();
    assert!(snapshot["rows"][0]["final_score"].is_null());
}

#[test]
fn empty_rankings_do_not_create_history() {
    let directory = tempfile::tempdir().unwrap();
    let computed_at = "2026-09-19T23:30:00Z".parse().unwrap();
    assert!(
        rankr::ranking::archive_snapshot(directory.path(), computed_at)
            .unwrap()
            .is_none()
    );
    assert!(!directory.path().join("rankings").exists());
}

#[test]
fn ranking_archive_write_failure_is_reported_without_changing_source_data() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_company(root, "KGHM");
    let expected = rank(root);
    fs::write(root.join("rankings"), "not a directory").unwrap();
    let computed_at = "2026-09-19T23:30:00Z".parse().unwrap();
    assert!(rankr::ranking::archive_snapshot(root, computed_at).is_err());
    assert_eq!(rank(root), expected);
}
