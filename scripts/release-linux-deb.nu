#!/usr/bin/env nu

use common.nu [get-cargo-info, output, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, output-build-results, check-nfpm, install-linux-cross-deps, nfpm-base-config, nfpm-contents-section, nfpm-dependencies-section]

def main [] {
    check-rust-toolchain
    check-nfpm

    let target = $env.TARGET? | default "x86_64-unknown-linux-gnu"
    let info = get-cargo-info
    let binary_name = $env.BINARY_NAME? | default $info.name
    let version = $info.version

    if $binary_name == "" {
        error "could not determine binary name"
    }
    if $version == "" {
        error "could not determine version"
    }

    let arch = target-to-deb-arch $target

    print $"(ansi green)Building .deb package:(ansi reset) ($binary_name) v($version) for ($arch)"

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

    let pkg_dir = "target/pkg-deb"
    rm -rf $pkg_dir
    mkdir $pkg_dir

    let abs_binary_path = $binary_path | path expand
    let nfpm_config = generate-nfpm-config $binary_name $version $arch $abs_binary_path
    let config_path = $"($pkg_dir)/nfpm.yaml"
    $nfpm_config | save -f $config_path

    let artifact = $"($binary_name)_($version)_($arch).deb"
    let artifact_path = $"($release_dir)/($artifact)"

    print $"(ansi green)Running nfpm...(ansi reset)"
    nfpm package --config $config_path --packager deb --target $artifact_path

    if not ($artifact_path | path exists) {
        error $"failed to create package: ($artifact_path)"
    }

    let checksums = generate-checksums $artifact_path
    print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
    hr-line
    ls $release_dir | where name =~ '\.deb' | print
    print $"(ansi green)Created:(ansi reset) ($artifact)"

    output "version" $version
    output "binary_name" $binary_name
    output "target" $target
    output "binary_path" $binary_path
    output-build-results $binary_name $version $target $artifact $artifact_path $checksums
}

def target-to-deb-arch [target: string]: nothing -> string {
    match $target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "amd64"
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "arm64"
        "armv7-unknown-linux-gnueabihf" => "armhf"
        "i686-unknown-linux-gnu" | "i686-unknown-linux-musl" => "i386"
        _ => {
            if $target =~ "x86_64" { "amd64" }
            else if $target =~ "aarch64" { "arm64" }
            else if $target =~ "armv7" { "armhf" }
            else if $target =~ "i686" { "i386" }
            else { error $"unsupported target for .deb: ($target)" }
        }
    }
}

def generate-nfpm-config [
    binary_name: string
    version: string
    arch: string
    binary_path: string
]: nothing -> string {
    let section = $env.PKG_SECTION? | default "utils"
    let priority = $env.PKG_PRIORITY? | default "optional"

    mut config = nfpm-base-config $binary_name $version $arch
    $config = $config + (nfpm-contents-section $binary_name $binary_path)
    $config = $config + $"
deb:
  section: \"($section)\"
  priority: \"($priority)\"
"
    $config = $config + (nfpm-dependencies-section)
    $config
}

