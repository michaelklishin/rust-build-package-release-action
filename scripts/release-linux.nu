#!/usr/bin/env nu

use common.nu [get-cargo-info, output, copy-docs, copy-includes, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, list-archivable-files, output-build-results, install-linux-cross-deps]

def main [] {
    check-rust-toolchain

    let target = $env.TARGET? | default "x86_64-unknown-linux-gnu"
    let info = get-cargo-info
    let binary_name = $env.BINARY_NAME? | default $info.name
    let version = $info.version
    let create_archive = $env.ARCHIVE? | default "" | $in == "true"

    if $binary_name == "" {
        error "could not determine binary name"
    }
    if $version == "" {
        error "could not determine version"
    }

    print $"(ansi green)Building(ansi reset) ($binary_name) v($version) for ($target)"

    let release_dir = $"target/($target)/release"
    rm -rf $release_dir
    mkdir $release_dir

    ensure-lockfile
    install-linux-cross-deps $target
    cargo-build $target $binary_name

    let binary_path = $"($release_dir)/($binary_name)"
    if not ($binary_path | path exists) {
        error $"binary not found: ($binary_path)"
    }

    copy-docs $release_dir
    copy-includes $release_dir

    let artifact_base = $"($binary_name)-($version)-($target)"

    output "version" $version
    output "binary_name" $binary_name
    output "target" $target
    output "binary_path" $binary_path

    if $create_archive {
        let artifact = $"($artifact_base).tar.gz"
        let artifact_path = $"($release_dir)/($artifact)"
        chmod +x $binary_path
        print $"(ansi green)Creating archive:(ansi reset) ($artifact)"
        let files = list-archivable-files $release_dir
        tar -C $release_dir -czf $artifact_path ...$files

        let checksums = generate-checksums $artifact_path
        print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
        hr-line
        ls $release_dir | print
        print $"(ansi green)Created:(ansi reset) ($artifact)"
        output-build-results $binary_name $version $target $artifact $artifact_path $checksums
    } else {
        let artifact = $artifact_base
        let artifact_path = $"($release_dir)/($artifact)"
        cp $binary_path $artifact_path
        chmod +x $artifact_path

        let checksums = generate-checksums $artifact_path
        print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
        hr-line
        ls $release_dir | print
        print $"(ansi green)Created:(ansi reset) ($artifact)"
        output-build-results $binary_name $version $target $artifact $artifact_path $checksums
    }
}
