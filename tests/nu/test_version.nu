#!/usr/bin/env nu

# Tests for version validation patterns

use std/assert

def main [] {
    test-semver-pattern
}

def test-semver-pattern [] {
    # Pattern matching semantic versions with optional pre-release and build metadata
    let semver_pattern = '^\d+\.\d+\.\d+(-[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?(\+[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?$'

    # Valid versions
    assert ("1.0.0" =~ $semver_pattern)
    assert ("0.1.0" =~ $semver_pattern)
    assert ("10.20.30" =~ $semver_pattern)
    assert ("1.2.3-alpha" =~ $semver_pattern)
    assert ("1.2.3-alpha.1" =~ $semver_pattern)
    assert ("1.2.3-beta.2" =~ $semver_pattern)
    assert ("1.0.0-rc.1" =~ $semver_pattern)
    assert ("2.0.0+build.123" =~ $semver_pattern)
    assert ("1.0.0-alpha+build" =~ $semver_pattern)
    assert ("1.0.0-alpha.1+build.456" =~ $semver_pattern)

    # Invalid versions
    assert (not ("1.0" =~ $semver_pattern))
    assert (not ("1" =~ $semver_pattern))
    assert (not ("v1.0.0" =~ $semver_pattern))
    assert (not ("1.0.0." =~ $semver_pattern))
    assert (not ("1.0.0-" =~ $semver_pattern))
    assert (not ("1.0.0+" =~ $semver_pattern))
    assert (not ("a.b.c" =~ $semver_pattern))
    assert (not ("1.0.0--double" =~ $semver_pattern))
}
