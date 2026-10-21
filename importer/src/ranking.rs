//! Offline combination of the existing fundamental and technical tables.

use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result};
use chrono::NaiveDate;
use serde::Serialize;

use crate::{fundamental_points_table, fundamental_table::SkippedFile, technical_table};

const FUNDAMENTAL_WEIGHT: i16 = 3;
const TECHNICAL_WEIGHT: i16 = 1;

#[derive(Debug, Serialize)]
pub struct RankingRow {
    pub code: String,
    pub gics_sector: String,
    pub fundamental_score: Option<i8>,
    pub technical_score: Option<i8>,
    pub sentiment_score: Option<i8>,
    pub fundamental_weight: i16,
    pub technical_weight: i16,
    pub sentiment_weight: i16,
    pub final_score: Option<i16>,
}

#[derive(Debug)]
pub struct Ranking {
    pub rows: Vec<RankingRow>,
    pub skipped: Vec<SkippedFile>,
}

/// Join the existing family tables by code without changing their selection rules.
pub fn rank_directory(directory: &Path, as_of: NaiveDate) -> Result<Ranking> {
    let fundamentals = fundamental_points_table::build_table(directory)
        .context("building the fundamental points table")?;
    let technical = technical_table::rank_directory(directory, as_of)
        .context("building the technical table")?;

    let mut skipped = fundamentals.skipped;
    for file in technical.skipped {
        skipped.push(SkippedFile {
            source_file: file.source_file,
            reason: file.reason,
        });
    }
    let mut histories = BTreeMap::new();
    for row in technical.rows {
        histories.insert(row.code.clone(), row);
    }

    let mut rows = Vec::new();
    for fundamental in fundamentals.rows {
        let Some(history) = histories.remove(&fundamental.code) else {
            skipped.push(SkippedFile {
                source_file: fundamental.source_file,
                reason: format!(
                    "{}: missing usable history.json for pairing",
                    fundamental.code
                ),
            });
            continue;
        };
        let fundamental_score = fundamental.points.fundamental_score;
        let technical_score = history.technical_score;
        // Both families are required. Sentiment is unavailable and is not a term.
        let final_score = match (fundamental_score, technical_score) {
            (Some(fundamental), Some(technical)) => Some(
                FUNDAMENTAL_WEIGHT * i16::from(fundamental)
                    + TECHNICAL_WEIGHT * i16::from(technical),
            ),
            _ => None,
        };
        rows.push(RankingRow {
            code: fundamental.code,
            gics_sector: fundamental.gics_sector,
            fundamental_score,
            technical_score,
            sentiment_score: None,
            fundamental_weight: FUNDAMENTAL_WEIGHT,
            technical_weight: TECHNICAL_WEIGHT,
            sentiment_weight: 0,
            final_score,
        });
    }
    for (code, history) in histories {
        skipped.push(SkippedFile {
            source_file: history.source_file,
            reason: format!("{code}: missing usable fundamentals.json for pairing"),
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
