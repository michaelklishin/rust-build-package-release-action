#!/usr/bin/env nu

# Downloads artifacts from a GitHub release

use common.nu [error]
use test-common.nu [verify-checksum]

# Downloads an artifact from GitHub releases
export def download-artifact [
    binary_name: string,
    version: string,
    arch: string,
    format: string
]: nothing -> string {
    let repo = $env.GITHUB_REPOSITORY? | default ""
    if $repo == "" {
        error "GITHUB_REPOSITORY not set"
    }

    let artifact_name = match $format {
        "deb" => $"($binary_name)_($version)_($arch).deb"
        "rpm" => $"($binary_name)-($version)-1.($arch).rpm"
        "windows-zip" => $"($binary_name)-($version)-x86_64-pc-windows-msvc.zip"
        "windows-msi" => $"($binary_name)-($version)-x86_64-pc-windows-msvc.msi"
        _ => { error $"unknown format: ($format)" }
    }

    let base_url = $"https://github.com/($repo)/releases/download/v($version)"
    let artifact_url = $"($base_url)/($artifact_name)"

    print $"(ansi green)Downloading artifact:(ansi reset) ($artifact_name)"
    download-file $artifact_url $artifact_name

    # Try to download checksum (supports sha256, sha512, b2)
    let checksum_result = try-download-checksum $base_url $artifact_name

    if $checksum_result.found {
        verify-checksum $artifact_name $checksum_result.file
    } else {
        print $"(ansi yellow)  No checksum file available(ansi reset)"
    }

    $artifact_name
}

# Tries to install curl if missing (best effort, warns on failure)
def ensure-curl [] {
    if (which curl | length) > 0 { return }
    print "  curl not found, attempting to install..."
    # Try apt-get (Debian/Ubuntu)
    let apt = do { apt-get update -qq } | complete
    if $apt.exit_code == 0 {
        do { apt-get install -y -qq curl } | complete
    }
    # Try dnf if curl still missing (Fedora/RHEL)
    if (which curl | length) == 0 {
        do { dnf install -y -q curl } | complete
    }
    if (which curl | length) == 0 {
        print $"(ansi yellow)  Warning: could not install curl, falling back to http module(ansi reset)"
    }
}

# Downloads via http module (fallback when curl unavailable)
def http-download [url: string, output: string]: nothing -> record<exit_code: int, stderr: string> {
    let gh_token = $env.GITHUB_TOKEN? | default ($env.GH_TOKEN? | default "")
    try {
        if $gh_token != "" {
            http get --raw --headers { Authorization: $"Bearer ($gh_token)" } $url | save -f $output
        } else {
            http get --raw $url | save -f $output
        }
        { exit_code: 0, stderr: "" }
    } catch {|e|
        { exit_code: 1, stderr: ($e.msg? | default "http request failed") }
    }
}

# Downloads a file via curl, falls back to http module
def curl-download [url: string, output: string]: nothing -> record<exit_code: int, stderr: string> {
    ensure-curl
    if (which curl | length) == 0 {
        return (http-download $url $output)
    }
    let gh_token = $env.GITHUB_TOKEN? | default ($env.GH_TOKEN? | default "")
    if $gh_token != "" {
        do { curl -fsSL -H $"Authorization: Bearer ($gh_token)" $url -o $output } | complete
    } else {
        do { curl -fsSL $url -o $output } | complete
    }
}

# Downloads a file using curl
def download-file [url: string, output: string] {
    let result = curl-download $url $output
    if $result.exit_code != 0 {
        error $"failed to download ($url): ($result.stderr)"
    }
}

# Tries to download a file, returns true if successful
def try-download-file [url: string, output: string]: nothing -> bool {
    (curl-download $url $output).exit_code == 0
}

# Tries to download checksum in multiple formats (sha256, sha512, b2)
def try-download-checksum [base_url: string, artifact_name: string]: nothing -> record<found: bool, file: string> {
    let checksum_exts = ["sha256" "sha512" "b2"]
    for ext in $checksum_exts {
        let checksum_file = $"($artifact_name).($ext)"
        let checksum_url = $"($base_url)/($checksum_file)"
        if (try-download-file $checksum_url $checksum_file) {
            return { found: true, file: $checksum_file }
        }
    }
    { found: false, file: "" }
}

# Downloads Windows artifacts (both zip and msi)
export def download-windows-artifacts [
    binary_name: string,
    version: string
]: nothing -> record<zip: string, msi: string, binary: string> {
    let repo = $env.GITHUB_REPOSITORY? | default ""
    if $repo == "" {
        error "GITHUB_REPOSITORY not set"
    }

    let base_url = $"https://github.com/($repo)/releases/download/v($version)"
    let zip_name = $"($binary_name)-($version)-x86_64-pc-windows-msvc.zip"
    let msi_name = $"($binary_name)-($version)-x86_64-pc-windows-msvc.msi"

    print $"(ansi green)Downloading Windows artifacts(ansi reset)"

    # Download zip
    print $"  Downloading ($zip_name)"
    download-file $"($base_url)/($zip_name)" $zip_name

    # Try checksum for zip (supports sha256, sha512, b2)
    let zip_checksum = try-download-checksum $base_url $zip_name
    if $zip_checksum.found {
        verify-checksum $zip_name $zip_checksum.file
    } else {
        print $"(ansi yellow)  No checksum file available(ansi reset)"
    }

    # Download MSI
    print $"  Downloading ($msi_name)"
    download-file $"($base_url)/($msi_name)" $msi_name

    # Try checksum for MSI (supports sha256, sha512, b2)
    let msi_checksum = try-download-checksum $base_url $msi_name
    if $msi_checksum.found {
        verify-checksum $msi_name $msi_checksum.file
    } else {
        print $"(ansi yellow)  No MSI checksum file available(ansi reset)"
    }

    # Extract zip (use PowerShell on Windows, unzip elsewhere)
    print "  Extracting archive"
    let is_windows = (sys host | get name) == "Windows"
    let result = if $is_windows {
        do { powershell -Command $"Expand-Archive -Path '($zip_name)' -DestinationPath 'extracted' -Force" } | complete
    } else {
        do { unzip -q $zip_name -d extracted } | complete
    }
    if $result.exit_code != 0 {
        error $"failed to extract archive: ($result.stderr)"
    }

    let binary_path = $"extracted/($binary_name).exe"
    if not ($binary_path | path exists) {
        error $"binary not found in archive: ($binary_path)"
    }

    { zip: $zip_name, msi: $msi_name, binary: $binary_path }
}
