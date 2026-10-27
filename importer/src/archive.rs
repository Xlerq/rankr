//! Recursive lookup of archived JSON files shared by the offline tables.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// A file or directory ignored while building a table, with the reason.
#[derive(Debug)]
pub struct SkippedFile {
    pub source_file: PathBuf,
    pub reason: String,
}

/// Find every file called `name` below `directory`. Unreadable entries are
/// recorded in `skipped`; only an unreadable `directory` itself is an error.
pub fn find_files(
    directory: &Path,
    name: &str,
    skipped: &mut Vec<SkippedFile>,
) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect(directory, name, &mut files, skipped)?;
    Ok(files)
}

fn collect(
    directory: &Path,
    name: &str,
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
            if let Err(error) = collect(&path, name, files, skipped) {
                skipped.push(SkippedFile {
                    source_file: path,
                    reason: error.to_string(),
                });
            }
        } else if file_type.is_file() && entry.file_name() == name {
            files.push(path);
        }
    }
    Ok(())
}
