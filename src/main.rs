use clap::{Parser, Subcommand};
use rust_release_action::{
    aur, changelog, collect_artifacts, format_release, homebrew, release, sbom, sign, testing,
    version, winget,
};
use std::{env, process};

#[derive(Parser)]
#[command(name = "rust-release-action")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ExtractChangelog,
    ValidateChangelog,
    ValidateVersion,
    GetVersion,
    GetReleaseVersion,
    GenerateSbom,
    GenerateHomebrew,
    GenerateAur,
    GenerateWinget,
    SignArtifact,
    FormatRelease,
    CollectArtifacts,
    Release,
    ReleaseLinux,
    ReleaseLinuxDeb,
    ReleaseLinuxRpm,
    ReleaseLinuxApk,
    ReleaseMacos,
    ReleaseMacosDmg,
    ReleaseWindows,
    ReleaseWindowsMsi,
    TestDeb,
    TestRpm,
    TestWindows,
}

fn main() {
    // Map INPUT_* env vars to the variables expected by command handlers,
    // matching the dispatch.nu logic.
    map_input_env_vars();

    let cli = Cli::parse();

    let result = match cli.command {
        Command::ExtractChangelog => changelog::run_extract_changelog(),
        Command::ValidateChangelog => changelog::run_validate_changelog(),
        Command::ValidateVersion => version::run_validate_version(),
        Command::GetVersion => version::run_get_version(),
        Command::GetReleaseVersion => version::run_get_release_version(),
        Command::GenerateSbom => sbom::run_generate_sbom(),
        Command::GenerateHomebrew => homebrew::run_generate_homebrew(),
        Command::GenerateAur => aur::run_generate_aur(),
        Command::GenerateWinget => winget::run_generate_winget(),
        Command::SignArtifact => sign::run_sign_artifact(),
        Command::FormatRelease => format_release::run_format_release(),
        Command::CollectArtifacts => collect_artifacts::run_collect_artifacts(),
        Command::Release => release::run_release(),
        Command::ReleaseLinux => release::run_release_linux(),
        Command::ReleaseLinuxDeb => release::run_release_linux_deb(),
        Command::ReleaseLinuxRpm => release::run_release_linux_rpm(),
        Command::ReleaseLinuxApk => release::run_release_linux_apk(),
        Command::ReleaseMacos => release::run_release_macos(),
        Command::ReleaseMacosDmg => release::run_release_macos_dmg(),
        Command::ReleaseWindows => release::run_release_windows(),
        Command::ReleaseWindowsMsi => release::run_release_windows_msi(),
        Command::TestDeb => testing::run_test_deb(),
        Command::TestRpm => testing::run_test_rpm(),
        Command::TestWindows => testing::run_test_windows(),
    };

    if let Err(e) = result {
        eprintln!("\x1b[31mERROR:\x1b[0m {e}");
        process::exit(1);
    }
}

/// Maps INPUT_* environment variables to the internal env vars expected by command handlers.
/// This mirrors the dispatch.nu logic where action.yml inputs are propagated.
fn map_input_env_vars() {
    // Safety: running single-threaded at this point during build setup
    unsafe { map_input_env_vars_inner() }
}

