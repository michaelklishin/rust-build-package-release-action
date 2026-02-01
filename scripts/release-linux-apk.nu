#!/usr/bin/env nu

use common.nu [get-cargo-info, output, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, output-build-results, check-nfpm, install-linux-cross-deps, nfpm-base-config, nfpm-contents-section, nfpm-dependencies-section]

def main [] {
    check-rust-toolchain
    check-nfpm

    let target = $env.TARGET? | default "x86_64-unknown-linux-musl"
    let info = get-cargo-info
    let binary_name = $env.BINARY_NAME? | default $info.name
    let version = $info.version

    if $binary_name == "" {
        error "could not determine binary name"
    }
    if $version == "" {
        error "could not determine version"
    }

    let arch = target-to-apk-arch $target

    print $"(ansi green)Building .apk package:(ansi reset) ($binary_name) v($version) for ($arch)"

    let release_dir = $"target/($target)/release"
    let binary_path = $"($release_dir)/($binary_name)"

    if not ($binary_path | path exists) {
        print $"(ansi yellow)Binary not found, building...(ansi reset)"
        rm -rf $release_dir
        mkdir $release_dir
        ensure-lockfile
        install-linux-cross-deps $target
        cargo-build $target $binary_name
    }

    if not ($binary_path | path exists) {
        error $"binary not found: ($binary_path)"
    }

    let pkg_dir = "target/pkg-apk"
    rm -rf $pkg_dir
    mkdir $pkg_dir

    let abs_binary_path = $binary_path | path expand
    let nfpm_config = generate-nfpm-config $binary_name $version $arch $abs_binary_path
    let config_path = $"($pkg_dir)/nfpm.yaml"
    $nfpm_config | save -f $config_path

    let release_num = $env.PKG_RELEASE? | default "0"
    let artifact = $"($binary_name)-($version)-r($release_num).apk"
    let artifact_path = $"($release_dir)/($artifact)"

    print $"(ansi green)Running nfpm...(ansi reset)"
    nfpm package --config $config_path --packager apk --target $artifact_path

    if not ($artifact_path | path exists) {
        error $"failed to create package: ($artifact_path)"
    }

    let checksums = generate-checksums $artifact_path
    print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
    hr-line
    ls $release_dir | where name =~ '\.apk' | print
    print $"(ansi green)Created:(ansi reset) ($artifact)"

    output "version" $version
    output "binary_name" $binary_name
    output "target" $target
    output "binary_path" $binary_path
    output-build-results $binary_name $version $target $artifact $artifact_path $checksums
}

def target-to-apk-arch [target: string]: nothing -> string {
    match $target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "x86_64"
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "aarch64"
        "armv7-unknown-linux-gnueabihf" | "armv7-unknown-linux-musleabihf" => "armv7"
        "i686-unknown-linux-gnu" | "i686-unknown-linux-musl" => "x86"
        _ => {
            if $target =~ "x86_64" { "x86_64" }
            else if $target =~ "aarch64" { "aarch64" }
            else if $target =~ "armv7" { "armv7" }
            else if $target =~ "i686" { "x86" }
            else { error $"unsupported target for .apk: ($target)" }
        }
    }
}

def generate-nfpm-config [
    binary_name: string
    version: string
    arch: string
    binary_path: string
]: nothing -> string {
    mut config = nfpm-base-config $binary_name $version $arch
    $config = $config + (nfpm-contents-section $binary_name $binary_path)
    $config = $config + (nfpm-dependencies-section)
    $config
}

