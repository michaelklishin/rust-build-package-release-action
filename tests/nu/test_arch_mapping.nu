#!/usr/bin/env nu

# Tests for architecture mapping functions

use std/assert

# Deb architecture mapping (must match release-linux-deb.nu)
def target-to-deb-arch [target: string]: nothing -> string {
    match $target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "amd64"
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "arm64"
        "armv7-unknown-linux-gnueabihf" => "armhf"
        "i686-unknown-linux-gnu" | "i686-unknown-linux-musl" => "i386"
        _ => {
            if $target =~ "x86_64" { "amd64" } else if $target =~ "aarch64" { "arm64" } else if $target =~ "armv7" { "armhf" } else if $target =~ "i686" { "i386" } else { "" }
        }
    }
}

# RPM architecture mapping (must match release-linux-rpm.nu)
def target-to-rpm-arch [target: string]: nothing -> string {
    match $target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "x86_64"
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "aarch64"
        "armv7-unknown-linux-gnueabihf" => "armv7hl"
        "i686-unknown-linux-gnu" | "i686-unknown-linux-musl" => "i686"
        _ => {
            if $target =~ "x86_64" { "x86_64" } else if $target =~ "aarch64" { "aarch64" } else if $target =~ "armv7" { "armv7hl" } else if $target =~ "i686" { "i686" } else { "" }
        }
    }
}

# APK architecture mapping (must match release-linux-apk.nu)
def target-to-apk-arch [target: string]: nothing -> string {
    match $target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => "x86_64"
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => "aarch64"
        "armv7-unknown-linux-gnueabihf" | "armv7-unknown-linux-musleabihf" => "armv7"
        "i686-unknown-linux-gnu" | "i686-unknown-linux-musl" => "x86"
        _ => {
            if $target =~ "x86_64" { "x86_64" } else if $target =~ "aarch64" { "aarch64" } else if $target =~ "armv7" { "armv7" } else if $target =~ "i686" { "x86" } else { "" }
        }
    }
}

def main [] {
    # Deb mappings
    assert equal (target-to-deb-arch "x86_64-unknown-linux-gnu") "amd64"
    assert equal (target-to-deb-arch "x86_64-unknown-linux-musl") "amd64"
    assert equal (target-to-deb-arch "aarch64-unknown-linux-gnu") "arm64"
    assert equal (target-to-deb-arch "aarch64-unknown-linux-musl") "arm64"
    assert equal (target-to-deb-arch "armv7-unknown-linux-gnueabihf") "armhf"
    assert equal (target-to-deb-arch "i686-unknown-linux-gnu") "i386"
    assert equal (target-to-deb-arch "i686-unknown-linux-musl") "i386"
    assert equal (target-to-deb-arch "x86_64-custom-target") "amd64"

    # RPM mappings
    assert equal (target-to-rpm-arch "x86_64-unknown-linux-gnu") "x86_64"
    assert equal (target-to-rpm-arch "x86_64-unknown-linux-musl") "x86_64"
    assert equal (target-to-rpm-arch "aarch64-unknown-linux-gnu") "aarch64"
    assert equal (target-to-rpm-arch "aarch64-unknown-linux-musl") "aarch64"
    assert equal (target-to-rpm-arch "armv7-unknown-linux-gnueabihf") "armv7hl"
    assert equal (target-to-rpm-arch "i686-unknown-linux-gnu") "i686"
    assert equal (target-to-rpm-arch "i686-unknown-linux-musl") "i686"
    assert equal (target-to-rpm-arch "x86_64-custom-target") "x86_64"
    assert equal (target-to-rpm-arch "aarch64-custom-target") "aarch64"

    # APK mappings
    assert equal (target-to-apk-arch "x86_64-unknown-linux-gnu") "x86_64"
    assert equal (target-to-apk-arch "x86_64-unknown-linux-musl") "x86_64"
    assert equal (target-to-apk-arch "aarch64-unknown-linux-gnu") "aarch64"
    assert equal (target-to-apk-arch "aarch64-unknown-linux-musl") "aarch64"
    assert equal (target-to-apk-arch "armv7-unknown-linux-gnueabihf") "armv7"
    assert equal (target-to-apk-arch "armv7-unknown-linux-musleabihf") "armv7"
    assert equal (target-to-apk-arch "i686-unknown-linux-gnu") "x86"
    assert equal (target-to-apk-arch "i686-unknown-linux-musl") "x86"
    assert equal (target-to-apk-arch "x86_64-alpine-linux-musl") "x86_64"
    assert equal (target-to-apk-arch "aarch64-alpine-linux-musl") "aarch64"
}
