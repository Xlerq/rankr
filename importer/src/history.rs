//! Bundled GPW/Yahoo identifiers and daily history parsing.

use std::collections::HashSet;

use chrono::NaiveDate;
use serde::Deserialize;
use thiserror::Error;

use crate::model::{DailyPriceHistory, Instrument, RawDocument, Source};

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("HTTP {0}; response does not contain successful daily history")]
    HttpStatus(u16),
    #[error("expected a Yahoo daily history response")]
    InvalidSource,
    #[error("raw daily history is missing its instrument")]
    MissingInstrument,
    #[error("invalid CSV: {0}")]
    Csv(#[from] csv::Error),
    #[error("duplicate daily candle for {0}")]
    DuplicateDate(NaiveDate),
    #[error("invalid Yahoo history: {0}")]
    Yahoo(String),
    #[error("invalid Yahoo JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("empty or ambiguous GPW/Yahoo symbol mapping")]
    InvalidSymbolMap,
    #[error("unknown GPW code {0:?} in data/raw/wig20_symbols.csv")]
    UnknownCode(String),
    #[error("no Yahoo mapping for GPW code {code} and ISIN {isin}")]
    MissingMapping { code: String, isin: String },
}

#[derive(Deserialize)]
struct SymbolMapping {
    gpw_code: String,
    isin: String,
    name: String,
    yahoo_symbol: String,
}

/// Identifiers from the WIG20 map are bundled so they survive `cargo install`.
pub struct YahooSymbolMap {
    entries: Vec<SymbolMapping>,
}

impl YahooSymbolMap {
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
                    || !entry.yahoo_symbol.ends_with(".WA")
                    || entry.yahoo_symbol.len() <= 3
                    || !entry
                        .yahoo_symbol
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.')
                    || !codes.insert(entry.gpw_code.to_ascii_uppercase())
                    || !isins.insert(&entry.isin)
                    || !symbols.insert(entry.yahoo_symbol.to_ascii_lowercase())
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

    pub fn instruments(&self) -> Vec<Instrument> {
        self.entries
            .iter()
            .map(|entry| Instrument {
                code: entry.gpw_code.clone(),
                isin: entry.isin.clone(),
                name: entry.name.clone(),
            })
            .collect()
    }

    pub fn symbol_for(&self, instrument: &Instrument) -> Result<&str, HistoryError> {
        self.entries
            .iter()
            .find(|entry| {
                entry.gpw_code.eq_ignore_ascii_case(&instrument.code)
                    && entry.isin == instrument.isin
            })
            .map(|entry| entry.yahoo_symbol.as_str())
            .ok_or_else(|| HistoryError::MissingMapping {
                code: instrument.code.clone(),
                isin: instrument.isin.clone(),
            })
    }
}

pub fn parse_history_document(raw: &RawDocument) -> Result<DailyPriceHistory, HistoryError> {
    if raw.source != Source::YahooFinance {
        return Err(HistoryError::InvalidSource);
    }
    crate::yahoo::parse_history(raw, false)
}
