# Changelog

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
