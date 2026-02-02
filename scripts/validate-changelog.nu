#!/usr/bin/env nu

# Validates that a changelog entry exists for the specified version

use common.nu [output, error]

def main [] {
    let version = $env.VERSION? | default ""
    if $version == "" {
        error "VERSION environment variable is required"
    }

    let changelog_path = $env.CHANGELOG_PATH? | default "CHANGELOG.md"

    if not ($changelog_path | path exists) {
        error $"changelog not found: ($changelog_path)"
    }

    let lines = open $changelog_path | lines
    let version_header = $"## v($version)"
    let alt_header = $"## ($version)"

    let found = $lines | any {|line|
        ($line | str starts-with $version_header) or ($line | str starts-with $alt_header)
    }

    if $found {
        print $"(ansi green)Changelog validated:(ansi reset) found entry for v($version)"
        output "version" $version
        output "valid" "true"
    } else {
        error $"No changelog entry found for version ($version). Expected header like '## v($version)' or '## ($version)'"
    }
}
