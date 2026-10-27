use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct SkippedFile {
    pub source_file: PathBuf,
    pub reason: String,
}

pub fn find_files(
    directory: &Path,
    name: &str,
    skipped: &mut Vec<SkippedFile>,
) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
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
            match find_files(&path, name, skipped) {
                Ok(found) => files.extend(found),
                Err(error) => skipped.push(SkippedFile {
                    source_file: path,
                    reason: error.to_string(),
                }),
            }
        } else if file_type.is_file() && entry.file_name() == name {
            files.push(path);
        }
    }
    Ok(files)
}
