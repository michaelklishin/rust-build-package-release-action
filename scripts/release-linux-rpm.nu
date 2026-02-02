#!/usr/bin/env nu

use common.nu [get-cargo-info, output, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, output-build-results, check-nfpm, install-linux-cross-deps, nfpm-base-config, nfpm-contents-section, nfpm-dependencies-section, run-pre-build-hook]

def main [] {
    let skip_build = $env.SKIP_BUILD? | default "" | $in == "true"
    let custom_binary_path = $env.BINARY_PATH? | default ""

    if not $skip_build {
        check-rust-toolchain
    }
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

    let arch = target-to-rpm-arch $target

    print $"(ansi green)Building .rpm package:(ansi reset) ($binary_name) v($version) for ($arch)"

    let release_dir = $"target/($target)/release"
    let binary_path = if $skip_build and $custom_binary_path != "" {
        $custom_binary_path
    } else {
        $"($release_dir)/($binary_name)"
    }

    if not ($binary_path | path exists) {
        if $skip_build {
            error $"binary not found: ($binary_path)"
        }
        print $"(ansi yellow)Binary not found, building...(ansi reset)"
        rm -rf $release_dir
        mkdir $release_dir
        ensure-lockfile
        run-pre-build-hook
        install-linux-cross-deps $target
        cargo-build $target $binary_name
    }

    if not ($binary_path | path exists) {
        error $"binary not found: ($binary_path)"
    }

    let pkg_dir = "target/pkg-rpm"
    rm -rf $pkg_dir
    mkdir $pkg_dir

    let abs_binary_path = $binary_path | path expand
    let nfpm_config = generate-nfpm-config $binary_name $version $arch $abs_binary_path
    let config_path = $"($pkg_dir)/nfpm.yaml"
    $nfpm_config | save -f $config_path

    let release_num = $env.PKG_RELEASE? | default "1"
    let artifact = $"($binary_name)-($version)-($release_num).($arch).rpm"
    let artifact_path = $"($release_dir)/($artifact)"

    print $"(ansi green)Running nfpm...(ansi reset)"
    nfpm package --config $config_path --packager rpm --target $artifact_path

    if not ($artifact_path | path exists) {
        error $"failed to create package: ($artifact_path)"
    }

    let checksums = generate-checksums $artifact_path
    print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
    hr-line
    ls $release_dir | where name =~ '\.rpm' | print
    print $"(ansi green)Created:(ansi reset) ($artifact)"

    output "version" $version
    output "binary_name" $binary_name
    output "target" $target
    output "binary_path" $binary_path
    output-build-results $binary_name $version $target $artifact $artifact_path $checksums
}

def target-to-rpm-arch [target: string]: nothing -> string {
    match $target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "x86_64"
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "aarch64"
        "armv7-unknown-linux-gnueabihf" => "armv7hl"
        "i686-unknown-linux-gnu" | "i686-unknown-linux-musl" => "i686"
        _ => {
            if $target =~ "x86_64" { "x86_64" }
            else if $target =~ "aarch64" { "aarch64" }
            else if $target =~ "armv7" { "armv7hl" }
            else if $target =~ "i686" { "i686" }
            else { error $"unsupported target for .rpm: ($target)" }
        }
    }
}

def generate-nfpm-config [
    binary_name: string
    version: string
    arch: string
    binary_path: string
]: nothing -> string {
    let release_num = $env.PKG_RELEASE? | default "1"
    let group = $env.PKG_GROUP? | default "Applications/System"
    let description = $env.PKG_DESCRIPTION? | default $"($binary_name) - built with rust-release-action"
    let summary = $env.PKG_SUMMARY? | default $description

    mut config = nfpm-base-config $binary_name $version $arch
    $config = $config + $"release: \"($release_num)\"\n"
    $config = $config + (nfpm-contents-section $binary_name $binary_path)
    $config = $config + $"
rpm:
  group: \"($group)\"
  summary: \"($summary)\"
  compression: gzip
"
    $config = $config + (nfpm-dependencies-section)
    $config
}

