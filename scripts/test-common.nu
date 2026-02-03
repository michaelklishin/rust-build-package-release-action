#!/usr/bin/env nu

# Shared functions for artifact testing

use common.nu [error]

# Verifies a checksum file against an artifact
export def verify-checksum [artifact_path: string, checksum_file: string] {
    print $"(ansi green)Verifying checksum...(ansi reset)"

    if not ($checksum_file | path exists) {
        error $"checksum file not found: ($checksum_file)"
    }

    let expected = parse-checksum-file $checksum_file
    let checksum_type = detect-checksum-type $checksum_file
    let actual = compute-checksum $artifact_path $checksum_type

    if $actual != $expected {
        error $"checksum mismatch: expected ($expected), got ($actual)"
    }
    print $"  ($checksum_type | str upcase): ($actual) ✓"
}

# Detects checksum type from file extension
export def detect-checksum-type [checksum_file: string]: nothing -> string {
    let ext = $checksum_file | path parse | get extension
    match $ext {
        "sha256" => "sha256"
        "sha512" => "sha512"
        "b2" => "b2"
        _ => "sha256"
    }
}

# Computes checksum of a file
export def compute-checksum [file_path: string, checksum_type: string]: nothing -> string {
    match $checksum_type {
        "sha256" => { open $file_path --raw | hash sha256 }
        "sha512" => {
            if (which sha512sum | is-not-empty) {
                sha512sum $file_path | split row " " | first
            } else if (which shasum | is-not-empty) {
                shasum -a 512 $file_path | split row " " | first
            } else {
                error "sha512sum or shasum not found"
            }
        }
        "b2" => {
            if (which b2sum | is-not-empty) {
                b2sum $file_path | split row " " | first
            } else {
                error "b2sum not found"
            }
        }
        _ => { error $"unsupported checksum type: ($checksum_type)" }
    }
}

# Parses a checksum file and returns the hash
export def parse-checksum-file [checksum_file: string]: nothing -> string {
    let content = open $checksum_file | str trim
    if ($content | is-empty) {
        error "checksum file is empty"
    }
    # Handle both "hash  filename" and "hash filename" formats
    let line = $content | lines | first
    $line | split row " " | where { |s| $s | is-not-empty } | first
}

# Gets version output from a binary using common flags
export def get-version-output [binary: string]: nothing -> string {
    let flags = ["--version" "-V" "version" "help"]

    for flag in $flags {
        let result = do { ^$binary $flag } | complete
        if $result.exit_code == 0 and ($result.stdout | str trim | is-not-empty) {
            return $result.stdout
        }
    }

    ""
}

# Verifies that version string appears in output
export def check-version-in-output [output: string, expected_version: string]: nothing -> bool {
    $output | str contains $expected_version
}

# Verifies an installed binary exists and reports correct version
export def verify-installed-binary [bin_path: string, expected_version: string] {
    print $"(ansi green)Verifying binary...(ansi reset)"

    if not ($bin_path | path exists) {
        error $"binary not found at ($bin_path)"
    }

    let version_output = get-version-output $bin_path
    if not (check-version-in-output $version_output $expected_version) {
        error $"version mismatch: expected ($expected_version) in output: ($version_output)"
    }
    print $"  Version ($expected_version) ✓"
}
