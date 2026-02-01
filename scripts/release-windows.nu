#!/usr/bin/env nu

use common.nu [get-cargo-info, output, copy-docs, copy-includes, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, list-archivable-files, output-build-results]

def main [] {
    check-rust-toolchain

    let target = $env.TARGET? | default "x86_64-pc-windows-msvc"
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
    rustup target add $target
    cargo-build $target $binary_name

    let binary_path = $"($release_dir)/($binary_name).exe"
    if not ($binary_path | path exists) {
        error $"binary not found: ($binary_path)"
    }

    copy-docs $release_dir
    copy-includes $release_dir

    let artifact_base = $"($binary_name)-($version)-($target)"

    output "version" $version
    output "binary_name" $binary_name
    output "target" $target
    output "binary_path" ($binary_path | str replace --all '\' '/')

    if $create_archive {
        let artifact = $"($artifact_base).zip"
        let original_dir = (pwd)
        print $"(ansi green)Creating archive:(ansi reset) ($artifact)"
        cd $release_dir
        let files = list-archivable-files "."
        7z a $artifact ...$files
        cd $original_dir

        let artifact_path = $"($release_dir)/($artifact)"
        let checksums = generate-checksums $artifact_path
        let normalised_path = $artifact_path | str replace --all '\' '/'

        print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
        hr-line
        ls $release_dir | print
        print $"(ansi green)Created:(ansi reset) ($artifact)"
        output-build-results $binary_name $version $target $artifact $normalised_path $checksums
    } else {
        let artifact = $"($artifact_base).exe"
        let artifact_path = $"($release_dir)/($artifact)"
        cp $binary_path $artifact_path

        let checksums = generate-checksums $artifact_path
        let normalised_path = $artifact_path | str replace --all '\' '/'

        print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
        hr-line
        ls $release_dir | print
        print $"(ansi green)Created:(ansi reset) ($artifact)"
        output-build-results $binary_name $version $target $artifact $normalised_path $checksums
    }
}
