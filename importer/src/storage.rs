//! Local JSON snapshots and atomic replacement of the growing price history.

use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::Serialize;
use tempfile::NamedTempFile;
use thiserror::Error;

use crate::model::{DailyPriceHistory, FundamentalSnapshot, RawDocument, ShortsSnapshot, Source};

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("file operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

/// An immutable fetch directory. Each collection preserves previous observations.
pub struct ArchivedFetch {
    directory: PathBuf,
}

impl ArchivedFetch {
    pub fn raw_path(&self) -> PathBuf {
        self.directory.join("raw.json")
    }

    /// A failed parse leaves raw.json available for offline recovery.
    pub fn save_snapshot(&self, snapshot: &FundamentalSnapshot) -> Result<PathBuf, StorageError> {
        let path = self.directory.join("fundamentals.json");
        write_json(&path, snapshot)?;
        Ok(path)
    }

    pub fn save_history(&self, history: &DailyPriceHistory) -> Result<PathBuf, StorageError> {
        let path = self.directory.join("history.json");
        write_json(&path, history)?;
        Ok(path)
    }

    pub fn save_shorts(&self, snapshot: &ShortsSnapshot) -> Result<PathBuf, StorageError> {
        let path = self.directory.join("shorts.json");
        write_json(&path, snapshot)?;
        Ok(path)
    }

    /// The first response is raw.json; subsequent pages are saved before parsing too.
    pub fn save_raw_page(&self, page: usize, raw: &RawDocument) -> Result<PathBuf, StorageError> {
        let path = self.directory.join(format!("raw_page_{page}.json"));
        write_json(&path, raw)?;
        Ok(path)
    }

    pub fn save_error(&self, error: &str) -> Result<(), StorageError> {
        #[derive(Serialize)]
        struct Failure<'a> {
            error: &'a str,
        }
        write_json(&self.directory.join("error.json"), &Failure { error })
    }
}

/// Archive raw data first, e.g. 2026-10-18_KGHM_history/raw.json.
pub fn archive_raw(root: &Path, raw: &RawDocument) -> Result<ArchivedFetch, StorageError> {
    fs::create_dir_all(root)?;
    let name = archive_name(raw);
    let directory = create_archive_directory(root, &name)?;
    if let Err(error) = write_json(&directory.join("raw.json"), raw) {
        // Remove only the empty directory created by this call.
        let _ = fs::remove_dir(&directory);
        return Err(error);
    }
    Ok(ArchivedFetch { directory })
}

fn archive_name(raw: &RawDocument) -> String {
    let date = raw.fetched_at.format("%Y-%m-%d");
    let kind = match raw.source {
        Source::GpwNotoria => "fundamentals",
        Source::YahooFinance => "history",
        Source::KnfShorts => "shorts",
    };
    match &raw.instrument {
        Some(instrument) => {
            // Company codes come from external data; keep them inside one directory name.
            let code = instrument.code.replace(
                |character: char| {
                    !character.is_ascii_alphanumeric() && character != '-' && character != '_'
                },
                "_",
            );
            format!("{date}_{code}_{kind}")
        }
        None => format!("{date}_{kind}"),
    }
}

fn create_archive_directory(root: &Path, name: &str) -> io::Result<PathBuf> {
    let mut directory = root.join(name);
    let mut number = 2;
    loop {
        // Creating the directory atomically also handles simultaneous collectors.
        match fs::create_dir(&directory) {
            Ok(()) => return Ok(directory),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                directory = root.join(format!("{name}_{number}"));
                number += 1;
            }
            Err(error) => return Err(error),
        }
    }
}

/// Publish a complete file and refuse to overwrite an existing observation.
fn write_json(path: &Path, value: &impl Serialize) -> Result<(), StorageError> {
    json_file(path, value)?
        .persist_noclobber(path)
        .map_err(|error| error.error)?;
    Ok(())
}

/// Replace only the growing history; a failed write leaves the old file intact.
pub fn replace_history(path: &Path, history: &DailyPriceHistory) -> Result<(), StorageError> {
    json_file(path, history)?
        .persist(path)
        .map_err(|error| error.error)?;
    Ok(())
}

fn json_file(path: &Path, value: &impl Serialize) -> Result<NamedTempFile, StorageError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(&mut temporary, value)?;
    temporary.write_all(b"\n")?;
    temporary.as_file().sync_all()?;
    Ok(temporary)
}
