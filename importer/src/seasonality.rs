//! Presentation of the existing monthly statistic and first-session-normalized paths.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;

use crate::technical::{self, CloseCandle, TechnicalError};

pub(crate) struct Seasonality {
    pub mean: Option<Decimal>,
    pub years: usize,
    pub current: Vec<Decimal>,
    pub all: Vec<Decimal>,
    pub five: Option<Vec<Decimal>>,
    pub ten: Option<Vec<Decimal>>,
}

pub(crate) fn calculate(
    candles: &[CloseCandle],
    as_of: NaiveDate,
) -> Result<Seasonality, TechnicalError> {
    // Reuse the scorer's mean, eligibility and price convention without changing points.
    let score = technical::score(candles, as_of)?;
    let mut months = BTreeMap::<(i32, u32), Vec<&CloseCandle>>::new();
    for candle in candles.iter().filter(|candle| candle.date <= as_of) {
        months
            .entry((candle.date.year(), candle.date.month()))
            .or_default()
            .push(candle);
    }
    for month in months.values_mut() {
        month.sort_by_key(|candle| candle.date);
    }
    let previous_month = |year| {
        if as_of.month() == 1 {
            (year - 1, 12)
        } else {
            (year, as_of.month() - 1)
        }
    };
    let mut completed = Vec::new();
    for (&(year, month), candles) in months.iter().rev() {
        if month == as_of.month()
            && year < as_of.year()
            && months.contains_key(&previous_month(year))
        {
            completed.push(normalize(candles)?);
        }
    }
    let current = months
        .get(&(as_of.year(), as_of.month()))
        .map(|candles| normalize(candles))
        .transpose()?
        .unwrap_or_default();
    Ok(Seasonality {
        mean: score.explanation.r,
        years: score.explanation.years,
        current,
        all: mean_path(&completed)?,
        five: if completed.len() >= 5 {
            Some(mean_path(&completed[..5])?)
        } else {
            None
        },
        ten: if completed.len() >= 10 {
            Some(mean_path(&completed[..10])?)
        } else {
            None
        },
    })
}

fn normalize(candles: &[&CloseCandle]) -> Result<Vec<Decimal>, TechnicalError> {
    let price = |candle: &CloseCandle| candle.adjusted_close.unwrap_or(candle.close);
    let Some(first) = candles.first() else {
        return Ok(Vec::new());
    };
    if price(first) <= Decimal::ZERO {
        return Err(TechnicalError::InvalidClose(first.date));
    }
    candles
        .iter()
        .map(|candle| {
            if price(candle) <= Decimal::ZERO {
                return Err(TechnicalError::InvalidClose(candle.date));
            }
            price(candle)
                .checked_div(price(first))
                .and_then(|ratio| ratio.checked_sub(Decimal::ONE))
                .ok_or(TechnicalError::Arithmetic)
        })
        .collect()
}

fn mean_path(paths: &[Vec<Decimal>]) -> Result<Vec<Decimal>, TechnicalError> {
    // Use only session indices observed in every selected year; never extend shorter months.
    let sessions = paths.iter().map(Vec::len).min().unwrap_or(0);
    (0..sessions)
        .map(|session| {
            paths
                .iter()
                .try_fold(Decimal::ZERO, |sum, path| {
                    sum.checked_add(path[session])
                        .ok_or(TechnicalError::Arithmetic)
                })?
                .checked_div(Decimal::from(paths.len() as u32))
                .ok_or(TechnicalError::Arithmetic)
        })
        .collect()
}

pub(crate) fn month_name(month: u32) -> &'static str {
    [
        "Styczeń",
        "Luty",
        "Marzec",
        "Kwiecień",
        "Maj",
        "Czerwiec",
        "Lipiec",
        "Sierpień",
        "Wrzesień",
        "Październik",
        "Listopad",
        "Grudzień",
    ][(month - 1) as usize]
}
