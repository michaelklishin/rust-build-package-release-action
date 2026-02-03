#!/usr/bin/env nu

use common.nu [get-cargo-info, output, copy-docs, copy-includes, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, output-build-results, run-pre-build-hook]

# Gets the manifest path for a package in a workspace
def get-package-manifest-path [package_name: string]: nothing -> string {
    let metadata = cargo metadata --format-version 1 --no-deps | from json
    let pkg = $metadata.packages | where name == $package_name | first
    $pkg.manifest_path
}

# Finds WiX source files, checking package directory for workspaces
def find-wix-files [package_name: string]: nothing -> list<string> {
    # First check root wix/ directory
    let root_files = glob "wix/*.wxs"
    if ($root_files | is-not-empty) {
        return $root_files
    }

    # For workspace packages, check the package's directory
    if $package_name != "" {
        let manifest_path = get-package-manifest-path $package_name
        let pkg_dir = $manifest_path | path dirname
        let pkg_pattern = $"($pkg_dir)/wix/*.wxs"
        glob $pkg_pattern
    } else {
        []
    }
}

# Checks that WiX Toolset is available, installs if missing
def check-wix-toolset [] {
    # Check if candle.exe (WiX compiler) is in PATH or WIX env var is set
    if (which candle | is-not-empty) {
        return
    }
    if ($env.WIX? | default "" | $in != "") {
        return
    }

    print $"(ansi yellow)WiX Toolset not found, installing via winget...(ansi reset)"

    # Try winget first (available on Windows Server 2022 runners)
    let result = do { winget install --id WiXToolset.WiX --version 3.14.1 --accept-source-agreements --accept-package-agreements } | complete
    if $result.exit_code == 0 {
        # Add WiX to PATH
        let wix_path = "C:\\Program Files (x86)\\WiX Toolset v3.14\\bin"
        if ($wix_path | path exists) {
            $env.PATH = ($env.PATH | prepend $wix_path)
            $env.WIX = "C:\\Program Files (x86)\\WiX Toolset v3.14"
            return
        }
    }

    # Fallback to chocolatey
    print $"(ansi yellow)Trying chocolatey...(ansi reset)"
    let result = do { choco install wixtoolset -y --version 3.14.1 } | complete
    if $result.exit_code != 0 {
        error "Failed to install WiX Toolset"
    }

    # Add WiX to PATH
    let wix_path = "C:\\Program Files (x86)\\WiX Toolset v3.14\\bin"
    if ($wix_path | path exists) {
        $env.PATH = ($env.PATH | prepend $wix_path)
        $env.WIX = "C:\\Program Files (x86)\\WiX Toolset v3.14"
    } else {
        error "WiX Toolset installed but bin directory not found"
    }
}

def main [] {
    let skip_build = $env.SKIP_BUILD? | default "" | $in == "true"
    let custom_binary_path = $env.BINARY_PATH? | default ""

    if not $skip_build {
        check-rust-toolchain
    }

    let target = $env.TARGET? | default "x86_64-pc-windows-msvc"
    let info = get-cargo-info
    let package_name = $env.PACKAGE? | default $info.name
    let binary_name = $env.BINARY_NAME? | default $info.name
    let version = $info.version

    if $binary_name == "" {
        error "could not determine binary name"
    }
    if $version == "" {
        error "could not determine version"
    }

    # Check for WiX source files early, before expensive operations
    let wxs_files = find-wix-files $package_name
    if ($wxs_files | is-empty) {
        error "No WiX source files found. Run 'cargo wix init' to generate them, or create wix/*.wxs files manually."
    }

    print $"(ansi green)Building(ansi reset) ($binary_name) v($version) MSI for ($target)"

    let release_dir = $"target/($target)/release"

    if $skip_build {
        if $custom_binary_path == "" {
            error "binary-path is required when skip-build is true"
        }
        if not ($custom_binary_path | path exists) {
            error $"binary not found: ($custom_binary_path)"
        }
        mkdir $release_dir
        cp $custom_binary_path $"($release_dir)/($binary_name).exe"
    } else {
        rm -rf $release_dir
        mkdir $release_dir
        ensure-lockfile
        run-pre-build-hook
        rustup target add $target
        cargo-build $target $binary_name
    }

    let binary_path = $"($release_dir)/($binary_name).exe"
    if not ($binary_path | path exists) {
        error $"binary not found: ($binary_path)"
    }

    copy-docs $release_dir
    copy-includes $release_dir

    # Copy binaries to target/release for cargo-wix
    cp -r ($"($release_dir)/*" | into glob) target/release/

    # Ensure WiX Toolset is installed
    check-wix-toolset

    if (which cargo-wix | is-empty) {
        print $"(ansi yellow)Installing cargo-wix...(ansi reset)"
        cargo install cargo-wix --version 0.3.8
    }

    let msi_path = $"target/wix/($binary_name)-($version)-($target).msi"
    print $"(ansi green)Creating MSI package...(ansi reset)"
    cargo wix --no-build --nocapture --package $package_name --output $msi_path

    if not ($msi_path | path exists) {
        error $"MSI not created: ($msi_path)"
    }

    let checksums = generate-checksums $msi_path

    let artifact_path = $msi_path | str replace --all '\' '/'
    let artifact = $artifact_path | path basename

    output "version" $version
    output "binary_name" $binary_name
    output "target" $target
    output "binary_path" ($binary_path | str replace --all '\' '/')

    print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
    hr-line
    ls target/wix | print
    print $"(ansi green)Created:(ansi reset) ($artifact)"
    output-build-results $binary_name $version $target $artifact $artifact_path $checksums
}
