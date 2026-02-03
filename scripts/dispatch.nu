#!/usr/bin/env nu

# Command dispatcher - routes to the appropriate script based on INPUT_COMMAND

use common.nu [error]

def main [] {
    let scripts = $env.GITHUB_ACTION_PATH | path join "scripts"
    let command = $env.INPUT_COMMAND? | default ""

    if $command == "" {
        error "command input is required"
    }

    # Map inputs to environment variables expected by scripts
    # Using $env.VAR = value instead of load-env so vars propagate to subprocesses

    # Version - auto-detect from tag if not provided
    let version_input = $env.INPUT_VERSION? | default ""
    $env.VERSION = if $version_input != "" {
        $version_input
    } else if ($env.GITHUB_REF_NAME? | default "" | str starts-with "v") {
        $env.GITHUB_REF_NAME | str substring 1..
    } else {
        ""
    }

    # Core inputs
    if ($env.INPUT_TARGET? | default "") != "" { $env.TARGET = $env.INPUT_TARGET }
    if ($env.INPUT_BINARY_NAME? | default "") != "" { $env.BINARY_NAME = $env.INPUT_BINARY_NAME }
    if ($env.INPUT_PACKAGE? | default "") != "" { $env.PACKAGE = $env.INPUT_PACKAGE }
    if ($env.INPUT_MANIFEST? | default "") != "" { $env.MANIFEST_PATH = $env.INPUT_MANIFEST }

    # Build options
    if ($env.INPUT_PRE_BUILD? | default "") != "" { $env.PRE_BUILD = $env.INPUT_PRE_BUILD }
    if ($env.INPUT_BINARY_PATH? | default "") != "" { $env.BINARY_PATH = $env.INPUT_BINARY_PATH }
    if ($env.INPUT_FEATURES? | default "") != "" { $env.FEATURES = $env.INPUT_FEATURES }
    if ($env.INPUT_PROFILE? | default "") != "" { $env.PROFILE = $env.INPUT_PROFILE }
    if ($env.INPUT_RUSTFLAGS? | default "") != "" { $env.TARGET_RUSTFLAGS = $env.INPUT_RUSTFLAGS }
    if ($env.INPUT_SKIP_BUILD? | default "") == "true" { $env.SKIP_BUILD = "true" }
    if ($env.INPUT_LOCKED? | default "") == "true" { $env.LOCKED = "true" }
    if ($env.INPUT_NO_DEFAULT_FEATURES? | default "") == "true" { $env.NO_DEFAULT_FEATURES = "true" }
    if ($env.INPUT_USE_ZIGBUILD? | default "") == "true" { $env.USE_ZIGBUILD = "true" }
    if ($env.INPUT_ARCHIVE? | default "") == "true" { $env.ARCHIVE = "true" }

    # Output options
    if ($env.INPUT_CHECKSUM? | default "") != "" { $env.CHECKSUM = $env.INPUT_CHECKSUM }
    if ($env.INPUT_INCLUDE? | default "") != "" { $env.ARCHIVE_INCLUDE = $env.INPUT_INCLUDE }

    # Changelog options
    if ($env.INPUT_CHANGELOG? | default "") != "" { $env.CHANGELOG_PATH = $env.INPUT_CHANGELOG }
    if ($env.INPUT_NOTES_OUTPUT? | default "") != "" { $env.OUTPUT_PATH = $env.INPUT_NOTES_OUTPUT }

    # Version validation
    if ($env.INPUT_TAG? | default "") != "" { $env.TAG = $env.INPUT_TAG }
    if ($env.INPUT_EXPECTED_VERSION? | default "") != "" { $env.EXPECTED_VERSION = $env.INPUT_EXPECTED_VERSION }
    if ($env.INPUT_VALIDATE_CARGO_TOML? | default "") == "true" { $env.VALIDATE_CARGO_TOML = "true" }

    # Package metadata
    if ($env.INPUT_PKG_DESCRIPTION? | default "") != "" { $env.PKG_DESCRIPTION = $env.INPUT_PKG_DESCRIPTION }
    if ($env.INPUT_PKG_MAINTAINER? | default "") != "" { $env.PKG_MAINTAINER = $env.INPUT_PKG_MAINTAINER }
    if ($env.INPUT_PKG_HOMEPAGE? | default "") != "" { $env.PKG_HOMEPAGE = $env.INPUT_PKG_HOMEPAGE }
    if ($env.INPUT_PKG_LICENSE? | default "") != "" { $env.PKG_LICENSE = $env.INPUT_PKG_LICENSE }
    if ($env.INPUT_PKG_VENDOR? | default "") != "" { $env.PKG_VENDOR = $env.INPUT_PKG_VENDOR }
    if ($env.INPUT_PKG_DEPENDS? | default "") != "" { $env.PKG_DEPENDS = $env.INPUT_PKG_DEPENDS }
    if ($env.INPUT_PKG_RECOMMENDS? | default "") != "" { $env.PKG_RECOMMENDS = $env.INPUT_PKG_RECOMMENDS }
    if ($env.INPUT_PKG_SUGGESTS? | default "") != "" { $env.PKG_SUGGESTS = $env.INPUT_PKG_SUGGESTS }
    if ($env.INPUT_PKG_CONFLICTS? | default "") != "" { $env.PKG_CONFLICTS = $env.INPUT_PKG_CONFLICTS }
    if ($env.INPUT_PKG_REPLACES? | default "") != "" { $env.PKG_REPLACES = $env.INPUT_PKG_REPLACES }
    if ($env.INPUT_PKG_PROVIDES? | default "") != "" { $env.PKG_PROVIDES = $env.INPUT_PKG_PROVIDES }
    if ($env.INPUT_PKG_CONTENTS? | default "") != "" { $env.PKG_CONTENTS = $env.INPUT_PKG_CONTENTS }
    if ($env.INPUT_PKG_SECTION? | default "") != "" { $env.PKG_SECTION = $env.INPUT_PKG_SECTION }
    if ($env.INPUT_PKG_PRIORITY? | default "") != "" { $env.PKG_PRIORITY = $env.INPUT_PKG_PRIORITY }
    if ($env.INPUT_PKG_GROUP? | default "") != "" { $env.PKG_GROUP = $env.INPUT_PKG_GROUP }
    if ($env.INPUT_PKG_RELEASE? | default "") != "" { $env.PKG_RELEASE = $env.INPUT_PKG_RELEASE }

    # SBOM options
    if ($env.INPUT_SBOM_FORMAT? | default "") != "" { $env.SBOM_FORMAT = $env.INPUT_SBOM_FORMAT }
    if ($env.INPUT_SBOM_DIR? | default "") != "" { $env.SBOM_OUTPUT_DIR = $env.INPUT_SBOM_DIR }

    # Homebrew options
    if ($env.INPUT_BREW_CLASS? | default "") != "" { $env.HOMEBREW_FORMULA_CLASS = $env.INPUT_BREW_CLASS }
    if ($env.INPUT_BREW_MACOS_ARM64_URL? | default "") != "" { $env.HOMEBREW_MACOS_ARM64_URL = $env.INPUT_BREW_MACOS_ARM64_URL }
    if ($env.INPUT_BREW_MACOS_ARM64_SHA256? | default "") != "" { $env.HOMEBREW_MACOS_ARM64_SHA256 = $env.INPUT_BREW_MACOS_ARM64_SHA256 }
    if ($env.INPUT_BREW_MACOS_X64_URL? | default "") != "" { $env.HOMEBREW_MACOS_X64_URL = $env.INPUT_BREW_MACOS_X64_URL }
    if ($env.INPUT_BREW_MACOS_X64_SHA256? | default "") != "" { $env.HOMEBREW_MACOS_X64_SHA256 = $env.INPUT_BREW_MACOS_X64_SHA256 }
    if ($env.INPUT_BREW_LINUX_ARM64_URL? | default "") != "" { $env.HOMEBREW_LINUX_ARM64_URL = $env.INPUT_BREW_LINUX_ARM64_URL }
    if ($env.INPUT_BREW_LINUX_ARM64_SHA256? | default "") != "" { $env.HOMEBREW_LINUX_ARM64_SHA256 = $env.INPUT_BREW_LINUX_ARM64_SHA256 }
    if ($env.INPUT_BREW_LINUX_X64_URL? | default "") != "" { $env.HOMEBREW_LINUX_X64_URL = $env.INPUT_BREW_LINUX_X64_URL }
    if ($env.INPUT_BREW_LINUX_X64_SHA256? | default "") != "" { $env.HOMEBREW_LINUX_X64_SHA256 = $env.INPUT_BREW_LINUX_X64_SHA256 }
    if ($env.INPUT_BREW_DIR? | default "") != "" { $env.HOMEBREW_OUTPUT_DIR = $env.INPUT_BREW_DIR }

    # Signing options
    if ($env.INPUT_ARTIFACT? | default "") != "" { $env.ARTIFACT_PATH = $env.INPUT_ARTIFACT }

    # Artifact collection
    if ($env.INPUT_ARTIFACTS_DIR? | default "") != "" { $env.ARTIFACTS_DIR = $env.INPUT_ARTIFACTS_DIR }
    if ($env.INPUT_BASE_URL? | default "") != "" { $env.BASE_URL = $env.INPUT_BASE_URL }

    # Release body options
    if ($env.INPUT_NOTES_FILE? | default "") != "" { $env.RELEASE_NOTES_FILE = $env.INPUT_NOTES_FILE }
    if ($env.INPUT_INCLUDE_CHECKSUMS? | default "") != "" { $env.INCLUDE_CHECKSUMS = $env.INPUT_INCLUDE_CHECKSUMS }
    if ($env.INPUT_INCLUDE_SIGNATURES? | default "") != "" { $env.INCLUDE_SIGNATURES = $env.INPUT_INCLUDE_SIGNATURES }
    if ($env.INPUT_HOMEBREW_TAP? | default "") != "" { $env.HOMEBREW_TAP = $env.INPUT_HOMEBREW_TAP }
    if ($env.INPUT_AUR_PACKAGE? | default "") != "" { $env.AUR_PACKAGE = $env.INPUT_AUR_PACKAGE }
    if ($env.INPUT_WINGET_ID? | default "") != "" { $env.WINGET_ID = $env.INPUT_WINGET_ID }

    # AUR options
    if ($env.INPUT_AUR_NAME? | default "") != "" { $env.AUR_PACKAGE_NAME = $env.INPUT_AUR_NAME }
    if ($env.INPUT_AUR_MAINTAINER? | default "") != "" { $env.AUR_MAINTAINER = $env.INPUT_AUR_MAINTAINER }
    if ($env.INPUT_AUR_SOURCE_URL? | default "") != "" { $env.AUR_SOURCE_URL = $env.INPUT_AUR_SOURCE_URL }
    if ($env.INPUT_AUR_SOURCE_SHA256? | default "") != "" { $env.AUR_SOURCE_SHA256 = $env.INPUT_AUR_SOURCE_SHA256 }
    if ($env.INPUT_AUR_MAKEDEPENDS? | default "") != "" { $env.AUR_MAKEDEPENDS = $env.INPUT_AUR_MAKEDEPENDS }
    if ($env.INPUT_AUR_OPTDEPENDS? | default "") != "" { $env.AUR_OPTDEPENDS = $env.INPUT_AUR_OPTDEPENDS }
    if ($env.INPUT_AUR_DIR? | default "") != "" { $env.AUR_OUTPUT_DIR = $env.INPUT_AUR_DIR }

    # Winget options
    if ($env.INPUT_WINGET_PUBLISHER? | default "") != "" { $env.WINGET_PUBLISHER = $env.INPUT_WINGET_PUBLISHER }
    if ($env.INPUT_WINGET_PUBLISHER_ID? | default "") != "" { $env.WINGET_PUBLISHER_ID = $env.INPUT_WINGET_PUBLISHER_ID }
    if ($env.INPUT_WINGET_PACKAGE_ID? | default "") != "" { $env.WINGET_PACKAGE_ID = $env.INPUT_WINGET_PACKAGE_ID }
    if ($env.INPUT_WINGET_LICENSE_URL? | default "") != "" { $env.WINGET_LICENSE_URL = $env.INPUT_WINGET_LICENSE_URL }
    if ($env.INPUT_WINGET_COPYRIGHT? | default "") != "" { $env.WINGET_COPYRIGHT = $env.INPUT_WINGET_COPYRIGHT }
    if ($env.INPUT_WINGET_TAGS? | default "") != "" { $env.WINGET_TAGS = $env.INPUT_WINGET_TAGS }
    if ($env.INPUT_WINGET_X64_URL? | default "") != "" { $env.WINGET_X64_URL = $env.INPUT_WINGET_X64_URL }
    if ($env.INPUT_WINGET_X64_SHA256? | default "") != "" { $env.WINGET_X64_SHA256 = $env.INPUT_WINGET_X64_SHA256 }
    if ($env.INPUT_WINGET_ARM64_URL? | default "") != "" { $env.WINGET_ARM64_URL = $env.INPUT_WINGET_ARM64_URL }
    if ($env.INPUT_WINGET_ARM64_SHA256? | default "") != "" { $env.WINGET_ARM64_SHA256 = $env.INPUT_WINGET_ARM64_SHA256 }
    if ($env.INPUT_WINGET_DIR? | default "") != "" { $env.WINGET_OUTPUT_DIR = $env.INPUT_WINGET_DIR }

    let script = match $command {
        "extract-changelog" => "extract-changelog.nu"
        "validate-changelog" => "validate-changelog.nu"
        "validate-version" => "validate-version.nu"
        "get-version" => "get-version.nu"
        "generate-sbom" => "generate-sbom.nu"
        "generate-homebrew" => "generate-homebrew.nu"
        "generate-aur" => "generate-aur.nu"
        "generate-winget" => "generate-winget.nu"
        "sign-artifact" => "sign-artifact.nu"
        "format-release" => "format-release.nu"
        "collect-artifacts" => "collect-artifacts.nu"
        "release" => "release.nu"
        "release-linux" => "release-linux.nu"
        "release-linux-deb" => "release-linux-deb.nu"
        "release-linux-rpm" => "release-linux-rpm.nu"
        "release-linux-apk" => "release-linux-apk.nu"
        "release-macos" => "release-macos.nu"
        "release-macos-dmg" => "release-macos-dmg.nu"
        "release-windows" => "release-windows.nu"
        "release-windows-msi" => "release-windows-msi.nu"
        _ => {
            error $"unknown command '($command)'"
        }
    }

    nu ($scripts | path join $script)
}
