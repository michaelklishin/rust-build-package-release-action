#!/usr/bin/env nu

# Tests for list-archivable-files function

use std/assert
use common.nu [list-archivable-files]

def main [] {
    test-list-archivable-files
}

def test-list-archivable-files [] {
    # Create a temporary directory with test files
    let temp_dir = $"/tmp/test-archivable-(random uuid)"
    mkdir $temp_dir

    # Create test files
    "binary content" | save $"($temp_dir)/myapp"
    "readme content" | save $"($temp_dir)/README.md"
    "license content" | save $"($temp_dir)/LICENSE"
    "archive content" | save $"($temp_dir)/myapp.tar.gz"
    "zip content" | save $"($temp_dir)/myapp.zip"
    "sha256 content" | save $"($temp_dir)/myapp.sha256"
    "sha512 content" | save $"($temp_dir)/myapp.sha512"
    "b2 content" | save $"($temp_dir)/myapp.b2"
    "sig content" | save $"($temp_dir)/myapp.sig"
    "pem content" | save $"($temp_dir)/myapp.pem"
    "sigstore content" | save $"($temp_dir)/myapp.sigstore.json"
    "spdx content" | save $"($temp_dir)/myapp.spdx.json"
    "cdx content" | save $"($temp_dir)/myapp.cdx.json"

    let files = list-archivable-files $temp_dir

    # Should include these
    assert ($files | any {|f| $f == "myapp" })
    assert ($files | any {|f| $f == "README.md" })
    assert ($files | any {|f| $f == "LICENSE" })

    # Should exclude these
    assert (not ($files | any {|f| $f == "myapp.tar.gz" }))
    assert (not ($files | any {|f| $f == "myapp.zip" }))
    assert (not ($files | any {|f| $f == "myapp.sha256" }))
    assert (not ($files | any {|f| $f == "myapp.sha512" }))
    assert (not ($files | any {|f| $f == "myapp.b2" }))
    assert (not ($files | any {|f| $f == "myapp.sig" }))
    assert (not ($files | any {|f| $f == "myapp.pem" }))
    assert (not ($files | any {|f| $f == "myapp.sigstore.json" }))
    assert (not ($files | any {|f| $f == "myapp.spdx.json" }))
    assert (not ($files | any {|f| $f == "myapp.cdx.json" }))

    # Clean up
    rm -rf $temp_dir
}
