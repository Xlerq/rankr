//! Offline fundamental ranking using the existing latest-snapshot ratio table.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

use crate::{
    archive::SkippedFile,
    fundamental_points::{FundamentalPoints, score},
    fundamental_table,
};

#[derive(Debug, Serialize)]
pub struct FundamentalPointsRow {
    pub code: String,
    pub gics_sector: String,
    pub gics_industry_group: String,
    #[serde(flatten)]
    pub points: FundamentalPoints,
    pub source_file: PathBuf,
}

#[derive(Debug)]
pub struct FundamentalPointsTable {
    pub rows: Vec<FundamentalPointsRow>,
    pub skipped: Vec<SkippedFile>,
}

#[derive(Deserialize)]
pub(crate) struct GicsEntry {
    gpw_code: String,
    gics_sector_code: u16,
    pub(crate) gics_sector: String,
    gics_industry_group: String,
}

pub fn build_table(directory: &Path) -> Result<FundamentalPointsTable> {
    let classifications = gics_map()?;
    let table = fundamental_table::build_table(directory)?;
    let mut skipped = table.skipped;
    let mut rows = Vec::new();
    for row in table.rows {
        let Some(gics) = classifications.get(&row.code) else {
            skipped.push(SkippedFile {
                reason: format!("missing GICS mapping for {}", row.code),
                source_file: row.source_file,
            });
            continue;
        };
        rows.push(FundamentalPointsRow {
            code: row.code,
            gics_sector: gics.gics_sector.clone(),
            gics_industry_group: gics.gics_industry_group.clone(),
            points: score(row.ratios, gics.gics_sector_code),
            source_file: row.source_file,
        });
    }
    rows.sort_by(|left, right| {
        right
            .points
            .fundamental_score
            .cmp(&left.points.fundamental_score)
            .then_with(|| left.code.cmp(&right.code))
    });
    Ok(FundamentalPointsTable { rows, skipped })
}

pub(crate) fn gics_map() -> Result<BTreeMap<String, GicsEntry>> {
    let mut reader =
        csv::Reader::from_reader(include_str!("../../data/raw/wig20_symbols.csv").as_bytes());
    let mut entries = BTreeMap::new();
    for entry in reader.deserialize() {
        let entry: GicsEntry = entry.context("reading bundled GICS mapping")?;
        let code = entry.gpw_code.clone();
        ensure!(
            !code.is_empty()
                && !entry.gics_sector.is_empty()
                && !entry.gics_industry_group.is_empty(),
            "incomplete GICS mapping for {code}"
        );
        ensure!(
            entries.insert(code.clone(), entry).is_none(),
            "duplicate GICS mapping for {code}"
        );
    }
    Ok(entries)
}
