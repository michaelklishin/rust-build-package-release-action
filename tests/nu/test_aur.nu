#!/usr/bin/env nu

# Tests for generate-aur.nu functions

use std/assert
use generate-aur.nu [generate-pkgbuild]

def main [] {
    test-generate-pkgbuild-basic
    test-generate-pkgbuild-with-deps
}

def test-generate-pkgbuild-basic [] {
    let config = {
        pkgname: "mytool"
        pkgver: "1.2.3"
        pkgdesc: "A test tool"
        url: "https://example.com"
        license: "MIT"
        maintainer: "Test User <test@example.com>"
        source_url: ""
        source_sha256: ""
        depends: ""
        makedepends: "cargo"
        optdepends: ""
        provides: ""
        conflicts: ""
        binary_name: "mytool"
    }

    let pkgbuild = generate-pkgbuild $config

    assert ($pkgbuild | str contains "pkgname=mytool")
    assert ($pkgbuild | str contains "pkgver=1.2.3")
    assert ($pkgbuild | str contains 'pkgdesc="A test tool"')
    assert ($pkgbuild | str contains 'url="https://example.com"')
    assert ($pkgbuild | str contains "license=('MIT')")
    assert ($pkgbuild | str contains "makedepends=('cargo')")
    assert ($pkgbuild | str contains "arch=('x86_64' 'aarch64')")
    assert ($pkgbuild | str contains 'install -Dm755 "target/release/mytool"')
    assert ($pkgbuild | str contains "Maintainer: Test User")
}

def test-generate-pkgbuild-with-deps [] {
    let config = {
        pkgname: "mytool"
        pkgver: "2.0.0"
        pkgdesc: "Tool with deps"
        url: ""
        license: "Apache-2.0"
        maintainer: ""
        source_url: "https://example.com/source.tar.gz"
        source_sha256: "abc123"
        depends: "glibc, openssl"
        makedepends: "cargo, rust"
        optdepends: "man-db: for man pages"
        provides: "mytool-bin"
        conflicts: "mytool-git"
        binary_name: "my-tool"
    }

    let pkgbuild = generate-pkgbuild $config

    assert ($pkgbuild | str contains "depends=('glibc' 'openssl')")
    assert ($pkgbuild | str contains "makedepends=('cargo' 'rust')")
    assert ($pkgbuild | str contains "optdepends=('man-db: for man pages')")
    assert ($pkgbuild | str contains "provides=('mytool-bin')")
    assert ($pkgbuild | str contains "conflicts=('mytool-git')")
    assert ($pkgbuild | str contains 'source=("https://example.com/source.tar.gz")')
    assert ($pkgbuild | str contains "sha256sums=('abc123')")
    assert ($pkgbuild | str contains 'install -Dm755 "target/release/my-tool"')
}
