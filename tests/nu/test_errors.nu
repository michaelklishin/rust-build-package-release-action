#!/usr/bin/env nu

# Tests for error handling and edge cases

use std/assert

def main [] {
    test-semver-edge-cases
    test-empty-inputs
}

def test-semver-edge-cases [] {
    let semver_pattern = '^\d+\.\d+\.\d+(-[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?(\+[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?$'

    # Edge cases that should be valid
    assert ("0.0.0" =~ $semver_pattern)
    assert ("999.999.999" =~ $semver_pattern)
    assert ("1.0.0-0" =~ $semver_pattern)
    assert ("1.0.0-alpha0" =~ $semver_pattern)
    assert ("1.0.0+20240101" =~ $semver_pattern)

    # Edge cases that should be invalid
    assert (not ("" =~ $semver_pattern))
    assert (not ("1" =~ $semver_pattern))
    assert (not ("1.0" =~ $semver_pattern))
    assert (not ("1.0.0." =~ $semver_pattern))
    assert (not (".1.0.0" =~ $semver_pattern))
    assert (not ("1..0.0" =~ $semver_pattern))
    assert (not ("1.0.0-" =~ $semver_pattern))
    assert (not ("1.0.0+" =~ $semver_pattern))
    assert (not ("1.0.0--alpha" =~ $semver_pattern))
    assert (not ("1.0.0-alpha..1" =~ $semver_pattern))
    assert (not ("v1.0.0" =~ $semver_pattern))
    assert (not (" 1.0.0" =~ $semver_pattern))
    assert (not ("1.0.0 " =~ $semver_pattern))
}

def test-empty-inputs [] {
    # Test that empty strings are handled correctly
    assert ("" | is-empty)
    assert (not ("value" | is-empty))

    # Nu shell's `default` only replaces null, not empty strings
    # Use `| default ""` pattern with `is-empty` check for empty string handling
    let val = "" | default "fallback"
    assert equal $val ""  # Empty string is NOT replaced

    let val = null | default "fallback"
    assert equal $val "fallback"  # Null IS replaced

    let val = "provided" | default "fallback"
    assert equal $val "provided"
}
