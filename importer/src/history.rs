//! Pure Stooq symbol mapping and daily CSV parsing. No HTTP or filesystem access.

use std::collections::HashSet;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use thiserror::Error;

use crate::model::{DailyCandle, DailyPriceHistory, Instrument, RawDocument, Source};

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("HTTP {0}; response does not contain successful daily history")]
    HttpStatus(u16),
    #[error("expected a Stooq daily history response")]
    InvalidSource,
    #[error("raw daily history is missing the instrument or daily Stooq symbol")]
    MissingInstrument,
    #[error("unexpected Stooq CSV header; check STOOQ_API_KEY and the symbol")]
    InvalidHeader,
    #[error("Stooq returned no daily candles; check STOOQ_API_KEY and the symbol")]
    EmptyHistory,
    #[error("invalid CSV: {0}")]
    Csv(#[from] csv::Error),
    #[error("invalid {field} in CSV row {row}")]
    InvalidField { row: usize, field: &'static str },
    #[error("invalid OHLC range in CSV row {0}")]
    InvalidRange(usize),
    #[error("duplicate daily candle for {0}")]
    DuplicateDate(NaiveDate),
    #[error("empty or ambiguous GPW/Stooq symbol mapping")]
    InvalidSymbolMap,
    #[error(
        "unknown GPW code {0:?}; use a gpw_code from data/raw/wig20_symbols.csv, not a Stooq ticker"
    )]
    UnknownCode(String),
    #[error(
        "no matching Stooq mapping for GPW code {code} and ISIN {isin}; update data/raw/wig20_symbols.csv and rebuild"
    )]
    MissingMapping { code: String, isin: String },
}

#[derive(Deserialize)]
struct SymbolMapping {
    gpw_code: String,
    isin: String,
    name: String,
    stooq_symbol: String,
}

/// Bundled identifiers survive `cargo install`; constituent selection remains live.
pub struct StooqSymbolMap {
    entries: Vec<SymbolMapping>,
}

impl StooqSymbolMap {
    pub fn bundled() -> Result<Self, HistoryError> {
        Self::from_csv(include_str!("../../data/raw/wig20_symbols.csv"))
    }

    pub fn from_csv(csv: &str) -> Result<Self, HistoryError> {
        let entries: Vec<SymbolMapping> = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(csv.as_bytes())
            .deserialize()
            .collect::<Result<_, _>>()?;
        let mut codes = HashSet::new();
        let mut isins = HashSet::new();
        let mut symbols = HashSet::new();
        if entries.is_empty()
            || entries.iter().any(|entry| {
                entry.gpw_code.is_empty()
                    || entry.isin.is_empty()
                    || entry.name.is_empty()
                    || entry.stooq_symbol.is_empty()
                    || !codes.insert(entry.gpw_code.to_ascii_uppercase())
                    || !isins.insert(&entry.isin)
                    || !symbols.insert(entry.stooq_symbol.to_ascii_lowercase())
            })
        {
            return Err(HistoryError::InvalidSymbolMap);
        }
        Ok(Self { entries })
    }

    pub fn instrument(&self, code: &str) -> Result<Instrument, HistoryError> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.gpw_code.eq_ignore_ascii_case(code))
            .ok_or_else(|| HistoryError::UnknownCode(code.into()))?;
        Ok(Instrument {
            code: entry.gpw_code.clone(),
            isin: entry.isin.clone(),
            name: entry.name.clone(),
        })
    }

    pub fn symbol_for(&self, instrument: &Instrument) -> Result<&str, HistoryError> {
        let index = Instrument::wig20_index();
        if instrument.code == index.code && instrument.isin == index.isin {
            return Ok("wig20");
        }
        self.entries
            .iter()
            .find(|entry| {
                entry.gpw_code.eq_ignore_ascii_case(&instrument.code)
                    && entry.isin == instrument.isin
            })
            .map(|entry| entry.stooq_symbol.as_str())
            .ok_or_else(|| HistoryError::MissingMapping {
                code: instrument.code.clone(),
                isin: instrument.isin.clone(),
            })
    }
}

pub fn parse_history_document(raw: &RawDocument) -> Result<DailyPriceHistory, HistoryError> {
    if !(200..300).contains(&raw.http_status) {
        return Err(HistoryError::HttpStatus(raw.http_status));
    }
    if raw.source != Source::Stooq {
        return Err(HistoryError::InvalidSource);
    }
    let instrument = raw
        .instrument
        .clone()
        .ok_or(HistoryError::MissingInstrument)?;
    let url = reqwest::Url::parse(&raw.url).map_err(|_| HistoryError::MissingInstrument)?;
    let symbols: Vec<_> = url
        .query_pairs()
        .filter(|(key, _)| key == "s")
        .map(|(_, value)| value.into_owned())
        .collect();
    let intervals: Vec<_> = url
        .query_pairs()
        .filter(|(key, _)| key == "i")
        .map(|(_, value)| value.into_owned())
        .collect();
    let [symbol] = symbols.as_slice() else {
        return Err(HistoryError::MissingInstrument);
    };
    if symbol.is_empty() || intervals != ["d"] {
        return Err(HistoryError::MissingInstrument);
    }

    Ok(DailyPriceHistory {
        instrument,
        stooq_symbol: symbol.clone(),
        source: raw.source,
        source_url: raw.url.clone(),
        fetched_at: raw.fetched_at,
        candles: parse_daily_csv(&raw.body)?,
    })
}

/// Preserve source decimals and adjustments. Sort dates without filling gaps.
pub fn parse_daily_csv(body: &str) -> Result<Vec<DailyCandle>, HistoryError> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(body.as_bytes());
    let headers = reader.headers()?;
    let english = ["Date", "Open", "High", "Low", "Close", "Volume"];
    let polish = [
        "Data",
        "Otwarcie",
        "Najwyzszy",
        "Najnizszy",
        "Zamkniecie",
        "Wolumen",
    ];
    if !headers.iter().eq(english) && !headers.iter().eq(polish) {
        return Err(HistoryError::InvalidHeader);
    }

    let mut candles = Vec::new();
    for (index, record) in reader.records().enumerate() {
        let record = record?;
        let row = index + 2;
        let invalid = |field| HistoryError::InvalidField { row, field };
        let date =
            NaiveDate::parse_from_str(&record[0], "%Y-%m-%d").map_err(|_| invalid("date"))?;
        let number = |column: usize, field| {
            Decimal::from_str_exact(&record[column])
                .ok()
                .filter(|value| *value >= Decimal::ZERO)
                .ok_or_else(|| invalid(field))
        };
        let candle = DailyCandle {
            date,
            open: number(1, "open")?,
            high: number(2, "high")?,
            low: number(3, "low")?,
            close: number(4, "close")?,
            volume: number(5, "volume")?,
        };
        if candle.high < candle.low
            || candle.open < candle.low
            || candle.open > candle.high
            || candle.close < candle.low
            || candle.close > candle.high
        {
            return Err(HistoryError::InvalidRange(row));
        }
        candles.push(candle);
    }
    if candles.is_empty() {
        return Err(HistoryError::EmptyHistory);
    }
    candles.sort_unstable_by_key(|candle| candle.date);
    for pair in candles.windows(2) {
        if pair[0].date == pair[1].date {
            return Err(HistoryError::DuplicateDate(pair[0].date));
        }
    }
    Ok(candles)
}
