use std::{fs, process::Command};

use chrono::{TimeZone, Utc};
use rankr_import::model::{Instrument, RawDocument, Source};
use serde_json::Value;

fn raw() -> RawDocument {
    RawDocument {
        source: Source::GpwNotoria,
        instrument: Some(Instrument {
            isin: "PL11BTS00015".into(),
            code: "11BIT".into(),
            name: "11 bit studios".into(),
        }),
        fetched_at: Utc.with_ymd_and_hms(2026, 9, 14, 18, 0, 0).unwrap(),
        url: "https://example.invalid/fundamentals".into(),
        http_status: 200,
        body: include_str!("../../data/raw/11bit_gpw_notoria_sample.html").into(),
    }
}

#[test]
fn parse_command_is_offline_and_emits_only_json() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("raw.json");
    fs::write(&input, serde_json::to_vec(&raw()).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .args(["parse", input.to_str().unwrap()])
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .output()
        .unwrap();

    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stderr.is_empty());
    let json: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["fundamentals"]["revenue"], "141034.32");
    assert_eq!(json["fundamentals"]["capex"], "-37993.69");
    assert_eq!(json["fundamentals"]["unit_multiplier"], 1000);
    assert_eq!(json["instrument"]["isin"], "PL11BTS00015");
    assert!(!String::from_utf8_lossy(&result.stdout).contains("parser_version"));
}

#[test]
fn parse_rejects_http_error_without_emitting_a_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("raw.json");
    let mut document = raw();
    document.http_status = 503;
    fs::write(&input, serde_json::to_vec(&document).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .args(["parse", input.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("HTTP 503"));
}

#[test]
fn history_parse_is_offline_and_preserves_daily_decimals() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("raw.json");
    let mut document = raw();
    document.source = Source::Stooq;
    document.instrument = Some(
        rankr_import::history::StooqSymbolMap::bundled()
            .unwrap()
            .instrument("KGHM")
            .unwrap(),
    );
    document.url = "https://stooq.com/q/d/l/?s=kgh&i=d".into();
    document.body = include_str!("fixtures/stooq_daily.csv").into();
    fs::write(&input, serde_json::to_vec(&document).unwrap()).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .args(["parse", input.to_str().unwrap()])
        .env_remove("STOOQ_API_KEY")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(result.stderr.is_empty());
    let json: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(json["source"], "stooq");
    assert_eq!(json["candles"][0]["close"], "118.95100");
}

#[test]
fn history_requires_a_key_before_attempting_any_network_request() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join(".env"), "# no API key\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .arg("history")
        .current_dir(directory.path())
        .env_remove("STOOQ_API_KEY")
        .env("HTTPS_PROXY", "http://127.0.0.1:1")
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("missing STOOQ_API_KEY"));
    assert!(!directory.path().join("data").exists());
}

#[test]
fn history_reads_dotenv_but_environment_takes_precedence() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join(".env"),
        "STOOQ_API_KEY='fixture-secret'\n",
    )
    .unwrap();
    // An unknown code stops before HTTP, after credentials have been loaded.
    let result = Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .args(["history", "kgh"])
        .current_dir(directory.path())
        .env_remove("STOOQ_API_KEY")
        .output()
        .unwrap();
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(error.contains("unknown GPW code"), "{error}");
    assert!(!error.contains("fixture-secret"));

    let result = Command::new(env!("CARGO_BIN_EXE_rankr-import"))
        .args(["history", "KGHM"])
        .current_dir(directory.path())
        .env("STOOQ_API_KEY", "")
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("missing STOOQ_API_KEY"));
}
