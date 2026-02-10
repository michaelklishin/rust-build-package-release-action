use crate::cargo_info::get_cargo_info;
use crate::env_or;
use crate::error::{Error, Result};
use crate::output::output;
use regex::Regex;
use serde_json::Value;
use std::path::Path;
use std::process::{self, Command};

/// Semver pattern: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]
pub fn semver_pattern() -> Regex {
    Regex::new(
        r"^\d+\.\d+\.\d+(-[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?(\+[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?$",
    )
    .unwrap()
}

/// Validates that a semver string is well-formed.
pub fn is_valid_semver(version: &str) -> bool {
    semver_pattern().is_match(version)
}

/// Extracts version from a tag string (strips leading 'v').
pub fn version_from_tag(tag: &str) -> Option<&str> {
    tag.strip_prefix('v')
}

pub fn run_validate_version() -> Result<()> {
    let tag = env_or("TAG", &env_or("GITHUB_REF_NAME", ""));
    let expected = env_or("EXPECTED_VERSION", &env_or("NEXT_RELEASE_VERSION", ""));
    let validate_cargo = env_or("VALIDATE_CARGO_TOML", "") == "true";

    if tag.is_empty() {
        eprintln!("\x1b[31mERROR:\x1b[0m GITHUB_REF_NAME is not available");
        eprintln!();
        eprintln!("Set TAG to the git tag being released (e.g., v1.2.3)");
        process::exit(1);
    }

    if expected.is_empty() {
        eprintln!("\x1b[31mERROR:\x1b[0m NEXT_RELEASE_VERSION variable is not set");
        eprintln!();
        eprintln!("Set it at: Settings > Secrets and variables > Actions > Variables");
        process::exit(1);
    }

    if !tag.starts_with('v') {
        eprintln!("\x1b[31mERROR:\x1b[0m Tag should start with 'v', got '{tag}'");
        eprintln!();
        eprintln!("Push a tag like: git tag v{expected} && git push origin v{expected}");
        process::exit(1);
    }

    let tag_version = &tag[1..];

    if !is_valid_semver(tag_version) {
        eprintln!("\x1b[31mERROR:\x1b[0m Invalid version format: {tag_version}");
        eprintln!();
        eprintln!("Expected semantic versioning: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]");
        eprintln!("Examples: 1.2.3, 1.0.0-alpha.1, 2.0.0-rc.1+build.123");
        process::exit(1);
    }

    if expected == tag_version {
        println!("\x1b[32mVersion validated:\x1b[0m {expected} matches tag {tag}");
        output("version", tag_version);
    } else {
        eprintln!(
            "\x1b[31mERROR:\x1b[0m NEXT_RELEASE_VERSION ({expected}) does not match tag ({tag})"
        );
        eprintln!();
        eprintln!("Either:");
        eprintln!(
            "  1. Update NEXT_RELEASE_VERSION to '{tag_version}' at: Settings > Secrets and variables > Actions > Variables"
        );
        eprintln!(
            "  2. Or push the correct tag: git tag v{expected} && git push origin v{expected}"
        );
        process::exit(1);
    }

    if validate_cargo {
        let cargo_info = get_cargo_info()?;
        let cargo_version = &cargo_info.version;
        if cargo_version.is_empty() {
            return Err(Error::User(
                "Could not read version from Cargo.toml".to_string(),
            ));
        }
        if cargo_version != tag_version {
            eprintln!(
                "\x1b[31mERROR:\x1b[0m Cargo.toml version ({cargo_version}) does not match tag ({tag_version})"
            );
            eprintln!();
            eprintln!("Update Cargo.toml version to '{tag_version}' before tagging");
            process::exit(1);
        }
        println!("\x1b[32mCargo.toml validated:\x1b[0m version {cargo_version} matches tag");
    }

    Ok(())
}

pub fn run_get_version() -> Result<()> {
    let manifest_path = env_or("MANIFEST_PATH", "Cargo.toml");
    if !Path::new(&manifest_path).exists() {
        return Err(Error::User(format!("manifest not found: {manifest_path}")));
    }

    let info = get_cargo_info()?;
    let version = &info.version;

    if version.is_empty() {
        eprintln!("\x1b[31mERROR:\x1b[0m no version found in Cargo.toml");
        eprintln!();
        eprintln!("Ensure [package] or [workspace.package] has a version field");
        process::exit(1);
    }

    let version_start = Regex::new(r"^\d+\.\d+\.\d+").unwrap();
    if !version_start.is_match(version) {
        return Err(Error::User(format!("invalid version format: {version}")));
    }

    println!("{version}");
    output("version", version);
    Ok(())
}

pub fn run_get_release_version() -> Result<()> {
    let version_override = env_or("VERSION", "");

    let version = if !version_override.is_empty() {
        version_override
    } else {
        let repo = env_or("GITHUB_REPOSITORY", "");
        if repo.is_empty() {
            return Err(Error::User("GITHUB_REPOSITORY not set".to_string()));
        }

        println!("\x1b[32mFetching latest release from\x1b[0m {repo}");

        let gh_token = env_or("GITHUB_TOKEN", &env_or("GH_TOKEN", ""));
        let tag = if !gh_token.is_empty() {
            let result = Command::new("gh")
                .args(["release", "view", "--repo", &repo, "--json", "tagName"])
                .output()
                .map_err(|e| Error::User(format!("failed to run gh: {e}")))?;
            if !result.status.success() {
                return Err(Error::User(format!(
                    "failed to fetch release: {}",
                    String::from_utf8_lossy(&result.stderr)
                )));
            }
            let json: Value = serde_json::from_slice(&result.stdout)?;
            json["tagName"].as_str().unwrap_or_default().to_string()
        } else {
            let api_url = format!("https://api.github.com/repos/{repo}/releases/latest");
            let result = Command::new("curl")
                .args(["-fsSL", &api_url])
                .output()
                .map_err(|e| Error::User(format!("failed to run curl: {e}")))?;
            if !result.status.success() {
                return Err(Error::User(format!(
                    "failed to fetch release: {}",
                    String::from_utf8_lossy(&result.stderr)
                )));
            }
            let json: Value = serde_json::from_slice(&result.stdout)?;
            json["tag_name"].as_str().unwrap_or_default().to_string()
        };

        if tag.is_empty() {
            return Err(Error::User("no releases found".to_string()));
        }

        tag.strip_prefix('v').unwrap_or(&tag).to_string()
    };

    let first_char = version.chars().next().unwrap_or(' ');
    if !first_char.is_ascii_digit() {
        return Err(Error::User(format!("invalid version format: {version}")));
    }

    println!("\x1b[32mVersion:\x1b[0m {version}");
    output("version", &version);
    Ok(())
}
