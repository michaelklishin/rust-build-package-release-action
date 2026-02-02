#!/usr/bin/env nu

use common.nu [get-cargo-info, output, copy-docs, copy-includes, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, list-archivable-files, output-build-results, install-linux-cross-deps, run-pre-build-hook]

def main [] {
    let skip_build = $env.SKIP_BUILD? | default "" | $in == "true"
    let custom_binary_path = $env.BINARY_PATH? | default ""

    if not $skip_build {
        check-rust-toolchain
    }

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

    let release_dir = $"target/($target)/release"

    if $skip_build {
        if $custom_binary_path == "" {
            error "binary-path is required when skip-build is true"
        }
        if not ($custom_binary_path | path exists) {
            error $"binary not found: ($custom_binary_path)"
        }
        print $"(ansi green)Packaging(ansi reset) ($binary_name) v($version) for ($target) (skip-build)"
        mkdir $release_dir
        cp $custom_binary_path $"($release_dir)/($binary_name)"
        chmod +x $"($release_dir)/($binary_name)"
    } else {
        print $"(ansi green)Building(ansi reset) ($binary_name) v($version) for ($target)"
        rm -rf $release_dir
        mkdir $release_dir
        ensure-lockfile
        run-pre-build-hook
        install-linux-cross-deps $target
        cargo-build $target $binary_name
    }

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

    # Always create the bare binary artifact
    let bare_artifact = $artifact_base
    let bare_artifact_path = $"($release_dir)/($bare_artifact)"
    cp $binary_path $bare_artifact_path
    chmod +x $bare_artifact_path

    # Output bare binary info
    output "bare_artifact" $bare_artifact
    output "bare_artifact_path" $bare_artifact_path

    if $create_archive {
        let artifact = $"($artifact_base).tar.gz"
        let artifact_path = $"($release_dir)/($artifact)"
        print $"(ansi green)Creating archive:(ansi reset) ($artifact)"
        let files = list-archivable-files $release_dir
        tar -C $release_dir -czf $artifact_path ...$files

        # Generate checksums for both bare binary and archive
        generate-checksums $bare_artifact_path
        let checksums = generate-checksums $artifact_path
        print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
        hr-line
        ls $release_dir | print
        print $"(ansi green)Created:(ansi reset) ($bare_artifact), ($artifact)"
        output-build-results $binary_name $version $target $artifact $artifact_path $checksums
    } else {
        let checksums = generate-checksums $bare_artifact_path
        print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
        hr-line
        ls $release_dir | print
        print $"(ansi green)Created:(ansi reset) ($bare_artifact)"
        output-build-results $binary_name $version $target $bare_artifact $bare_artifact_path $checksums
    }
}
