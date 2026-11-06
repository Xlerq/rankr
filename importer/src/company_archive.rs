//! Index archive paths once, so a company request opens only its own source files.

use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;

use crate::archive::find_files;

#[derive(Clone, Debug)]
pub(crate) struct SavedScore {
    pub date: NaiveDate,
    pub score: Option<i8>,
}

#[derive(Default)]
pub(crate) struct CompanyFiles {
    pub fundamentals: Vec<PathBuf>,
    pub histories: Vec<PathBuf>,
    pub shorts: Vec<PathBuf>,
    pub scores: Vec<SavedScore>,
}

#[derive(Default)]
pub(crate) struct ArchiveIndex {
    companies: BTreeMap<String, CompanyFiles>,
}

#[derive(Deserialize)]
pub(crate) struct CompanyCode {
    pub code: String,
}

#[derive(Deserialize)]
struct Identity {
    code: Option<String>,
    instrument: Option<CompanyCode>,
}

impl ArchiveIndex {
    pub fn read(directory: &Path) -> io::Result<Self> {
        let mut index = Self::default();
        for name in ["fundamentals.json", "history.json", "shorts.json"] {
            let mut paths = find_files(directory, name, &mut Vec::new())?;
            paths.sort();
            for path in paths {
                // Modern archives encode the company in the directory name.
                // Older, timestamp-only archives are identified once at startup.
                let code = named_code(&path).or_else(|| {
                    let identity: Identity = read_json(&path).ok()?;
                    identity
                        .code
                        .or_else(|| identity.instrument.map(|instrument| instrument.code))
                });
                let Some(code) = code else { continue };
                let company = index.companies.entry(code).or_default();
                match name {
                    "fundamentals.json" => company.fundamentals.push(path),
                    "history.json" => company.histories.push(path),
                    _ => company.shorts.push(path),
                }
            }
        }
        index.read_scores(directory)?;
        Ok(index)
    }

    pub fn company(&self, code: &str) -> Option<&CompanyFiles> {
        self.companies.get(code)
    }

    fn read_scores(&mut self, directory: &Path) -> io::Result<()> {
        #[derive(Deserialize)]
        struct Row {
            code: String,
            final_score: Option<i8>,
        }
        #[derive(Deserialize)]
        struct Snapshot {
            computed_at: DateTime<Utc>,
            as_of: NaiveDate,
            rows: Vec<Row>,
        }
        let mut paths = find_files(directory, "ranking.json", &mut Vec::new())?;
        paths.sort();
        let mut daily = BTreeMap::<(String, NaiveDate), (DateTime<Utc>, Option<i8>)>::new();
        for path in paths {
            let snapshot: Snapshot = match read_json(&path) {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    eprintln!("{}: {error}", path.display());
                    continue;
                }
            };
            for row in snapshot.rows {
                let entry = daily
                    .entry((row.code, snapshot.as_of))
                    .or_insert((snapshot.computed_at, row.final_score));
                // Multiple runs on one UTC day: display that day's latest saved observation.
                // A later null replaces an earlier score; it must not become a zero bar.
                if snapshot.computed_at > entry.0 {
                    *entry = (snapshot.computed_at, row.final_score);
                }
            }
        }
        for ((code, date), (_, score)) in daily {
            self.companies
                .entry(code)
                .or_default()
                .scores
                .push(SavedScore { date, score });
        }
        Ok(())
    }
}

fn named_code(path: &Path) -> Option<String> {
    let name = path.parent()?.file_name()?.to_str()?;
    let (date, rest) = name.split_once('_')?;
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    let (code, kind) = rest.split_once('_')?;
    if code.is_empty()
        || !["fundamentals", "history", "shorts"]
            .iter()
            .any(|prefix| kind.starts_with(prefix))
    {
        return None;
    }
    Some(code.to_owned())
}

pub(crate) fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}
