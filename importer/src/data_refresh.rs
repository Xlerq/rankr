//! Refresh each mapped company using UTC dates from the archive directory names.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::Result;
use chrono::{NaiveDate, Utc};

use crate::{
    data_collect,
    history::YahooSymbolMap,
    shorts_collect,
    source::{GpwClient, KnfClient, YahooClient},
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum DataKind {
    Fundamentals,
    History,
    Shorts,
}

impl DataKind {
    fn filename(self) -> &'static str {
        match self {
            Self::Fundamentals => "fundamentals.json",
            Self::History => "history.json",
            Self::Shorts => "shorts.json",
        }
    }
}

const KINDS: [DataKind; 3] = [DataKind::Fundamentals, DataKind::History, DataKind::Shorts];

fn needs_refresh(kind: DataKind, latest: Option<NaiveDate>, today: NaiveDate) -> bool {
    let Some(latest) = latest else {
        return true;
    };
    match kind {
        DataKind::Fundamentals | DataKind::Shorts => latest != today,
        DataKind::History => !(0..=7).contains(&(today - latest).num_days()),
    }
}

pub async fn refresh(directory: &Path) -> Result<()> {
    let symbols = YahooSymbolMap::bundled()?;
    let gpw = GpwClient::new()?;
    let yahoo = YahooClient::new()?;
    let knf = KnfClient::new()?;
    let today = Utc::now().date_naive();
    let dates = latest_dates(directory, today);
    let mut collected = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for instrument in symbols.instruments() {
        for kind in KINDS {
            let latest = dates.get(&(instrument.code.clone(), kind)).copied();
            if !needs_refresh(kind, latest, today) {
                skipped += 1;
                eprintln!(
                    "{}: {} is current; skipped",
                    instrument.code,
                    kind.filename()
                );
                continue;
            }

            eprintln!("{}: downloading {}", instrument.code, kind.filename());
            let result = match kind {
                DataKind::Fundamentals => {
                    data_collect::collect_fundamentals(&gpw, &instrument, directory).await
                }
                DataKind::History => {
                    // Keep the existing Yahoo collection pace.
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    data_collect::collect_history(&yahoo, &symbols, &instrument, directory, false)
                        .await
                }
                DataKind::Shorts => shorts_collect::collect_one(&knf, &instrument, directory).await,
            };
            match result {
                Ok(()) => collected += 1,
                Err(error) => {
                    failed += 1;
                    eprintln!("{}: {}: {error:#}", instrument.code, kind.filename());
                }
            }
        }
    }
    eprintln!("Refresh finished: {collected} collected, {skipped} current, {failed} failed");
    Ok(())
}

fn latest_dates(directory: &Path, today: NaiveDate) -> BTreeMap<(String, DataKind), NaiveDate> {
    let mut dates = BTreeMap::new();
    let mut directories = vec![directory.to_path_buf()];
    while let Some(directory) = directories.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    eprintln!("{}: {error}", directory.display());
                }
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    eprintln!("{}: {error}", directory.display());
                    continue;
                }
            };
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) => {
                    eprintln!("{}: {error}", path.display());
                    continue;
                }
            };
            if file_type.is_dir() {
                directories.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let Some(kind) = KINDS
                .into_iter()
                .find(|kind| entry.file_name() == kind.filename())
            else {
                continue;
            };
            let Some((date, code)) = observation_date_and_code(&directory) else {
                continue;
            };
            if date <= today {
                let latest = dates.entry((code.to_owned(), kind)).or_insert(date);
                *latest = (*latest).max(date);
            }
        }
    }
    dates
}

fn observation_date_and_code(directory: &Path) -> Option<(NaiveDate, &str)> {
    // archive_raw names observations YYYY-MM-DD_CODE_kind, optionally ending in _2, _3, ...
    // Only a completed JSON file counts; raw.json and error.json do not mark data as current.
    let name = directory.file_name()?.to_str()?;
    let (date, rest) = name.split_once('_')?;
    let (code, _) = rest.split_once('_')?;
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    Some((date, code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Days;

    #[test]
    fn refresh_decision_uses_daily_and_weekly_windows() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap();
        for kind in [DataKind::Fundamentals, DataKind::Shorts] {
            assert!(!needs_refresh(kind, Some(today), today));
            assert!(needs_refresh(kind, Some(today - Days::new(1)), today));
        }
        assert!(!needs_refresh(
            DataKind::History,
            Some(today - Days::new(7)),
            today
        ));
        assert!(needs_refresh(
            DataKind::History,
            Some(today - Days::new(8)),
            today
        ));
        for kind in KINDS {
            assert!(needs_refresh(kind, None, today));
        }
    }
}
