#!/usr/bin/env nu

# Tests RPM packages by installing and verifying the binary

use common.nu [error, output]
use test-common.nu [verify-checksum, verify-installed-binary]
use download-release.nu [download-artifact]

def main [] {
    let download_from_release = ($env.DOWNLOAD_FROM_RELEASE? | default "false") == "true"
    let binary_name = $env.BINARY_NAME? | default ""
    let version = $env.VERSION? | default ""
    let arch = $env.ARCH? | default ""

    if $binary_name == "" {
        error "binary-name is required"
    }
    if $version == "" {
        error "version is required"
    }

    let artifact_path = if $download_from_release {
        if $arch == "" {
            error "arch is required when download-from-release is true"
        }
        let valid_archs = ["x86_64" "aarch64" "i686" "armv7hl"]
        if not ($arch in $valid_archs) {
            error $"invalid arch '($arch)': must be one of ($valid_archs | str join ', ')"
        }
        download-artifact $binary_name $version $arch "rpm"
    } else {
        let path = $env.ARTIFACT_PATH? | default ""
        if $path == "" {
            error "artifact is required when download-from-release is false"
        }
        if not ($path | path exists) {
            error $"artifact not found: ($path)"
        }
        # Verify checksum if provided separately
        let checksum_file = $env.CHECKSUM_FILE? | default ""
        if $checksum_file != "" {
            verify-checksum $path $checksum_file
        }
        $path
    }

    print $"(ansi green)Testing RPM package:(ansi reset) ($artifact_path)"
    print $"(ansi green)Expected version:(ansi reset) ($version)"

    install-rpm $artifact_path
    verify-installed-binary $"/usr/bin/($binary_name)" $version
    uninstall-rpm $binary_name

    print $"(ansi green)All tests passed(ansi reset)"
    output "result" "success"
}

def install-rpm [artifact_path: string] {
    print $"(ansi green)Installing package...(ansi reset)"
    let result = do { sudo rpm -i $artifact_path } | complete
    if $result.exit_code != 0 {
        error $"failed to install package: ($result.stderr)"
    }
    print "  Package installed ✓"
}

def uninstall-rpm [binary_name: string] {
    print $"(ansi green)Uninstalling package...(ansi reset)"
    let result = do { sudo rpm -e $binary_name } | complete
    if $result.exit_code != 0 {
        error $"failed to uninstall package: ($result.stderr)"
    }
    print "  Package uninstalled ✓"
}
