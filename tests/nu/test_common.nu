#!/usr/bin/env nu

# Tests for common.nu functions

use std/assert
use common.nu [format-dependency-list, build-summary]

def main [] {
    test-format-dependency-list
    test-build-summary
}

def test-format-dependency-list [] {
    # Empty input
    assert equal (format-dependency-list "depends" "") ""
    assert equal (format-dependency-list "depends" "   ") ""
    assert equal (format-dependency-list "depends" ", ,") ""

    # Single item
    let single = format-dependency-list "depends" "libc6"
    assert ($single | str starts-with "depends:\n")
    assert ($single | str contains "libc6")

    # Multiple items
    let multi = format-dependency-list "requires" "glibc, openssl, zlib"
    assert ($multi | str starts-with "requires:\n")
    assert ($multi | str contains "glibc")
    assert ($multi | str contains "openssl")
    assert ($multi | str contains "zlib")

    # Whitespace handling
    let spaced = format-dependency-list "depends" "  pkg1  ,  pkg2  "
    assert ($spaced | str contains "pkg1")
    assert ($spaced | str contains "pkg2")
    assert (not ($spaced | str contains "  pkg"))
}

def test-build-summary [] {
    let checksums = {sha256: "abc123", sha512: "def456", b2: "ghi789"}
    let result = build-summary "mybin" "1.2.3" "x86_64-unknown-linux-gnu" "mybin-1.2.3.tar.gz" "/path/to/artifact" $checksums

    let parsed = $result | from json
    assert equal $parsed.binary_name "mybin"
    assert equal $parsed.version "1.2.3"
    assert equal $parsed.target "x86_64-unknown-linux-gnu"
    assert equal $parsed.artifact "mybin-1.2.3.tar.gz"
    assert equal $parsed.artifact_path "/path/to/artifact"
    assert equal $parsed.sha256 "abc123"
    assert equal $parsed.sha512 "def456"
    assert equal $parsed.b2 "ghi789"
}
