# Examples

Workflow examples for rust-release-action.

## Workflows

| File | Description |
|------|-------------|
| [basic-release.yml](basic-release.yml) | Single-platform Linux release |
| [multi-platform.yml](multi-platform.yml) | Linux, macOS, and Windows builds |
| [linux-packages.yml](linux-packages.yml) | Debian, RPM, and Alpine packages |
| [installers.yml](installers.yml) | macOS DMG and Windows MSI |
| [package-managers.yml](package-managers.yml) | Homebrew, AUR, and Winget manifests |
| [security.yml](security.yml) | SBOM and Sigstore signing |
| [complete.yml](complete.yml) | Everything combined |

## Usage

Copy a workflow to your `.github/workflows/` directory:

```bash
cp examples/basic-release.yml .github/workflows/release.yml
```

Before using:

 * Replace `michaelklishin/rust-release-action@v1` with a pinned version
 * Update package metadata (`pkg-maintainer`, `pkg-description`, etc.)
 * Adjust target triples for your platforms
