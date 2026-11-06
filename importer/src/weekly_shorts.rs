//! Presentation-only weekly observations; sentiment scoring still uses its original pair.

use std::collections::BTreeMap;

use chrono::{Datelike, Duration, NaiveDate};
use rust_decimal::Decimal;

use crate::charts::ShortTotal;

pub(crate) struct WeeklyShort {
    /// Monday of the ISO week, based on the fetch timestamp in UTC.
    pub week: NaiveDate,
    pub total: Decimal,
    pub change: Option<Decimal>,
}

pub(crate) fn group(totals: &[ShortTotal]) -> Vec<WeeklyShort> {
    let mut selected = BTreeMap::<NaiveDate, &ShortTotal>::new();
    for observation in totals {
        let date = observation.fetched_at.date_naive();
        let week = date
            .checked_sub_signed(Duration::days(i64::from(
                date.weekday().num_days_from_monday(),
            )))
            .unwrap_or(date);
        let entry = selected.entry(week).or_insert(observation);
        if observation.fetched_at >= entry.fetched_at {
            *entry = observation;
        }
    }
    let mut previous: Option<(NaiveDate, Decimal)> = None;
    selected
        .into_iter()
        .map(|(week, observation)| {
            // Without an observation in the preceding ISO week, the weekly change is unknown.
            let change = previous
                .filter(|(date, _)| (week - *date).num_days() == 7)
                .and_then(|(_, total)| observation.total.checked_sub(total));
            previous = Some((week, observation.total));
            WeeklyShort {
                week,
                total: observation.total,
                change,
            }
        })
        .collect()
}
