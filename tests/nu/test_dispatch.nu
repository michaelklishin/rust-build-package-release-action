#!/usr/bin/env nu

# Integration tests for dispatch.nu - verifies env var propagation to subprocesses

use std/assert

# Test that dispatch.nu correctly propagates INPUT_* env vars to child scripts
#[test]
def "test dispatch get-version" [] {
    let test_toml = "/tmp/test-dispatch-cargo.toml"
    "[package]\nname = \"dispatch-test\"\nversion = \"5.6.7\"" | save -f $test_toml

    let scripts_dir = $env.FILE_PWD | path dirname | path dirname | path join "scripts"
    let action_path = $scripts_dir | path dirname

    let result = with-env {
        INPUT_COMMAND: "get-version"
        INPUT_MANIFEST: $test_toml
        GITHUB_ACTION_PATH: $action_path
        NU_LIB_DIRS: [$scripts_dir]
    } {
        nu ($scripts_dir | path join "dispatch.nu") | str trim
    }

    assert equal $result "5.6.7"
    rm -f $test_toml
}

# Test that VERSION env var is set correctly from INPUT_VERSION
#[test]
def "test dispatch version passthrough" [] {
    let test_toml = "/tmp/test-dispatch-version.toml"
    "[package]\nname = \"version-test\"\nversion = \"1.0.0\"" | save -f $test_toml

    let scripts_dir = $env.FILE_PWD | path dirname | path dirname | path join "scripts"
    let action_path = $scripts_dir | path dirname

    # get-version should work regardless of INPUT_VERSION
    let result = with-env {
        INPUT_COMMAND: "get-version"
        INPUT_MANIFEST: $test_toml
        INPUT_VERSION: "9.9.9"
        GITHUB_ACTION_PATH: $action_path
        NU_LIB_DIRS: [$scripts_dir]
    } {
        nu ($scripts_dir | path join "dispatch.nu") | str trim
    }

    # get-version reads from Cargo.toml, not VERSION env
    assert equal $result "1.0.0"
    rm -f $test_toml
}

# Test VERSION auto-detection from GITHUB_REF_NAME
#[test]
def "test dispatch version from tag" [] {
    let scripts_dir = $env.FILE_PWD | path dirname | path dirname | path join "scripts"
    let action_path = $scripts_dir | path dirname
    let changelog = $action_path | path join "CHANGELOG.md"

    # extract-changelog uses VERSION which should be auto-detected from tag
    let result = with-env {
        INPUT_COMMAND: "extract-changelog"
        INPUT_CHANGELOG: $changelog
        GITHUB_REF_NAME: "v1.0.0"
        GITHUB_ACTION_PATH: $action_path
        NU_LIB_DIRS: [$scripts_dir]
    } {
        nu ($scripts_dir | path join "dispatch.nu") | str trim
    }

    # Should extract v1.0.0 section from changelog
    assert ($result | str contains "Introduce a unified")
}
