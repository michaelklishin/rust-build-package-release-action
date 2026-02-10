use crate::error::{Error, Result};
use blake2::{Blake2b512, Digest};
use sha2::{Sha256, Sha512};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

/// Checksums produced for a file.
#[derive(Debug, Clone, Default)]
pub struct Checksums {
    pub sha256: String,
    pub sha512: String,
    pub b2: String,
}

/// Compute SHA-256 of a file.
pub fn sha256_file(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute SHA-512 of a file.
pub fn sha512_file(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha512::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute BLAKE2b-512 of a file.
pub fn blake2_file(path: &Path) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Blake2b512::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Compute SHA-256 of a byte slice.
pub fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Generate checksums for a file based on the CHECKSUM env var.
/// Returns the computed checksums and writes .sha256/.sha512/.b2 sidecar files.
pub fn generate_checksums(file_path: &Path) -> Result<Checksums> {
    let checksum_types = env::var("CHECKSUM").unwrap_or_else(|_| "sha256".to_string());
    let basename = file_path.file_name().unwrap_or_default().to_string_lossy();

    let mut checksums = Checksums::default();

    if checksum_types.contains("sha256") || checksum_types.is_empty() {
        let hash = sha256_file(file_path)?;
        let checksum_file = format!("{}.sha256", file_path.display());
        let mut f = fs::File::create(&checksum_file)?;
        writeln!(f, "{hash}  {basename}")?;
        println!("\x1b[32mSHA256:\x1b[0m {hash}");
        checksums.sha256 = hash;
    }

    if checksum_types.contains("sha512") {
        let hash = sha512_file(file_path)?;
        let checksum_file = format!("{}.sha512", file_path.display());
        let mut f = fs::File::create(&checksum_file)?;
        writeln!(f, "{hash}  {basename}")?;
        println!("\x1b[32mSHA512:\x1b[0m {hash}");
        checksums.sha512 = hash;
    }

    if checksum_types.contains("b2") {
        let hash = blake2_file(file_path)?;
        let checksum_file = format!("{}.b2", file_path.display());
        let mut f = fs::File::create(&checksum_file)?;
        writeln!(f, "{hash}  {basename}")?;
        println!("\x1b[32mBLAKE2:\x1b[0m {hash}");
        checksums.b2 = hash;
    }

    Ok(checksums)
}

/// Detect checksum type from file extension.
pub fn detect_checksum_type(checksum_file: &str) -> &'static str {
    if checksum_file.ends_with(".sha512") {
        "sha512"
    } else if checksum_file.ends_with(".b2") {
        "b2"
    } else {
        "sha256"
    }
}

/// Parse a checksum file and return the hash value (first token of first line).
pub fn parse_checksum_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path)?;
    let content = content.trim();
    if content.is_empty() {
        return Err(Error::User("checksum file is empty".into()));
    }
    let first_line = content.lines().next().unwrap_or("");
    let hash = first_line
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    Ok(hash)
}

/// Compute checksum of a file using the specified type.
pub fn compute_checksum(path: &Path, checksum_type: &str) -> Result<String> {
    match checksum_type {
        "sha256" => sha256_file(path),
        "sha512" => sha512_file(path),
        "b2" => blake2_file(path),
        other => Err(Error::User(format!("unsupported checksum type: {other}"))),
    }
}

/// Verify a checksum file against an artifact.
pub fn verify_checksum(artifact_path: &Path, checksum_file_path: &Path) -> Result<()> {
    println!("\x1b[32mVerifying checksum...\x1b[0m");

    if !checksum_file_path.exists() {
        return Err(Error::User(format!(
            "checksum file not found: {}",
            checksum_file_path.display()
        )));
    }

    let expected = parse_checksum_file(checksum_file_path)?;
    let checksum_type = detect_checksum_type(&checksum_file_path.to_string_lossy());
    let actual = compute_checksum(artifact_path, checksum_type)?;

    if actual != expected {
        return Err(Error::User(format!(
            "checksum mismatch: expected {expected}, got {actual}"
        )));
    }

    println!("  {}: {actual} \u{2713}", checksum_type.to_uppercase());
    Ok(())
}
