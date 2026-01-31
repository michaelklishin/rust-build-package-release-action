#!/usr/bin/env nu

use common.nu [output, error]

def main [] {
    let version = $env.VERSION? | default ""
    if $version == "" {
        error "VERSION environment variable is required"
    }

    let changelog_path = $env.CHANGELOG_PATH? | default "CHANGELOG.md"
    let output_path = $env.OUTPUT_PATH? | default "release_notes.md"

    if not ($changelog_path | path exists) {
        error $"changelog not found: ($changelog_path)"
    }

    let lines = open $changelog_path | lines
    let version_header = $"## v($version)"
    let alt_header = $"## ($version)"

    let start_idx = $lines | enumerate | where {|row|
        ($row.item | str starts-with $version_header) or ($row.item | str starts-with $alt_header)
    } | get -o 0.index

    if $start_idx == null {
        error $"version ($version) not found in changelog"
    }

    let remaining = $lines | skip ($start_idx + 1)
    let end_offset = $remaining | enumerate | where {|row|
        $row.item =~ '^## v?\d+\.\d+\.\d+'
    } | get -o 0.index | default ($remaining | length)

    # Include the header line in the output
    let section = $lines | skip $start_idx | take ($end_offset + 1)
    let notes = $section | str join "\n"

    if ($notes | str trim) == "" {
        error $"no content found for version ($version)"
    }

    $notes | save -f $output_path
    print $"(ansi green)Extracted(ansi reset) release notes for v($version) to ($output_path)"
    output "release_notes_file" $output_path
}
