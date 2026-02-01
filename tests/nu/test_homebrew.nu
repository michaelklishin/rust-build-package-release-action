#!/usr/bin/env nu

# Tests for generate-homebrew.nu functions

use std/assert
use generate-homebrew.nu [to-class-name, generate-formula]

def main [] {
    test-to-class-name
    test-generate-formula-basic
    test-generate-formula-all-platforms
    test-generate-formula-macos-only
    test-generate-formula-linux-only
}

def test-to-class-name [] {
    assert equal (to-class-name "mytool") "Mytool"
    assert equal (to-class-name "my-tool") "MyTool"
    assert equal (to-class-name "my-awesome-cli") "MyAwesomeCli"
    assert equal (to-class-name "a") "A"
    assert equal (to-class-name "rabbitmqadmin") "Rabbitmqadmin"
    assert equal (to-class-name "rabbitmqadmin-ng") "RabbitmqadminNg"
}

def test-generate-formula-basic [] {
    let config = {
        class: "MyTool"
        binary_name: "my-tool"
        version: "1.2.3"
        description: "A test tool"
        homepage: "https://example.com"
        license: "MIT"
        macos_arm64_url: ""
        macos_arm64_sha256: ""
        macos_x64_url: ""
        macos_x64_sha256: ""
        linux_arm64_url: ""
        linux_arm64_sha256: ""
        linux_x64_url: ""
        linux_x64_sha256: ""
    }

    let formula = generate-formula $config

    assert ($formula | str contains "class MyTool < Formula")
    assert ($formula | str contains 'desc "A test tool"')
    assert ($formula | str contains 'homepage "https://example.com"')
    assert ($formula | str contains 'version "1.2.3"')
    assert ($formula | str contains 'license "MIT"')
    assert ($formula | str contains 'bin.install "my-tool"')
    assert ($formula | str contains '"#{bin}/my-tool", "--version"')
}

def test-generate-formula-all-platforms [] {
    let config = {
        class: "MyTool"
        binary_name: "my-tool"
        version: "1.0.0"
        description: "Test"
        homepage: ""
        license: ""
        macos_arm64_url: "https://example.com/macos-arm64.tar.gz"
        macos_arm64_sha256: "abc123"
        macos_x64_url: "https://example.com/macos-x64.tar.gz"
        macos_x64_sha256: "def456"
        linux_arm64_url: "https://example.com/linux-arm64.tar.gz"
        linux_arm64_sha256: "ghi789"
        linux_x64_url: "https://example.com/linux-x64.tar.gz"
        linux_x64_sha256: "jkl012"
    }

    let formula = generate-formula $config

    assert ($formula | str contains "on_macos do")
    assert ($formula | str contains "on_linux do")
    assert ($formula | str contains "Hardware::CPU.arm?")
    assert ($formula | str contains 'url "https://example.com/macos-arm64.tar.gz"')
    assert ($formula | str contains 'sha256 "abc123"')
    assert ($formula | str contains 'url "https://example.com/linux-x64.tar.gz"')
    assert ($formula | str contains 'sha256 "jkl012"')
}

def test-generate-formula-macos-only [] {
    let config = {
        class: "MacTool"
        binary_name: "mac-tool"
        version: "2.0.0"
        description: "macOS only"
        homepage: ""
        license: ""
        macos_arm64_url: "https://example.com/arm64.tar.gz"
        macos_arm64_sha256: "sha256hash"
        macos_x64_url: ""
        macos_x64_sha256: ""
        linux_arm64_url: ""
        linux_arm64_sha256: ""
        linux_x64_url: ""
        linux_x64_sha256: ""
    }

    let formula = generate-formula $config

    assert ($formula | str contains "on_macos do")
    assert ($formula | str contains "on_arm do")
    assert (not ($formula | str contains "on_linux do"))
}

def test-generate-formula-linux-only [] {
    let config = {
        class: "LinuxTool"
        binary_name: "linux-tool"
        version: "3.0.0"
        description: "Linux only"
        homepage: ""
        license: ""
        macos_arm64_url: ""
        macos_arm64_sha256: ""
        macos_x64_url: ""
        macos_x64_sha256: ""
        linux_arm64_url: ""
        linux_arm64_sha256: ""
        linux_x64_url: "https://example.com/linux-x64.tar.gz"
        linux_x64_sha256: "linuxhash"
    }

    let formula = generate-formula $config

    assert (not ($formula | str contains "on_macos do"))
    assert ($formula | str contains "on_linux do")
    assert ($formula | str contains "on_intel do")
}
