//! Pure scoring of changes in disclosed KNF net short positions.

use chrono::{Datelike, Duration, NaiveDate};
use rust_decimal::Decimal;

use crate::model::ShortsSnapshot;

/// Compare an older and a newer observation of the same ISIN.
/// Any positive gap between fetched_at timestamps is sufficient.
pub fn score(older: Option<&ShortsSnapshot>, newer: Option<&ShortsSnapshot>) -> Option<i8> {
    let older = older?;
    let newer = newer?;
    if older.isin.trim().is_empty() || older.isin != newer.isin {
        return None;
    }
    if older.fetched_at >= newer.fetched_at {
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

/// Monday of the ISO week in UTC, shared by scoring and the weekly board.
pub(crate) fn week_start(date: NaiveDate) -> NaiveDate {
    date.checked_sub_signed(Duration::days(i64::from(
        date.weekday().num_days_from_monday(),
    )))
    .unwrap_or(date)
}

/// Sources are ordered newest first. Require the current and immediately preceding
/// ISO week; neither gaps nor a stale last observation are treated as zero.
pub(crate) fn weekly_pair<'a>(
    shorts: impl Iterator<Item = &'a ShortsSnapshot>,
    as_of: NaiveDate,
) -> (Option<&'a ShortsSnapshot>, Option<&'a ShortsSnapshot>) {
    let current = week_start(as_of);
    let previous = current.checked_sub_signed(Duration::days(7));
    let mut newer: Option<&ShortsSnapshot> = None;
    let mut older = None;
    for snapshot in shorts {
        if snapshot.fetched_at.date_naive() > as_of {
            continue;
        }
        let week = week_start(snapshot.fetched_at.date_naive());
        if week == current && newer.is_none() {
            newer = Some(snapshot);
        }
        if Some(week) == previous
            && older.is_none()
            && newer.is_some_and(|newer| newer.isin == snapshot.isin)
        {
            older = Some(snapshot);
        }
    }
    (newer, older)
}
