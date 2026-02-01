#!/usr/bin/env nu

# Tests for generate-winget.nu functions

use std/assert
use generate-winget.nu [generate-version-manifest, generate-locale-manifest, generate-installer-manifest, parse-tags]

def main [] {
    test-parse-tags
    test-generate-version-manifest
    test-generate-locale-manifest
    test-generate-installer-manifest
}

def test-parse-tags [] {
    # Empty input
    assert equal (parse-tags "") []
    assert equal (parse-tags "   ") []
    assert equal (parse-tags ", ,") []

    # Single tag
    assert equal (parse-tags "cli") ["cli"]

    # Multiple tags
    assert equal (parse-tags "cli, tool, rust") ["cli" "tool" "rust"]

    # Whitespace handling
    assert equal (parse-tags "  cli  ,  tool  ") ["cli" "tool"]
}

def test-generate-version-manifest [] {
    let manifest = generate-version-manifest "Publisher.Package" "1.2.3"

    assert ($manifest | str contains "PackageIdentifier: Publisher.Package")
    assert ($manifest | str contains "PackageVersion: 1.2.3")
    assert ($manifest | str contains "DefaultLocale: en-US")
    assert ($manifest | str contains "ManifestType: version")
    assert ($manifest | str contains "ManifestVersion: 1.6.0")
}

def test-generate-locale-manifest [] {
    let config = {
        id: "Publisher.Package"
        version: "1.2.3"
        publisher: "Test Publisher"
        name: "myapp"
        description: "A test application"
        homepage: "https://example.com"
        license: "MIT"
        license_url: "https://example.com/license"
        copyright: "Copyright 2024"
        tags: "cli, tool"
    }

    let manifest = generate-locale-manifest $config

    assert ($manifest | str contains "PackageIdentifier: Publisher.Package")
    assert ($manifest | str contains "PackageVersion: 1.2.3")
    assert ($manifest | str contains "Publisher: Test Publisher")
    assert ($manifest | str contains "PackageName: myapp")
    assert ($manifest | str contains "ShortDescription: A test application")
    assert ($manifest | str contains "PackageUrl: https://example.com")
    assert ($manifest | str contains "License: MIT")
    assert ($manifest | str contains "LicenseUrl: https://example.com/license")
    assert ($manifest | str contains "Copyright: Copyright 2024")
    assert ($manifest | str contains "Tags:")
    assert ($manifest | str contains "  - cli")
    assert ($manifest | str contains "  - tool")
    assert ($manifest | str contains "ManifestType: defaultLocale")
}

def test-generate-installer-manifest [] {
    let config = {
        id: "Publisher.Package"
        version: "1.0.0"
        x64_url: "https://example.com/app-x64.zip"
        x64_sha256: "abc123"
        arm64_url: "https://example.com/app-arm64.zip"
        arm64_sha256: "def456"
    }

    let manifest = generate-installer-manifest $config

    assert ($manifest | str contains "PackageIdentifier: Publisher.Package")
    assert ($manifest | str contains "PackageVersion: 1.0.0")
    assert ($manifest | str contains "InstallerType: portable")
    assert ($manifest | str contains "Commands:")
    assert ($manifest | str contains "  - Package")
    assert ($manifest | str contains "Installers:")
    assert ($manifest | str contains "  - Architecture: x64")
    assert ($manifest | str contains "    InstallerUrl: https://example.com/app-x64.zip")
    assert ($manifest | str contains "    InstallerSha256: ABC123")
    assert ($manifest | str contains "  - Architecture: arm64")
    assert ($manifest | str contains "    InstallerUrl: https://example.com/app-arm64.zip")
    assert ($manifest | str contains "    InstallerSha256: DEF456")
    assert ($manifest | str contains "ManifestType: installer")
}
