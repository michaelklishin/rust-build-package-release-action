# Changelog

## v1.0.0 (Feb 1, 2026)

### New Features

 * Introduce a unified `release` command that auto-selects the platform from the target triple
 * A new `validate-changelog` command for fail-fast changelog validation
 * A new `collect-artifacts` command with consolidated SHA256SUMS generation
 * A new `validate-cargo-toml` option for the `validate-version` command
 * Pre-build hooks via the `pre-build` input for WASM/frontend projects
 * Skip-build mode via the `skip-build` input for container workflows
 * Version auto-detection from git tags for the `extract-changelog` and `validate-changelog` commands
 * Minimal quickstart workflow in the README

### Enhancements

 * Simplified the multi-platform matrix (no per-platform command mapping needed)
 * Added the `examples/` directory with workflow templates

## v0.13.0 (Feb 1, 2026)

### Bug Fixes

 * Fix archives to include LICENSE and README files
 * Exclude previous artifacts and checksums from archives
 * Fix Windows archive path handling after directory change
 * Use MANIFEST_PATH in get-cargo-info
 * Include LICENSE files without suffix (e.g., LICENSE vs LICENSE-MIT)
 * Release workflow now triggers on pre-release tags
 * Extract shared helpers to reduce code duplication

## v0.12.0 (Jan 31, 2026)

### Enhancements

 * Add checksum generation (SHA256, SHA512, BLAKE2) with `checksum` input
 * Add `features` input for enabling Cargo features
 * Add `include` input for additional files in archives
 * Add outputs: `binary_name`, `binary_path`, `target`, `sha256`, `summary`
 * Support pre-release version tags (alpha, beta, rc)

## v0.11.0 (Jan 31, 2026)

### Enhancements

 * Add `locked` input for reproducible builds using `--locked` flag
 * Release commands now output `version` for use in downstream workflow steps

## v0.10.0 (Jan 31, 2026)

### Enhancements

 * Add `archive` input to create .tar.gz archives on Linux/macOS and .zip on Windows
 * Set `CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER` for armv7 cross-compilation

## v0.9.0 (Jan 31, 2026)

### Enhancements

 * Upgrade Nu shell from 0.102.0 to 0.110.0
 * Switch from `cargo build` to `cargo rustc` for finer control
 * Add `release-windows-msi` command for Windows MSI installers via cargo-wix
 * Include version header in extracted changelog (matches upstream repos)
 * Add coloured output with ANSI codes for better visibility
 * Add `hr-line` helper for visual progress markers
 * Add `error` helper for consistent error formatting
 * Use modern Nu shell syntax (`get -o` instead of deprecated `get -i`)
 * Improve error messages with `ERROR:` prefix and helpful hints

## v0.8.0 (Jan 31, 2026)

### Enhancements

 * Add `package` input for Cargo workspace builds
 * Add `no-default-features` input for static/musl builds
 * Add `target-rustflags` input for custom build flags
 * Automatic static linking for musl targets (`-C target-feature=+crt-static`)
 * Add Fedora support for cross-compilation
 * Add `armv7-unknown-linux-gnueabihf` target support
 * Generate `Cargo.lock` if missing
 * Refactor build logic into shared `cargo-build` function

## v0.7.0 (Jan 31, 2026)

### Initial Release

 * `extract-changelog` command for extracting version-specific release notes from CHANGELOG.md
 * `validate-version` command for ensuring git tags match expected versions
 * `get-version` command for reading version from Cargo.toml (supports workspace manifests)
 * `release-linux`, `release-macos`, `release-windows` build commands
 * Composite GitHub Action with Nu shell scripts
 * Cross-platform CI testing (Linux, macOS, Windows)
