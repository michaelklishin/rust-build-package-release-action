#!/usr/bin/env nu

# Tests for zigbuild support

use std/assert

def main [] {
    test-use-zigbuild-env-parsing
    test-musl-target-detection
}

def test-use-zigbuild-env-parsing [] {
    assert equal (parse-zigbuild-env "true") true
    assert equal (parse-zigbuild-env "false") false
    assert equal (parse-zigbuild-env "") false
    assert equal (parse-zigbuild-env "TRUE") false
    assert equal (parse-zigbuild-env "yes") false
}

def parse-zigbuild-env [val: string]: nothing -> bool {
    $val == "true"
}

def test-musl-target-detection [] {
    let musl_targets = [
        "aarch64-unknown-linux-musl"
        "x86_64-unknown-linux-musl"
        "armv7-unknown-linux-musleabihf"
    ]

    for target in $musl_targets {
        assert (is-musl-target $target) $"($target) should be musl"
    }

    let non_musl_targets = [
        "x86_64-unknown-linux-gnu"
        "aarch64-apple-darwin"
        "x86_64-pc-windows-msvc"
    ]

    for target in $non_musl_targets {
        assert (not (is-musl-target $target)) $"($target) should not be musl"
    }
}

def is-musl-target [target: string]: nothing -> bool {
    $target =~ "musl"
}
