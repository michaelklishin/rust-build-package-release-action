use crate::error::{Error, Result};
use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;

pub fn list_archivable_files(dir: &Path) -> Vec<String> {
    let exclude = Regex::new(
        r"\.(tar\.gz|zip|sha256|sha512|b2|sig|pem|sigstore\.json|spdx\.json|cdx\.json)$",
    )
    .unwrap();
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if !exclude.is_match(&name) {
                files.push(name);
            }
        }
    }
    files.sort();
    files
}

/// Copies LICENSE* and README.md to the destination.
pub fn copy_docs(dest: &Path) -> Result<()> {
    for entry in glob::glob("LICENSE*").unwrap().flatten() {
        let dest_file = dest.join(entry.file_name().unwrap_or_default());
        fs::copy(&entry, &dest_file)?;
    }
    if Path::new("README.md").exists() {
        fs::copy("README.md", dest.join("README.md"))?;
    }
    Ok(())
}

/// Copies additional include files to the destination.
pub fn copy_includes(dest: &Path) -> Result<()> {
    let includes = env::var("ARCHIVE_INCLUDE").unwrap_or_default();
    if includes.is_empty() {
        return Ok(());
    }
    for pattern in includes.split(',') {
        let pattern = pattern.trim();
        if pattern.is_empty() {
            continue;
        }
        let matches = glob::glob(pattern)
            .map_err(|e| Error::User(format!("invalid glob pattern '{pattern}': {e}")))?;
        for entry in matches.flatten() {
            if entry.is_file() {
                let dest_file = dest.join(entry.file_name().unwrap_or_default());
                fs::copy(&entry, &dest_file)?;
            }
        }
    }
    Ok(())
}
