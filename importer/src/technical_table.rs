//! Offline ranking of archived histories using the unchanged technical score.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::technical::{CloseCandle, score};

#[derive(Debug, Serialize)]
pub struct TechnicalRow {
    pub code: String,
    pub as_of: NaiveDate,
    pub seasonality_points: Option<i8>,
    pub trend_points: i8,
    pub technical_score: Option<i8>,
    pub r: Option<Decimal>,
    pub years: usize,
    pub source_file: PathBuf,
}

#[derive(Debug)]
pub struct SkippedFile {
    pub source_file: PathBuf,
    pub reason: String,
}

#[derive(Debug)]
pub struct TechnicalTable {
    pub rows: Vec<TechnicalRow>,
    pub skipped: Vec<SkippedFile>,
}

#[derive(Deserialize)]
struct Company {
    code: String,
}

/// Reuse CloseCandle's precise date/close decoding, ignoring other provider fields.
#[derive(Deserialize)]
struct HistoryInput {
    instrument: Company,
    fetched_at: DateTime<Utc>,
    candles: Vec<CloseCandle>,
}

struct SelectedHistory {
    input: HistoryInput,
    source_file: PathBuf,
}

/// Select the latest fetch per company, then score and sort the usable histories.
/// An empty result is returned with its diagnostics for the CLI to report.
pub fn rank_directory(directory: &Path, as_of: NaiveDate) -> io::Result<TechnicalTable> {
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    find_history_files(directory, &mut files, &mut skipped)?;
    // Equal fetch timestamps keep the first path in lexical order.
    files.sort();

    let mut selected: BTreeMap<String, SelectedHistory> = BTreeMap::new();
    for source_file in files {
        let input = match read_history(&source_file) {
            Ok(input) => input,
            Err(error) => {
                skipped.push(SkippedFile {
                    source_file,
                    reason: format!("{error:#}"),
                });
                continue;
            }
        };
        let code = input.instrument.code.clone();
        let candidate = SelectedHistory { input, source_file };
        if let Some(current) = selected.get_mut(&code) {
            let discarded = if candidate.input.fetched_at > current.input.fetched_at {
                std::mem::replace(current, candidate)
            } else {
                candidate
            };
            skipped.push(SkippedFile {
                source_file: discarded.source_file,
                reason: format!(
                    "duplicate history for {code}; kept {} (fetched_at {})",
                    current.source_file.display(),
                    current.input.fetched_at
                ),
            });
        } else {
            selected.insert(code, candidate);
        }
    }

    let mut rows = Vec::new();
    for (code, history) in selected {
        match score(&history.input.candles, as_of) {
            Ok(result) => rows.push(TechnicalRow {
                code,
                as_of: result.as_of,
                seasonality_points: result.seasonality_points,
                trend_points: result.trend_points,
                technical_score: result.technical_score,
                r: result.explanation.r,
                years: result.explanation.years,
                source_file: history.source_file,
            }),
            Err(error) => skipped.push(SkippedFile {
                source_file: history.source_file,
                reason: format!("{code}: {error}"),
            }),
        }
    }
    rows.sort_by(|left, right| {
        right
            .technical_score
            .cmp(&left.technical_score)
            .then_with(|| left.code.cmp(&right.code))
    });
    Ok(TechnicalTable { rows, skipped })
}

fn read_history(path: &Path) -> Result<HistoryInput> {
    let reader = BufReader::new(File::open(path).context("opening history JSON")?);
    let input: HistoryInput = serde_json::from_reader(reader).context("invalid history JSON")?;
    ensure!(
        !input.instrument.code.trim().is_empty(),
        "missing company code in instrument"
    );
    Ok(input)
}

fn find_history_files(
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
            if let Err(error) = find_history_files(&path, files, skipped) {
                skipped.push(SkippedFile {
                    source_file: path,
                    reason: error.to_string(),
                });
            }
        } else if file_type.is_file() && entry.file_name() == "history.json" {
            files.push(path);
        }
    }
    Ok(())
}
