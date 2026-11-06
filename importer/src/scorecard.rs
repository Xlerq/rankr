//! Presentation data selected through the same archive tables as the ranking.

use std::{fs::File, io::BufReader, path::Path};

use anyhow::Result;
use chrono::NaiveDate;

use crate::{
    fundamental_points_table::{self, FundamentalPointsRow},
    model::{FundamentalSnapshot, ShortsSnapshot},
    ranking::{self, RankingRow},
    technical::{self, TechnicalHistory, TechnicalScore},
    technical_table,
};

pub(crate) struct CompanyCard {
    pub row: RankingRow,
    pub fundamental: Option<(FundamentalPointsRow, FundamentalSnapshot)>,
    pub technical: Option<TechnicalScore>,
    pub newer_shorts: Option<ShortsSnapshot>,
    pub older_shorts: Option<ShortsSnapshot>,
}

pub(crate) fn load(directory: &Path, code: &str, as_of: NaiveDate) -> Result<Option<CompanyCard>> {
    let Some(row) = ranking::rank_directory(directory, as_of)?
        .rows
        .into_iter()
        .find(|row| row.code == code)
    else {
        return Ok(None);
    };
    let fundamental = fundamental_points_table::build_table(directory)?
        .rows
        .into_iter()
        .find(|row| row.code == code)
        .map(|row| -> Result<_> {
            let report = serde_json::from_reader(BufReader::new(File::open(&row.source_file)?))?;
            Ok((row, report))
        })
        .transpose()?;
    let technical = technical_table::rank_directory(directory, as_of)?
        .rows
        .into_iter()
        .find(|row| row.code == code)
        .map(|row| -> Result<_> {
            let history: TechnicalHistory =
                serde_json::from_reader(BufReader::new(File::open(row.source_file)?))?;
            Ok(technical::score(&history.candles, as_of)?)
        })
        .transpose()?;
    let (newer_shorts, older_shorts) = ranking::shorts_pair(directory, code)?;
    Ok(Some(CompanyCard {
        row,
        fundamental,
        technical,
        newer_shorts,
        older_shorts,
    }))
}
