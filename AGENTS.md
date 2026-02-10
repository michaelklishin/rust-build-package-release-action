# Instructions for AI Agents

## Project Overview

`rust-build-package-release-action` is an **opinionated** GitHub Action that automates release workflows for Rust projects,
built as a single Rust binary crate with clap subcommands (edition 2024, rust-version 1.85).

This is a conventions-based, opinionated release process extracted from:

 * [rabbitmq/rabbitmqadmin-ng](https://github.com/rabbitmq/rabbitmqadmin-ng)
 * [michaelklishin/rabbitmq-lqt](https://github.com/michaelklishin/rabbitmq-lqt)
 * [michaelklishin/frm](https://github.com/michaelklishin/frm)

## Testing

Run all tests:

```bash
cargo test
```

Run clippy:

```bash
cargo clippy --all-targets -- -D warnings
```

See `CONTRIBUTING.md` as well.

For verifying YAML file syntax, use `yq`, Ruby or Python YAML modules (whichever is available).

## Key Files

 * `action.yml`: GitHub Action definition
 * `Cargo.toml`: crate manifest
 * `src/main.rs`: clap CLI and command dispatcher (routes subcommands to modules)
 * `src/lib.rs`: shared utilities (`env_or`, `parse_comma_list`) and module re-exports
 * `src/error.rs`: error type and `Result` alias
 * `src/output.rs`: GitHub Actions output helpers (`output`, `output_multiline`, `print_hr`)
 * `src/platform.rs`: target triple parsing and platform detection
 * `src/version.rs`: version validation, `get-version`, `get-release-version`
 * `src/cargo_info.rs`: reads package metadata from `Cargo.toml`
 * `src/changelog.rs`: extracts and validates changelog entries
 * `src/build.rs`: cargo build orchestration
 * `src/archive.rs`: archive file listing, doc copying, include handling
 * `src/checksum.rs`: SHA-256, SHA-512, BLAKE2 checksum generation and verification
 * `src/collect_artifacts.rs`: collects artifacts, computes checksums, outputs structured data for Homebrew/Winget
 * `src/release.rs`: unified release command that auto-selects platform from target triple
 * `src/nfpm.rs`: nfpm config generation for .deb/.rpm/.apk packages
 * `src/homebrew.rs`: Homebrew formula generator
 * `src/aur.rs`: Arch Linux PKGBUILD generator
 * `src/winget.rs`: Winget manifest generator
 * `src/format_release.rs`: GitHub Release body formatter
 * `src/sbom.rs`: SPDX and CycloneDX SBOM generation via cargo-sbom
 * `src/sign.rs`: Sigstore/cosign artifact signing
 * `src/download.rs`: downloads artifacts from GitHub releases
 * `src/testing.rs`: tests Debian/RPM packages and Windows binaries
 * `src/tools.rs`: external tool installation (cross, cargo-zigbuild, nfpm, etc.)
 * `tests/test_helpers.rs`: shared test utilities (`create_temp_file`, `create_temp_text_file`)
 * `tests/*_tests.rs`: unit tests for each module
 * `tests/*_proptests.rs`: property-based tests (proptest) for version, changelog, platform
 * `examples/*.yml`: workflow examples for common scenarios

## Rust Style

 * Use `use` statements at the top module level, never in function scope
 * Avoid fully qualified type paths (e.g. `std::fmt::Display`) unless needed for disambiguation
 * All tests go in `tests/` as integration tests, not inline `#[cfg(test)]` modules
 * Use `LazyLock<Mutex<()>>` to serialise tests that modify process-wide state (env vars, CWD)
 * Mark `env::set_var` and `env::remove_var` calls as `unsafe` with a safety comment
 * Format with `cargo fmt --all`, lint with `cargo clippy --all-features --all`
 * Only add important comments

## Git Conventions

 * Never add yourself to commit co-authors list
 * Never mention yourself in commit messages

## Markdown Style

 * Never add full stops to Markdown list items

## Reviews

Perform up to twenty iterative reviews after completing a task. Look for:

 * Meaningful improvements
 * Test coverage gaps
 * Deviations from the instructions in `AGENTS.md`

Stop iterating when three iterations in a row show no meaningful improvements.
