use crate::cargo_info::get_cargo_info;
use crate::env_or;
use crate::error::{Error, Result};
use crate::output::{output, print_hr};
use crate::tools::{check_cargo_sbom, check_rust_toolchain};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Generate SPDX and CycloneDX SBOMs.
fn generate_sbom_files(
    output_dir: &str,
    binary_name: &str,
    version: &str,
) -> Result<(String, String)> {
    let spdx_path = format!("{output_dir}/{binary_name}-{version}.spdx.json");
    let cyclonedx_path = format!("{output_dir}/{binary_name}-{version}.cdx.json");

    println!("\x1b[32mGenerating SPDX SBOM...\x1b[0m");
    let result = Command::new("cargo")
        .args(["sbom", "--output-format", "spdx_json_2_3"])
        .output()
        .map_err(|e| Error::User(format!("cargo-sbom failed: {e}")))?;
    if !result.status.success() {
        return Err(Error::User(format!(
            "cargo-sbom SPDX generation failed: {}",
            String::from_utf8_lossy(&result.stderr)
        )));
    }
    fs::write(&spdx_path, &result.stdout)?;

    println!("\x1b[32mGenerating CycloneDX SBOM...\x1b[0m");
    let result = Command::new("cargo")
        .args(["sbom", "--output-format", "cyclone_dx_json_1_4"])
        .output()
        .map_err(|e| Error::User(format!("cargo-sbom failed: {e}")))?;
    if !result.status.success() {
        return Err(Error::User(format!(
            "cargo-sbom CycloneDX generation failed: {}",
            String::from_utf8_lossy(&result.stderr)
        )));
    }
    fs::write(&cyclonedx_path, &result.stdout)?;

    if !Path::new(&spdx_path).exists() {
        return Err(Error::User(format!(
            "SPDX SBOM was not created: {spdx_path}"
        )));
    }
    if !Path::new(&cyclonedx_path).exists() {
        return Err(Error::User(format!(
            "CycloneDX SBOM was not created: {cyclonedx_path}"
        )));
    }

    Ok((spdx_path, cyclonedx_path))
}

pub fn run_generate_sbom() -> Result<()> {
    check_rust_toolchain()?;
    check_cargo_sbom()?;

    let info = get_cargo_info()?;
    let binary_name = env_or("BINARY_NAME", &info.name);
    let version = &info.version;

    if binary_name.is_empty() {
        return Err(Error::User("could not determine binary name".into()));
    }
    if version.is_empty() {
        return Err(Error::User("could not determine version".into()));
    }

    println!("\x1b[32mGenerating SBOM:\x1b[0m {binary_name} v{version}");

    let output_dir = env_or("SBOM_OUTPUT_DIR", "target/sbom");
    fs::create_dir_all(&output_dir)?;

    let (spdx, cyclonedx) = generate_sbom_files(&output_dir, &binary_name, version)?;

    println!();
    println!("\x1b[32mSBOM files:\x1b[0m");
    print_hr();

    output("version", version);
    output("binary_name", &binary_name);
    output("sbom_spdx", &spdx);
    output("sbom_cyclonedx", &cyclonedx);
    Ok(())
}
