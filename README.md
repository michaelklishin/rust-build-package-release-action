# rust-release-action

An opinionated GitHub Action that automates release workflows for Rust projects using Nu shell scripts.

This is a conventions-based release process extracted from:

 * [rabbitmq/rabbitmqadmin-rs](https://github.com/rabbitmq/rabbitmqadmin-rs)
 * [michaelklishin/rabbitmq-lqt](https://github.com/michaelklishin/rabbitmq-lqt)
 * [michaelklishin/frm](https://github.com/michaelklishin/frm)

## Features

 * Parse `CHANGELOG.md` and extract release notes for a specific version
 * Validate git tags match expected versions via `NEXT_RELEASE_VERSION` variable
 * Extract version from `Cargo.toml` (supports workspace manifests)
 * Build and package binaries for Linux, macOS, and Windows

## Conventions

This action expects:

 1. **Changelog format**: versions as `## v{version} ({date})` headers
 2. **Tag format**: tags prefixed with `v` (e.g., `v1.2.3`)
 3. **Version variable**: `NEXT_RELEASE_VERSION` GitHub Actions variable
 4. **Semantic versioning**: `MAJOR.MINOR.PATCH` format

## Usage

### Extract Changelog

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: extract-changelog
    version: '1.2.3'
```

### Validate Version

Tag and expected version are inferred from `GITHUB_REF_NAME` and `NEXT_RELEASE_VERSION`:

```yaml
- uses: michaelklishin/rust-release-action@v0
  id: validate
  with:
    command: validate-version
```

Or specify explicitly:

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: validate-version
    tag: v1.2.3
    expected-version: '1.2.3'
```

### Get Version from Cargo.toml

```yaml
- uses: michaelklishin/rust-release-action@v0
  id: version
  with:
    command: get-version
```

### Build Release Binaries

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-linux
    target: x86_64-unknown-linux-gnu
```

### Workspace Builds

For Cargo workspaces, specify the package name:

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-linux
    target: x86_64-unknown-linux-gnu
    package: my-cli-bin
    binary-name: my-cli
```

### Static Builds (musl)

For musl targets, static linking is enabled automatically. Use `no-default-features` to disable features that require dynamic linking (e.g., native-tls):

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-linux
    target: x86_64-unknown-linux-musl
    no-default-features: 'true'
```

### Windows MSI Installer

Build a Windows MSI installer using cargo-wix:

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-windows-msi
    target: x86_64-pc-windows-msvc
```

This requires a `wix/main.wxs` file in your project. See [cargo-wix documentation](https://github.com/volks73/cargo-wix) for setup.

### Monorepo Support

Use `working-directory` for projects in subdirectories:

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: get-version
    working-directory: crates/my-cli
```

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `command` | Command to run (required) | - |
| `version` | Version to extract | - |
| `changelog` | Path to CHANGELOG.md | `CHANGELOG.md` |
| `output` | Output file for release notes | `release_notes.md` |
| `tag` | Git tag for validation | `GITHUB_REF_NAME` |
| `expected-version` | Expected version | `NEXT_RELEASE_VERSION` |
| `manifest` | Path to Cargo.toml | `Cargo.toml` |
| `target` | Rust target triple | platform default |
| `binary-name` | Binary name | package name |
| `package` | Cargo package name for workspaces | - |
| `no-default-features` | Build with --no-default-features | `false` |
| `target-rustflags` | Extra RUSTFLAGS for the build | - |
| `working-directory` | Working directory | `.` |

## Outputs

| Output | Description |
|--------|-------------|
| `version` | Extracted or validated version |
| `release_notes_file` | Path to release notes file |
| `artifact` | Artifact filename |
| `artifact_path` | Full path to artifact |

## Complete Workflow Example

```yaml
name: Release

on:
  workflow_dispatch:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'

permissions:
  contents: write

concurrency:
  group: release-${{ github.ref }}
  cancel-in-progress: false

jobs:
  validate:
    runs-on: ubuntu-22.04
    timeout-minutes: 5
    outputs:
      version: ${{ steps.validate.outputs.version }}
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2

      - uses: michaelklishin/rust-release-action@v0
        id: validate
        with:
          command: validate-version

  build:
    needs: validate
    timeout-minutes: 30
    strategy:
      fail-fast: false
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-22.04
            command: release-linux
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-24.04-arm
            command: release-linux
          - target: x86_64-unknown-linux-musl
            os: ubuntu-22.04
            command: release-linux
          - target: aarch64-apple-darwin
            os: macos-14
            command: release-macos
          - target: x86_64-pc-windows-msvc
            os: windows-2022
            command: release-windows
          - target: aarch64-pc-windows-msvc
            os: windows-11-arm
            command: release-windows
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2

      - name: Setup Rust
        run: |
          rustup toolchain install stable --profile minimal
          rustup default stable

      - uses: Swatinem/rust-cache@9d47c6ad4b02e050fd481d890b2ea34778fd09d6 # v2.7.8
        with:
          key: ${{ matrix.target }}

      - uses: michaelklishin/rust-release-action@v0
        id: build
        with:
          command: ${{ matrix.command }}
          target: ${{ matrix.target }}

      - uses: actions/upload-artifact@ea165f8d65b6e75b540449e92b4886f43607fa02 # v4.6.2
        with:
          name: ${{ matrix.target }}
          path: target/${{ matrix.target }}/release/
          retention-days: 2

  release:
    needs: [validate, build]
    runs-on: ubuntu-22.04
    timeout-minutes: 10
    steps:
      - uses: actions/checkout@11bd71901bbe5b1630ceea73d27597364c9af683 # v4.2.2

      - uses: michaelklishin/rust-release-action@v0
        with:
          command: extract-changelog
          version: ${{ needs.validate.outputs.version }}

      - uses: actions/download-artifact@d3f86a106a0bac45b974a628896c90dbdf5c8093 # v4.3.0
        with:
          path: artifacts

      - name: Collect release files
        run: |
          mkdir -p release
          find artifacts -type f -name '*-${{ needs.validate.outputs.version }}-*' -exec cp {} release/ \;

      - uses: softprops/action-gh-release@da05d552573ad5aba039eaac05058a918a7bf631 # v2.2.2
        with:
          tag_name: v${{ needs.validate.outputs.version }}
          name: v${{ needs.validate.outputs.version }}
          body_path: release_notes.md
          files: release/*
```

## Best Practices

 * Actions pinned to commit SHAs
 * Minimal token permissions
 * Swatinem/rust-cache for faster builds
 * `--locked` flag for reproducible builds
 * Specific runner versions (no `-latest`)
 * Timeouts on all jobs
 * Concurrency settings to prevent duplicate runs

## License

Licensed under either of:

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
 * MIT license ([LICENSE-MIT](LICENSE-MIT))