unsafe fn map_input_env_vars_inner() {
    // Version - auto-detect from tag if not provided
    let version_input = env::var("INPUT_VERSION").unwrap_or_default();
    let version = if !version_input.is_empty() {
        version_input
    } else {
        let ref_name = env::var("GITHUB_REF_NAME").unwrap_or_default();
        if let Some(v) = ref_name.strip_prefix('v') {
            v.to_string()
        } else {
            String::new()
        }
    };
    if !version.is_empty() {
        unsafe { env::set_var("VERSION", &version) };
    }

    // Simple 1:1 mappings (INPUT_X -> Y)
    let mappings: &[(&str, &str)] = &[
        ("INPUT_TARGET", "TARGET"),
        ("INPUT_BINARY_NAME", "BINARY_NAME"),
        ("INPUT_PACKAGE", "PACKAGE"),
        ("INPUT_MANIFEST", "MANIFEST_PATH"),
        ("INPUT_PRE_BUILD", "PRE_BUILD"),
        ("INPUT_BINARY_PATH", "BINARY_PATH"),
        ("INPUT_FEATURES", "FEATURES"),
        ("INPUT_PROFILE", "PROFILE"),
        ("INPUT_RUSTFLAGS", "TARGET_RUSTFLAGS"),
        ("INPUT_CHECKSUM", "CHECKSUM"),
        ("INPUT_INCLUDE", "ARCHIVE_INCLUDE"),
        ("INPUT_CHANGELOG", "CHANGELOG_PATH"),
        ("INPUT_NOTES_OUTPUT", "OUTPUT_PATH"),
        ("INPUT_TAG", "TAG"),
        ("INPUT_EXPECTED_VERSION", "EXPECTED_VERSION"),
        ("INPUT_PKG_DESCRIPTION", "PKG_DESCRIPTION"),
        ("INPUT_PKG_MAINTAINER", "PKG_MAINTAINER"),
        ("INPUT_PKG_HOMEPAGE", "PKG_HOMEPAGE"),
        ("INPUT_PKG_LICENSE", "PKG_LICENSE"),
        ("INPUT_PKG_VENDOR", "PKG_VENDOR"),
        ("INPUT_PKG_DEPENDS", "PKG_DEPENDS"),
        ("INPUT_PKG_RECOMMENDS", "PKG_RECOMMENDS"),
        ("INPUT_PKG_SUGGESTS", "PKG_SUGGESTS"),
        ("INPUT_PKG_CONFLICTS", "PKG_CONFLICTS"),
        ("INPUT_PKG_REPLACES", "PKG_REPLACES"),
        ("INPUT_PKG_PROVIDES", "PKG_PROVIDES"),
        ("INPUT_PKG_CONTENTS", "PKG_CONTENTS"),
        ("INPUT_PKG_SECTION", "PKG_SECTION"),
        ("INPUT_PKG_PRIORITY", "PKG_PRIORITY"),
        ("INPUT_PKG_GROUP", "PKG_GROUP"),
        ("INPUT_PKG_RELEASE", "PKG_RELEASE"),
        ("INPUT_SBOM_FORMAT", "SBOM_FORMAT"),
        ("INPUT_SBOM_DIR", "SBOM_OUTPUT_DIR"),
        ("INPUT_BREW_CLASS", "HOMEBREW_FORMULA_CLASS"),
        ("INPUT_BREW_MACOS_ARM64_URL", "HOMEBREW_MACOS_ARM64_URL"),
        (
            "INPUT_BREW_MACOS_ARM64_SHA256",
            "HOMEBREW_MACOS_ARM64_SHA256",
        ),
        ("INPUT_BREW_MACOS_X64_URL", "HOMEBREW_MACOS_X64_URL"),
        ("INPUT_BREW_MACOS_X64_SHA256", "HOMEBREW_MACOS_X64_SHA256"),
        ("INPUT_BREW_LINUX_ARM64_URL", "HOMEBREW_LINUX_ARM64_URL"),
        (
            "INPUT_BREW_LINUX_ARM64_SHA256",
            "HOMEBREW_LINUX_ARM64_SHA256",
        ),
        ("INPUT_BREW_LINUX_X64_URL", "HOMEBREW_LINUX_X64_URL"),
        ("INPUT_BREW_LINUX_X64_SHA256", "HOMEBREW_LINUX_X64_SHA256"),
        ("INPUT_BREW_DIR", "HOMEBREW_OUTPUT_DIR"),
        ("INPUT_ARTIFACT", "ARTIFACT_PATH"),
        ("INPUT_ARTIFACTS_DIR", "ARTIFACTS_DIR"),
        ("INPUT_BASE_URL", "BASE_URL"),
        ("INPUT_NOTES_FILE", "RELEASE_NOTES_FILE"),
        ("INPUT_INCLUDE_CHECKSUMS", "INCLUDE_CHECKSUMS"),
        ("INPUT_INCLUDE_SIGNATURES", "INCLUDE_SIGNATURES"),
        ("INPUT_HOMEBREW_TAP", "HOMEBREW_TAP"),
        ("INPUT_AUR_PACKAGE", "AUR_PACKAGE"),
        ("INPUT_WINGET_ID", "WINGET_ID"),
        ("INPUT_AUR_NAME", "AUR_PACKAGE_NAME"),
        ("INPUT_AUR_MAINTAINER", "AUR_MAINTAINER"),
        ("INPUT_AUR_SOURCE_URL", "AUR_SOURCE_URL"),
        ("INPUT_AUR_SOURCE_SHA256", "AUR_SOURCE_SHA256"),
        ("INPUT_AUR_MAKEDEPENDS", "AUR_MAKEDEPENDS"),
        ("INPUT_AUR_OPTDEPENDS", "AUR_OPTDEPENDS"),
        ("INPUT_AUR_DIR", "AUR_OUTPUT_DIR"),
        ("INPUT_WINGET_PUBLISHER", "WINGET_PUBLISHER"),
        ("INPUT_WINGET_PUBLISHER_ID", "WINGET_PUBLISHER_ID"),
        ("INPUT_WINGET_PACKAGE_ID", "WINGET_PACKAGE_ID"),
        ("INPUT_WINGET_LICENSE_URL", "WINGET_LICENSE_URL"),
        ("INPUT_WINGET_COPYRIGHT", "WINGET_COPYRIGHT"),
        ("INPUT_WINGET_TAGS", "WINGET_TAGS"),
        ("INPUT_WINGET_X64_URL", "WINGET_X64_URL"),
        ("INPUT_WINGET_X64_SHA256", "WINGET_X64_SHA256"),
        ("INPUT_WINGET_ARM64_URL", "WINGET_ARM64_URL"),
        ("INPUT_WINGET_ARM64_SHA256", "WINGET_ARM64_SHA256"),
        ("INPUT_WINGET_DIR", "WINGET_OUTPUT_DIR"),
        ("INPUT_CHECKSUM_FILE", "CHECKSUM_FILE"),
        ("INPUT_MSI_PATH", "MSI_PATH"),
        ("INPUT_MSI_CHECKSUM_FILE", "MSI_CHECKSUM_FILE"),
        ("INPUT_ARCH", "ARCH"),
    ];

    for (input_key, target_key) in mappings {
        if let Ok(val) = env::var(input_key) {
            if !val.is_empty() {
                unsafe { env::set_var(target_key, &val) };
            }
        }
    }

    // Boolean mappings (INPUT_X == "true" -> Y = "true")
    let bool_mappings: &[(&str, &str)] = &[
        ("INPUT_SKIP_BUILD", "SKIP_BUILD"),
        ("INPUT_LOCKED", "LOCKED"),
        ("INPUT_NO_DEFAULT_FEATURES", "NO_DEFAULT_FEATURES"),
        ("INPUT_USE_ZIGBUILD", "USE_ZIGBUILD"),
        ("INPUT_ARCHIVE", "ARCHIVE"),
        ("INPUT_VALIDATE_CARGO_TOML", "VALIDATE_CARGO_TOML"),
        ("INPUT_DOWNLOAD_FROM_RELEASE", "DOWNLOAD_FROM_RELEASE"),
    ];

    for (input_key, target_key) in bool_mappings {
        if let Ok(val) = env::var(input_key) {
            if val == "true" {
                unsafe { env::set_var(target_key, "true") };
            }
        }
    }
}
