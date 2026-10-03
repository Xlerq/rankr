//! Offline selection of the latest fundamental snapshot and its raw ratios.

use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, BufReader},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::fundamental_ratios::{FundamentalRatioInput, FundamentalRatios, calculate};

#[derive(Debug, Serialize)]
pub struct FundamentalRow {
    pub code: String,
    pub report_period: String,
    pub currency: String,
    pub consolidated: bool,
    pub unit_multiplier: u32,
    #[serde(flatten)]
    pub ratios: FundamentalRatios,
    pub source_file: PathBuf,
}

#[derive(Debug)]
pub struct SkippedFile {
    pub source_file: PathBuf,
    pub reason: String,
}

#[derive(Debug)]
pub struct FundamentalTable {
    pub rows: Vec<FundamentalRow>,
    pub skipped: Vec<SkippedFile>,
}

#[derive(Debug, Default)]
pub struct RatioNullCounts {
    pub net_margin: usize,
    pub operating_margin: usize,
    pub equity_ratio: usize,
    pub debt_to_equity: usize,
    pub ocf_to_net_income: usize,
}

impl FundamentalTable {
    pub fn null_counts(&self) -> RatioNullCounts {
        let mut counts = RatioNullCounts::default();
        for row in &self.rows {
            counts.net_margin += usize::from(row.ratios.net_margin.is_none());
            counts.operating_margin += usize::from(row.ratios.operating_margin.is_none());
            counts.equity_ratio += usize::from(row.ratios.equity_ratio.is_none());
            counts.debt_to_equity += usize::from(row.ratios.debt_to_equity.is_none());
            counts.ocf_to_net_income += usize::from(row.ratios.ocf_to_net_income.is_none());
        }
        counts
    }
}

#[derive(Deserialize)]
struct Company {
    code: String,
}

#[derive(Deserialize)]
struct ReportInput {
    report_period: String,
    currency: String,
    consolidated: bool,
    #[serde(flatten)]
    amounts: FundamentalRatioInput,
}

#[derive(Deserialize)]
struct SnapshotInput {
    instrument: Company,
    fetched_at: DateTime<Utc>,
    fundamentals: ReportInput,
}

struct SelectedSnapshot {
    input: SnapshotInput,
    source_file: PathBuf,
}

/// Read archived snapshots, keeping only the latest fetched_at for each code.
/// Empty results retain their diagnostics so the CLI can report them before failing.
pub fn build_table(directory: &Path) -> io::Result<FundamentalTable> {
    let mut files = Vec::new();
    let mut skipped = Vec::new();
    find_fundamental_files(directory, &mut files, &mut skipped)?;
    // As in technical-table, equal timestamps keep the first lexical path.
    files.sort();

    let mut selected: BTreeMap<String, SelectedSnapshot> = BTreeMap::new();
    for source_file in files {
        let input = match read_snapshot(&source_file) {
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
        let candidate = SelectedSnapshot { input, source_file };
        if let Some(current) = selected.get_mut(&code) {
            let discarded = if candidate.input.fetched_at > current.input.fetched_at {
                std::mem::replace(current, candidate)
            } else {
                candidate
            };
            skipped.push(SkippedFile {
                source_file: discarded.source_file,
                reason: format!(
                    "duplicate snapshot for {code}; kept {} (fetched_at {})",
                    current.source_file.display(),
                    current.input.fetched_at
                ),
            });
        } else {
            selected.insert(code, candidate);
        }
    }

    let mut rows = Vec::new();
    // BTreeMap iteration gives the requested company-code order.
    for (code, snapshot) in selected {
        let report = snapshot.input.fundamentals;
        match calculate(&report.amounts) {
            Ok(ratios) => rows.push(FundamentalRow {
                code,
                report_period: report.report_period,
                currency: report.currency,
                consolidated: report.consolidated,
                unit_multiplier: report.amounts.unit_multiplier,
                ratios,
                source_file: snapshot.source_file,
            }),
            Err(error) => skipped.push(SkippedFile {
                source_file: snapshot.source_file,
                reason: format!("{code}: {error}"),
            }),
        }
    }
    Ok(FundamentalTable { rows, skipped })
}

fn read_snapshot(path: &Path) -> Result<SnapshotInput> {
    let reader = BufReader::new(File::open(path).context("opening fundamentals JSON")?);
    let input: SnapshotInput =
        serde_json::from_reader(reader).context("invalid fundamentals JSON")?;
    ensure!(
        !input.instrument.code.trim().is_empty(),
        "missing company code in instrument"
    );
    Ok(input)
}

fn find_fundamental_files(
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
            if let Err(error) = find_fundamental_files(&path, files, skipped) {
                skipped.push(SkippedFile {
                    source_file: path,
                    reason: error.to_string(),
                });
            }
        } else if file_type.is_file() && entry.file_name() == "fundamentals.json" {
            files.push(path);
        }
    }
    Ok(())
}
