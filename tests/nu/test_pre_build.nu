#!/usr/bin/env nu

# Tests for run-pre-build-hook function

use std/assert
use common.nu [run-pre-build-hook]

def main [] {
    test-pre-build-hook-empty
    # Skip shell command test on Windows (shell syntax differs)
    if (sys host | get name) != "Windows" {
        test-pre-build-hook-success
    }
}

def test-pre-build-hook-empty [] {
    # When PRE_BUILD is not set or empty, should do nothing
    $env.PRE_BUILD = ""

    # This should not error
    run-pre-build-hook

    # Clean up
    hide-env PRE_BUILD
}

def test-pre-build-hook-success [] {
    # Test successful command execution (Unix only, uses shell pipes/std stream redirection)
    let temp_dir = $env.TMPDIR? | default "/tmp"
    let temp_file = $"($temp_dir)/test-pre-build-(random uuid)"
    $env.PRE_BUILD = $"echo 'test content' > ($temp_file)"

    run-pre-build-hook

    # Verify the command ran
    assert ($temp_file | path exists)
    let content = open $temp_file | str trim
    assert equal $content "test content"

    # Clean up
    rm -f $temp_file
    hide-env PRE_BUILD
}
