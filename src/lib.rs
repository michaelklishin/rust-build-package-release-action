use std::env;

pub mod archive;
pub mod aur;
pub mod build;
pub mod cargo_info;
pub mod changelog;
pub mod checksum;
pub mod collect_artifacts;
pub mod download;
pub mod error;
pub mod format_release;
pub mod homebrew;
pub mod nfpm;
pub mod output;
pub mod platform;
pub mod release;
pub mod sbom;
pub mod sign;
pub mod testing;
pub mod tools;
pub mod version;
pub mod winget;

/// Read an environment variable with a fallback default.
pub fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

/// Parse a comma-separated string into trimmed non-empty values.
pub fn parse_comma_list(raw: &str) -> Vec<String> {
    if raw.is_empty() {
        return Vec::new();
    }
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
