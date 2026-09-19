use chrono::{NaiveDate, TimeZone, Utc};
use rankr_import::{
    history::{HistoryError, YahooSymbolMap, parse_daily_csv, parse_history_document},
    model::{Instrument, RawDocument, Source},
    storage::{archive_raw, read_raw},
};

const CSV: &str = include_str!("fixtures/stooq_daily.csv");

fn raw() -> RawDocument {
    RawDocument {
        source: Source::Stooq,
        instrument: Some(
            YahooSymbolMap::bundled()
                .unwrap()
                .instrument("KGHM")
                .unwrap(),
        ),
        fetched_at: Utc.with_ymd_and_hms(2026, 9, 17, 18, 0, 0).unwrap(),
        url: "https://stooq.com/q/d/l/?s=kgh&i=d".into(),
        http_status: 200,
        body: CSV.into(),
    }
}

#[test]
fn preserves_csv_precision_fractional_volume_and_close_scale() {
    let history = parse_history_document(&raw()).unwrap();
    assert_eq!(history.source, Source::Stooq);
    assert_eq!(history.instrument.code, "KGHM");
    assert_eq!(history.symbol, "kgh");
    assert_eq!(history.candles.len(), 3);
    assert_eq!(
        history.candles[0].date,
        NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()
    );
    let json = serde_json::to_value(&history).unwrap();
    assert_eq!(json["source"], "stooq");
    assert_eq!(json["candles"][0]["open"], "121.871");
    assert_eq!(json["candles"][0]["high"], "122.614");
    assert_eq!(json["candles"][0]["low"], "117.812");
    assert_eq!(json["candles"][0]["close"], "118.95100");
    assert_eq!(json["candles"][0]["volume"], "475373.24396983");
    assert!(json.get("price").is_none());
}

#[test]
fn handles_polish_headers_bom_crlf_and_sorts_without_filling_gaps() {
    let csv = CSV
        .replace(
            "Date,Open,High,Low,Close,Volume",
            "\u{feff}Data,Otwarcie,Najwyzszy,Najnizszy,Zamkniecie,Wolumen",
        )
        .replace("2024-01-03", "2009-01-02")
        .replace('\n', "\r\n");
    let candles = parse_daily_csv(&csv).unwrap();
    assert_eq!(candles.len(), 3);
    assert_eq!(candles[0].date.to_string(), "2009-01-02");
    assert_eq!(candles[2].date.to_string(), "2024-01-04");
}

#[test]
fn rejects_invalid_ranges_missing_values_and_lossy_numbers() {
    for row in [
        "2024-01-02,3,2,1,1,10", // open above high
        "2024-01-02,0,2,1,1,10", // open below low
        "2024-01-02,1,2,1,3,10", // close above high
        "2024-01-02,1,2,1,0,10", // close below low
        "2024-01-02,1,0,2,1,10", // high below low
        "2024-01-02,1,2,-1,1,10",
        "2024-01-02,1,2,1,1,-10",
        "2024-01-02,1,2,1,NaN,10",
        "2024-01-02,1,2,1,,10",
        "2024-01-02,1,2,1,1",
        "2024-01-02,1,2,1,1,10,extra",
        "2024-02-30,1,2,1,1,10",
        "2024-01-02,1,2,1,1.00000000000000000000000000001,10",
    ] {
        let csv = format!("Date,Open,High,Low,Close,Volume\n{row}\n");
        assert!(
            parse_daily_csv(&csv).is_err(),
            "accepted invalid row: {row}"
        );
    }
}

