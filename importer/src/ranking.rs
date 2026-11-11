//! Offline combination of the existing three family scores.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::{
    archive::{SkippedFile, find_files},
    fundamental_points_table,
    model::ShortsSnapshot,
    sentiment, storage, technical_table,
};

/// Increment when scoring formulas, observation periods, thresholds or family weights change.
pub const SCORING_VERSION: &str = "v2-weekly-sentiment";

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

/// Calculate and archive a ranking for the supplied UTC timestamp.
pub fn archive_snapshot(directory: &Path, computed_at: DateTime<Utc>) -> Result<Option<PathBuf>> {
    let ranking = rank_directory(directory, computed_at.date_naive())?;
    storage::save_ranking(directory, computed_at, &ranking.rows)
        .context("saving the ranking snapshot")
}

/// Join family scores by code. Sentiment uses the two ISO weeks ending at as_of;
/// the technical calculation uses only candles before as_of.
pub fn rank_directory(directory: &Path, as_of: NaiveDate) -> Result<Ranking> {
    let fundamentals = fundamental_points_table::build_table(directory)
        .context("building the fundamental points table")?;
    let technical = technical_table::rank_directory(directory, as_of)
        .context("building the technical table")?;
    let classifications = fundamental_points_table::gics_map()?;

    let mut skipped = fundamentals.skipped;
    skipped.extend(technical.skipped);
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
        let (newer, older) = sentiment::weekly_pair(
            shorts
                .iter()
                .map(|archived| &archived.snapshot)
                .filter(|snapshot| snapshot.code.as_str() == code.as_str()),
            as_of,
        );
        let sentiment_score = sentiment::score(older, newer);
        let final_score = combine_scores(fundamental_score, technical_score, sentiment_score);
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

pub(crate) fn combine_scores(
    fundamental: Option<i8>,
    technical: Option<i8>,
    sentiment: Option<i8>,
) -> Option<i8> {
    Some(fundamental? + technical? + sentiment?)
}

/// Select the latest observation from each of the two scoring weeks.
pub(crate) fn shorts_pair(
    shorts: &[ShortsSnapshot],
    as_of: NaiveDate,
) -> (Option<ShortsSnapshot>, Option<ShortsSnapshot>) {
    let (newer, older) = sentiment::weekly_pair(shorts.iter(), as_of);
    (newer.cloned(), older.cloned())
}

/// Newest first, with the same timestamp/path tie rule as the ranking.
pub(crate) fn company_shorts(files: &[PathBuf], code: &str) -> Vec<ShortsSnapshot> {
    read_shorts_files(files.to_vec(), &mut Vec::new())
        .into_iter()
        .map(|archived| archived.snapshot)
        .filter(|snapshot| snapshot.code == code)
        .collect()
}

fn read_shorts(
    directory: &Path,
    skipped: &mut Vec<SkippedFile>,
) -> io::Result<Vec<ArchivedShorts>> {
    let files = find_files(directory, "shorts.json", skipped)?;
    Ok(read_shorts_files(files, skipped))
}

fn read_shorts_files(files: Vec<PathBuf>, skipped: &mut Vec<SkippedFile>) -> Vec<ArchivedShorts> {
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
    snapshots
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
