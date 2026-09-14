//! Local JSON archive. HTTP and HTML parsing do not depend on this module.

use std::{
    fs::{self, File},
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
};

use serde::{Serialize, de::DeserializeOwned};
use tempfile::{Builder, NamedTempFile};
use thiserror::Error;

use crate::model::{FundamentalSnapshot, RawDocument};

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

    pub fn save_error(&self, error: &str) -> Result<(), StorageError> {
        #[derive(Serialize)]
        struct Failure<'a> {
            error: &'a str,
        }
        write_json(&self.directory.join("error.json"), &Failure { error })
    }
}

/// Archive raw data first. Random suffixes prevent collisions between runs.
pub fn archive_raw(root: &Path, raw: &RawDocument) -> Result<ArchivedFetch, StorageError> {
    fs::create_dir_all(root)?;
    let prefix = raw.fetched_at.format("%Y%m%dT%H%M%S%.9fZ-").to_string();
    let directory = Builder::new().prefix(&prefix).tempdir_in(root)?;
    write_json(&directory.path().join("raw.json"), raw)?;
    Ok(ArchivedFetch {
        directory: directory.keep(),
    })
}

pub fn read_raw(path: &Path) -> Result<RawDocument, StorageError> {
    read_json(path)
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, StorageError> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}

/// Publish a complete file and refuse to overwrite an existing observation.
fn write_json(path: &Path, value: &impl Serialize) -> Result<(), StorageError> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(&mut temporary, value)?;
    temporary.write_all(b"\n")?;
    temporary.as_file().sync_all()?;
    temporary.persist_noclobber(path).map_err(|error| error.error)?;
    Ok(())
}

