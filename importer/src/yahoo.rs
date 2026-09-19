//! Pure parsing of Yahoo's chart response. No HTTP, price adjustment or scoring.

use std::collections::HashSet;

use chrono::{DateTime, Duration, Utc};
use chrono_tz::Europe::Warsaw;
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::Number;

use crate::{
    history::{HistoryError, YahooSymbolMap},
    model::{DailyCandle, DailyPriceHistory, RawDocument, SkippedCandle, SkippedCandleReason},
};

#[derive(Deserialize)]
struct Envelope {
    chart: Chart,
}

#[derive(Deserialize)]
struct Chart {
    result: Option<Vec<Series>>,
    error: Option<ApiError>,
}

#[derive(Deserialize)]
struct ApiError {
    code: String,
    description: String,
}

#[derive(Deserialize)]
struct Series {
    meta: Metadata,
    timestamp: Option<Vec<i64>>,
    indicators: Indicators,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Metadata {
    symbol: String,
    currency: String,
    exchange_name: String,
    exchange_timezone_name: String,
    instrument_type: String,
    data_granularity: String,
    current_trading_period: Option<TradingPeriods>,
}

#[derive(Deserialize)]
struct TradingPeriods {
    regular: TradingPeriod,
}

#[derive(Deserialize)]
struct TradingPeriod {
    start: i64,
    end: i64,
}

#[derive(Deserialize)]
struct Indicators {
    quote: Vec<Quotes>,
    #[serde(default)]
    adjclose: Vec<AdjustedClose>,
}

#[derive(Deserialize)]
struct Quotes {
    open: Vec<Option<Number>>,
    high: Vec<Option<Number>>,
    low: Vec<Option<Number>>,
    close: Vec<Option<Number>>,
    volume: Vec<Option<Number>>,
}

#[derive(Deserialize)]
struct AdjustedClose {
    adjclose: Vec<Option<Number>>,
}

fn invalid(reason: impl Into<String>) -> HistoryError {
    HistoryError::Yahoo(reason.into())
}

pub(crate) fn parse_history(raw: &RawDocument) -> Result<DailyPriceHistory, HistoryError> {
    if !(200..300).contains(&raw.http_status) {
        return Err(HistoryError::HttpStatus(raw.http_status));
    }
    let instrument = raw
        .instrument
        .as_ref()
        .ok_or(HistoryError::MissingInstrument)?;
    let expected = YahooSymbolMap::bundled()?;
    let symbol = expected.symbol_for(instrument)?;
    let url = reqwest::Url::parse(&raw.url).map_err(|_| invalid("invalid source URL"))?;
    let intervals: Vec<_> = url.query_pairs().filter(|(k, _)| k == "interval").collect();
    if url
        .path_segments()
        .and_then(|mut segments| segments.next_back())
        != Some(symbol)
        || intervals.len() != 1
        || intervals[0].1 != "1d"
    {
        return Err(invalid(
            "source URL does not identify the requested daily company history",
        ));
    }

    let envelope: Envelope = serde_json::from_str(&raw.body)?;
    if let Some(error) = envelope.chart.error {
        return Err(invalid(format!("{}: {}", error.code, error.description)));
    }
    let results = envelope
        .chart
        .result
        .ok_or_else(|| invalid("missing chart result"))?;
    let [series] = results.as_slice() else {
        return Err(invalid("expected exactly one chart result"));
    };
    let meta = &series.meta;
    if meta.symbol != symbol
        || meta.currency != "PLN"
        || meta.exchange_name != "WSE"
        || meta.exchange_timezone_name != "Europe/Warsaw"
        || meta.instrument_type != "EQUITY"
        || meta.data_granularity != "1d"
    {
        return Err(invalid(
            "chart metadata does not match a daily GPW equity in PLN",
        ));
    }
    let timestamps = series
        .timestamp
        .as_ref()
        .ok_or_else(|| invalid("missing daily timestamps"))?;
    let [quote] = series.indicators.quote.as_slice() else {
        return Err(invalid("expected exactly one OHLCV series"));
    };
    let [adjusted] = series.indicators.adjclose.as_slice() else {
        return Err(invalid("missing adjusted-close series"));
    };
    let columns = [
        &quote.open,
        &quote.high,
        &quote.low,
        &quote.close,
        &quote.volume,
        &adjusted.adjclose,
    ];
    if columns
        .iter()
        .any(|column| column.len() != timestamps.len())
    {
        return Err(invalid("timestamp and price array lengths differ"));
    }

    let today = raw.fetched_at.with_timezone(&Warsaw).date_naive();
    let mut dates = HashSet::new();
    let mut candles = Vec::with_capacity(timestamps.len());
    let mut skipped_candles = Vec::new();
    for (i, timestamp) in timestamps.iter().enumerate() {
        let timestamp = DateTime::from_timestamp(*timestamp, 0)
            .ok_or_else(|| invalid("invalid daily timestamp"))?;
        let date = timestamp.with_timezone(&Warsaw).date_naive();
        if date > today {
            return Err(invalid("daily timestamp is later than fetch time"));
        }
        if !dates.insert(date) {
            return Err(HistoryError::DuplicateDate(date));
        }
        let mut skip = |reason| skipped_candles.push(SkippedCandle { date, reason });
        // Older sessions are closed. Today's candle requires the source's session
        // end plus a 15-minute publication margin; a live bar is never called EOD.
        if date == today && !session_finished(meta, raw.fetched_at) {
            skip(SkippedCandleReason::UnfinishedSession);
            continue;
        }
        if timestamp > raw.fetched_at {
            return Err(invalid("daily timestamp is later than fetch time"));
        }
        if columns.iter().any(|column| column[i].is_none()) {
            skip(SkippedCandleReason::MissingValues);
            continue;
        }
        let values = columns.map(|column| {
            // arbitrary_precision retains source digits; never round via f64.
            Decimal::from_str_exact(&column[i].as_ref().expect("checked above").to_string()).ok()
        });
        let [
            Some(open),
            Some(high),
            Some(low),
            Some(close),
            Some(volume),
            Some(adjusted_close),
        ] = values
        else {
            skip(SkippedCandleReason::InvalidValues);
            continue;
        };
        if [open, high, low, close, adjusted_close]
            .iter()
            .any(|v| *v <= Decimal::ZERO)
            || volume < Decimal::ZERO
        {
            skip(SkippedCandleReason::InvalidValues);
            continue;
        }
        if high < low || open < low || open > high || close < low || close > high {
            skip(SkippedCandleReason::InvalidRange);
            continue;
        }
        candles.push(DailyCandle {
            date,
            open,
            high,
            low,
            close,
            adjusted_close: Some(adjusted_close),
            volume,
        });
    }
    if candles.is_empty() {
        return Err(invalid(format!(
            "no complete, finished daily candles ({} omitted rows)",
            skipped_candles.len()
        )));
    }
    candles.sort_unstable_by_key(|candle| candle.date);
    skipped_candles.sort_unstable_by_key(|candle| candle.date);
    Ok(DailyPriceHistory {
        instrument: instrument.clone(),
        symbol: symbol.into(),
        currency: meta.currency.clone(),
        source: raw.source,
        source_url: raw.url.clone(),
        fetched_at: raw.fetched_at,
        candles,
        skipped_candles,
    })
}

fn session_finished(meta: &Metadata, fetched_at: DateTime<Utc>) -> bool {
    let Some(periods) = &meta.current_trading_period else {
        return false;
    };
    let (Some(start), Some(end)) = (
        DateTime::from_timestamp(periods.regular.start, 0),
        DateTime::from_timestamp(periods.regular.end, 0),
    ) else {
        return false;
    };
    let today = fetched_at.with_timezone(&Warsaw).date_naive();
    start < end
        && start.with_timezone(&Warsaw).date_naive() == today
        && end.with_timezone(&Warsaw).date_naive() == today
        && end
            .checked_add_signed(Duration::minutes(15))
            .is_some_and(|ready| fetched_at >= ready)
}
