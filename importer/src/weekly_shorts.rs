//! Presentation-only weekly observations; sentiment scoring still uses its original pair.

use std::collections::BTreeMap;

use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::charts::ShortTotal;

pub(crate) struct WeeklyShort {
    /// Monday of the ISO week, based on the fetch timestamp in UTC.
    pub week: NaiveDate,
    pub total: Decimal,
    pub fetched_at: DateTime<Utc>,
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
    selected
        .into_iter()
        .map(|(week, observation)| WeeklyShort {
            week,
            total: observation.total,
            fetched_at: observation.fetched_at,
        })
        .collect()
}

pub(crate) struct BoardRow {
    pub code: String,
    pub weekly: BTreeMap<NaiveDate, WeeklyShort>,
    pub newer: Option<ShortTotal>,
    pub older: Option<ShortTotal>,
    pub sentiment_score: Option<i8>,
}

pub(crate) struct KnfBoard {
    pub weeks: Vec<NaiveDate>,
    pub rows: Vec<BoardRow>,
}

pub(crate) fn board(index: &crate::company_archive::ArchiveIndex, codes: &[String]) -> KnfBoard {
    let mut weeks = std::collections::BTreeSet::new();
    let rows = codes
        .iter()
        .map(|code| {
            let snapshots = index
                .company(code)
                .map(|files| crate::ranking::company_shorts(&files.shorts, code))
                .unwrap_or_default();
            let observations = snapshots
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
                .collect::<Vec<_>>();
            let (newer, older) = crate::ranking::shorts_pair(&snapshots);
            let sentiment_score = crate::sentiment::score(older.as_ref(), newer.as_ref());
            let total = |snapshot: crate::model::ShortsSnapshot| ShortTotal {
                fetched_at: snapshot.fetched_at,
                total: snapshot
                    .positions
                    .iter()
                    .map(|position| position.net_short_position_percent)
                    .sum(),
            };
            let weekly = group(&observations)
                .into_iter()
                .map(|observation| {
                    weeks.insert(observation.week);
                    (observation.week, observation)
                })
                .collect();
            BoardRow {
                code: code.clone(),
                weekly,
                newer: newer.map(total),
                older: older.map(total),
                sentiment_score,
            }
        })
        .collect();
    KnfBoard {
        weeks: weeks.into_iter().collect(),
        rows,
    }
}
