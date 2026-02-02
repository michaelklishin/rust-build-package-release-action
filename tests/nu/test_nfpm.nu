#!/usr/bin/env nu

# Tests for nfpm configuration generation functions

use std/assert
use common.nu [nfpm-base-config, nfpm-contents-section, nfpm-dependencies-section, format-dependency-list]

def main [] {
    test-nfpm-base-config
    test-nfpm-base-config-with-metadata
    test-nfpm-contents-section
    test-nfpm-dependencies-section
}

def test-nfpm-base-config [] {
    # Test with minimal env vars (defaults)
    # Use hide-env to ensure defaults are used
    hide-env -i PKG_DESCRIPTION
    hide-env -i PKG_MAINTAINER
    hide-env -i PKG_HOMEPAGE
    hide-env -i PKG_LICENSE
    hide-env -i PKG_VENDOR

    let config = nfpm-base-config "myapp" "1.2.3" "amd64"

    assert ($config | str contains 'name: "myapp"')
    assert ($config | str contains 'arch: "amd64"')
    assert ($config | str contains "platform: linux")
    assert ($config | str contains 'version: "1.2.3"')
    assert ($config | str contains 'maintainer: "Unknown <unknown@example.com>"')
    assert ($config | str contains 'description: "myapp - built with rust-release-action"')
}

def test-nfpm-base-config-with-metadata [] {
    $env.PKG_DESCRIPTION = "A CLI tool for testing"
    $env.PKG_MAINTAINER = "Test User <test@example.com>"
    $env.PKG_HOMEPAGE = "https://example.com"
    $env.PKG_LICENSE = "MIT"
    $env.PKG_VENDOR = "Test Corp"

    let config = nfpm-base-config "testcli" "2.0.0" "arm64"

    assert ($config | str contains 'name: "testcli"')
    assert ($config | str contains 'arch: "arm64"')
    assert ($config | str contains 'version: "2.0.0"')
    assert ($config | str contains 'maintainer: "Test User <test@example.com>"')
    assert ($config | str contains 'description: "A CLI tool for testing"')
    assert ($config | str contains 'homepage: "https://example.com"')
    assert ($config | str contains 'license: "MIT"')
    assert ($config | str contains 'vendor: "Test Corp"')

    # Clean up
    hide-env -i PKG_DESCRIPTION
    hide-env -i PKG_MAINTAINER
    hide-env -i PKG_HOMEPAGE
    hide-env -i PKG_LICENSE
    hide-env -i PKG_VENDOR
}

def test-nfpm-contents-section [] {
    hide-env -i PKG_CONTENTS

    let config = nfpm-contents-section "myapp" "/path/to/myapp"

    assert ($config | str contains "contents:")
    assert ($config | str contains 'src: "/path/to/myapp"')
    assert ($config | str contains 'dst: "/usr/bin/myapp"')
    assert ($config | str contains "mode: 0755")
}

def test-nfpm-dependencies-section [] {
    # Test with no dependencies
    hide-env -i PKG_DEPENDS
    hide-env -i PKG_RECOMMENDS
    hide-env -i PKG_SUGGESTS
    hide-env -i PKG_CONFLICTS
    hide-env -i PKG_REPLACES
    hide-env -i PKG_PROVIDES

    let empty_config = nfpm-dependencies-section
    assert equal $empty_config ""

    # Test with dependencies
    $env.PKG_DEPENDS = "libc6, libssl3"
    $env.PKG_RECOMMENDS = "curl"
    $env.PKG_CONFLICTS = "oldapp"

    let config = nfpm-dependencies-section

    assert ($config | str contains "depends:")
    assert ($config | str contains '"libc6"')
    assert ($config | str contains '"libssl3"')
    assert ($config | str contains "recommends:")
    assert ($config | str contains '"curl"')
    assert ($config | str contains "conflicts:")
    assert ($config | str contains '"oldapp"')
    assert (not ($config | str contains "suggests:"))
    assert (not ($config | str contains "replaces:"))
    assert (not ($config | str contains "provides:"))

    # Clean up
    hide-env -i PKG_DEPENDS
    hide-env -i PKG_RECOMMENDS
    hide-env -i PKG_SUGGESTS
    hide-env -i PKG_CONFLICTS
    hide-env -i PKG_REPLACES
    hide-env -i PKG_PROVIDES
}
