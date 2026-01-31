#!/usr/bin/env nu

use common.nu [get-cargo-info, output, error]

def main [] {
    let manifest_path = $env.MANIFEST_PATH? | default "Cargo.toml"

    if not ($manifest_path | path exists) {
        error $"manifest not found: ($manifest_path)"
    }

    let info = get-cargo-info
    let version = $info.version

    if $version == "" {
        print $"(ansi red)ERROR:(ansi reset) no version found in Cargo.toml"
        print ""
        print "Ensure [package] or [workspace.package] has a version field"
        exit 1
    }

    if not ($version =~ '^\d+\.\d+\.\d+') {
        error $"invalid version format: ($version)"
    }

    print $version
    output "version" $version
}
