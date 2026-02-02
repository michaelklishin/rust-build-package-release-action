#!/usr/bin/env nu

# Tests for collect-artifacts.nu functions

use std/assert
use collect-artifacts.nu [detect-platform]

def main [] {
    test-detect-platform-macos
    test-detect-platform-linux
    test-detect-platform-windows
    test-detect-platform-packages
}

def test-detect-platform-macos [] {
    # macOS ARM64 patterns
    assert equal (detect-platform "myapp-1.0.0-aarch64-apple-darwin.tar.gz") "macos-arm64"
    assert equal (detect-platform "myapp-darwin-arm64.tar.gz") "macos-arm64"
    assert equal (detect-platform "myapp-macos-arm64.tar.gz") "macos-arm64"

    # macOS x64 patterns
    assert equal (detect-platform "myapp-1.0.0-x86_64-apple-darwin.tar.gz") "macos-x64"
    assert equal (detect-platform "myapp-darwin-x86_64.tar.gz") "macos-x64"
    assert equal (detect-platform "myapp-macos-x64.tar.gz") "macos-x64"

    # macOS DMG
    assert equal (detect-platform "myapp-1.0.0.dmg") "macos-dmg"
}

def test-detect-platform-linux [] {
    # Linux ARM64 patterns
    assert equal (detect-platform "myapp-1.0.0-aarch64-unknown-linux-gnu.tar.gz") "linux-arm64"
    assert equal (detect-platform "myapp-linux-arm64.tar.gz") "linux-arm64"
    assert equal (detect-platform "myapp-linux-aarch64.tar.gz") "linux-arm64"

    # Linux x64 patterns
    assert equal (detect-platform "myapp-1.0.0-x86_64-unknown-linux-gnu.tar.gz") "linux-x64"
    assert equal (detect-platform "myapp-linux-x64.tar.gz") "linux-x64"
    assert equal (detect-platform "myapp-linux-amd64.tar.gz") "linux-x64"

    # Linux packages
    assert equal (detect-platform "myapp-1.0.0.deb") "linux-deb"
    assert equal (detect-platform "myapp-1.0.0.rpm") "linux-rpm"
    assert equal (detect-platform "myapp-1.0.0.apk") "linux-apk"
}

def test-detect-platform-windows [] {
    # Windows x64 patterns
    assert equal (detect-platform "myapp-1.0.0-x86_64-pc-windows-msvc.zip") "windows-x64"
    assert equal (detect-platform "myapp-windows-x64.zip") "windows-x64"
    assert equal (detect-platform "myapp-windows-x86_64.zip") "windows-x64"

    # Windows ARM64 patterns
    assert equal (detect-platform "myapp-1.0.0-aarch64-pc-windows-msvc.zip") "windows-arm64"
    assert equal (detect-platform "myapp-windows-arm64.zip") "windows-arm64"

    # Windows MSI
    assert equal (detect-platform "myapp-1.0.0.msi") "windows-msi"
}

def test-detect-platform-packages [] {
    # Package formats should be detected by extension
    assert equal (detect-platform "package.deb") "linux-deb"
    assert equal (detect-platform "package.rpm") "linux-rpm"
    assert equal (detect-platform "package.apk") "linux-apk"
    assert equal (detect-platform "installer.dmg") "macos-dmg"
    assert equal (detect-platform "setup.msi") "windows-msi"

    # Unknown format
    assert equal (detect-platform "unknown-file.txt") "unknown"
}
