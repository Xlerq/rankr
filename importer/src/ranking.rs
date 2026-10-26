//! Offline combination of the existing three family scores.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    fundamental_points_table, fundamental_table::SkippedFile, model::ShortsSnapshot, sentiment,
    technical_table,
};

#[derive(Debug, Serialize)]
pub struct RankingRow {
    pub code: String,
    pub gics_sector: String,
    pub fundamental_score: Option<i8>,
    pub technical_score: Option<i8>,
    pub sentiment_score: Option<i8>,
    pub final_score: Option<i8>,
}

#[derive(Debug)]
pub struct Ranking {
    pub rows: Vec<RankingRow>,
    pub skipped: Vec<SkippedFile>,
}

struct ArchivedShorts {
    snapshot: ShortsSnapshot,
    source_file: PathBuf,
}

/// Join the family results by code. Latest snapshots are independent of as_of;
/// only the existing technical calculation limits candles to dates before as_of.
pub fn rank_directory(directory: &Path, as_of: NaiveDate) -> Result<Ranking> {
    let fundamentals = fundamental_points_table::build_table(directory)
        .context("building the fundamental points table")?;
    let technical = technical_table::rank_directory(directory, as_of)
        .context("building the technical table")?;
    let classifications = fundamental_points_table::gics_map()?;

    let mut skipped = fundamentals.skipped;
    for file in technical.skipped {
        skipped.push(SkippedFile {
            source_file: file.source_file,
            reason: file.reason,
        });
    }
    let shorts = read_shorts(directory, &mut skipped)?;
    let mut latest_shorts = BTreeMap::new();
    // read_shorts orders newest first and resolves timestamp ties by lexical path.
    for archived in &shorts {
        latest_shorts
            .entry(archived.snapshot.code.clone())
            .or_insert(&archived.snapshot);
    }

    let fundamentals: BTreeMap<_, _> = fundamentals
        .rows
        .into_iter()
        .map(|row| (row.code, row.points.fundamental_score))
        .collect();
    let technical: BTreeMap<_, _> = technical
        .rows
        .into_iter()
        .map(|row| (row.code, row.technical_score))
        .collect();
    let codes: BTreeSet<_> = fundamentals
        .keys()
        .chain(technical.keys())
        .chain(latest_shorts.keys())
        .collect();

    let mut rows = Vec::new();
    for code in codes {
        let Some(gics) = classifications.get(code) else {
            skipped.push(SkippedFile {
                source_file: directory.to_path_buf(),
                reason: format!("missing GICS mapping for {code}"),
            });
            continue;
        };
        let fundamental_score = fundamentals.get(code).copied().flatten();
        let technical_score = technical.get(code).copied().flatten();
        let sentiment_score = match latest_shorts.get(code) {
            Some(newer) => {
                let older = shorts.iter().find(|archived| {
                    let snapshot = &archived.snapshot;
                    snapshot.isin == newer.isin && snapshot.fetched_at < newer.fetched_at
                });
                sentiment::score(older.map(|archived| &archived.snapshot), Some(newer))
            }
            None => None,
        };
        let final_score = match (fundamental_score, technical_score, sentiment_score) {
            (Some(fundamental), Some(technical), Some(sentiment)) => {
                Some(fundamental + technical + sentiment)
            }
            _ => None,
        };
        rows.push(RankingRow {
            code: code.clone(),
            gics_sector: gics.gics_sector.clone(),
            fundamental_score,
            technical_score,
            sentiment_score,
            final_score,
        });
    }
    rows.sort_by(|left, right| {
        right
            .final_score
            .cmp(&left.final_score)
            .then_with(|| left.code.cmp(&right.code))
    });
    Ok(Ranking { rows, skipped })
}

fn read_shorts(
    directory: &Path,
    skipped: &mut Vec<SkippedFile>,
) -> io::Result<Vec<ArchivedShorts>> {
    let mut files = Vec::new();
    find_shorts_files(directory, &mut files, skipped)?;
    let mut snapshots = Vec::new();
    for source_file in files {
        match read_snapshot(&source_file) {
            Ok(snapshot) => snapshots.push(ArchivedShorts {
                snapshot,
                source_file,
            }),
            Err(error) => skipped.push(SkippedFile {
                source_file,
                reason: format!("{error:#}"),
            }),
        }
    }
    snapshots.sort_by(|left, right| {
        right
            .snapshot
            .fetched_at
            .cmp(&left.snapshot.fetched_at)
            .then_with(|| left.source_file.cmp(&right.source_file))
    });
    Ok(snapshots)
}

fn read_snapshot(path: &Path) -> Result<ShortsSnapshot> {
    let reader = BufReader::new(File::open(path).context("opening shorts JSON")?);
    let snapshot: ShortsSnapshot =
        serde_json::from_reader(reader).context("invalid shorts JSON")?;
    ensure!(
        !snapshot.code.trim().is_empty() && !snapshot.isin.trim().is_empty(),
        "missing company code or ISIN in shorts snapshot"
    );
    Ok(snapshot)
}

fn find_shorts_files(
    directory: &Path,
    files: &mut Vec<PathBuf>,
    skipped: &mut Vec<SkippedFile>,
) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                skipped.push(SkippedFile {
                    source_file: directory.to_path_buf(),
                    reason: error.to_string(),
                });
                continue;
            }
        };
        let path = entry.path();
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => {
                skipped.push(SkippedFile {
                    source_file: path,
                    reason: error.to_string(),
                });
                continue;
            }
        };
        if file_type.is_dir() {
            if let Err(error) = find_shorts_files(&path, files, skipped) {
                skipped.push(SkippedFile {
                    source_file: path,
                    reason: error.to_string(),
                });
            }
        } else if file_type.is_file() && entry.file_name() == "shorts.json" {
            files.push(path);
        }
    }
    Ok(())
}
