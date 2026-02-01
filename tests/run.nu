#!/usr/bin/env nu

# Test runner for rust-release-action Nu shell scripts

def run-test [file: string, scripts_dir: string]: nothing -> bool {
    let result = do { ^nu --env-config '' --config '' -I $scripts_dir $file } | complete
    $result.exit_code == 0
}

def main [] {
    let test_dir = $env.FILE_PWD
    let scripts_dir = ($test_dir | path dirname | path join "scripts")

    print $"(ansi green_bold)Running tests...(ansi reset)"
    print ""

    let test_files = glob $"($test_dir)/nu/*.nu"
    let results = $test_files | each {|file|
        let name = $file | path basename
        print $"  ($name)..."
        let passed = run-test $file $scripts_dir
        if $passed {
            print $"    (ansi green)PASS(ansi reset)"
        } else {
            print $"    (ansi red)FAIL(ansi reset)"
        }
        $passed
    }

    let passed = $results | where $it == true | length
    let failed = $results | where $it == false | length

    print ""
    print $"(ansi green_bold)Results:(ansi reset) ($passed) passed, ($failed) failed"

    if $failed > 0 {
        exit 1
    }
}
