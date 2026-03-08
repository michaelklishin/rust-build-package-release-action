use crate::cargo_info::get_cargo_info;
use crate::env_or;
use crate::error::{Error, Result};
use crate::output::{output, print_hr};
use crate::tools;
use crate::version::{is_valid_semver, version_from_tag};
use std::env;

/// Build the args for `cargo publish`.
pub fn build_publish_args(dry_run: bool) -> Vec<String> {
    let package = env::var("PACKAGE").unwrap_or_default();
    let features = env::var("FEATURES").unwrap_or_default();
    let no_default_features = env::var("NO_DEFAULT_FEATURES").unwrap_or_default() == "true";
    let locked = env::var("LOCKED").unwrap_or_default() == "true";
    let manifest_path = env_or("MANIFEST_PATH", "Cargo.toml");

    let mut args = vec!["publish".to_string()];

    if dry_run {
        args.push("--dry-run".to_string());
    }

    if !package.is_empty() {
        args.push("--package".to_string());
        args.push(package);
    }

    if manifest_path != "Cargo.toml" {
        args.push("--manifest-path".to_string());
        args.push(manifest_path);
    }

    if no_default_features {
        args.push("--no-default-features".to_string());
    }

    if !features.is_empty() {
        args.push("--features".to_string());
        args.push(features);
    }

    if locked {
        args.push("--locked".to_string());
    }

    args
}

/// Validate that the tag version matches Cargo.toml before publishing.
fn validate_version_for_publish() -> Result<String> {
    let tag = env_or("TAG", &env_or("GITHUB_REF_NAME", ""));

    if tag.is_empty() {
        return Err(Error::User(
            "no tag available: set TAG or trigger from a tag push".to_string(),
        ));
    }

    let tag_version = version_from_tag(&tag).ok_or_else(|| {
        Error::User(format!(
            "tag '{tag}' does not start with 'v' (expected format: v1.2.3)"
        ))
    })?;

    if !is_valid_semver(tag_version) {
        return Err(Error::User(format!("invalid semver in tag: {tag_version}")));
    }

    let cargo_info = get_cargo_info()?;
    if cargo_info.version.is_empty() {
        return Err(Error::User(
            "could not read version from Cargo.toml".to_string(),
        ));
    }

    if cargo_info.version != tag_version {
        return Err(Error::User(format!(
            "Cargo.toml version ({}) does not match tag ({tag_version})\n\n\
             Update Cargo.toml version to '{tag_version}' before publishing",
            cargo_info.version
        )));
    }

    println!(
        "\x1b[32mVersion validated:\x1b[0m {} matches tag {}",
        cargo_info.version, tag
    );

    Ok(tag_version.to_string())
}

/// Check that CARGO_REGISTRY_TOKEN is available.
fn check_registry_token() -> Result<()> {
    if env::var("CARGO_REGISTRY_TOKEN")
        .unwrap_or_default()
        .is_empty()
    {
        return Err(Error::User(
            "CARGO_REGISTRY_TOKEN is not set\n\n\
             For Trusted Publishing, add these steps before publish-crate:\n\n\
               - uses: rust-lang/crates-io-auth-action@v1\n\
                 id: auth\n\n\
             Then pass the token:\n\n\
               env:\n\
                 CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}\n\n\
             Your workflow also needs:\n\
               permissions:\n\
                 id-token: write"
                .to_string(),
        ));
    }
    Ok(())
}

/// Run `cargo publish` and return whether the crate was actually published
/// (false when the version already exists on crates.io).
fn run_cargo_publish(args: &[String]) -> Result<bool> {
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    match tools::run_command("cargo", &refs) {
        Ok(_) => Ok(true),
        Err(Error::Command { stderr, .. }) if stderr.contains("already exists") => {
            println!("\x1b[33mVersion already published — skipping (idempotent)\x1b[0m");
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

pub fn run_publish_crate() -> Result<()> {
    let dry_run = env_or("PUBLISH_DRY_RUN", "false") == "true";

    print_hr();
    if dry_run {
        println!("\x1b[32mPublishing crate (dry run)\x1b[0m");
    } else {
        println!("\x1b[32mPublishing crate to crates.io\x1b[0m");
    }
    print_hr();

    tools::check_rust_toolchain()?;
    let version = validate_version_for_publish()?;

    if !dry_run {
        check_registry_token()?;

        // Dry run first to catch packaging errors before consuming a version on crates.io
        println!("\x1b[32mRunning dry-run validation...\x1b[0m");
        run_cargo_publish(&build_publish_args(true))?;
        println!("\x1b[32mDry run passed\x1b[0m");
    }

    let published = run_cargo_publish(&build_publish_args(dry_run))?;

    if dry_run {
        println!("\x1b[32mDry run completed successfully\x1b[0m");
    } else if published {
        println!("\x1b[32mPublished version {version} to crates.io\x1b[0m");
    } else {
        println!("\x1b[32mVersion {version} was already on crates.io\x1b[0m");
    }

    output("version", &version);
    output(
        "published",
        if published && !dry_run {
            "true"
        } else {
            "false"
        },
    );

    Ok(())
}
