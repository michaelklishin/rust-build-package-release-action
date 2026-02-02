#!/usr/bin/env nu

# Tests for changelog parsing logic

use std/assert

def main [] {
    test-version-header-matching
    test-section-extraction
}

def test-version-header-matching [] {
    # Test that version headers are correctly identified
    let lines = [
        "# Changelog"
        ""
        "## v1.2.3 (Jan 1, 2025)"
        ""
        "- Feature A"
        "- Bug fix B"
        ""
        "## v1.2.2 (Dec 15, 2024)"
        ""
        "- Old feature"
    ]

    # Test with v prefix
    let version_header = "## v1.2.3"
    let start_idx = $lines | enumerate | where {|row|
        $row.item | str starts-with $version_header
    } | get -o 0.index

    assert equal $start_idx 2

    # Test without v prefix (alternate header format)
    let alt_header = "## 1.2.3"
    let alt_lines = [
        "# Changelog"
        ""
        "## 1.2.3 (Jan 1, 2025)"
        ""
        "- Feature A"
    ]

    let alt_start_idx = $alt_lines | enumerate | where {|row|
        $row.item | str starts-with $alt_header
    } | get -o 0.index

    assert equal $alt_start_idx 2
}

def test-section-extraction [] {
    let lines = [
        "# Changelog"
        ""
        "## v2.0.0 (Feb 1, 2025)"
        ""
        "### Breaking Changes"
        "- Changed API"
        ""
        "### Features"
        "- New feature"
        ""
        "## v1.9.0 (Jan 15, 2025)"
        ""
        "- Old stuff"
    ]

    let version = "2.0.0"
    let version_header = $"## v($version)"

    let start_idx = $lines | enumerate | where {|row|
        $row.item | str starts-with $version_header
    } | get -o 0.index

    assert equal $start_idx 2

    let remaining = $lines | skip ($start_idx + 1)
    let next_version_pattern = '^## v?\d+\.\d+\.\d+'
    let end_offset = $remaining | enumerate | where {|row|
        $row.item =~ $next_version_pattern
    } | get -o 0.index | default ($remaining | length)

    # Should find v1.9.0 header at offset 7 (relative to remaining)
    assert equal $end_offset 7

    let section = $lines | skip $start_idx | take ($end_offset + 1)
    assert equal ($section | length) 8
    assert equal ($section | first) "## v2.0.0 (Feb 1, 2025)"
    assert ($section | str join "\n" | str contains "Breaking Changes")
    assert ($section | str join "\n" | str contains "New feature")
}
