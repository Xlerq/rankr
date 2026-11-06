//! Load and score just one company's indexed archives, using the existing formulas.

use std::path::PathBuf;

use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    charts::{OhlcCandle, ShortTotal},
    company_archive::{ArchiveIndex, CompanyCode, SavedScore, read_json},
    fundamental_points::{self, FundamentalPoints},
    fundamental_points_table,
    fundamental_ratios::{self, FundamentalRatioInput},
    model::{FundamentalSnapshot, ShortsSnapshot},
    ranking::{self, RankingRow},
    sentiment,
    technical::{self, CloseCandle, TechnicalScore},
};

pub(crate) struct CompanyCard {
    pub row: RankingRow,
    pub fundamental: Option<(FundamentalPoints, FundamentalSnapshot)>,
    pub technical: Option<TechnicalScore>,
    pub newer_shorts: Option<ShortsSnapshot>,
    pub older_shorts: Option<ShortsSnapshot>,
    pub short_totals: Vec<ShortTotal>,
    pub candles: Vec<OhlcCandle>,
    pub currency: String,
    pub saved_scores: Vec<SavedScore>,
}

#[derive(Deserialize)]
struct History {
    instrument: CompanyCode,
    fetched_at: DateTime<Utc>,
    #[serde(default)]
    currency: String,
    candles: Vec<HistoryCandle>,
}

#[derive(Deserialize)]
struct HistoryCandle {
    #[serde(flatten)]
    close: CloseCandle,
    open: Option<Decimal>,
    high: Option<Decimal>,
    low: Option<Decimal>,
}

pub(crate) fn load(
    index: &ArchiveIndex,
    code: &str,
    as_of: NaiveDate,
) -> Result<Option<CompanyCard>> {
    let classifications = fundamental_points_table::gics_map()?;
    let Some(gics) = classifications.get(code) else {
        return Ok(None);
    };
    let Some(files) = index.company(code) else {
        return Ok(None);
    };
    let fundamental = latest::<FundamentalSnapshot>(
        &files.fundamentals,
        |snapshot| snapshot.instrument.code == code,
        |snapshot| snapshot.fetched_at,
    )
    .and_then(|snapshot| {
        let f = &snapshot.fundamentals;
        let input = FundamentalRatioInput {
            unit_multiplier: f.unit_multiplier,
            revenue: f.revenue,
            net_income_parent: f.net_income_parent,
            operating_profit: f.operating_profit,
            equity_parent: f.equity_parent,
            assets: f.assets,
            long_term_liabilities: f.long_term_liabilities,
            short_term_liabilities: f.short_term_liabilities,
            operating_cash_flow: f.operating_cash_flow,
        };
        let ratios = match fundamental_ratios::calculate(&input) {
            Ok(ratios) => ratios,
            Err(error) => {
                eprintln!("{code}: {error}");
                return None;
            }
        };
        Some((
            fundamental_points::score(ratios, gics.gics_sector_code),
            snapshot,
        ))
    });
    let history = latest::<History>(
        &files.histories,
        |history| history.instrument.code == code,
        |history| history.fetched_at,
    );
    let technical = history.as_ref().and_then(|history| {
        let closes: Vec<_> = history
            .candles
            .iter()
            .map(|candle| candle.close.clone())
            .collect();
        match technical::score(&closes, as_of) {
            Ok(score) => Some(score),
            Err(error) => {
                eprintln!("{code}: {error}");
                None
            }
        }
    });
    // Missing OHLC fields are never fabricated from close-only legacy data.
    let mut candles: Vec<_> = history
        .as_ref()
        .into_iter()
        .flat_map(|history| &history.candles)
        .filter_map(|candle| {
            let candle = OhlcCandle {
                date: candle.close.date,
                open: candle.open?,
                high: candle.high?,
                low: candle.low?,
                close: candle.close.close,
            };
            (candle.date <= as_of
                && candle.low > Decimal::ZERO
                && candle.low <= candle.open.min(candle.close)
                && candle.high >= candle.open.max(candle.close))
            .then_some(candle)
        })
        .collect();
    candles.sort_by_key(|candle| candle.date);
    let shorts = ranking::company_shorts(&files.shorts, code);
    let (newer_shorts, older_shorts) = ranking::shorts_pair(&shorts);
    let short_totals = shorts
        .iter()
        .rev()
        .map(|snapshot| ShortTotal {
            fetched_at: snapshot.fetched_at,
            total: snapshot
                .positions
                .iter()
                .map(|position| position.net_short_position_percent)
                .sum(),
        })
        .collect();
    let fundamental_score = fundamental
        .as_ref()
        .and_then(|(points, _)| points.fundamental_score);
    let technical_score = technical.as_ref().and_then(|score| score.technical_score);
    let sentiment_score = sentiment::score(older_shorts.as_ref(), newer_shorts.as_ref());
    let row = RankingRow {
        code: code.to_owned(),
        gics_sector: gics.gics_sector.clone(),
        fundamental_score,
        technical_score,
        sentiment_score,
        final_score: ranking::combine_scores(fundamental_score, technical_score, sentiment_score),
    };
    Ok(Some(CompanyCard {
        row,
        fundamental,
        technical,
        newer_shorts,
        older_shorts,
        short_totals,
        candles,
        currency: history.map(|history| history.currency).unwrap_or_default(),
        saved_scores: files
            .scores
            .iter()
            .filter(|score| score.date <= as_of)
            .cloned()
            .collect(),
    }))
}

fn latest<T: serde::de::DeserializeOwned>(
    paths: &[PathBuf],
    matches: impl Fn(&T) -> bool,
    timestamp: impl Fn(&T) -> DateTime<Utc>,
) -> Option<T> {
    let mut selected = None;
    for path in paths {
        let candidate: T = match read_json(path) {
            Ok(candidate) => candidate,
            Err(error) => {
                eprintln!("{}: {error:#}", path.display());
                continue;
            }
        };
        if matches(&candidate)
            && selected
                .as_ref()
                .is_none_or(|current| timestamp(&candidate) > timestamp(current))
        {
            selected = Some(candidate);
        }
    }
    selected
}
