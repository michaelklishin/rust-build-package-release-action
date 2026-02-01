#!/usr/bin/env nu

use common.nu [get-cargo-info, output, copy-docs, copy-includes, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, output-build-results]

def main [] {
    check-rust-toolchain

    let target = $env.TARGET? | default "x86_64-pc-windows-msvc"
    let info = get-cargo-info
    let binary_name = $env.BINARY_NAME? | default $info.name
    let version = $info.version

    if $binary_name == "" {
        error "could not determine binary name"
    }
    if $version == "" {
        error "could not determine version"
    }

    print $"(ansi green)Building(ansi reset) ($binary_name) v($version) MSI for ($target)"

    let release_dir = $"target/($target)/release"
    rm -rf $release_dir
    mkdir $release_dir

    ensure-lockfile
    rustup target add $target
    cargo-build $target $binary_name

    let binary_path = $"($release_dir)/($binary_name).exe"
    if not ($binary_path | path exists) {
        error $"binary not found: ($binary_path)"
    }

    copy-docs $release_dir
    copy-includes $release_dir

    # Copy binaries to target/release for cargo-wix
    cp -r ($"($release_dir)/*" | into glob) target/release/

    if (which cargo-wix | is-empty) {
        print $"(ansi yellow)Installing cargo-wix...(ansi reset)"
        cargo install cargo-wix --version 0.3.8
    }

    let msi_path = $"target/wix/($binary_name)-($version)-($target).msi"
    print $"(ansi green)Creating MSI package...(ansi reset)"
    cargo wix --no-build --nocapture --package $binary_name --output $msi_path

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
