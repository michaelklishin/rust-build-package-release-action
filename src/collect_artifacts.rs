use crate::checksum::sha256_file;
use crate::env_or;
use crate::error::{Error, Result};
use crate::output::{output, output_multiline};
use crate::platform::detect_platform_short;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize)]
struct ArtifactEntry {
    artifact: String,
    path: String,
    sha256: String,
    platform: String,
    url: String,
}

pub fn run_collect_artifacts() -> Result<()> {
    let artifacts_dir = env_or("ARTIFACTS_DIR", "artifacts");
    let base_url = env_or("BASE_URL", "");
    let artifacts_path = Path::new(&artifacts_dir);
    if !artifacts_path.exists() {
        return Err(Error::User(format!(
            "artifacts directory not found: {artifacts_dir}"
        )));
    }

    println!("\x1b[32mCollecting artifacts from:\x1b[0m {artifacts_dir}");

    let archive_re = Regex::new(r"\.(tar\.gz|zip|dmg|msi|deb|rpm|apk)$").unwrap();
    let mut artifact_names: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(artifacts_path) {
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
            if archive_re.is_match(&name) {
                artifact_names.push(name);
            }
        }
    }

    if artifact_names.is_empty() {
        return Err(Error::User(format!(
            "no artifacts found in {artifacts_dir}"
        )));
    }

    artifact_names.sort();

    let mut collection: Vec<ArtifactEntry> = Vec::new();
    for artifact in &artifact_names {
        let artifact_path = format!("{artifacts_dir}/{artifact}");
        let sha256 = sha256_file(Path::new(&artifact_path))?;
        let platform = detect_platform_short(artifact).to_string();
        let url = if !base_url.is_empty() {
            format!("{base_url}/{artifact}")
        } else {
            String::new()
        };

        collection.push(ArtifactEntry {
            artifact: artifact.clone(),
            path: artifact_path,
            sha256,
            platform,
            url,
        });
    }

    println!("\x1b[32mFound:\x1b[0m {} artifacts", collection.len());
    for a in &collection {
        println!("  {}: {}", a.platform, a.artifact);
    }

    // Output individual platform checksums
    for (platform, prefix) in &[
        ("macos-arm64", "macos_arm64"),
        ("macos-x64", "macos_x64"),
        ("linux-arm64", "linux_arm64"),
        ("linux-x64", "linux_x64"),
        ("windows-x64", "windows_x64"),
        ("windows-arm64", "windows_arm64"),
    ] {
        if let Some(a) = collection.iter().find(|a| a.platform == *platform) {
            output(&format!("{prefix}_sha256"), &a.sha256);
            output(&format!("{prefix}_url"), &a.url);
            output(&format!("{prefix}_artifact"), &a.artifact);
        }
    }

    // Output full collection as JSON
    let json = serde_json::to_string_pretty(&collection)?;
    output_multiline("collection", &json);

    // Generate consolidated checksums file
    let checksums_content: String = collection
        .iter()
        .map(|a| format!("{}  {}", a.sha256, a.artifact))
        .collect::<Vec<_>>()
        .join("\n");

    let checksums_path = format!("{artifacts_dir}/SHA256SUMS");
    let mut f = fs::File::create(&checksums_path)?;
    writeln!(f, "{checksums_content}")?;
    println!("\x1b[32mCreated:\x1b[0m {checksums_path}");
    output("checksums_file", &checksums_path);
    Ok(())
}
