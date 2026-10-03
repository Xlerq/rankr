//! Raw financial ratios from one snapshot, calculated with monetary unit scaling.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Only the monetary fields needed for the five ratios are read from a report.
#[derive(Debug, Deserialize)]
pub struct FundamentalRatioInput {
    pub unit_multiplier: u32,
    pub revenue: Option<Decimal>,
    pub net_income_parent: Option<Decimal>,
    pub operating_profit: Option<Decimal>,
    pub equity_parent: Option<Decimal>,
    pub assets: Option<Decimal>,
    pub long_term_liabilities: Option<Decimal>,
    pub short_term_liabilities: Option<Decimal>,
    pub operating_cash_flow: Option<Decimal>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FundamentalRatios {
    pub net_margin: Option<Decimal>,
    pub operating_margin: Option<Decimal>,
    pub equity_ratio: Option<Decimal>,
    pub debt_to_equity: Option<Decimal>,
    pub ocf_to_net_income: Option<Decimal>,
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum RatioError {
    #[error("fundamental ratio calculation exceeds Decimal precision/range")]
    Arithmetic,
}

/// Calculate dimensionless ratios without annualization or currency conversion.
pub fn calculate(input: &FundamentalRatioInput) -> Result<FundamentalRatios, RatioError> {
    let multiplier = Decimal::from(input.unit_multiplier);
    let revenue = scale(input.revenue, multiplier)?;
    let net_income = scale(input.net_income_parent, multiplier)?;
    let operating_profit = scale(input.operating_profit, multiplier)?;
    let equity = scale(input.equity_parent, multiplier)?;
    let assets = scale(input.assets, multiplier)?;
    let long_term = scale(input.long_term_liabilities, multiplier)?;
    let short_term = scale(input.short_term_liabilities, multiplier)?;
    let operating_cash_flow = scale(input.operating_cash_flow, multiplier)?;
    let liabilities = match (long_term, short_term) {
        (Some(long_term), Some(short_term)) => Some(
            long_term
                .checked_add(short_term)
                .ok_or(RatioError::Arithmetic)?,
        ),
        _ => None,
    };

    Ok(FundamentalRatios {
        net_margin: divide(net_income, revenue)?,
        operating_margin: divide(operating_profit, revenue)?,
        equity_ratio: divide(equity, assets)?,
        debt_to_equity: divide(liabilities, equity)?,
        ocf_to_net_income: divide(operating_cash_flow, net_income)?,
    })
}

fn scale(value: Option<Decimal>, multiplier: Decimal) -> Result<Option<Decimal>, RatioError> {
    value
        .map(|amount| amount.checked_mul(multiplier).ok_or(RatioError::Arithmetic))
        .transpose()
}

fn divide(
    numerator: Option<Decimal>,
    denominator: Option<Decimal>,
) -> Result<Option<Decimal>, RatioError> {
    let (Some(numerator), Some(denominator)) = (numerator, denominator) else {
        return Ok(None);
    };
    if denominator.is_zero() {
        return Ok(None);
    }
    numerator
        .checked_div(denominator)
        .map(Some)
        .ok_or(RatioError::Arithmetic)
}
