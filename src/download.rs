use crate::checksum::verify_checksum;
use crate::env_or;
use crate::error::{Error, Result};
use crate::tools::command_exists;
use std::path::Path;
use std::process::Command;

/// Ensure curl is available.
fn ensure_curl() {
    if command_exists("curl") {
        return;
    }
    println!("  curl not found, attempting to install...");
    let _ = Command::new("apt-get").args(["update", "-qq"]).status();
    let _ = Command::new("apt-get")
        .args(["install", "-y", "-qq", "curl"])
        .status();
    if !command_exists("curl") {
        let _ = Command::new("dnf")
            .args(["install", "-y", "-q", "curl"])
            .status();
    }
}

/// Download a file using curl.
fn download_file(url: &str, output: &str) -> Result<()> {
    let result = curl_download(url, output)?;
    if !result {
        return Err(Error::User(format!("failed to download {url}")));
    }
    Ok(())
}

/// Download via curl with optional auth token.
fn curl_download(url: &str, output: &str) -> Result<bool> {
    ensure_curl();
    let gh_token = env_or("GITHUB_TOKEN", &env_or("GH_TOKEN", ""));
    let result = if !gh_token.is_empty() {
        Command::new("curl")
            .args([
                "-fsSL",
                "-H",
                &format!("Authorization: Bearer {gh_token}"),
                url,
                "-o",
                output,
            ])
            .output()
    } else {
        Command::new("curl")
            .args(["-fsSL", url, "-o", output])
            .output()
    };

    match result {
        Ok(o) => Ok(o.status.success()),
        Err(e) => Err(Error::User(format!("curl failed: {e}"))),
    }
}

/// Try to download a file, return true if successful.
fn try_download_file(url: &str, output: &str) -> bool {
    curl_download(url, output).unwrap_or(false)
}

/// Try to download checksum in multiple formats.
fn try_download_checksum(base_url: &str, artifact_name: &str) -> Option<String> {
    let exts = ["sha256", "sha512", "b2"];
    for ext in &exts {
        let checksum_file = format!("{artifact_name}.{ext}");
        let checksum_url = format!("{base_url}/{checksum_file}");
        if try_download_file(&checksum_url, &checksum_file) {
            return Some(checksum_file);
        }
    }
    None
}

/// Download an artifact from GitHub releases.
pub fn download_artifact(
    binary_name: &str,
    version: &str,
    arch: &str,
    format: &str,
) -> Result<String> {
    let repo = env_or("GITHUB_REPOSITORY", "");
    if repo.is_empty() {
        return Err(Error::User("GITHUB_REPOSITORY not set".into()));
    }

    let artifact_name = match format {
        "deb" => format!("{binary_name}_{version}_{arch}.deb"),
        "rpm" => format!("{binary_name}-{version}-1.{arch}.rpm"),
        "windows-zip" => format!("{binary_name}-{version}-x86_64-pc-windows-msvc.zip"),
        "windows-msi" => format!("{binary_name}-{version}-x86_64-pc-windows-msvc.msi"),
        _ => return Err(Error::User(format!("unknown format: {format}"))),
    };

    let base_url = format!("https://github.com/{repo}/releases/download/v{version}");
    let artifact_url = format!("{base_url}/{artifact_name}");

    println!("\x1b[32mDownloading artifact:\x1b[0m {artifact_name}");
    download_file(&artifact_url, &artifact_name)?;

    if let Some(checksum_file) = try_download_checksum(&base_url, &artifact_name) {
        verify_checksum(Path::new(&artifact_name), Path::new(&checksum_file))?;
    } else {
        println!("\x1b[33m  No checksum file available\x1b[0m");
    }

    Ok(artifact_name)
}

pub struct WindowsArtifacts {
    pub binary: String,
    pub msi: String,
}

/// Download Windows artifacts (both zip and msi).
pub fn download_windows_artifacts(binary_name: &str, version: &str) -> Result<WindowsArtifacts> {
    let repo = env_or("GITHUB_REPOSITORY", "");
    if repo.is_empty() {
        return Err(Error::User("GITHUB_REPOSITORY not set".into()));
    }

    let base_url = format!("https://github.com/{repo}/releases/download/v{version}");
    let zip_name = format!("{binary_name}-{version}-x86_64-pc-windows-msvc.zip");
    let msi_name = format!("{binary_name}-{version}-x86_64-pc-windows-msvc.msi");

    println!("\x1b[32mDownloading Windows artifacts\x1b[0m");

    println!("  Downloading {zip_name}");
    download_file(&format!("{base_url}/{zip_name}"), &zip_name)?;

    if let Some(checksum_file) = try_download_checksum(&base_url, &zip_name) {
        verify_checksum(Path::new(&zip_name), Path::new(&checksum_file))?;
    }

    println!("  Downloading {msi_name}");
    download_file(&format!("{base_url}/{msi_name}"), &msi_name)?;

    if let Some(checksum_file) = try_download_checksum(&base_url, &msi_name) {
        verify_checksum(Path::new(&msi_name), Path::new(&checksum_file))?;
    }

    // Extract zip
    println!("  Extracting archive");
    #[cfg(target_os = "windows")]
    {
        let result = Command::new("powershell")
            .args([
                "-Command",
                &format!("Expand-Archive -Path '{zip_name}' -DestinationPath 'extracted' -Force"),
            ])
            .output()
            .map_err(|e| Error::User(format!("extraction failed: {e}")))?;
        if !result.status.success() {
            return Err(Error::User(format!(
                "failed to extract archive: {}",
                String::from_utf8_lossy(&result.stderr)
            )));
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let result = Command::new("unzip")
            .args(["-q", &zip_name, "-d", "extracted"])
            .output()
            .map_err(|e| Error::User(format!("extraction failed: {e}")))?;
        if !result.status.success() {
            return Err(Error::User(format!(
                "failed to extract archive: {}",
                String::from_utf8_lossy(&result.stderr)
            )));
        }
    }

    let binary_path = format!("extracted/{binary_name}.exe");
    if !Path::new(&binary_path).exists() {
        return Err(Error::User(format!(
            "binary not found in archive: {binary_path}"
        )));
    }

    Ok(WindowsArtifacts {
        binary: binary_path,
        msi: msi_name,
    })
}
