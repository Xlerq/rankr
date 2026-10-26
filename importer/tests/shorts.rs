use chrono::{TimeZone, Utc};
use rankr::{
    model::{Instrument, RawDocument, ShortsSnapshot, Source},
    shorts::{ShortsError, latest_positions, parse_page},
};
use serde_json::{Value, json};

fn raw(body: &str) -> RawDocument {
    RawDocument {
        source: Source::KnfShorts,
        instrument: Some(Instrument {
            code: "BUDIMEX".into(),
            isin: "PLBUDMX00013".into(),
            name: "Budimex SA".into(),
        }),
        fetched_at: Utc.with_ymd_and_hms(2026, 10, 21, 12, 0, 0).unwrap(),
        url: "https://rss.knf.gov.pl/rss_pub/JSON".into(),
        http_status: 200,
        body: body.into(),
    }
}

#[test]
fn keeps_latest_position_then_latest_correction_per_holder_without_summing() {
    let raw = raw(include_str!("fixtures/knf_shorts.json"));
    let page = parse_page(&raw).unwrap();
    assert_eq!(page.total, 4);
    assert_eq!(page.positions.len(), 4);
    let mut reversed = page.positions.clone();
    reversed.reverse();

    let positions = latest_positions(page.positions);
    assert_eq!(positions, latest_positions(reversed));
    assert_eq!(positions.len(), 2);
    assert_eq!(positions[0].holder, "Alpha Capital");
    assert_eq!(positions[0].net_short_position_percent.to_string(), "0.63");
    assert_eq!(positions[0].position_date.to_string(), "2026-10-18");
    assert_eq!(positions[0].modify_date.to_string(), "2026-10-20");
    assert_eq!(positions[1].holder, "Beta & Partners");
}

#[test]
fn percentages_stay_exact_json_numbers_in_percent_units() {
    let raw = raw(include_str!("fixtures/knf_shorts.json"));
    let positions = latest_positions(parse_page(&raw).unwrap().positions);
    let json = serde_json::to_value(positions).unwrap();
    assert!(json[0]["net_short_position_percent"].is_number());
    assert_eq!(json[0]["net_short_position_percent"].to_string(), "0.63");
    assert_eq!(
        json[1]["net_short_position_percent"].to_string(),
        "1.234567890123456789"
    );
}

#[test]
fn no_entry_is_an_empty_list_with_company_and_fetch_metadata() {
    let raw = raw(include_str!("fixtures/knf_empty.json"));
    let page = parse_page(&raw).unwrap();
    assert_eq!(page.total, 0);
    let instrument = raw.instrument.unwrap();
    let snapshot = ShortsSnapshot {
        code: instrument.code,
        isin: instrument.isin,
        fetched_at: raw.fetched_at,
        positions: latest_positions(page.positions),
    };
    assert_eq!(
        serde_json::to_value(snapshot).unwrap(),
        json!({
            "code": "BUDIMEX",
            "isin": "PLBUDMX00013",
            "fetched_at": "2026-10-21T12:00:00Z",
            "positions": []
        })
    );
}

#[test]
fn errors_and_incomplete_responses_do_not_become_empty_positions() {
    for body in [
        "not JSON",
        r#"{"status":"error","message":"unavailable"}"#,
        r#"{"status":"error","total":0,"records":[]}"#,
        r#"{"status":"success"}"#,
        r#"{"status":"success","total":1,"records":[]}"#,
        r#"{"status":"success","total":0,"records":null}"#,
    ] {
        assert!(parse_page(&raw(body)).is_err(), "accepted {body}");
    }
    let mut document = raw(include_str!("fixtures/knf_empty.json"));
    document.http_status = 503;
    assert!(matches!(
        parse_page(&document),
        Err(ShortsError::HttpStatus(503))
    ));
    document.http_status = 200;
    document.source = Source::YahooFinance;
    assert!(matches!(
        parse_page(&document),
        Err(ShortsError::InvalidSource)
    ));
    document.source = Source::KnfShorts;
    document.instrument = None;
    assert!(matches!(
        parse_page(&document),
        Err(ShortsError::MissingInstrument)
    ));
}

#[test]
fn rejects_wrong_issuer_missing_fields_and_invalid_values() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/knf_shorts.json")).unwrap();
    for (field, value) in [
        ("ISIN", json!("PLKGHM000017")),
        ("HOLDER_FULL_NAME", json!(" \t ")),
        ("NET_SHORT_POSITION_O", json!("-0.5")),
        ("NET_SHORT_POSITION_O", Value::Null),
        ("NET_SHORT_POSITION_O", json!("unknown")),
        ("POSITION_DATE", json!("2026-02-30")),
        ("MODIFY_DATE", Value::Null),
    ] {
        let mut input = fixture.clone();
        input["records"][0][field] = value;
        assert!(
            parse_page(&raw(&input.to_string())).is_err(),
            "accepted {field}"
        );
    }
    let mut input = fixture;
    input["records"][0]
        .as_object_mut()
        .unwrap()
        .remove("NET_SHORT_POSITION_O");
    assert!(parse_page(&raw(&input.to_string())).is_err());
}
