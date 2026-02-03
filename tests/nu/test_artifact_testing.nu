#!/usr/bin/env nu

# Tests for artifact testing functions

use std/assert
use ../../scripts/test-common.nu [parse-checksum-file, check-version-in-output, detect-checksum-type, compute-checksum, verify-checksum]

def main [] {
    test-parse-checksum-file
    test-detect-checksum-type
    test-compute-checksum-sha256
    test-verify-checksum-roundtrip
    test-check-version-in-output
    test-version-patterns
}

def test-parse-checksum-file [] {
    let temp_dir = $"/tmp/test-checksum-(random uuid)"
    mkdir $temp_dir

    # Standard sha256sum format (two spaces)
    let checksum1 = "abc123def456  myfile.tar.gz\n"
    $checksum1 | save $"($temp_dir)/test1.sha256"
    let result1 = parse-checksum-file $"($temp_dir)/test1.sha256"
    assert equal $result1 "abc123def456"

    # Single space format
    let checksum2 = "fedcba987654321 another-file.zip"
    $checksum2 | save $"($temp_dir)/test2.sha256"
    let result2 = parse-checksum-file $"($temp_dir)/test2.sha256"
    assert equal $result2 "fedcba987654321"

    # With leading/trailing whitespace
    let checksum3 = "  hash123456789   file.deb  \n"
    $checksum3 | save $"($temp_dir)/test3.sha256"
    let result3 = parse-checksum-file $"($temp_dir)/test3.sha256"
    assert equal $result3 "hash123456789"

    # Multiple lines (should take first)
    let checksum4 = "first111  file1.tar.gz\nsecond222  file2.tar.gz"
    $checksum4 | save $"($temp_dir)/test4.sha256"
    let result4 = parse-checksum-file $"($temp_dir)/test4.sha256"
    assert equal $result4 "first111"

    rm -rf $temp_dir
}

def test-detect-checksum-type [] {
    assert equal (detect-checksum-type "file.sha256") "sha256"
    assert equal (detect-checksum-type "file.sha512") "sha512"
    assert equal (detect-checksum-type "file.b2") "b2"
    assert equal (detect-checksum-type "/path/to/artifact.tar.gz.sha256") "sha256"
    assert equal (detect-checksum-type "/path/to/artifact.tar.gz.sha512") "sha512"
    assert equal (detect-checksum-type "/path/to/artifact.tar.gz.b2") "b2"

    # Unknown extension defaults to sha256
    assert equal (detect-checksum-type "file.unknown") "sha256"
    assert equal (detect-checksum-type "file") "sha256"
}

def test-compute-checksum-sha256 [] {
    let temp_dir = $"/tmp/test-compute-checksum-(random uuid)"
    mkdir $temp_dir

    let test_file = $"($temp_dir)/test.txt"
    "hello world\n" | save $test_file

    let hash = compute-checksum $test_file "sha256"
    # Known SHA256 of "hello world\n"
    assert equal $hash "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447" $"Unexpected hash: ($hash)"

    rm -rf $temp_dir
}

def test-verify-checksum-roundtrip [] {
    let temp_dir = $"/tmp/test-verify-roundtrip-(random uuid)"
    mkdir $temp_dir

    let artifact = $"($temp_dir)/artifact.bin"
    let checksum_file = $"($temp_dir)/artifact.bin.sha256"

    # Create artifact with known content
    "test artifact content\n" | save $artifact

    # Compute and save checksum
    let hash = compute-checksum $artifact "sha256"
    $"($hash)  artifact.bin\n" | save $checksum_file

    # Verify should succeed (no error thrown)
    verify-checksum $artifact $checksum_file

    rm -rf $temp_dir
}

def test-check-version-in-output [] {
    # Exact match in typical --version output
    assert (check-version-in-output "myapp 1.2.3" "1.2.3")
    assert (check-version-in-output "myapp version 1.2.3" "1.2.3")
    assert (check-version-in-output "v1.2.3" "1.2.3")

    # Version with pre-release suffix
    assert (check-version-in-output "myapp 2.0.0-beta.1" "2.0.0-beta.1")
    assert (check-version-in-output "tool 1.0.0-rc.2" "1.0.0-rc.2")

    # Multiline output
    let multiline = "myapp 3.0.0
Built with Rust
Copyright 2024"
    assert (check-version-in-output $multiline "3.0.0")

    # Version not found
    assert (not (check-version-in-output "myapp 1.2.3" "1.2.4"))
    assert (not (check-version-in-output "no version here" "1.0.0"))

    # Partial match is valid
    assert (check-version-in-output "version: 1.2.3-alpha+build.456" "1.2.3")
}

def test-version-patterns [] {
    # Common version output formats from CLIs
    let patterns = [
        ["myapp 1.0.0", "1.0.0"]
        ["myapp version 2.3.4", "2.3.4"]
        ["myapp v3.0.0", "3.0.0"]
        ["Version: 1.2.3", "1.2.3"]
        ["1.0.0-beta", "1.0.0-beta"]
        ["myapp 1.0.0 (abc123)", "1.0.0"]
        ["myapp 1.0.0\nBuilt on 2024-01-01", "1.0.0"]
    ]

    for pair in $patterns {
        let output = $pair | first
        let version = $pair | last
        assert (check-version-in-output $output $version) $"Failed for: ($output)"
    }
}
