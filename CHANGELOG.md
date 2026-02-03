# Changelog

## v1.20.0 (Feb 3, 2026)

### New Features

 * `get-release-version` fetches the latest release from GitHub and (if available)
   verifies their SHA-256 checksums
 * `test-deb`, `test-rpm`, `test-windows` commands for smoke-testing artifacts
 * `download-from-release` mode for automatic artifact fetching
 * New example workflows: `verify-artifacts.yml`, `build-with-verification.yml`

### Enhancements

 * All commands now support the `GITHUB_TOKEN` env variable (where applicable)
   with automatic fallback to token-unaware (anonymous) operations for public repos

## v1.19.0 (Feb 3, 2026)

### Enhancements

 * Improve SBOM section formatting in release body
 * Simplify `generate-aur.nu` variable handling and error messages
 * Add tests for `get-cargo-info`, `copy-docs`, `copy-includes`

## v1.18.0 (Feb 3, 2026)

### Bug Fixes

 * Fix Windows SDK `INCLUDE` path conflict by renaming internal env var to `ARCHIVE_INCLUDE`
 * Add workspace-aware WiX file lookup for MSI builds using `cargo metadata`
 * Fix Windows path normalization in workspace WiX lookup

## v1.17.0 (Feb 2, 2026)

### Bug Fixes

 * Fix Nushell syntax error where `(skip-build)` was interpreted as a command in print statements
 * Add early prerequisite check for WiX source files in MSI builds

## v1.16.0 (Feb 2, 2026)

### Enhancements

 * Rename "Downloads" section to "Build Assets" in release body

## v1.15.0 (Feb 2, 2026)

### New Features

 * Support for cross-compilation via `cargo-zigbuild` (for the painful-to-get-right `aarch64-unknown-linux-musl` target)

## v1.14.0 (Feb 2, 2026)

### Bug Fixes

 * Use `get -o 0` instead of `first?` for Nu 0.110 compatibility
 * Improve AUR error messages, fallback to `binary-name` input
 * Fix empty table handling in `format-release.nu`
 * Fix filesize to int conversion in `format-size`
 * Add retries to the DMG creation step to work around certain "resource busy" errors on macOS runners

## v1.13.0 (Feb 2, 2026)

### Bug Fixes

 * Fix regex escaping in `collect-artifacts.nu`

## v1.12.0 (Feb 1, 2026)

### Bug Fixes

 * Auto-install WiX Toolset for Windows MSI builds

## v1.11.0 (Feb 1, 2026)

### New Features

 * Support for optional Cargo project features (auto-detection via `cargo metadata`)
 * Auto-enable `mimalloc` feature for musl builds when available (improves performance)

## v1.10.0 (Feb 1, 2026)

### Bug Fixes

 * Use the Cargo package name, not the binary name, for `cargo-wix --package`

## v1.9.0 (Feb 1, 2026)

### Bug Fixes

 * Fix `cosign` path resolution on Windows

## v1.8.0 (Feb 1, 2026)

### Bug Fixes

 * Fix `cosign` 3.x signing (auto-detect GitHub Actions environment)

## v1.7.0 (Feb 1, 2026)

### Bug Fixes

 * Fix `cosign` installation on Windows

## v1.6.0 (Feb 1, 2026)

### Bug Fixes

 * Fix `nfpm` downloads on x86-64 Linux

## v1.5.0 (Feb 1, 2026)

### Bug Fixes

 * Fix mutable variable capture in cosign signing closure

## v1.4.0 (Feb 1, 2026)

### Bug Fixes

 * Fix nfpm config: `section` and `priority` are top-level fields, not nested under `deb:`

## v1.3.0 (Feb 1, 2026)

### Bug Fixes

 * Use direct `$env.VAR` assignment instead of `load-env` to propagate variables to subprocesses

## v1.2.0 (Feb 1, 2026)

### Bug Fixes

 * Remove `vars.NEXT_RELEASE_VERSION` from composite action (the `vars.*` context is not available there)
 * Use `get -o` instead of the deprecated `get -i` flag in dispatch.nu

## v1.1.0 (Feb 1, 2026)

### Bug Fixes

 * Don't run over the Actions' inline element length limit

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
