#!/usr/bin/env nu

# Collects artifacts from a directory, computes checksums, and outputs structured data
# for use with generate-homebrew and generate-winget commands

use common.nu [output, output-multiline, error]

def main [] {
    let artifacts_dir = $env.ARTIFACTS_DIR? | default "artifacts"
    let base_url = $env.BASE_URL? | default ""
    let binary_name = $env.BINARY_NAME? | default ""

    if not ($artifacts_dir | path exists) {
        error $"artifacts directory not found: ($artifacts_dir)"
    }

    print $"(ansi green)Collecting artifacts from:(ansi reset) ($artifacts_dir)"

    # Find all artifact files (archives and binaries)
    let archive_pattern = '\.(tar\.gz|zip|dmg|msi|deb|rpm|apk)$'
    let artifacts = ls $artifacts_dir
        | where type == file
        | where { |f| $f.name =~ $archive_pattern }
        | get name
        | path basename

    if ($artifacts | is-empty) {
        error $"no artifacts found in ($artifacts_dir)"
    }

    # Collect artifact info with checksums and platform detection
    let collection = $artifacts | each {|artifact|
        let artifact_path = $"($artifacts_dir)/($artifact)"
        let sha256 = open $artifact_path --raw | hash sha256
        let platform = detect-platform $artifact
        let url = if $base_url != "" { $"($base_url)/($artifact)" } else { "" }

        {
            artifact: $artifact
            path: $artifact_path
            sha256: $sha256
            platform: $platform
            url: $url
        }
    }

    print $"(ansi green)Found:(ansi reset) ($collection | length) artifacts"
    $collection | each {|a| print $"  ($a.platform): ($a.artifact)" }

    # Output individual platform checksums for Homebrew/Winget
    let macos_arm64 = $collection | where platform == "macos-arm64" | first?
    let macos_x64 = $collection | where platform == "macos-x64" | first?
    let linux_arm64 = $collection | where platform == "linux-arm64" | first?
    let linux_x64 = $collection | where platform == "linux-x64" | first?
    let windows_x64 = $collection | where platform == "windows-x64" | first?
    let windows_arm64 = $collection | where platform == "windows-arm64" | first?

    if $macos_arm64 != null {
        output "macos_arm64_sha256" $macos_arm64.sha256
        output "macos_arm64_url" $macos_arm64.url
        output "macos_arm64_artifact" $macos_arm64.artifact
    }
    if $macos_x64 != null {
        output "macos_x64_sha256" $macos_x64.sha256
        output "macos_x64_url" $macos_x64.url
        output "macos_x64_artifact" $macos_x64.artifact
    }
    if $linux_arm64 != null {
        output "linux_arm64_sha256" $linux_arm64.sha256
        output "linux_arm64_url" $linux_arm64.url
        output "linux_arm64_artifact" $linux_arm64.artifact
    }
    if $linux_x64 != null {
        output "linux_x64_sha256" $linux_x64.sha256
        output "linux_x64_url" $linux_x64.url
        output "linux_x64_artifact" $linux_x64.artifact
    }
    if $windows_x64 != null {
        output "windows_x64_sha256" $windows_x64.sha256
        output "windows_x64_url" $windows_x64.url
        output "windows_x64_artifact" $windows_x64.artifact
    }
    if $windows_arm64 != null {
        output "windows_arm64_sha256" $windows_arm64.sha256
        output "windows_arm64_url" $windows_arm64.url
        output "windows_arm64_artifact" $windows_arm64.artifact
    }

    # Output full collection as JSON
    output-multiline "collection" ($collection | to json)

    # Generate consolidated checksums file
    let checksums_content = $collection | each {|a|
        $"($a.sha256)  ($a.artifact)"
    } | str join "\n"

    let checksums_path = $"($artifacts_dir)/SHA256SUMS"
    $"($checksums_content)\n" | save -f $checksums_path
    print $"(ansi green)Created:(ansi reset) ($checksums_path)"
    output "checksums_file" $checksums_path
}

# Detect platform from artifact filename
def detect-platform [filename: string]: nothing -> string {
    if ($filename =~ "darwin.*arm64|aarch64.*apple|apple.*aarch64|macos.*arm64") {
        "macos-arm64"
    } else if ($filename =~ "darwin.*x86_64|x86_64.*apple|apple.*x86_64|macos.*x64|macos.*x86_64") {
        "macos-x64"
    } else if ($filename =~ "linux.*aarch64|aarch64.*linux|linux.*arm64") {
        "linux-arm64"
    } else if ($filename =~ "linux.*x86_64|x86_64.*linux|linux.*x64|linux.*amd64") {
        "linux-x64"
    } else if ($filename =~ "windows.*x86_64|x86_64.*windows|windows.*x64|pc-windows.*x86_64") {
        "windows-x64"
    } else if ($filename =~ "windows.*aarch64|aarch64.*windows|windows.*arm64") {
        "windows-arm64"
    } else if ($filename =~ "\.deb$") {
        "linux-deb"
    } else if ($filename =~ "\.rpm$") {
        "linux-rpm"
    } else if ($filename =~ "\.apk$") {
        "linux-apk"
    } else if ($filename =~ "\.dmg$") {
        "macos-dmg"
    } else if ($filename =~ "\.msi$") {
        "windows-msi"
    } else {
        "unknown"
    }
}
