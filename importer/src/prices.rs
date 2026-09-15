//! Pure parsers for current market quotes. No network or filesystem access.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::Warsaw;
use rust_decimal::{Decimal, prelude::ToPrimitive};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use serde_json::Number;
use thiserror::Error;

use crate::{
    model::{MarketPair, PriceInstrument, PriceSnapshot, PriceUnit, RawDocument, Source},
    parser::parse_number,
};

#[derive(Debug, Error)]
pub enum PriceError {
    #[error("HTTP {0}; response does not contain a successful quote")]
    HttpStatus(u16),
    #[error("expected a GPW or TradingView price response")]
    InvalidSource,
    #[error("missing or invalid price field: {0}")]
    InvalidField(&'static str),
    #[error("quote instrument does not match the requested instrument")]
    InstrumentMismatch,
    #[error("invalid price JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid OHLC range")]
    InvalidRange,
}

pub fn parse_price_document(raw: &RawDocument) -> Result<PriceSnapshot, PriceError> {
    if !(200..300).contains(&raw.http_status) {
        return Err(PriceError::HttpStatus(raw.http_status));
    }
    let snapshot = match raw.source {
        Source::GpwPrices => parse_gpw(raw)?,
        Source::TradingViewIdc => parse_tradingview(raw)?,
        _ => return Err(PriceError::InvalidSource),
    };
    validate(&snapshot)?;
    Ok(snapshot)
}

fn parse_gpw(raw: &RawDocument) -> Result<PriceSnapshot, PriceError> {
    let instrument = raw.instrument.as_ref().ok_or(PriceError::InvalidField("instrument"))?;
    let document = Html::parse_document(&raw.body);
    let identity = one_text(&document, "#getH1", "instrument identity")?;
    if identity != format!("{} ({})", instrument.code, instrument.isin) {
        return Err(PriceError::InstrumentMismatch);
    }

    let updated = one_text(&document, ".currentTimeMin", "last update time")?;
    let updated = updated.strip_prefix("Ostatnia aktualizacja:")
        .ok_or(PriceError::InvalidField("last update time"))?.trim();
    let local = NaiveDateTime::parse_from_str(updated, "%d-%m-%Y %H:%M")
        .map_err(|_| PriceError::InvalidField("last update time"))?;
    let updated_at = Warsaw.from_local_datetime(&local).single()
        .ok_or(PriceError::InvalidField("ambiguous last update time"))?.with_timezone(&Utc);

    let price = polish_number(&one_text(&document, ".PaL.header .summary", "price")?, "price")?
        .ok_or(PriceError::InvalidField("price"))?;
    let range = one_text(&document, ".PaL.header .max_min", "daily range")?;
    let (low, high) = range.strip_prefix("min ").and_then(|value| value.split_once("max "))
        .ok_or(PriceError::InvalidField("daily range"))?;

    let rows = Selector::parse("table tr").expect("static selector");
    let cells = Selector::parse("th, td").expect("static selector");
    let mut values = BTreeMap::new();
    for row in document.select(&rows) {
        let cells: Vec<_> = row.select(&cells).map(text).collect();
        let [label, value] = cells.as_slice() else { continue };
        let field = match label.as_str() {
            "Kurs otwarcia" => "open",
            "Wol. obrotu (szt.)" => "volume",
            "Obroty (tys. zł)" => "turnover_pln",
            _ => continue,
        };
        if values.insert(field, value.clone()).is_some() {
            return Err(PriceError::InvalidField(field));
        }
    }
    // This source is the primary GPW listing, quoted in PLN. Reject pages
    // without its PLN turnover label instead of silently accepting another view.
    if !values.contains_key("turnover_pln") {
        return Err(PriceError::InvalidField("PLN quotation"));
    }
    let open = values.get("open").map(|value| polish_number(value, "open")).transpose()?.flatten();
    let volume = values.get("volume").map(|value| polish_number(value, "volume")).transpose()?.flatten();
    let volume = volume.map(|value| {
        if !value.fract().is_zero() { return Err(PriceError::InvalidField("volume")); }
        value.to_u64().ok_or(PriceError::InvalidField("volume"))
    }).transpose()?;

    Ok(PriceSnapshot {
        instrument: PriceInstrument {
            code: instrument.code.clone(),
            isin: Some(instrument.isin.clone()),
            currency: "PLN".into(),
            unit: PriceUnit::Share,
        },
        source: raw.source,
        source_url: raw.url.clone(),
        fetched_at: raw.fetched_at,
        source_updated_at: Some(updated_at),
        period_started_at: None,
        price,
        open,
        high: polish_number(high, "high")?,
        low: polish_number(low, "low")?,
        volume,
    })
}

fn parse_tradingview(raw: &RawDocument) -> Result<PriceSnapshot, PriceError> {
    #[derive(Deserialize)]
    struct Quote {
        name: String,
        currency: String,
        close: Number,
        open: Option<Number>,
        high: Option<Number>,
        low: Option<Number>,
        time: i64,
    }

    let quote: Quote = serde_json::from_str(&raw.body)?;
    let pair = MarketPair::from_code(&quote.name).ok_or(PriceError::InstrumentMismatch)?;
    let requested = reqwest::Url::parse(&raw.url).map_err(|_| PriceError::InstrumentMismatch)?;
    let symbols: Vec<_> = requested.query_pairs().filter(|(key, _)| key == "symbol")
        .map(|(_, value)| value.into_owned()).collect();
    if symbols != [format!("FX_IDC:{}", pair.code())]
        || raw.instrument.is_some()
        || quote.currency != pair.instrument().currency
    {
        return Err(PriceError::InstrumentMismatch);
    }

    let period_started_at = DateTime::from_timestamp(quote.time, 0)
        .filter(|time| time.timestamp() > 0)
        .ok_or(PriceError::InvalidField("daily bar start"))?;
    Ok(PriceSnapshot {
        instrument: pair.instrument(),
        source: raw.source,
        source_url: raw.url.clone(),
        fetched_at: raw.fetched_at,
        source_updated_at: None,
        period_started_at: Some(period_started_at),
        price: json_number(&quote.close, "price")?,
        open: quote.open.as_ref().map(|value| json_number(value, "open")).transpose()?,
        high: quote.high.as_ref().map(|value| json_number(value, "high")).transpose()?,
        low: quote.low.as_ref().map(|value| json_number(value, "low")).transpose()?,
        volume: None,
    })
}

fn validate(snapshot: &PriceSnapshot) -> Result<(), PriceError> {
    for (field, value) in [
        ("price", Some(snapshot.price)), ("open", snapshot.open),
        ("high", snapshot.high), ("low", snapshot.low),
    ] {
        if value.is_some_and(|value| value <= Decimal::ZERO) {
            return Err(PriceError::InvalidField(field));
        }
    }
    for value in [Some(snapshot.price), snapshot.open].into_iter().flatten() {
        if snapshot.low.is_some_and(|low| value < low)
            || snapshot.high.is_some_and(|high| value > high)
        {
            return Err(PriceError::InvalidRange);
        }
    }
    if snapshot.source_updated_at.is_some_and(|time| time > snapshot.fetched_at)
        || snapshot.period_started_at.is_some_and(|time| time > snapshot.fetched_at)
    {
        return Err(PriceError::InvalidField("source time is later than fetch time"));
    }
    Ok(())
}

fn polish_number(value: &str, field: &'static str) -> Result<Option<Decimal>, PriceError> {
    parse_number(value).map_err(|_| PriceError::InvalidField(field))
}

fn json_number(value: &Number, field: &'static str) -> Result<Decimal, PriceError> {
    // serde_json's arbitrary_precision feature preserves the original digits.
    Decimal::from_str_exact(&value.to_string()).map_err(|_| PriceError::InvalidField(field))
}

fn one_text(document: &Html, css: &str, field: &'static str) -> Result<String, PriceError> {
    let selector = Selector::parse(css).expect("static selector");
    let mut elements = document.select(&selector);
    let first = elements.next().ok_or(PriceError::InvalidField(field))?;
    if elements.next().is_some() { return Err(PriceError::InvalidField(field)); }
    Ok(text(first))
}

fn text(element: ElementRef<'_>) -> String {
    element.text().flat_map(str::split_whitespace).collect::<Vec<_>>().join(" ")
}
