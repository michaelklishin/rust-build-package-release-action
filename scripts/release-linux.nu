#!/usr/bin/env nu

use common.nu [get-cargo-info, output, copy-docs, ensure-lockfile, cargo-build, hr-line, error]

def main [] {
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

    print $"(ansi green)Building(ansi reset) ($binary_name) v($version) for ($target)"

    let release_dir = $"target/($target)/release"
    rm -rf $release_dir
    mkdir $release_dir

    ensure-lockfile
    install-dependencies $target
    cargo-build $target $binary_name

    let src = $"($release_dir)/($binary_name)"
    if not ($src | path exists) {
        error $"binary not found: ($src)"
    }

    let artifact = $"($binary_name)-($version)-($target)"
    let artifact_path = $"($release_dir)/($artifact)"
    cp $src $artifact_path
    chmod +x $artifact_path
    copy-docs $release_dir

    print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
    hr-line
    ls $release_dir | print

    print $"(ansi green)Created:(ansi reset) ($artifact)"
    output "artifact" $artifact
    output "artifact_path" $artifact_path
}

def install-dependencies [target: string] {
    # Detect the OS
    let is_ubuntu = (which apt-get | is-not-empty)
    let is_fedora = (which dnf | is-not-empty)

    if $target =~ "musl" {
        if $is_ubuntu {
            sudo apt-get update -qq
            sudo apt-get install -y -qq musl-tools
        }
    } else if $target == "aarch64-unknown-linux-gnu" {
        let arch = (^uname -m | str trim)
        if $arch != "aarch64" {
            if $is_ubuntu {
                sudo apt-get update -qq
                sudo apt-get install -y -qq gcc-aarch64-linux-gnu
            } else if $is_fedora {
                sudo dnf install -y gcc-aarch64-linux-gnu
            }
            $env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "aarch64-linux-gnu-gcc"
        }
    } else if $target == "armv7-unknown-linux-gnueabihf" {
        if $is_ubuntu {
            sudo apt-get update -qq
            sudo apt-get install -y -qq pkg-config gcc-arm-linux-gnueabihf
        } else if $is_fedora {
            sudo dnf install -y pkg-config gcc-arm-linux-gnueabihf
        }
    }

    rustup target add $target
}
