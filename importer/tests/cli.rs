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
        rankr_import::history::YahooSymbolMap::bundled()
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
fn daily_commands_do_not_read_dotenv_or_require_an_api_key() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(directory.path().join(".env"), "invalid dotenv syntax = '\n").unwrap();
    for command in ["prices", "history"] {
        let result = Command::new(env!("CARGO_BIN_EXE_rankr-import"))
            .args([command, "NOTACOMPANY"])
            .current_dir(directory.path())
            .env_remove("STOOQ_API_KEY")
            .env("HTTPS_PROXY", "http://127.0.0.1:1")
            .output()
            .unwrap();
        let error = String::from_utf8_lossy(&result.stderr);
        assert!(!result.status.success());
        assert!(error.contains("unknown GPW code"), "{error}");
        assert!(!error.contains("API_KEY"));
        assert!(!error.contains("dotenv"));
        assert!(!directory.path().join("data").exists());
    }
}

#[test]
fn yahoo_parse_is_offline_and_preserves_adjusted_close_separately() {
    let directory = tempfile::tempdir().unwrap();
    let input = directory.path().join("raw.json");
    let mut document = raw();
    document.source = Source::YahooFinance;
    document.instrument = Some(
        rankr_import::history::YahooSymbolMap::bundled()
            .unwrap()
            .instrument("KGHM")
            .unwrap(),
    );
    document.url = "https://query1.finance.yahoo.com/v8/finance/chart/KGH.WA?interval=1d&period1=0&period2=1789776000".into();
    document.body = include_str!("fixtures/yahoo_daily.json").into();
    fs::write(&input, serde_json::to_vec(&document).unwrap()).unwrap();
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
    assert_eq!(json["source"], "yahoo_finance");
    assert_eq!(json["symbol"], "KGH.WA");
    assert_eq!(json["candles"][0]["close"], "118.95100");
    assert_eq!(json["candles"][0]["adjusted_close"], "112.234567890123");
    assert!(json.get("stooq_symbol").is_none());
}
