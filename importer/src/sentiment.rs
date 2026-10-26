//! Pure scoring of changes in disclosed KNF net short positions.

use std::collections::BTreeSet;

use rust_decimal::Decimal;

use crate::model::ShortsSnapshot;

/// Compare an older and a newer observation of the same ISIN.
/// Any positive gap between fetched_at timestamps is sufficient.
/// Missing or incomparable observations return None (JSON null), never zero.
pub fn score(older: Option<&ShortsSnapshot>, newer: Option<&ShortsSnapshot>) -> Option<i8> {
    let older = older?;
    let newer = newer?;
    if older.isin.trim().is_empty() || older.isin != newer.isin {
        return None;
    }
    if older.fetched_at >= newer.fetched_at {
        return None;
    }

    let (older_holders, older_total) = holders_and_total(older)?;
    let (newer_holders, newer_total) = holders_and_total(newer)?;
    if older_holders != newer_holders {
        return None;
    }

    if newer_total > older_total {
        Some(-1)
    } else if newer_total < older_total {
        Some(1)
    } else {
        Some(0)
    }
}

fn holders_and_total(snapshot: &ShortsSnapshot) -> Option<(BTreeSet<String>, Decimal)> {
    if snapshot.positions.is_empty() {
        return None;
    }
    let mut holders = BTreeSet::new();
    let mut total = Decimal::ZERO;
    let minimum = Decimal::new(5, 1);
    for position in &snapshot.positions {
        // Match the importer's whitespace and case handling of holder identities.
        let holder = position
            .holder
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_uppercase();
        if holder.is_empty()
            || position.net_short_position_percent < minimum
            || !holders.insert(holder)
        {
            // A stored snapshot must contain exactly one position per holder.
            return None;
        }
        total = total.checked_add(position.net_short_position_percent)?;
    }
    Some((holders, total))
}
