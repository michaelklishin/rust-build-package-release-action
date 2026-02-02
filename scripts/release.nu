#!/usr/bin/env nu

# Unified release command that auto-selects the platform-specific release script

use common.nu [error]

def main [] {
    let target = $env.TARGET? | default ""

    if $target == "" {
        error "TARGET is required for the unified release command"
    }

    let scripts = $env.GITHUB_ACTION_PATH? | default "." | path join "scripts"

    # Determine platform from target triple
    let script = if ($target | str contains "linux") {
        "release-linux.nu"
    } else if ($target | str contains "darwin") or ($target | str contains "apple") {
        "release-macos.nu"
    } else if ($target | str contains "windows") {
        "release-windows.nu"
    } else {
        error $"Cannot determine platform from target: ($target). Use release-linux, release-macos, or release-windows directly."
    }

    print $"(ansi green)Auto-selected:(ansi reset) ($script) for target ($target)"

    # Execute the platform-specific script
    nu $"($scripts)/($script)"
}
