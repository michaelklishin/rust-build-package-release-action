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

    print $"(ansi green)Formatting release:(ansi reset) v($version)"

    mut body = ""

    # Include release notes if file exists
    if ($release_notes_file | path exists) {
        let notes = open $release_notes_file | str trim
        if $notes != "" {
            $body = $notes + "\n\n"
        }
    }

    # Build artifacts table
    if ($artifacts_dir | path exists) {
        let artifacts = list-release-artifacts $artifacts_dir
        if ($artifacts | is-not-empty) {
            $body = $body + "## Downloads\n\n"
            $body = $body + (format-artifacts-table $artifacts $include_checksums)
        }

        # Add checksum section
        if $include_checksums {
            let checksums = collect-checksums $artifacts_dir
            if $checksums != "" {
                $body = $body + "\n## Checksums\n\n"
                $body = $body + "```\n" + $checksums + "```\n"
            }
        }

        # Add signature section
        if $include_signatures {
            let sig_pattern = '.sig$|.pem$|.sigstore.json$'
            let has_sigs = ls $artifacts_dir | where { |f| $f.name =~ $sig_pattern } | is-not-empty
            if $has_sigs {
                $body = $body + "\n## Signatures\n\n"
                $body = $body + "All artifacts are signed with [Sigstore](https://www.sigstore.dev/). "
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

# Lists release artifacts (excludes checksums, signatures, and metadata)
def list-release-artifacts [dir: string]: nothing -> table {
    let exclude_pattern = '.sha256$|.sha512$|.b2$|.sig$|.pem$|.sigstore.json$'
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
export def format-size [bytes: int]: nothing -> string {
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
    } else {
        "Other"
    }
}

# Formats artifacts as a Markdown table
def format-artifacts-table [artifacts: table, include_checksums: bool]: nothing -> string {
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
    let checksum_files = ls $dir | where { |f| $f.name =~ $checksum_pattern } | get name
    if ($checksum_files | is-empty) {
        return ""
    }

    mut checksums = ""
    for file in $checksum_files {
        let content = open $file | str trim
        $checksums = $checksums + $content + "\n"
    }
    $checksums
}
