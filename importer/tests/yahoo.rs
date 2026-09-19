use chrono::{TimeZone, Utc};
use rankr_import::{
    history::{HistoryError, YahooSymbolMap, parse_history_document},
    model::{RawDocument, SkippedCandleReason, Source},
};
use serde_json::{Value, json};

fn raw() -> RawDocument {
    RawDocument {
        source: Source::YahooFinance,
        instrument: Some(YahooSymbolMap::bundled().unwrap().instrument("KGHM").unwrap()),
        fetched_at: Utc.with_ymd_and_hms(2024, 1, 6, 12, 0, 0).unwrap(),
        url: "https://query1.finance.yahoo.com/v8/finance/chart/KGH.WA?interval=1d&period1=0&period2=1704542400".into(),
        http_status: 200,
        body: include_str!("fixtures/yahoo_daily.json").into(),
    }
}

fn changed(edit: impl FnOnce(&mut Value)) -> RawDocument {
    let mut raw = raw();
    let mut data: Value = serde_json::from_str(&raw.body).unwrap();
    edit(&mut data["chart"]["result"][0]);
    raw.body = serde_json::to_string(&data).unwrap();
    raw
}

#[test]
fn preserves_daily_ohlcv_and_adjusted_close_without_recomputing_dividends() {
    let history = parse_history_document(&raw()).unwrap();
    assert_eq!(history.source, Source::YahooFinance);
    assert_eq!(history.symbol, "KGH.WA");
    assert_eq!(history.currency, "PLN");
    assert_eq!(history.candles.len(), 3);
    assert!(history.skipped_candles.is_empty());
    let candle = &history.candles[0];
    assert_eq!(candle.date.to_string(), "2024-01-02");
    assert_eq!(candle.close.to_string(), "118.95100");
    assert_eq!(
        candle.adjusted_close.unwrap().to_string(),
        "112.234567890123"
    );
    assert_eq!(candle.volume.to_string(), "475373");
    let saved: rankr_import::model::DailyPriceHistory =
        serde_json::from_str(&serde_json::to_string(&history).unwrap()).unwrap();
    assert_eq!(saved, history);
}

#[test]
fn rejects_mismatched_symbols_markets_and_non_daily_responses() {
    for (field, value) in [
        ("symbol", "PKO.WA"),
        ("currency", "USD"),
        ("exchangeName", "NMS"),
        ("exchangeTimezoneName", "UTC"),
        ("instrumentType", "INDEX"),
        ("dataGranularity", "1mo"),
    ] {
        let raw = changed(|s| s["meta"][field] = json!(value));
        assert!(
            parse_history_document(&raw).is_err(),
            "accepted {field}={value}"
        );
    }
    let mut document = raw();
    document.url = document.url.replace("KGH.WA", "PKO.WA");
    assert!(parse_history_document(&document).is_err());
    document = raw();
    document.url.push_str("&interval=1mo");
    assert!(parse_history_document(&document).is_err());
    document = raw();
    document.instrument.as_mut().unwrap().isin = "PL0000000000".into();
    assert!(matches!(
        parse_history_document(&document),
        Err(HistoryError::MissingMapping { .. })
    ));
}

#[test]
fn reports_missing_and_invalid_candles_without_filling_or_repairing_them() {
    for (field, value, reason) in [
        ("close", Value::Null, SkippedCandleReason::MissingValues),
        ("volume", Value::Null, SkippedCandleReason::MissingValues),
        ("open", json!(-1), SkippedCandleReason::InvalidValues),
        ("volume", json!(-1), SkippedCandleReason::InvalidValues),
        ("high", json!(100), SkippedCandleReason::InvalidRange),
        ("low", json!(150), SkippedCandleReason::InvalidRange),
    ] {
        let document = changed(|s| s["indicators"]["quote"][0][field][1] = value);
        let history = parse_history_document(&document).unwrap();
        assert_eq!(history.candles.len(), 2);
        assert_eq!(history.skipped_candles.len(), 1);
        assert_eq!(history.skipped_candles[0].date.to_string(), "2024-01-03");
        assert_eq!(history.skipped_candles[0].reason, reason);
        assert_eq!(history.candles[1].date.to_string(), "2024-01-04");
    }
    let document = changed(|s| s["indicators"]["adjclose"][0]["adjclose"][1] = Value::Null);
    assert_eq!(
        parse_history_document(&document).unwrap().skipped_candles[0].reason,
        SkippedCandleReason::MissingValues
    );
    let document = changed(|s| s["indicators"]["quote"][0]["close"] = json!([null, null, null]));
    assert!(parse_history_document(&document).is_err());
}

