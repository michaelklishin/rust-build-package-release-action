#!/usr/bin/env nu

use common.nu [output, output-multiline, hr-line, error]

def main [] {
    let version = $env.VERSION? | default ""
    if $version == "" {
        error "VERSION is required"
    }

    let artifacts_dir = $env.ARTIFACTS_DIR? | default "release"
    let release_notes_file = $env.RELEASE_NOTES_FILE? | default "release_notes.md"
    let include_checksums = ($env.INCLUDE_CHECKSUMS? | default "true") == "true"
    let include_signatures = ($env.INCLUDE_SIGNATURES? | default "true") == "true"

    # Optional package manager info
    let homebrew_tap = $env.HOMEBREW_TAP? | default ""
    let aur_package = $env.AUR_PACKAGE? | default ""
    let winget_id = $env.WINGET_ID? | default ""

    print $"(ansi green)Formatting release:(ansi reset) v($version)"

    mut body = ""

    # 1. Release notes from changelog (rabbitmqadmin-ng format)
    if ($release_notes_file | path exists) {
        let notes = open $release_notes_file | str trim
        if $notes != "" {
            $body = $notes + "\n\n"
        }
    }

    # 2. Installation section (package managers)
    let install_section = format-installation-section $homebrew_tap $aur_package $winget_id
    if $install_section != "" {
        $body = $body + $install_section
    }

    # 3. Downloads table
    if ($artifacts_dir | path exists) {
        let artifacts = list-release-artifacts $artifacts_dir
        if ($artifacts | is-not-empty) {
            $body = $body + "## Downloads\n\n"
            $body = $body + (format-artifacts-table $artifacts)
        }

        # 4. SBOM section
        let sbom_section = format-sbom-section $artifacts_dir
        if $sbom_section != "" {
            $body = $body + $sbom_section
        }

        # 5. Checksums section
        if $include_checksums {
            let checksums = collect-checksums $artifacts_dir
            if $checksums != "" {
                $body = $body + "\n## Checksums\n\n"
                $body = $body + "```\n" + $checksums + "```\n"
            }
        }

        # 6. Signatures section
        if $include_signatures {
            let sig_pattern = '.sig$|.pem$|.sigstore.json$'
            let has_sigs = ls $artifacts_dir | where { |f| $f.name =~ $sig_pattern } | is-not-empty
            if $has_sigs {
                $body = $body + "\n## Signatures\n\n"
                $body = $body + "All release artifacts are signed with [Sigstore](https://www.sigstore.dev/). "
                $body = $body + "Verify with:\n\n"
                $body = $body + "```bash\n"
                $body = $body + "cosign verify-blob --bundle <artifact>.sigstore.json <artifact>\n"
                $body = $body + "```\n"
            }
        }
    }

    print $"(char nl)(ansi green)Release body:(ansi reset)"
    hr-line
    print $body
    hr-line

    output "version" $version
    output-multiline "body" $body
}

# Formats installation instructions for package managers
def format-installation-section [homebrew_tap: string, aur_package: string, winget_id: string]: nothing -> string {
    mut has_any = false
    mut section = "## Installation\n\n"

    if $homebrew_tap != "" {
        $has_any = true
        $section = $section + "**Homebrew:**\n```bash\n"
        $section = $section + $"brew install ($homebrew_tap)\n"
        $section = $section + "```\n\n"
    }

    if $aur_package != "" {
        $has_any = true
        $section = $section + "**Arch Linux (AUR):**\n```bash\n"
        $section = $section + $"yay -S ($aur_package)\n"
        $section = $section + "```\n\n"
    }

    if $winget_id != "" {
        $has_any = true
        $section = $section + "**Windows (winget):**\n```powershell\n"
        $section = $section + $"winget install ($winget_id)\n"
        $section = $section + "```\n\n"
    }

    if $has_any { $section } else { "" }
}

# Formats SBOM files section
def format-sbom-section [dir: string]: nothing -> string {
    let sbom_pattern = '.spdx.json$|.cdx.json$'
    let sbom_files = ls $dir | where { |f| $f.name =~ $sbom_pattern }

    if ($sbom_files | is-empty) {
        return ""
    }

    mut section = "\n## SBOM\n\n"

    for file in $sbom_files {
        let name = $file.name | path basename
        if $name =~ ".spdx.json$" {
            $section = $section + $"- `($name)` - SPDX format\n"
        } else if $name =~ ".cdx.json$" {
            $section = $section + $"- `($name)` - CycloneDX format\n"
        }
    }

    $section + "\n"
}

# Lists release artifacts (excludes checksums, signatures, SBOM, and metadata)
def list-release-artifacts [dir: string]: nothing -> table {
    let exclude_pattern = '.sha256$|.sha512$|.b2$|.sig$|.pem$|.sigstore.json$|.spdx.json$|.cdx.json$'
    ls $dir
        | where type == file
        | where { |f| not ($f.name =~ $exclude_pattern) }
        | each {|f|
            let name = $f.name | path basename
            let size = format-size $f.size
            let platform = detect-platform $name
            { name: $name, size: $size, platform: $platform }
        }
}

# Formats file size in human-readable format
export def format-size [bytes]: nothing -> string {
    let bytes = $bytes | into int
    if $bytes < 1024 { return $"($bytes) B" }
    let kb = $bytes / 1024
    if $kb < 1024 { return $"($kb | math round -p 1) KB" }
    let mb = $kb / 1024
    $"($mb | math round -p 1) MB"
}

# Detects platform from artifact name
export def detect-platform [name: string]: nothing -> string {
    if $name =~ "darwin|macos|osx" {
        if $name =~ "arm64|aarch64" { "macOS (Apple Silicon)" } else { "macOS (Intel)" }
    } else if $name =~ "windows|win" {
        if $name =~ "arm64|aarch64" { "Windows (ARM64)" } else { "Windows (x64)" }
    } else if $name =~ "linux" {
        if $name =~ "musl" {
            if $name =~ "arm64|aarch64" { "Linux (ARM64, musl)" } else { "Linux (x64, musl)" }
        } else {
            if $name =~ "arm64|aarch64" { "Linux (ARM64)" } else if $name =~ "armv7" { "Linux (ARMv7)" } else { "Linux (x64)" }
        }
    } else if $name =~ ".deb$" {
        "Debian/Ubuntu"
    } else if $name =~ ".rpm$" {
        "RHEL/Fedora"
    } else if $name =~ ".apk$" {
        "Alpine Linux"
    } else if $name =~ ".dmg$" {
        "macOS Installer"
    } else if $name =~ ".msi$" {
        "Windows Installer"
    } else if $name =~ ".pkg.tar.zst$" {
        "Arch Linux"
    } else {
        "Other"
    }
}

# Formats artifacts as a Markdown table
def format-artifacts-table [artifacts: table]: nothing -> string {
    mut table = "| Platform | File | Size |\n"
    $table = $table + "|----------|------|------|\n"

    for artifact in $artifacts {
        $table = $table + $"| ($artifact.platform) | `($artifact.name)` | ($artifact.size) |\n"
    }

    $table
}

# Collects all checksum file contents
def collect-checksums [dir: string]: nothing -> string {
    let checksum_pattern = '.sha256$|.sha512$|.b2$'
    let checksum_files = ls $dir | where { |f| $f.name =~ $checksum_pattern }
    if ($checksum_files | is-empty) {
        return ""
    }
    let checksum_files = $checksum_files | get name

    mut checksums = ""
    for file in $checksum_files {
        let content = open $file | str trim
        $checksums = $checksums + $content + "\n"
    }
    $checksums
}
