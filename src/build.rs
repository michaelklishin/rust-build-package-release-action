use crate::checksum::Checksums;
use crate::error::Result;
use crate::output::output;
use crate::output::output_multiline;
use crate::tools;
use serde_json::Value;
use std::env;
use std::process::Command;

/// Check if a Cargo feature exists in the project.
fn has_cargo_feature(feature: &str) -> bool {
    let package = env::var("PACKAGE").unwrap_or_default();
    let mut args = vec![
        "metadata".to_string(),
        "--format-version".to_string(),
        "1".to_string(),
        "--no-deps".to_string(),
    ];
    if !package.is_empty() {
        args.push("--package".to_string());
        args.push(package.clone());
    }

    let result = Command::new("cargo").args(&args).output();
    let output = match result {
        Ok(o) if o.status.success() => o,
        _ => return false,
    };

    let data: Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let packages = match data["packages"].as_array() {
        Some(p) => p,
        None => return false,
    };

    let pkg = if !package.is_empty() {
        packages
            .iter()
            .find(|p| p["name"].as_str() == Some(&package))
    } else {
        packages.first()
    };

    match pkg {
        Some(p) => p["features"]
            .as_object()
            .is_some_and(|f| f.contains_key(feature)),
        None => false,
    }
}

/// Build with cargo rustc or cargo-zigbuild using the environment configuration.
pub fn cargo_build(target: &str, binary_name: &str) -> Result<()> {
    let package = env::var("PACKAGE").unwrap_or_default();
    let no_default_features = env::var("NO_DEFAULT_FEATURES").unwrap_or_default() == "true";
    let mut features = env::var("FEATURES").unwrap_or_default();
    let locked = env::var("LOCKED").unwrap_or_default() == "true";
    let profile = env::var("PROFILE").unwrap_or_else(|_| "release".into());
    let target_rustflags = env::var("TARGET_RUSTFLAGS").unwrap_or_default();
    let use_zigbuild = env::var("USE_ZIGBUILD").unwrap_or_default() == "true";

    if !target_rustflags.is_empty() {
        // Safety: running single-threaded at this point during build setup
        unsafe { env::set_var("RUSTFLAGS", &target_rustflags) };
    }

    // For musl targets without zigbuild, set static linking
    if target.contains("musl")
        && !use_zigbuild
        && env::var("RUSTFLAGS").unwrap_or_default().is_empty()
    {
        // Safety: running single-threaded at this point during build setup
        unsafe { env::set_var("RUSTFLAGS", "-C target-feature=+crt-static") };
    }

    // For musl targets, auto-enable mimalloc if available
    if target.contains("musl") && !features.contains("mimalloc") && has_cargo_feature("mimalloc") {
        println!("\x1b[32mEnabling mimalloc feature for musl build\x1b[0m");
        features = if features.is_empty() {
            "mimalloc".to_string()
        } else {
            format!("{features},mimalloc")
        };
    }

    let mut args: Vec<String> = if use_zigbuild {
        tools::check_zigbuild()?;
        vec!["zigbuild".into(), "--target".into(), target.into()]
    } else {
        vec![
            "rustc".into(),
            "--target".into(),
            target.into(),
            "-q".into(),
        ]
    };

    if profile == "release" {
        args.push("--release".into());
    } else if profile != "dev" {
        args.push("--profile".into());
        args.push(profile);
    }

    if !package.is_empty() {
        args.push("--package".into());
        args.push(package);
    }

    if !binary_name.is_empty() {
        args.push("--bin".into());
        args.push(binary_name.into());
    }

    if no_default_features {
        args.push("--no-default-features".into());
    }

    if !features.is_empty() {
        args.push("--features".into());
        args.push(features);
    }

    if locked {
        args.push("--locked".into());
    }

    let args_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    tools::run_command_inherit("cargo", &args_refs)
}

/// Outputs build results to GITHUB_OUTPUT.
pub fn output_build_results(
    binary_name: &str,
    version: &str,
    target: &str,
    artifact: &str,
    artifact_path: &str,
    checksums: &Checksums,
) {
    output("artifact", artifact);
    output("artifact_path", artifact_path);
    output("sha256", &checksums.sha256);
    output("sha512", &checksums.sha512);
    output("b2", &checksums.b2);

    if !checksums.sha256.is_empty() {
        output("checksum_file", &format!("{artifact_path}.sha256"));
    }

    let summary = build_summary(
        binary_name,
        version,
        target,
        artifact,
        artifact_path,
        checksums,
    );
    output_multiline("summary", &summary);
}

/// Generates a JSON summary of the build.
pub fn build_summary(
    binary_name: &str,
    version: &str,
    target: &str,
    artifact: &str,
    artifact_path: &str,
    checksums: &Checksums,
) -> String {
    let summary = serde_json::json!({
        "binary_name": binary_name,
        "version": version,
        "target": target,
        "artifact": artifact,
        "artifact_path": artifact_path,
        "sha256": checksums.sha256,
        "sha512": checksums.sha512,
        "b2": checksums.b2,
    });
    serde_json::to_string_pretty(&summary).unwrap_or_default()
}
