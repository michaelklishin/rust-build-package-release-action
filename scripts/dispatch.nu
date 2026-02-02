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
    setup-env

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

# Maps INPUT_* environment variables to the names expected by scripts
def setup-env [] {
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
    map-env "INPUT_TARGET" "TARGET"
    map-env "INPUT_BINARY_NAME" "BINARY_NAME"
    map-env "INPUT_PACKAGE" "PACKAGE"
    map-env "INPUT_MANIFEST" "MANIFEST_PATH"

    # Build options
    map-env "INPUT_PRE_BUILD" "PRE_BUILD"
    map-env "INPUT_BINARY_PATH" "BINARY_PATH"
    map-env "INPUT_FEATURES" "FEATURES"
    map-env "INPUT_PROFILE" "PROFILE"
    map-env "INPUT_RUSTFLAGS" "TARGET_RUSTFLAGS"
    map-bool "INPUT_SKIP_BUILD" "SKIP_BUILD"
    map-bool "INPUT_LOCKED" "LOCKED"
    map-bool "INPUT_NO_DEFAULT_FEATURES" "NO_DEFAULT_FEATURES"
    map-bool "INPUT_ARCHIVE" "ARCHIVE"

    # Output options
    map-env "INPUT_CHECKSUM" "CHECKSUM"
    map-env "INPUT_INCLUDE" "INCLUDE"

    # Changelog options
    map-env "INPUT_CHANGELOG" "CHANGELOG_PATH"
    map-env "INPUT_NOTES_OUTPUT" "OUTPUT_PATH"

    # Version validation
    map-env "INPUT_TAG" "TAG"
    map-env "INPUT_EXPECTED_VERSION" "EXPECTED_VERSION"
    map-bool "INPUT_VALIDATE_CARGO_TOML" "VALIDATE_CARGO_TOML"

    # Package metadata
    map-env "INPUT_PKG_DESCRIPTION" "PKG_DESCRIPTION"
    map-env "INPUT_PKG_MAINTAINER" "PKG_MAINTAINER"
    map-env "INPUT_PKG_HOMEPAGE" "PKG_HOMEPAGE"
    map-env "INPUT_PKG_LICENSE" "PKG_LICENSE"
    map-env "INPUT_PKG_VENDOR" "PKG_VENDOR"
    map-env "INPUT_PKG_DEPENDS" "PKG_DEPENDS"
    map-env "INPUT_PKG_RECOMMENDS" "PKG_RECOMMENDS"
    map-env "INPUT_PKG_SUGGESTS" "PKG_SUGGESTS"
    map-env "INPUT_PKG_CONFLICTS" "PKG_CONFLICTS"
    map-env "INPUT_PKG_REPLACES" "PKG_REPLACES"
    map-env "INPUT_PKG_PROVIDES" "PKG_PROVIDES"
    map-env "INPUT_PKG_CONTENTS" "PKG_CONTENTS"
    map-env "INPUT_PKG_SECTION" "PKG_SECTION"
    map-env "INPUT_PKG_PRIORITY" "PKG_PRIORITY"
    map-env "INPUT_PKG_GROUP" "PKG_GROUP"
    map-env "INPUT_PKG_RELEASE" "PKG_RELEASE"

    # SBOM options
    map-env "INPUT_SBOM_FORMAT" "SBOM_FORMAT"
    map-env "INPUT_SBOM_DIR" "SBOM_OUTPUT_DIR"

    # Homebrew options
    map-env "INPUT_BREW_CLASS" "HOMEBREW_FORMULA_CLASS"
    map-env "INPUT_BREW_MACOS_ARM64_URL" "HOMEBREW_MACOS_ARM64_URL"
    map-env "INPUT_BREW_MACOS_ARM64_SHA256" "HOMEBREW_MACOS_ARM64_SHA256"
    map-env "INPUT_BREW_MACOS_X64_URL" "HOMEBREW_MACOS_X64_URL"
    map-env "INPUT_BREW_MACOS_X64_SHA256" "HOMEBREW_MACOS_X64_SHA256"
    map-env "INPUT_BREW_LINUX_ARM64_URL" "HOMEBREW_LINUX_ARM64_URL"
    map-env "INPUT_BREW_LINUX_ARM64_SHA256" "HOMEBREW_LINUX_ARM64_SHA256"
    map-env "INPUT_BREW_LINUX_X64_URL" "HOMEBREW_LINUX_X64_URL"
    map-env "INPUT_BREW_LINUX_X64_SHA256" "HOMEBREW_LINUX_X64_SHA256"
    map-env "INPUT_BREW_DIR" "HOMEBREW_OUTPUT_DIR"

    # Signing options
    map-env "INPUT_ARTIFACT" "ARTIFACT_PATH"

    # Artifact collection
    map-env "INPUT_ARTIFACTS_DIR" "ARTIFACTS_DIR"
    map-env "INPUT_BASE_URL" "BASE_URL"

    # Release body options
    map-env "INPUT_NOTES_FILE" "RELEASE_NOTES_FILE"
    map-env "INPUT_INCLUDE_CHECKSUMS" "INCLUDE_CHECKSUMS"
    map-env "INPUT_INCLUDE_SIGNATURES" "INCLUDE_SIGNATURES"
    map-env "INPUT_HOMEBREW_TAP" "HOMEBREW_TAP"
    map-env "INPUT_AUR_PACKAGE" "AUR_PACKAGE"
    map-env "INPUT_WINGET_ID" "WINGET_ID"

    # AUR options
    map-env "INPUT_AUR_NAME" "AUR_PACKAGE_NAME"
    map-env "INPUT_AUR_MAINTAINER" "AUR_MAINTAINER"
    map-env "INPUT_AUR_SOURCE_URL" "AUR_SOURCE_URL"
    map-env "INPUT_AUR_SOURCE_SHA256" "AUR_SOURCE_SHA256"
    map-env "INPUT_AUR_MAKEDEPENDS" "AUR_MAKEDEPENDS"
    map-env "INPUT_AUR_OPTDEPENDS" "AUR_OPTDEPENDS"
    map-env "INPUT_AUR_DIR" "AUR_OUTPUT_DIR"

    # Winget options
    map-env "INPUT_WINGET_PUBLISHER" "WINGET_PUBLISHER"
    map-env "INPUT_WINGET_PUBLISHER_ID" "WINGET_PUBLISHER_ID"
    map-env "INPUT_WINGET_PACKAGE_ID" "WINGET_PACKAGE_ID"
    map-env "INPUT_WINGET_LICENSE_URL" "WINGET_LICENSE_URL"
    map-env "INPUT_WINGET_COPYRIGHT" "WINGET_COPYRIGHT"
    map-env "INPUT_WINGET_TAGS" "WINGET_TAGS"
    map-env "INPUT_WINGET_X64_URL" "WINGET_X64_URL"
    map-env "INPUT_WINGET_X64_SHA256" "WINGET_X64_SHA256"
    map-env "INPUT_WINGET_ARM64_URL" "WINGET_ARM64_URL"
    map-env "INPUT_WINGET_ARM64_SHA256" "WINGET_ARM64_SHA256"
    map-env "INPUT_WINGET_DIR" "WINGET_OUTPUT_DIR"
}

# Maps an INPUT_* env var to script env var if non-empty
def map-env [from: string, to: string] {
    let value = $env | get -o $from | default ""
    if $value != "" {
        load-env { $to: $value }
    }
}

# Maps a boolean INPUT_* env var to script env var
def map-bool [from: string, to: string] {
    let value = $env | get -o $from | default ""
    if $value == "true" {
        load-env { $to: "true" }
    }
}