#[test]
fn refuses_to_align_different_array_lengths_or_accept_duplicate_dates() {
    for field in ["open", "high", "low", "close", "volume"] {
        let document = changed(|s| {
            s["indicators"]["quote"][0][field]
                .as_array_mut()
                .unwrap()
                .pop();
        });
        assert!(parse_history_document(&document).is_err());
    }
    let document = changed(|s| s["indicators"]["adjclose"] = json!([]));
    assert!(parse_history_document(&document).is_err());
    let document = changed(|s| {
        s["indicators"]["adjclose"][0]["adjclose"]
            .as_array_mut()
            .unwrap()
            .pop();
    });
    assert!(parse_history_document(&document).is_err());
    let document = changed(|s| s["timestamp"][1] = s["timestamp"][0].clone());
    assert!(matches!(
        parse_history_document(&document),
        Err(HistoryError::DuplicateDate(_))
    ));
}

#[test]
fn uses_warsaw_calendar_dates_in_winter_and_summer_and_sorts_them() {
    for (month, hour, expected) in [(1, 23, "2024-01-02"), (7, 22, "2024-07-02")] {
        let timestamp = Utc
            .with_ymd_and_hms(2024, month, 1, hour, 30, 0)
            .unwrap()
            .timestamp();
        let mut document = changed(|s| s["timestamp"][0] = json!(timestamp));
        document.fetched_at = Utc.with_ymd_and_hms(2024, 7, 6, 12, 0, 0).unwrap();
        let history = parse_history_document(&document).unwrap();
        assert!(
            history
                .candles
                .iter()
                .any(|c| c.date.to_string() == expected)
        );
        assert!(history.candles.windows(2).all(|p| p[0].date < p[1].date));
    }
}

#[test]
fn excludes_the_current_session_until_close_and_the_publication_margin() {
    let mut document = raw();
    for (hour, minute, expected) in [(7, 0, 2), (12, 0, 2), (16, 5, 2), (16, 19, 2), (16, 20, 3)] {
        document.fetched_at = Utc.with_ymd_and_hms(2024, 1, 4, hour, minute, 0).unwrap();
        let history = parse_history_document(&document).unwrap();
        assert_eq!(history.candles.len(), expected);
        if expected == 2 {
            assert_eq!(
                history.skipped_candles[0].reason,
                SkippedCandleReason::UnfinishedSession
            );
        }
    }
    document = changed(|s| s["meta"]["currentTradingPeriod"] = Value::Null);
    document.fetched_at = Utc.with_ymd_and_hms(2024, 1, 4, 18, 0, 0).unwrap();
    assert_eq!(parse_history_document(&document).unwrap().candles.len(), 2);
}

#[test]
fn does_not_replace_missing_daily_close_with_a_latest_quote() {
    let document = changed(|s| {
        s["indicators"]["quote"][0]["close"][2] = Value::Null;
        s["meta"]["regularMarketPrice"] = json!(999);
    });
    let history = parse_history_document(&document).unwrap();
    assert_eq!(
        history.candles.last().unwrap().date.to_string(),
        "2024-01-03"
    );
    assert_eq!(
        history.skipped_candles[0].reason,
        SkippedCandleReason::MissingValues
    );
}

#[test]
fn rejects_http_api_errors_empty_series_and_future_dates() {
    let mut document = raw();
    document.http_status = 429;
    assert!(matches!(
        parse_history_document(&document),
        Err(HistoryError::HttpStatus(429))
    ));
    document = raw();
    document.body =
        r#"{"chart":{"result":null,"error":{"code":"Not Found","description":"No data found"}}}"#
            .into();
    assert!(
        parse_history_document(&document)
            .unwrap_err()
            .to_string()
            .contains("Not Found")
    );
    for body in [
        "<html>Access denied</html>",
        r#"{"chart":{"result":[],"error":null}}"#,
    ] {
        document.body = body.into();
        assert!(parse_history_document(&document).is_err());
    }
    document = changed(|s| s["timestamp"][0] = json!(2000000000));
    assert!(parse_history_document(&document).is_err());
}
