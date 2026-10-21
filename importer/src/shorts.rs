//! Pure parsing of the current KNF register. No network, storage or scoring.

use std::collections::BTreeMap;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use scraper::Html;
use serde::Deserialize;
use thiserror::Error;

use crate::model::{RawDocument, ShortPosition, Source};

#[derive(Debug, Error)]
pub enum ShortsError {
    #[error("HTTP {0}; response does not contain a successful KNF page")]
    HttpStatus(u16),
    #[error("expected a KNF short-position response")]
    InvalidSource,
    #[error("raw document is missing the company code or ISIN")]
    MissingInstrument,
    #[error("invalid KNF JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid KNF response: {0}")]
    InvalidResponse(&'static str),
    #[error("KNF returned a different ISIN: {0}")]
    WrongIsin(String),
}

#[derive(Debug)]
pub struct ShortsPage {
    pub total: usize,
    pub positions: Vec<ShortPosition>,
}

#[derive(Deserialize)]
struct Response {
    status: String,
    total: usize,
    records: Vec<Record>,
}

#[derive(Deserialize)]
struct Record {
    #[serde(rename = "ISIN")]
    isin: String,
    #[serde(rename = "HOLDER_FULL_NAME")]
    holder: String,
    #[serde(rename = "NET_SHORT_POSITION_O")]
    percent: Decimal,
    #[serde(rename = "POSITION_DATE")]
    position_date: NaiveDate,
    #[serde(rename = "MODIFY_DATE")]
    modify_date: NaiveDate,
}

/// Parse one page, retaining its row count for pagination before deduplication.
pub fn parse_page(raw: &RawDocument) -> Result<ShortsPage, ShortsError> {
    if !(200..300).contains(&raw.http_status) {
        return Err(ShortsError::HttpStatus(raw.http_status));
    }
    if raw.source != Source::KnfShorts {
        return Err(ShortsError::InvalidSource);
    }
    let instrument = raw
        .instrument
        .as_ref()
        .filter(|company| !company.code.trim().is_empty() && !company.isin.trim().is_empty())
        .ok_or(ShortsError::MissingInstrument)?;
    let response: Response = serde_json::from_str(&raw.body)?;
    if response.status != "success" {
        return Err(ShortsError::InvalidResponse("status is not success"));
    }
    if response.records.len() > response.total
        || (response.total > 0 && response.records.is_empty())
    {
        return Err(ShortsError::InvalidResponse(
            "inconsistent total and records",
        ));
    }

    let mut positions = Vec::new();
    for record in response.records {
        // The endpoint uses a contains filter; require an exact ISIN match here.
        if record.isin.trim() != instrument.isin {
            return Err(ShortsError::WrongIsin(record.isin));
        }
        // KNF returns HTML entities such as &amp; in holder names.
        let holder = Html::parse_fragment(&record.holder)
            .root_element()
            .text()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if holder.is_empty() || record.percent < Decimal::ZERO {
            return Err(ShortsError::InvalidResponse(
                "empty holder or negative position",
            ));
        }
        positions.push(ShortPosition {
            holder,
            net_short_position_percent: record.percent,
            position_date: record.position_date,
            modify_date: record.modify_date,
        });
    }
    Ok(ShortsPage {
        total: response.total,
        positions,
    })
}

/// Select the latest position per holder, then the latest correction of that date.
/// Older reports of the same position must never be added together.
pub fn latest_positions(positions: Vec<ShortPosition>) -> Vec<ShortPosition> {
    let mut latest: BTreeMap<String, ShortPosition> = BTreeMap::new();
    for position in positions {
        let key = position.holder.to_uppercase();
        if let Some(previous) = latest.get(&key)
            && (previous.position_date, previous.modify_date)
                >= (position.position_date, position.modify_date)
        {
            continue;
        }
        latest.insert(key, position);
    }
    latest.into_values().collect()
}
