use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Instrument {
    pub isin: String,
    pub code: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    GpwNotoria,
    GpwBenchmark,
    GpwPrices,
    TradingViewIdc,
}

/// An original response, including responses that could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawDocument {
    pub source: Source,
    pub instrument: Option<Instrument>,
    pub fetched_at: DateTime<Utc>,
    pub url: String,
    pub http_status: u16,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FundamentalSnapshot {
    pub instrument: Instrument,
    pub source: Source,
    pub source_url: String,
    pub fetched_at: DateTime<Utc>,
    pub fundamentals: Fundamentals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketPair {
    UsdPln,
    EurPln,
    XauUsd,
}

impl MarketPair {
    pub const ALL: [Self; 3] = [Self::UsdPln, Self::EurPln, Self::XauUsd];

    pub fn from_code(code: &str) -> Option<Self> {
        match code.replace('/', "").to_ascii_uppercase().as_str() {
            "USDPLN" => Some(Self::UsdPln),
            "EURPLN" => Some(Self::EurPln),
            "XAUUSD" => Some(Self::XauUsd),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Self::UsdPln => "USDPLN",
            Self::EurPln => "EURPLN",
            Self::XauUsd => "XAUUSD",
        }
    }

    pub fn instrument(self) -> PriceInstrument {
        PriceInstrument {
            code: self.code().into(),
            isin: None,
            currency: if self == Self::XauUsd { "USD" } else { "PLN" }.into(),
            unit: if self == Self::XauUsd {
                PriceUnit::TroyOunce
            } else {
                PriceUnit::BaseCurrency
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriceUnit {
    Share,
    BaseCurrency,
    TroyOunce,
}

/// ISIN identifies shares; currency/metal pairs do not have an ISIN.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceInstrument {
    pub code: String,
    pub isin: Option<String>,
    pub currency: String,
    /// One share, one unit of the pair's base currency, or one troy ounce.
    pub unit: PriceUnit,
}

/// Latest available quote, not a confirmed end-of-day candle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceSnapshot {
    pub instrument: PriceInstrument,
    pub source: Source,
    pub source_url: String,
    pub fetched_at: DateTime<Utc>,
    /// GPW's displayed last-update time (minute precision). Unknown for IDC.
    pub source_updated_at: Option<DateTime<Utc>>,
    /// Start of IDC's daily bar; this is not the time of its last trade.
    pub period_started_at: Option<DateTime<Utc>>,
    pub price: Decimal,
    pub open: Option<Decimal>,
    pub high: Option<Decimal>,
    pub low: Option<Decimal>,
    /// Cumulative session volume in shares; unavailable for the FX/metal feed.
    pub volume: Option<u64>,
}

/// Financial amounts retain the source's sign, currency and unit scale.
/// Reported ratios are dimensionless and are not multiplied by `unit_multiplier`.
/// `None` represents unavailable data; it must never be treated as zero.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fundamentals {
    pub report_period: String,
    pub currency: String,
    pub unit_multiplier: u32,
    pub consolidated: bool,

    pub revenue: Option<Decimal>,
    pub sales_profit: Option<Decimal>,
    pub other_operating_income: Option<Decimal>,
    pub operating_profit: Option<Decimal>,
    pub financial_income: Option<Decimal>,
    pub pretax_profit: Option<Decimal>,
    pub net_income_parent: Option<Decimal>,
    pub depreciation_amortization: Option<Decimal>,
    pub assets: Option<Decimal>,
    pub non_current_assets: Option<Decimal>,
    pub current_assets: Option<Decimal>,
    pub equity_parent: Option<Decimal>,
    pub share_capital: Option<Decimal>,
    pub long_term_liabilities: Option<Decimal>,
    pub short_term_liabilities: Option<Decimal>,
    pub operating_cash_flow: Option<Decimal>,
    pub investing_cash_flow: Option<Decimal>,
    pub capex: Option<Decimal>,
    pub financing_cash_flow: Option<Decimal>,
    pub net_cash_flow: Option<Decimal>,
    pub ebitda: Option<Decimal>,

    pub reported_roe: Option<Decimal>,
    pub reported_roa: Option<Decimal>,
    pub reported_current_ratio: Option<Decimal>,
    pub reported_quick_ratio: Option<Decimal>,
    pub reported_debt_service_ratio: Option<Decimal>,

    /// Unrecognized rows retain their original labels and displayed values.
    pub extra_fields: BTreeMap<String, String>,
}
