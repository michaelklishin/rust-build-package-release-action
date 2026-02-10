use crate::env_or;
use crate::error::{Error, Result};
use crate::output::{output, output_multiline, print_hr};
use crate::platform::detect_platform_display;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Format file size in human-readable format.
pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let kb = bytes as f64 / 1024.0;
    if kb < 1024.0 {
        return format!("{:.1} KB", kb);
    }
    let mb = kb / 1024.0;
    format!("{:.1} MB", mb)
}

struct ArtifactInfo {
    name: String,
    size: String,
    platform: &'static str,
}

/// Lists release artifacts (excludes checksums, signatures, SBOM, and metadata).
fn list_release_artifacts(dir: &Path) -> Vec<ArtifactInfo> {
    let exclude =
        Regex::new(r"\.(sha256|sha512|b2|sig|pem|sigstore\.json|spdx\.json|cdx\.json)$").unwrap();

    let mut artifacts = Vec::new();
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
            if exclude.is_match(&name) {
                continue;
            }
            let size = path.metadata().map(|m| m.len()).unwrap_or(0);
            artifacts.push(ArtifactInfo {
                name: name.clone(),
                size: format_size(size),
                platform: detect_platform_display(&name),
            });
        }
    }
    artifacts.sort_by(|a, b| a.name.cmp(&b.name));
    artifacts
}

/// Formats artifacts as a Markdown table.
fn format_artifacts_table(artifacts: &[ArtifactInfo]) -> String {
    let mut table = "| Platform | File | Size |\n".to_string();
    table.push_str("|----------|------|------|\n");
    for a in artifacts {
        table.push_str(&format!("| {} | `{}` | {} |\n", a.platform, a.name, a.size));
    }
    table
}

/// Formats installation instructions for package managers.
fn format_installation_section(homebrew_tap: &str, aur_package: &str, winget_id: &str) -> String {
    let mut has_any = false;
    let mut section = "## Installation\n\n".to_string();

    if !homebrew_tap.is_empty() {
        has_any = true;
        section.push_str("**Homebrew:**\n```bash\n");
        section.push_str(&format!("brew install {homebrew_tap}\n"));
        section.push_str("```\n\n");
    }

    if !aur_package.is_empty() {
        has_any = true;
        section.push_str("**Arch Linux (AUR):**\n```bash\n");
        section.push_str(&format!("yay -S {aur_package}\n"));
        section.push_str("```\n\n");
    }

    if !winget_id.is_empty() {
        has_any = true;
        section.push_str("**Windows (winget):**\n```powershell\n");
        section.push_str(&format!("winget install {winget_id}\n"));
        section.push_str("```\n\n");
    }

    if has_any { section } else { String::new() }
}

/// Formats SBOM files section.
fn format_sbom_section(dir: &Path) -> String {
    let sbom_re = Regex::new(r"\.(spdx|cdx)\.json$").unwrap();
    let mut sbom_files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if sbom_re.is_match(&name) {
                sbom_files.push(name);
            }
        }
    }

    if sbom_files.is_empty() {
        return String::new();
    }

    sbom_files.sort();
    let mut section = "\n## SBOM\n\n".to_string();
    for name in &sbom_files {
        if name.ends_with(".spdx.json") {
            section.push_str(&format!(" * `{name}`: in the SPDX format\n"));
        } else if name.ends_with(".cdx.json") {
            section.push_str(&format!(" * `{name}`: in the CycloneDX format\n"));
        }
    }
    section.push('\n');
    section
}

/// Collects all checksum file contents.
fn collect_checksums(dir: &Path) -> String {
    let checksum_re = Regex::new(r"\.(sha256|sha512|b2)$").unwrap();
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if checksum_re.is_match(&name) {
                files.push(entry.path());
            }
        }
    }

    if files.is_empty() {
        return String::new();
    }

    files.sort();
    let mut checksums = String::new();
    for file in &files {
        if let Ok(content) = fs::read_to_string(file) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                checksums.push_str(trimmed);
                checksums.push('\n');
            }
        }
    }
    checksums
}

pub fn run_format_release() -> Result<()> {
    let version = env_or("VERSION", "");
    if version.is_empty() {
        return Err(Error::User("VERSION is required".into()));
    }

    let artifacts_dir = env_or("ARTIFACTS_DIR", "release");
    let release_notes_file = env_or("RELEASE_NOTES_FILE", "release_notes.md");
    let include_checksums = env_or("INCLUDE_CHECKSUMS", "true") == "true";
    let include_signatures = env_or("INCLUDE_SIGNATURES", "true") == "true";

    let homebrew_tap = env_or("HOMEBREW_TAP", "");
    let aur_package = env_or("AUR_PACKAGE", "");
    let winget_id = env_or("WINGET_ID", "");

    println!("\x1b[32mFormatting release:\x1b[0m v{version}");

    let mut body = String::new();

    // 1. Release notes from changelog
    if Path::new(&release_notes_file).exists() {
        let notes = fs::read_to_string(&release_notes_file)?;
        let notes = notes.trim();
        if !notes.is_empty() {
            body.push_str(notes);
            body.push_str("\n\n");
        }
    }

    // 2. Installation section
    let install_section = format_installation_section(&homebrew_tap, &aur_package, &winget_id);
    if !install_section.is_empty() {
        body.push_str(&install_section);
    }

    let artifacts_path = Path::new(&artifacts_dir);
    if artifacts_path.exists() {
        // 3. Build Assets table
        let artifacts = list_release_artifacts(artifacts_path);
        if !artifacts.is_empty() {
            body.push_str("## Build Assets\n\n");
            body.push_str(&format_artifacts_table(&artifacts));
        }

        // 4. SBOM section
        let sbom_section = format_sbom_section(artifacts_path);
        if !sbom_section.is_empty() {
            body.push_str(&sbom_section);
        }

        // 5. Checksums section
        if include_checksums {
            let checksums = collect_checksums(artifacts_path);
            if !checksums.is_empty() {
                body.push_str("\n## Checksums\n\n");
                body.push_str("```\n");
                body.push_str(&checksums);
                body.push_str("```\n");
            }
        }

        // 6. Signatures section
        if include_signatures {
            let sig_re = Regex::new(r"\.(sig|pem|sigstore\.json)$").unwrap();
            let has_sigs = fs::read_dir(artifacts_path)
                .into_iter()
                .flatten()
                .flatten()
                .any(|e| sig_re.is_match(&e.file_name().to_string_lossy()));
            if has_sigs {
                body.push_str("\n## Signatures\n\n");
                body.push_str(
                    "All release artifacts are signed with [Sigstore](https://www.sigstore.dev/). ",
                );
                body.push_str("Verify with:\n\n");
                body.push_str("```bash\n");
                body.push_str("cosign verify-blob --bundle <artifact>.sigstore.json <artifact>\n");
                body.push_str("```\n");
            }
        }
    }

    println!();
    println!("\x1b[32mRelease body:\x1b[0m");
    print_hr();
    print!("{body}");
    print_hr();

    output("version", &version);
    output_multiline("body", &body);
    Ok(())
}