#[test]
fn rejects_empty_responses_duplicate_dates_and_non_daily_documents() {
    for csv in [
        "",
        "No data",
        "<html>Check your API key</html>",
        "Date,Open,High,Low,Close,Volume\n",
    ] {
        assert!(parse_daily_csv(csv).is_err());
    }
    assert!(matches!(
        parse_daily_csv(&CSV.replace("2024-01-03", "2024-01-02")),
        Err(HistoryError::DuplicateDate(_))
    ));
    let mut document = raw();
    document.http_status = 403;
    assert!(matches!(
        parse_history_document(&document),
        Err(HistoryError::HttpStatus(403))
    ));
    document.http_status = 200;
    document.source = Source::GpwPrices;
    assert!(matches!(
        parse_history_document(&document),
        Err(HistoryError::InvalidSource)
    ));
    document.source = Source::Stooq;
    document.url = document.url.replace("i=d", "i=m");
    assert!(parse_history_document(&document).is_err());
}

#[test]
fn diagnoses_stooq_access_errors_returned_with_http_200() {
    let mut document = raw();
    document.body = "\u{feff}Access denied\r\n".into();
    assert!(matches!(
        parse_history_document(&document),
        Err(HistoryError::AccessDenied)
    ));

    for body in [
        "<!DOCTYPE html><html><body><noscript>This site requires JavaScript to verify your browser. Please enable JavaScript and reload.</noscript></body></html>",
        "<!DOCTYPE html><html><body><noscript>Ta strona wymaga JavaScriptu do weryfikacji przeglądarki. Włącz JavaScript i odśwież stronę.</noscript></body></html>",
        "<html><script>fetch(\"/__verify\", {method: \"POST\"})</script></html>",
    ] {
        document.body = body.into();
        assert!(matches!(
            parse_history_document(&document),
            Err(HistoryError::BrowserVerificationRequired)
        ));
    }

    // An unrelated HTML response must not be diagnosed as browser verification.
    document.body = "<html>Service unavailable</html>".into();
    assert!(matches!(
        parse_history_document(&document),
        Err(HistoryError::InvalidHeader)
    ));
}

#[test]
fn resolves_gpw_codes_and_checks_isin_instead_of_guessing_symbols() {
    let symbols = YahooSymbolMap::bundled().unwrap();
    let kghm = symbols.instrument("kGhM").unwrap();
    assert_eq!(symbols.symbol_for(&kghm).unwrap(), "KGH.WA");
    assert_eq!(
        symbols
            .symbol_for(&symbols.instrument("PKOBP").unwrap())
            .unwrap(),
        "PKO.WA"
    );
    assert!(symbols.symbol_for(&Instrument::wig20_index()).is_err());
    assert!(symbols.instrument("kgh").is_err());
    let mut wrong_isin = kghm;
    wrong_isin.isin = "PL0000000000".into();
    assert!(matches!(
        symbols.symbol_for(&wrong_isin),
        Err(HistoryError::MissingMapping { .. })
    ));
    assert!(
        symbols
            .symbol_for(&Instrument {
                code: "NEWCOMPANY".into(),
                isin: "PL0000000000".into(),
                name: "New company".into(),
            })
            .is_err()
    );
}

#[test]
fn rejects_ambiguous_or_empty_symbol_maps() {
    let header = "gpw_code,isin,name,yahoo_symbol\n";
    let row = "KGHM,PLKGHM000017,KGHM,KGH.WA\n";
    assert!(YahooSymbolMap::from_csv(header).is_err());
    assert!(YahooSymbolMap::from_csv(&format!("{header}{row}{row}")).is_err());
    assert!(YahooSymbolMap::from_csv(&format!("{header}KGHM,PLKGHM000017,KGHM,\n")).is_err());
}

#[test]
fn history_archive_is_immutable_and_retains_the_original_csv() {
    let directory = tempfile::tempdir().unwrap();
    let document = raw();
    let archived = archive_raw(directory.path(), &document).unwrap();
    assert_eq!(read_raw(&archived.raw_path()).unwrap().body, CSV);
    let history = parse_history_document(&document).unwrap();
    let path = archived.save_history(&history).unwrap();
    let saved: rankr_import::model::DailyPriceHistory =
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(saved, history);
    assert!(archived.save_history(&history).is_err());
}
