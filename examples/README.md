# Examples

Workflow examples for rust-build-package-release-action.

## Workflows

| File | Description |
|------|-------------|
| [basic-release.yml](basic-release.yml) | Single-platform Linux release |
| [multi-platform.yml](multi-platform.yml) | Linux, macOS, and Windows builds |
| [linux-packages.yml](linux-packages.yml) | Debian, RPM, and Alpine packages |
| [installers.yml](installers.yml) | macOS DMG and Windows MSI |
| [package-managers.yml](package-managers.yml) | Homebrew, AUR, and Winget manifests |
| [macos-multi-arch.yml](macos-multi-arch.yml) | Intel and Apple Silicon macOS builds with Homebrew formula |
| [publish-crate-only.yml](publish-crate-only.yml) | Publish to crates.io via Trusted Publishing |
| [trusted-publishing.yml](trusted-publishing.yml) | Full release with crates.io Trusted Publishing |
| [supply_chain_security.yml](supply_chain_security.yml) | SBOM and Sigstore signing |
| [verify-artifacts.yml](verify-artifacts.yml) | Test packages across distributions |
| [build-with-verification.yml](build-with-verification.yml) | Build and test in one workflow |
| [complete.yml](complete.yml) | Everything combined |

## Usage

Copy a workflow to your `.github/workflows/` directory:

```bash
cp examples/basic-release.yml .github/workflows/release.yml
```

Before using:

 * Replace `michaelklishin/rust-build-package-release-action@v3` with a pinned commit SHA for supply-chain safety
 * Update package metadata (`pkg-maintainer`, `pkg-description`, etc.)
 * Adjust target triples for your platforms
