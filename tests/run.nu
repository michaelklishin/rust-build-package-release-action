#!/usr/bin/env nu

# Test runner for rust-release-action Nu shell scripts

def main [] {
    let test_dir = $env.FILE_PWD
    let scripts_dir = ($test_dir | path dirname | path join "scripts")
    $env.NU_LIB_DIRS = [$scripts_dir]

    print $"(ansi green_bold)Running tests...(ansi reset)"
    print ""

    let test_files = glob $"($test_dir)/nu/*.nu"
    mut passed = 0
    mut failed = 0

    for file in $test_files {
        let name = $file | path basename
        print $"  ($name)..."
        try {
            nu $file
            print $"    (ansi green)PASS(ansi reset)"
            $passed = $passed + 1
        } catch {|e|
            print $"    (ansi red)FAIL(ansi reset): ($e.msg)"
            $failed = $failed + 1
        }
    }

    print ""
    print $"(ansi green_bold)Results:(ansi reset) ($passed) passed, ($failed) failed"

    if $failed > 0 {
        exit 1
    }
}
