//! Fixed fundamental thresholds. GICS selects a template, never a peer group.

use rust_decimal::Decimal;
use serde::Serialize;

use crate::fundamental_ratios::FundamentalRatios;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct FundamentalPoints {
    #[serde(flatten)]
    pub ratios: FundamentalRatios,
    pub net_margin_points: Option<i8>,
    pub operating_margin_points: Option<i8>,
    pub equity_ratio_points: Option<i8>,
    pub debt_to_equity_points: Option<i8>,
    pub ocf_to_net_income_points: Option<i8>,
    pub fundamental_score: Option<i8>,
}

/// Score one company's ratios without access to other companies or external data.
pub fn score(mut ratios: FundamentalRatios, gics_sector_code: u16) -> FundamentalPoints {
    let equity_ratio_points;
    if gics_sector_code == 40 {
        // The entire Financials sector uses only equity, including insurers and KRUK.
        ratios.net_margin = None;
        ratios.operating_margin = None;
        ratios.debt_to_equity = None;
        ratios.ocf_to_net_income = None;
        equity_ratio_points = points(ratios.equity_ratio, Decimal::new(8, 2), Decimal::new(10, 2));
    } else {
        equity_ratio_points = points(
            ratios.equity_ratio,
            Decimal::new(30, 2),
            Decimal::new(50, 2),
        );
    }

    let net_margin_points = points(ratios.net_margin, Decimal::new(-2, 2), Decimal::new(2, 2));
    let operating_margin_points = points(
        ratios.operating_margin,
        Decimal::new(-2, 2),
        Decimal::new(2, 2),
    );
    // Lower debt is positive, so reverse the direction of the common threshold rule.
    let debt_to_equity_points = points(
        ratios.debt_to_equity,
        Decimal::new(50, 2),
        Decimal::new(150, 2),
    )
    .map(|value| -value);
    let ocf_to_net_income_points = points(ratios.ocf_to_net_income, Decimal::ZERO, Decimal::ONE);

    let mut fundamental_score = None;
    for value in [
        net_margin_points,
        operating_margin_points,
        equity_ratio_points,
        debt_to_equity_points,
        ocf_to_net_income_points,
    ]
    .into_iter()
    .flatten()
    {
        fundamental_score = Some(fundamental_score.unwrap_or(0) + value);
    }

    FundamentalPoints {
        ratios,
        net_margin_points,
        operating_margin_points,
        equity_ratio_points,
        debt_to_equity_points,
        ocf_to_net_income_points,
        fundamental_score,
    }
}

/// Both boundaries belong to the neutral band. Missing data stays missing.
fn points(value: Option<Decimal>, low: Decimal, high: Decimal) -> Option<i8> {
    let value = value?;
    if value < low {
        Some(-1)
    } else if value > high {
        Some(1)
    } else {
        Some(0)
    }
}
