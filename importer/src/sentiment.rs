//! Pure scoring of changes in disclosed KNF net short positions.

use rust_decimal::Decimal;

use crate::model::ShortsSnapshot;

/// Compare an older and a newer observation of the same ISIN.
/// Any positive gap between fetched_at timestamps is sufficient.
/// Missing pairs and two empty lists return None (JSON null).
pub fn score(older: Option<&ShortsSnapshot>, newer: Option<&ShortsSnapshot>) -> Option<i8> {
    let older = older?;
    let newer = newer?;
    if older.isin.trim().is_empty() || older.isin != newer.isin {
        return None;
    }
    if older.fetched_at >= newer.fetched_at {
        return None;
    }

    if older.positions.is_empty() && newer.positions.is_empty() {
        return None;
    }

    let older_total = total(older);
    let newer_total = total(newer);
    if newer_total > older_total {
        Some(-1)
    } else if newer_total < older_total {
        Some(1)
    } else {
        Some(0)
    }
}

fn total(snapshot: &ShortsSnapshot) -> Decimal {
    snapshot
        .positions
        .iter()
        .map(|position| position.net_short_position_percent)
        .sum()
}
