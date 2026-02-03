# Instructions for AI Agents

## Project Overview

`rust-release-action` is an **opinionated** GitHub Action that automates release workflows for Rust projects,
built as a collection of Nu shell scripts.

This is a conventions-based, opinionated release process extracted from:

 * [rabbitmq/rabbitmqadmin-ng](https://github.com/rabbitmq/rabbitmqadmin-ng)
 * [michaelklishin/rabbitmq-lqt](https://github.com/michaelklishin/rabbitmq-lqt)
 * [michaelklishin/frm](https://github.com/michaelklishin/frm)

## Testing

Run all tests:

```bash
nu tests/run.nu
```

Run individual scripts:

```bash
nu scripts/extract-changelog.nu
nu scripts/validate-version.nu
nu scripts/get-version.nu
```

Note: this requires taking care of some script-specific environment variables.

See `CONTRIBUTING.md` as well.

For verifying YAML file syntax, use `yq`, Ruby or Python YAML modules (whichever is available).

## Key Files

 * `action.yml`: GitHub Action definition
 * `scripts/dispatch.nu`: command dispatcher (routes inputs to scripts)
 * `scripts/common.nu`: shared utilities (`cargo-build`, `generate-checksums`, `output-build-results`)
 * `scripts/extract-changelog.nu`: extracts release details from a change log file (see `rabbitmqadmin-ng` for example)
 * `scripts/validate-changelog.nu`: validates that a changelog entry exists for a version
 * `scripts/validate-version.nu`: version validation logic, including pre-release versions
 * `scripts/collect-artifacts.nu`: collects artifacts, computes checksums, outputs structured data for Homebrew/Winget
 * `scripts/release.nu`: unified release command that auto-selects platform from target triple
 * `scripts/get-version.nu`: reads version from `Cargo.toml`
 * `scripts/generate-sbom.nu`: generates SPDX and CycloneDX SBOMs via cargo-sbom
 * `scripts/release-linux.nu`: Linux build script
 * `scripts/release-linux-deb.nu`: Linux .deb package via nfpm
 * `scripts/release-linux-rpm.nu`: Linux .rpm package via nfpm
 * `scripts/release-linux-apk.nu`: Linux .apk package via nfpm
 * `scripts/release-macos.nu`: macOS build script
 * `scripts/release-macos-dmg.nu`: macOS .dmg installer via hdiutil
 * `scripts/release-windows.nu`: Windows build script
 * `scripts/release-windows-msi.nu`: Windows MSI installer via cargo-wix
 * `scripts/generate-homebrew.nu`: Homebrew formula generator
 * `scripts/generate-aur.nu`: Arch Linux PKGBUILD generator
 * `scripts/generate-winget.nu`: Winget manifest generator
 * `scripts/sign-artifact.nu`: Sigstore/cosign signing
 * `scripts/format-release.nu`: GitHub Release body formatter
 * `tests/run.nu`: Nu shell test runner
 * `tests/nu/*.nu`: unit tests for pure functions

## Nu Script Style

 * Format all Nu scripts with [nufmt](https://github.com/nushell/nufmt) before committing
 * Use 4-space indentation (nufmt default)
 * Avoid multiline string interpolation in heredoc style; use array-based string building with `| str join "\n"` instead
 * Use `def main []` as script entry point
 * Use `$env.VARIABLE?` with `| default ""` for optional env vars
 * Exit with code 1 on errors, produce brief but helpful messages
 * Write to `$env.GITHUB_OUTPUT` for action outputs
 * Only add important comments
 * For regex patterns in `where` clauses, assign the pattern to a variable first to avoid breaking `nufmt`

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
