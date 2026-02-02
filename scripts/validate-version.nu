#!/usr/bin/env nu

use common.nu [output, error, get-cargo-info]

def main [] {
    let tag = $env.TAG? | default ($env.GITHUB_REF_NAME? | default "")
    let expected = $env.EXPECTED_VERSION? | default ($env.NEXT_RELEASE_VERSION? | default "")
    let validate_cargo = $env.VALIDATE_CARGO_TOML? | default "" | $in == "true"

    if $tag == "" {
        print $"(ansi red)ERROR:(ansi reset) GITHUB_REF_NAME is not available"
        print ""
        print "Set TAG to the git tag being released (e.g., v1.2.3)"
        exit 1
    }

    if $expected == "" {
        print $"(ansi red)ERROR:(ansi reset) NEXT_RELEASE_VERSION variable is not set"
        print ""
        print "Set it at: Settings > Secrets and variables > Actions > Variables"
        exit 1
    }

    # Check if this looks like a version tag (supports semver with pre-release)
    if not ($tag | str starts-with "v") {
        print $"(ansi red)ERROR:(ansi reset) Tag should start with 'v', got '($tag)'"
        print ""
        print $"Push a tag like: git tag v($expected) && git push origin v($expected)"
        exit 1
    }

    # Extract version from tag (v0.14.0 -> 0.14.0, v1.0.0-beta.1 -> 1.0.0-beta.1)
    let tag_version = $tag | str substring 1..

    # Validate version format (semver with optional pre-release and build metadata)
    let semver_pattern = '^\d+\.\d+\.\d+(-[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?(\+[a-zA-Z0-9]+(\.[a-zA-Z0-9]+)*)?$'
    if not ($tag_version =~ $semver_pattern) {
        print $"(ansi red)ERROR:(ansi reset) Invalid version format: ($tag_version)"
        print ""
        print "Expected semantic versioning: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]"
        print "Examples: 1.2.3, 1.0.0-alpha.1, 2.0.0-rc.1+build.123"
        exit 1
    }

    if $expected == $tag_version {
        print $"(ansi green)Version validated:(ansi reset) ($expected) matches tag ($tag)"
        output "version" $tag_version
    } else {
        print $"(ansi red)ERROR:(ansi reset) NEXT_RELEASE_VERSION (($expected)) does not match tag (($tag))"
        print ""
        print "Either:"
        print $"  1. Update NEXT_RELEASE_VERSION to '($tag_version)' at: Settings > Secrets and variables > Actions > Variables"
        print $"  2. Or push the correct tag: git tag v($expected) && git push origin v($expected)"
        exit 1
    }

    if $validate_cargo {
        let cargo_info = get-cargo-info
        let cargo_version = $cargo_info.version
        if $cargo_version == "" {
            error "Could not read version from Cargo.toml"
        }
        if $cargo_version != $tag_version {
            print $"(ansi red)ERROR:(ansi reset) Cargo.toml version (($cargo_version)) does not match tag (($tag_version))"
            print ""
            print $"Update Cargo.toml version to '($tag_version)' before tagging"
            exit 1
        }
        print $"(ansi green)Cargo.toml validated:(ansi reset) version ($cargo_version) matches tag"
    }
}
