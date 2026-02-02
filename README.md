# rust-release-action

An opinionated, conventions-based GitHub Action that automates release workflows for Rust projects using Nu shell scripts.

## Features

 * Cross-platform builds for Linux, macOS, and Windows
 * Linux packages: `.deb`, `.rpm`, `.apk` via [nfpm](https://nfpm.goreleaser.com/)
 * macOS `.dmg` installers via `hdiutil`
 * Windows `.msi` installers via [cargo-wix](https://github.com/volks73/cargo-wix)
 * Homebrew formula, AUR PKGBUILD, and Winget manifest generation
 * Sigstore/cosign artifact signing
 * SBOM generation in SPDX and CycloneDX formats
 * Changelog parsing and GitHub Release body formatting
 * SHA256, SHA512, and BLAKE2 checksums

## Conventions

This action expects:

 1. **Changelog format**: versions as `## v{version} ({date})` headers
 2. **Tag format**: tags prefixed with `v` (e.g., `v1.2.3`, `v1.0.0-beta.1`)
 3. **Versioning**: `MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]` format

---

## Quick Start

```yaml
# Build a release binary (auto-selects platform from target)
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release
    target: x86_64-unknown-linux-gnu
    archive: 'true'
```

Or use platform-specific commands:

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-linux
    target: x86_64-unknown-linux-gnu
    locked: 'true'
```

### Minimal Complete Workflow

Copy this to `.github/workflows/release.yml` to get started:

```yaml
name: Release
on:
  push:
    tags: ['v*']

permissions:
  contents: write

jobs:
  build:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - run: rustup toolchain install stable --profile minimal
      - uses: michaelklishin/rust-release-action@v0
        id: build
        with:
          command: release
          target: x86_64-unknown-linux-gnu
          archive: 'true'
      - uses: softprops/action-gh-release@v2
        with:
          files: target/x86_64-unknown-linux-gnu/release/*.tar.gz
```

This builds a Linux binary, creates a `.tar.gz` archive with checksums, and uploads it to a GitHub Release. Add more targets using a [matrix strategy](#complete-workflow-example).

---

## Inputs Reference

### Core Inputs

Universal options used across most commands.

| Input | Description | Default |
|-------|-------------|---------|
| `command` | **Required.** Command to run (see [Commands](#commands)) | — |
| `version` | Version string without `v` prefix | — |
| `target` | Rust target triple | Platform default |
| `binary-name` | Binary name | Package name from Cargo.toml |
| `package` | Cargo package name (for workspaces) | — |
| `manifest` | Path to Cargo.toml | `Cargo.toml` |
| `working-directory` | Working directory for commands | `.` |

### Build Options

Standard Cargo build flags. These map directly to familiar `cargo build` options.

| Input | Description | Default |
|-------|-------------|---------|
| `pre-build` | Shell command to run before `cargo build` (e.g., frontend build step) | — |
| `skip-build` | Skip cargo build and use existing binary | `false` |
| `binary-path` | Path to existing binary when `skip-build` is true | — |
| `features` | Cargo features to enable | — |
| `profile` | Cargo build profile | `release` |
| `locked` | Build with `--locked` for reproducible builds | `false` |
| `no-default-features` | Build with `--no-default-features` | `false` |
| `rustflags` | Extra RUSTFLAGS for the build | — |

**Example: Static musl build with specific features**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-linux
    target: x86_64-unknown-linux-musl
    no-default-features: 'true'
    features: 'rustls-tls'
    locked: 'true'
```

**Example: Pre-build hook for WASM/frontend projects**

For projects that require a frontend build (WASM, npm, etc.) before `cargo build`:

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-linux
    target: x86_64-unknown-linux-gnu
    pre-build: 'cd frontend && npm install && npm run build'
    archive: 'true'
```

**Example: Package pre-built binary (skip-build)**

For Alpine/container workflows where the build happens in a separate step:

```yaml
# Build in Alpine container
- name: Build in Alpine
  run: |
    cargo build --release --target x86_64-unknown-linux-musl

# Package the pre-built binary
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-linux
    target: x86_64-unknown-linux-musl
    skip-build: 'true'
    binary-path: 'target/x86_64-unknown-linux-musl/release/myapp'
    archive: 'true'
```

### Output Options

Control artifact generation and checksums.

| Input | Description | Default |
|-------|-------------|---------|
| `archive` | Create archive (`.tar.gz` on Linux/macOS, `.zip` on Windows) | `false` |
| `checksum` | Checksum algorithms: `sha256`, `sha512`, `b2` (comma-separated) | `sha256` |
| `include` | Extra files to include in archive (glob patterns, comma-separated) | — |

**Example: Archive with multiple checksums**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-linux
    target: x86_64-unknown-linux-gnu
    archive: 'true'
    checksum: 'sha256,sha512'
    include: 'config/*.toml,docs/*.md'
```

### Changelog Options

For `extract-changelog` and `validate-changelog` commands.

| Input | Description | Default |
|-------|-------------|---------|
| `version` | Version to extract/validate | Auto-detected from git tag |
| `changelog` | Path to CHANGELOG.md | `CHANGELOG.md` |
| `notes-output` | Output file for extracted release notes | `release_notes.md` |

**Note:** When `version` is not provided, it's auto-detected from `GITHUB_REF_NAME` (strips `v` prefix from tags like `v1.2.3`).

**Example: Extract changelog**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: extract-changelog
    # version auto-detected from git tag (e.g., v1.2.3 -> 1.2.3)
```

### Version Validation

For `validate-version` command.

| Input | Description | Default |
|-------|-------------|---------|
| `tag` | Git tag to validate | `GITHUB_REF_NAME` |
| `expected-version` | Expected version to match (required) | — |
| `validate-cargo-toml` | Also verify Cargo.toml version matches tag | `false` |

**Example: Validate version**

```yaml
- uses: michaelklishin/rust-release-action@v1
  with:
    command: validate-version
    expected-version: ${{ vars.NEXT_RELEASE_VERSION }}
```

**Example: Validate version with Cargo.toml check**

```yaml
- uses: michaelklishin/rust-release-action@v1
  with:
    command: validate-version
    expected-version: ${{ vars.NEXT_RELEASE_VERSION }}
    validate-cargo-toml: 'true'
```

### Changelog Validation

For `validate-changelog` command. Fails fast if no changelog entry exists for the release version.

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: validate-changelog
    version: '1.2.3'  # Or omit to auto-detect from git tag
```

### Artifact Collection

For `collect-artifacts` command. Scans a directory, computes checksums, and outputs structured data for Homebrew/Winget generation.

| Input | Description | Default |
|-------|-------------|---------|
| `artifacts-dir` | Directory containing artifacts | `artifacts` |
| `base-url` | Base URL for download links | — |

**Example: Collect artifacts and generate Homebrew formula**

```yaml
- uses: actions/download-artifact@v4
  with:
    path: artifacts

- uses: michaelklishin/rust-release-action@v0
  id: collect
  with:
    command: collect-artifacts
    artifacts-dir: artifacts
    base-url: 'https://github.com/${{ github.repository }}/releases/download/v${{ needs.validate.outputs.version }}'

- uses: michaelklishin/rust-release-action@v0
  with:
    command: generate-homebrew
    version: ${{ needs.validate.outputs.version }}
    brew-macos-arm64-sha256: ${{ steps.collect.outputs.macos_arm64_sha256 }}
    brew-linux-x64-sha256: ${{ steps.collect.outputs.linux_x64_sha256 }}
    # ... URLs constructed from base-url + artifact names
```

Outputs: `collection` (JSON), `checksums_file`, `macos_arm64_sha256`, `macos_x64_sha256`, `linux_arm64_sha256`, `linux_x64_sha256`, `windows_x64_sha256`, `windows_arm64_sha256`

### Package Metadata (`pkg-*`)

Shared metadata for Linux packages (deb/rpm/apk), Homebrew, AUR, and Winget.

| Input | Description | Default |
|-------|-------------|---------|
| `pkg-description` | Package description | — |
| `pkg-maintainer` | Maintainer (`Name <email>`) | — |
| `pkg-homepage` | Project homepage URL | — |
| `pkg-license` | License identifier (e.g., `MIT`, `Apache-2.0`) | — |
| `pkg-vendor` | Vendor/organization name | — |
| `pkg-depends` | Runtime dependencies (comma-separated) | — |
| `pkg-recommends` | Recommended packages (comma-separated) | — |
| `pkg-suggests` | Suggested packages (comma-separated) | — |
| `pkg-conflicts` | Conflicting packages (comma-separated) | — |
| `pkg-replaces` | Packages this replaces (comma-separated) | — |
| `pkg-provides` | Virtual packages provided (comma-separated) | — |
| `pkg-contents` | Extra files (`src:dst,src:dst`) | — |
| `pkg-section` | Debian section | `utils` |
| `pkg-priority` | Debian priority | `optional` |
| `pkg-group` | RPM group | `Applications/System` |
| `pkg-release` | Package release/revision number | — |

**Example: Debian package with dependencies**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: release-linux-deb
    target: x86_64-unknown-linux-gnu
    pkg-maintainer: 'Your Name <you@example.com>'
    pkg-description: 'A CLI tool for doing things'
    pkg-homepage: 'https://github.com/you/project'
    pkg-license: 'MIT'
    pkg-depends: 'libc6,libssl3'
```

### SBOM Options (`sbom-*`)

For `generate-sbom` command.

| Input | Description | Default |
|-------|-------------|---------|
| `sbom-format` | Formats: `spdx`, `cyclonedx`, or both | `spdx,cyclonedx` |
| `sbom-dir` | Output directory | `target/sbom` |

**Example: Generate SBOMs**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: generate-sbom
    sbom-format: 'spdx,cyclonedx'
```

### Homebrew Options (`brew-*`)

For `generate-homebrew` command. SHA256 values come from build step outputs.

| Input | Description | Default |
|-------|-------------|---------|
| `brew-class` | Ruby class name for formula | Auto-generated |
| `brew-macos-arm64-url` | macOS ARM64 artifact URL | — |
| `brew-macos-arm64-sha256` | macOS ARM64 SHA256 | — |
| `brew-macos-x64-url` | macOS x64 artifact URL | — |
| `brew-macos-x64-sha256` | macOS x64 SHA256 | — |
| `brew-linux-arm64-url` | Linux ARM64 artifact URL | — |
| `brew-linux-arm64-sha256` | Linux ARM64 SHA256 | — |
| `brew-linux-x64-url` | Linux x64 artifact URL | — |
| `brew-linux-x64-sha256` | Linux x64 SHA256 | — |
| `brew-dir` | Output directory | `target/homebrew` |

**Example: Generate Homebrew formula**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: generate-homebrew
    version: ${{ needs.build.outputs.version }}
    brew-macos-arm64-url: 'https://github.com/you/proj/releases/download/v1.0.0/proj-1.0.0-aarch64-apple-darwin.tar.gz'
    brew-macos-arm64-sha256: ${{ needs.build-macos-arm64.outputs.sha256 }}
    brew-linux-x64-url: 'https://github.com/you/proj/releases/download/v1.0.0/proj-1.0.0-x86_64-unknown-linux-gnu.tar.gz'
    brew-linux-x64-sha256: ${{ needs.build-linux-x64.outputs.sha256 }}
```

### Signing Options

For `sign-artifact` command.

| Input | Description | Default |
|-------|-------------|---------|
| `artifact` | Path to artifact for Sigstore signing | — |

**Example: Sign artifact**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: sign-artifact
    artifact: 'target/release/myapp-1.0.0.tar.gz'
```

### Release Body Options

For `format-release` command.

| Input | Description | Default |
|-------|-------------|---------|
| `artifacts-dir` | Directory containing release artifacts | `release` |
| `notes-file` | Release notes file to include | `release_notes.md` |
| `include-checksums` | Include checksums section | `true` |
| `include-signatures` | Include signatures section | `true` |
| `homebrew-tap` | Homebrew tap for installation instructions | — |
| `aur-package` | AUR package name for installation instructions | — |
| `winget-id` | Winget package ID for installation instructions | — |

**Example: Format release body with installation instructions**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: format-release
    version: '1.0.0'
    artifacts-dir: 'release'
    homebrew-tap: 'you/tap/mytool'
    aur-package: 'mytool'
    winget-id: 'You.MyTool'
```

### AUR Options (`aur-*`)

For `generate-aur` command.

| Input | Description | Default |
|-------|-------------|---------|
| `aur-name` | AUR package name | Binary name |
| `aur-maintainer` | Maintainer (`Name <email>`) | — |
| `aur-source-url` | Source tarball URL | — |
| `aur-source-sha256` | Source SHA256 | — |
| `aur-makedepends` | Build dependencies (comma-separated) | `cargo` |
| `aur-optdepends` | Optional dependencies | — |
| `aur-dir` | Output directory | `target/aur` |

**Example: Generate AUR PKGBUILD**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: generate-aur
    version: '1.0.0'
    aur-maintainer: 'Your Name <you@example.com>'
    aur-source-url: 'https://github.com/you/project/archive/refs/tags/v1.0.0.tar.gz'
    aur-source-sha256: ${{ steps.source.outputs.sha256 }}
    pkg-description: 'A CLI tool'
    pkg-license: 'MIT'
```

### Winget Options (`winget-*`)

For `generate-winget` command.

| Input | Description | Default |
|-------|-------------|---------|
| `winget-publisher` | Publisher display name | — |
| `winget-publisher-id` | Publisher ID | Publisher without spaces |
| `winget-package-id` | Package ID | Binary name |
| `winget-license-url` | License URL | — |
| `winget-copyright` | Copyright notice | — |
| `winget-tags` | Package tags (comma-separated) | — |
| `winget-x64-url` | Windows x64 artifact URL | — |
| `winget-x64-sha256` | Windows x64 SHA256 | — |
| `winget-arm64-url` | Windows ARM64 artifact URL | — |
| `winget-arm64-sha256` | Windows ARM64 SHA256 | — |
| `winget-dir` | Output directory | `target/winget` |

**Example: Generate Winget manifest**

```yaml
- uses: michaelklishin/rust-release-action@v0
  with:
    command: generate-winget
    version: '1.0.0'
    winget-publisher: 'Your Name'
    winget-x64-url: 'https://github.com/you/project/releases/download/v1.0.0/project-1.0.0-x86_64-pc-windows-msvc.zip'
    winget-x64-sha256: ${{ needs.build-windows.outputs.sha256 }}
    pkg-description: 'A CLI tool'
    pkg-license: 'MIT'
```

---

## Commands

| Command | Description |
|---------|-------------|
| `release` | Unified build command (auto-selects platform from target triple) |
| `extract-changelog` | Extract release notes from CHANGELOG.md |
| `validate-changelog` | Validate changelog has entry for version |
| `validate-version` | Validate git tag matches expected version (optionally checks Cargo.toml) |
| `get-version` | Get version from Cargo.toml |
| `collect-artifacts` | Collect artifacts, compute checksums, generate SHA256SUMS |
| `generate-sbom` | Generate SPDX and CycloneDX SBOMs |
| `generate-homebrew` | Generate Homebrew formula |
| `generate-aur` | Generate AUR PKGBUILD and .SRCINFO |
| `generate-winget` | Generate Winget manifest files |
| `sign-artifact` | Sign artifact with Sigstore/cosign |
| `format-release` | Format GitHub Release body |
| `release-linux` | Build Linux binary or tarball |
| `release-linux-deb` | Build Debian package |
| `release-linux-rpm` | Build RPM package |
| `release-linux-apk` | Build Alpine APK package |
| `release-macos` | Build macOS binary or tarball |
| `release-macos-dmg` | Build macOS DMG installer |
| `release-windows` | Build Windows binary or zip |
| `release-windows-msi` | Build Windows MSI installer |

**Note:** The unified `release` command auto-detects the platform from the `target` input:
- Targets containing `linux` use `release-linux`
- Targets containing `darwin` or `apple` use `release-macos`
- Targets containing `windows` use `release-windows`

This simplifies matrix builds by eliminating the need for per-platform command mapping.

---

## Outputs

| Output | Description |
|--------|-------------|
| `version` | Version from get-version, validate-version, or release commands |
| `release_notes_file` | Path to release notes file |
| `release_notes` | Release notes content |
| `artifact` | Artifact filename (archive when archive=true, bare binary otherwise) |
| `artifact_path` | Full path to artifact (archive when archive=true, bare binary otherwise) |
| `bare_artifact` | Bare binary filename (always produced) |
| `bare_artifact_path` | Full path to bare binary (always produced) |
| `binary_name` | Binary name that was built |
| `binary_path` | Path to raw binary (before archiving) |
| `target` | Target triple used for the build |
| `sha256` | SHA256 checksum |
| `sha512` | SHA512 checksum |
| `b2` | BLAKE2 checksum |
| `summary` | JSON build summary |
| `sbom_spdx` | Path to SPDX SBOM file |
| `sbom_cyclonedx` | Path to CycloneDX SBOM file |
| `formula_file` | Path to Homebrew formula |
| `formula_class` | Homebrew formula class name |
| `formula` | Homebrew formula content |
| `signature_path` | Path to signature file |
| `certificate_path` | Path to signing certificate |
| `bundle_path` | Path to Sigstore bundle |
| `body` | Formatted release body |
| `pkgbuild_path` | Path to AUR PKGBUILD |
| `srcinfo_path` | Path to AUR .SRCINFO |
| `pkgbuild` | PKGBUILD content |
| `manifest_dir` | Winget manifest directory |
| `manifest_id` | Winget manifest ID |

---

## Complete Workflow Example

```yaml
name: Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+*'

permissions:
  contents: write

jobs:
  validate:
    runs-on: ubuntu-22.04
    outputs:
      version: ${{ steps.validate.outputs.version }}
    steps:
      - uses: actions/checkout@v4
      - uses: michaelklishin/rust-release-action@v0
        id: validate
        with:
          command: validate-version

  build:
    needs: validate
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-22.04
            command: release-linux
          - target: aarch64-apple-darwin
            os: macos-14
            command: release-macos
          - target: x86_64-pc-windows-msvc
            os: windows-2022
            command: release-windows
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - run: rustup toolchain install stable --profile minimal
      - uses: michaelklishin/rust-release-action@v0
        id: build
        with:
          command: ${{ matrix.command }}
          target: ${{ matrix.target }}
          locked: 'true'
          archive: 'true'
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.target }}
          path: target/${{ matrix.target }}/release/*

  release:
    needs: [validate, build]
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4
      - uses: michaelklishin/rust-release-action@v0
        with:
          command: extract-changelog
          version: ${{ needs.validate.outputs.version }}
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
      - run: |
          mkdir -p release
          find artifacts -type f -name '*${{ needs.validate.outputs.version }}*' -exec cp {} release/ \;
      - uses: softprops/action-gh-release@v2
        with:
          body_path: release_notes.md
          files: release/*
```

---

## License

Licensed under either of:

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
 * MIT license ([LICENSE-MIT](LICENSE-MIT))
