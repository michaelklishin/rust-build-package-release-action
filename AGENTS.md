# Instructions for AI Agents

## Project Overview

`rust-release-action` is an **opinionated** GitHub Action that automates release workflows for Rust projects,
built as a collection of Nu shell scripts.

This is a conventions-based, opinionated release process extracted from:
 
 * [rabbitmq/rabbitmqadmin-ng](https://github.com/rabbitmq/rabbitmqadmin-ng)
 * [michaelklishin/rabbitmq-lqt](https://github.com/michaelklishin/rabbitmq-lqt)
 * [michaelklishin/frm](https://github.com/michaelklishin/frm)

## Testing

```bash
nu scripts/extract-changelog.nu
nu scripts/validate-version.nu
nu scripts/get-version.nu
```

Set required environment variables before running scripts.

## Key Files

 * `action.yml`: GitHub Action definition
 * `scripts/common.nu`: shared utilities (`cargo-build`, `generate-checksums`, `output-build-results`)
 * `scripts/extract-changelog.nu`: extracts release details from a change log file (see `rabbitmqadmin-ng` for example)
 * `scripts/validate-version.nu`: version validation logic, including pre-release versions
 * `scripts/get-version.nu`: reads version from `Cargo.toml`
 * `scripts/release-linux.nu`: Linux build script
 * `scripts/release-linux-deb.nu`: Linux .deb package via nfpm
 * `scripts/release-linux-rpm.nu`: Linux .rpm package via nfpm
 * `scripts/release-linux-apk.nu`: Linux .apk package via nfpm
 * `scripts/release-macos.nu`: macOS build script
 * `scripts/release-windows.nu`: Windows build script
 * `scripts/release-windows-msi.nu`: Windows MSI installer via cargo-wix
 * `tests/run.nu`: Nu shell test runner
 * `tests/nu/*.nu`: unit tests for pure functions

## Nu Script Style

 * Use `def main []` as script entry point
 * Use `$env.VARIABLE?` with `| default ""` for optional env vars
 * Exit with code 1 on errors, include helpful messages
 * Write to `$env.GITHUB_OUTPUT` for action outputs
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
