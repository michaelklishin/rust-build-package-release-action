#!/usr/bin/env nu

# Tests for format-release.nu functions

use std/assert
use format-release.nu [format-size, detect-platform]

def main [] {
    test-format-size
    test-detect-platform
}

def test-format-size [] {
    assert equal (format-size 500) "500 B"
    assert equal (format-size 1024) "1.0 KB"
    assert equal (format-size 2048) "2.0 KB"
    assert equal (format-size 1048576) "1.0 MB"
    assert equal (format-size 5242880) "5.0 MB"
}

def test-detect-platform [] {
    assert equal (detect-platform "mytool-1.0.0-darwin-arm64.tar.gz") "macOS (Apple Silicon)"
    assert equal (detect-platform "mytool-1.0.0-darwin-x86_64.tar.gz") "macOS (Intel)"
    assert equal (detect-platform "mytool-1.0.0-macos-aarch64.tar.gz") "macOS (Apple Silicon)"
    assert equal (detect-platform "mytool-1.0.0-linux-x86_64.tar.gz") "Linux (x64)"
    assert equal (detect-platform "mytool-1.0.0-linux-aarch64.tar.gz") "Linux (ARM64)"
    assert equal (detect-platform "mytool-1.0.0-linux-x86_64-musl.tar.gz") "Linux (x64, musl)"
    assert equal (detect-platform "mytool-1.0.0-linux-aarch64-musl.tar.gz") "Linux (ARM64, musl)"
    assert equal (detect-platform "mytool-1.0.0-windows-x64.zip") "Windows (x64)"
    assert equal (detect-platform "mytool-1.0.0-windows-arm64.zip") "Windows (ARM64)"
    assert equal (detect-platform "mytool_1.0.0_amd64.deb") "Debian/Ubuntu"
    assert equal (detect-platform "mytool-1.0.0-1.x86_64.rpm") "RHEL/Fedora"
    assert equal (detect-platform "mytool-1.0.0-r0.apk") "Alpine Linux"
    assert equal (detect-platform "mytool-1.0.0.dmg") "macOS Installer"
    assert equal (detect-platform "mytool-1.0.0.msi") "Windows Installer"
}
