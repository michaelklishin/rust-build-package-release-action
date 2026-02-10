use crate::error::{Error, Result};
use crate::output::{output, output_multiline};
use regex::Regex;
use std::path::Path;
use std::{env, fs};

/// Extracts a version's section from a changelog.
pub fn extract_changelog_section(content: &str, version: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let version_header = format!("## v{version}");
    let alt_header = format!("## {version}");

    let start_idx = lines
        .iter()
        .position(|line| line.starts_with(&version_header) || line.starts_with(&alt_header))
        .ok_or_else(|| Error::User(format!("version {version} not found in changelog")))?;

    let remaining = &lines[start_idx + 1..];
    let next_version_re = Regex::new(r"^## v?\d+\.\d+\.\d+").unwrap();
    let end_offset = remaining
        .iter()
        .position(|line| next_version_re.is_match(line))
        .unwrap_or(remaining.len());

    let section = &lines[start_idx..start_idx + 1 + end_offset];
    let notes = section.join("\n");

    if notes.trim().is_empty() {
        return Err(Error::User(format!(
            "no content found for version {version}"
        )));
    }

    Ok(notes)
}

/// Validates that a changelog contains an entry for the given version.
pub fn validate_changelog_entry(content: &str, version: &str) -> bool {
    let version_header = format!("## v{version}");
    let alt_header = format!("## {version}");
    content
        .lines()
        .any(|line| line.starts_with(&version_header) || line.starts_with(&alt_header))
}

pub fn run_extract_changelog() -> Result<()> {
    let version = env::var("VERSION").unwrap_or_default();
    if version.is_empty() {
        return Err(Error::User(
            "VERSION environment variable is required".to_string(),
        ));
    }

    let changelog_path = env::var("CHANGELOG_PATH").unwrap_or_else(|_| "CHANGELOG.md".into());
    let output_path = env::var("OUTPUT_PATH").unwrap_or_else(|_| "release_notes.md".into());

    if !Path::new(&changelog_path).exists() {
        return Err(Error::User(format!(
            "changelog not found: {changelog_path}"
        )));
    }

    let content = fs::read_to_string(&changelog_path)?;
    let notes = extract_changelog_section(&content, &version)?;

    fs::write(&output_path, &notes)?;
    println!("\x1b[32mExtracted\x1b[0m release notes for v{version} to {output_path}");
    output("version", &version);
    output("release_notes_file", &output_path);
    output_multiline("release_notes", &notes);
    Ok(())
}

pub fn run_validate_changelog() -> Result<()> {
    let version = env::var("VERSION").unwrap_or_default();
    if version.is_empty() {
        return Err(Error::User(
            "VERSION environment variable is required".to_string(),
        ));
    }

    let changelog_path = env::var("CHANGELOG_PATH").unwrap_or_else(|_| "CHANGELOG.md".into());

    if !Path::new(&changelog_path).exists() {
        return Err(Error::User(format!(
            "changelog not found: {changelog_path}"
        )));
    }

    let content = fs::read_to_string(&changelog_path)?;

    if validate_changelog_entry(&content, &version) {
        println!("\x1b[32mChangelog validated:\x1b[0m found entry for v{version}");
        output("version", &version);
        output("valid", "true");
        Ok(())
    } else {
        Err(Error::User(format!(
            "No changelog entry found for version {version}. Expected header like '## v{version}' or '## {version}'"
        )))
    }
}
