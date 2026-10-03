//! Technical v1: calendar-month seasonality points plus SMA200 trend points.
//! Only daily closes strictly before the ranking date participate.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, de};
use thiserror::Error;

const SMA_SESSIONS: usize = 200;

/// Projection of history.json. Provider metadata and adjusted_close are ignored.
#[derive(Debug, Deserialize)]
pub struct TechnicalHistory {
    pub candles: Vec<CloseCandle>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CloseCandle {
    pub date: NaiveDate,
    #[serde(deserialize_with = "deserialize_close")]
    pub close: Decimal,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TechnicalScore {
    pub as_of: NaiveDate,
    pub seasonality_points: Option<i8>,
    pub trend_points: i8,
    /// An incomplete sum remains missing when seasonality is unavailable.
    pub technical_score: Option<i8>,
    pub explanation: TechnicalExplanation,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TechnicalExplanation {
    pub r: Option<Decimal>,
    pub years: usize,
    pub last_session: Option<NaiveDate>,
    pub close: Option<Decimal>,
    pub sma200: Option<Decimal>,
    pub d: Option<Decimal>,
    pub trend_sessions: usize,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TechnicalError {
    #[error("duplicate daily close for {0}")]
    DuplicateDate(NaiveDate),
    #[error("close must be positive on {0}")]
    InvalidClose(NaiveDate),
    #[error("technical calculation exceeds Decimal precision/range")]
    Arithmetic,
}

/// Evaluate one instrument without filesystem, network, or database access.
pub fn score(candles: &[CloseCandle], as_of: NaiveDate) -> Result<TechnicalScore, TechnicalError> {
    let mut past: Vec<_> = candles
        .iter()
        .filter(|candle| candle.date < as_of)
        .collect();
    past.sort_unstable_by_key(|candle| candle.date);
    for (index, candle) in past.iter().enumerate() {
        if candle.close <= Decimal::ZERO {
            return Err(TechnicalError::InvalidClose(candle.date));
        }
        if index > 0 && past[index - 1].date == candle.date {
            return Err(TechnicalError::DuplicateDate(candle.date));
        }
    }

    let (r, years) = seasonality(&past, as_of)?;
    let seasonality_points = r.map(|r| {
        if r > Decimal::new(2, 2) {
            1
        } else if r < Decimal::new(-2, 2) {
            -1
        } else {
            0
        }
    });
    let last = past.last();
    let window = &past[past.len().saturating_sub(SMA_SESSIONS)..];
    let mut sma200 = None;
    let mut d = None;
    let mut trend_points = 0;
    if window.len() == SMA_SESSIONS {
        let sum = window.iter().try_fold(Decimal::ZERO, |sum, candle| {
            sum.checked_add(candle.close)
                .ok_or(TechnicalError::Arithmetic)
        })?;
        let sma = sum
            .checked_div(Decimal::from(SMA_SESSIONS as u32))
            .ok_or(TechnicalError::Arithmetic)?;
        let close = last.expect("a 200-session window is nonempty").close;
        let difference = close
            .checked_sub(sma)
            .ok_or(TechnicalError::Arithmetic)?
            .abs();
        let distance = difference
            .checked_div(sma)
            .ok_or(TechnicalError::Arithmetic)?;
        let band = sma
            .checked_mul(Decimal::new(3, 2))
            .ok_or(TechnicalError::Arithmetic)?;
        // Compare before division so rounding of d cannot move a band boundary.
        if difference >= band {
            trend_points = if close > sma { 1 } else { -1 };
        }
        sma200 = Some(sma);
        d = Some(distance);
    }

    Ok(TechnicalScore {
        as_of,
        seasonality_points,
        trend_points,
        technical_score: seasonality_points.map(|points| points + trend_points),
        explanation: TechnicalExplanation {
            r,
            years,
            last_session: last.map(|candle| candle.date),
            close: last.map(|candle| candle.close),
            sma200,
            d,
            trend_sessions: window.len(),
        },
    })
}

fn seasonality(
    candles: &[&CloseCandle],
    as_of: NaiveDate,
) -> Result<(Option<Decimal>, usize), TechnicalError> {
    // Sorted input leaves the last available session of each calendar month.
    let month_ends: BTreeMap<_, _> = candles
        .iter()
        .map(|candle| ((candle.date.year(), candle.date.month()), candle.close))
        .collect();
    let mut sum = Decimal::ZERO;
    let mut years = 0_u32;
    for (&(year, month), &close) in &month_ends {
        if month != as_of.month() || year >= as_of.year() {
            continue;
        }
        let previous = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        let Some(&previous_close) = month_ends.get(&previous) else {
            continue;
        };
        let monthly_return = close
            .checked_div(previous_close)
            .and_then(|ratio| ratio.checked_sub(Decimal::ONE))
            .ok_or(TechnicalError::Arithmetic)?;
        sum = sum
            .checked_add(monthly_return)
            .ok_or(TechnicalError::Arithmetic)?;
        years += 1;
    }
    let mean = if years == 0 {
        None
    } else {
        Some(
            sum.checked_div(Decimal::from(years))
                .ok_or(TechnicalError::Arithmetic)?,
        )
    };
    Ok((mean, years as usize))
}

fn deserialize_close<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    // Accept both importer decimal strings and precise JSON numeric literals.
    let value = serde_json::Value::deserialize(deserializer)?;
    let digits = match value {
        serde_json::Value::String(value) => value,
        serde_json::Value::Number(value) => value.to_string(),
        _ => {
            return Err(de::Error::custom(
                "close must be a decimal string or number",
            ));
        }
    };
    Decimal::from_str_exact(&digits).map_err(de::Error::custom)
}
