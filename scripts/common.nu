#!/usr/bin/env nu

# Reads Cargo.toml and returns the package name and version
export def get-cargo-info []: nothing -> record<name: string, version: string> {
    let manifest = $env.MANIFEST_PATH? | default "Cargo.toml"
    let cargo = open $manifest
    let name = $cargo | get -o package.name | default ""
    let version = $cargo | get -o package.version | default ($cargo | get -o workspace.package.version | default "")
    { name: $name, version: $version }
}

# Writes key=value to GITHUB_OUTPUT if available
export def output [key: string, value: string] {
    if ($env.GITHUB_OUTPUT? | is-not-empty) {
        $"($key)=($value)\n" | save --append $env.GITHUB_OUTPUT
    }
}

# Writes a multiline value to GITHUB_OUTPUT using heredoc syntax
export def output-multiline [key: string, value: string] {
    if ($env.GITHUB_OUTPUT? | is-not-empty) {
        let delimiter = "EOF_RUST_RELEASE_ACTION"
        $"($key)<<($delimiter)\n($value)\n($delimiter)\n" | save --append $env.GITHUB_OUTPUT
    }
}

# Copies LICENSE* and README.md to the destination
export def copy-docs [dest: string] {
    glob LICENSE* | each {|f| cp $f $dest }
    if ("README.md" | path exists) { cp README.md $dest }
}

# Copies additional include files to the destination
export def copy-includes [dest: string] {
    let includes = $env.INCLUDE? | default ""
    if $includes != "" {
        $includes | split row "," | each {|pattern|
            let pattern = $pattern | str trim
            if $pattern != "" {
                glob $pattern | each {|f| cp $f $dest }
            }
        }
    }
}

# Lists files in a directory, excluding archives and checksums
export def list-archivable-files [dir: string]: nothing -> list<string> {
    ls $dir
        | where type == file
        | where { |f| not ($f.name =~ '\.(tar\.gz|zip|sha256|sha512|b2)$') }
        | get name
        | path basename
}

# Ensures Cargo.lock exists
export def ensure-lockfile [] {
    if not ("Cargo.lock" | path exists) {
        print $"(ansi yellow)Generating Cargo.lock...(ansi reset)"
        cargo generate-lockfile
    }
}

# Prints a horizontal line marker
export def hr-line [] {
    print $"(ansi green)---------------------------------------------------------------------------->(ansi reset)"
}

# Prints an error message and exits
export def error [msg: string] {
    print $"(ansi red)ERROR:(ansi reset) ($msg)"
    exit 1
}

# Checks that Rust toolchain is available
export def check-rust-toolchain [] {
    if (which cargo | is-empty) {
        print $"(ansi red)ERROR:(ansi reset) Rust toolchain not found"
        print ""
        print "Add a Rust setup step before this action:"
        print "  - uses: dtolnay/rust-toolchain@stable"
        print ""
        print "Or install manually:"
        print "  rustup toolchain install stable --profile minimal"
        exit 1
    }
}

# Generates checksums for a file
export def generate-checksums [file_path: string]: nothing -> record<sha256: string, sha512: string, b2: string> {
    let checksum_types = $env.CHECKSUM? | default "sha256"

    mut checksums = {sha256: "", sha512: "", b2: ""}

    if ($checksum_types | str contains "sha256") or $checksum_types == "" {
        let hash = (open $file_path --raw | hash sha256)
        let checksum_file = $"($file_path).sha256"
        $"($hash)  ($file_path | path basename)\n" | save -f $checksum_file
        $checksums.sha256 = $hash
        print $"(ansi green)SHA256:(ansi reset) ($hash)"
    }

    if ($checksum_types | str contains "sha512") {
        let checksum_file = $"($file_path).sha512"
        if (which sha512sum | is-not-empty) {
            let result = (sha512sum $file_path | split row " " | first)
            $"($result)  ($file_path | path basename)\n" | save -f $checksum_file
            $checksums.sha512 = $result
            print $"(ansi green)SHA512:(ansi reset) ($result)"
        } else if (which shasum | is-not-empty) {
            let result = (shasum -a 512 $file_path | split row " " | first)
            $"($result)  ($file_path | path basename)\n" | save -f $checksum_file
            $checksums.sha512 = $result
            print $"(ansi green)SHA512:(ansi reset) ($result)"
        }
    }

    if ($checksum_types | str contains "b2") {
        if (which b2sum | is-not-empty) {
            let result = (b2sum $file_path | split row " " | first)
            let checksum_file = $"($file_path).b2"
            $"($result)  ($file_path | path basename)\n" | save -f $checksum_file
            $checksums.b2 = $result
            print $"(ansi green)BLAKE2:(ansi reset) ($result)"
        }
    }

    $checksums
}

# Builds with cargo rustc using the environment configuration
export def cargo-build [target: string, binary_name: string] {
    let package = $env.PACKAGE? | default ""
    let no_default_features = $env.NO_DEFAULT_FEATURES? | default "" | $in == "true"
    let features = $env.FEATURES? | default ""
    let locked = $env.LOCKED? | default "" | $in == "true"
    let profile = $env.PROFILE? | default "release"
    let target_rustflags = $env.TARGET_RUSTFLAGS? | default ""

    if $target_rustflags != "" {
        $env.RUSTFLAGS = $target_rustflags
    }

    # For musl targets, enables static linking
    if ($target =~ "musl") and ($env.RUSTFLAGS? | default "" | is-empty) {
        $env.RUSTFLAGS = "-C target-feature=+crt-static"
    }

    mut args = ["rustc" "--target" $target "-q"]

    # Handle profile
    if $profile == "release" {
        $args = ($args | append "--release")
    } else if $profile != "dev" {
        $args = ($args | append ["--profile" $profile])
    }

    if $package != "" {
        $args = ($args | append ["--package" $package])
    }

    if $binary_name != "" {
        $args = ($args | append ["--bin" $binary_name])
    }

    if $no_default_features {
        $args = ($args | append "--no-default-features")
    }

    if $features != "" {
        $args = ($args | append ["--features" $features])
    }

    if $locked {
        $args = ($args | append "--locked")
    }

    cargo ...$args
}

# Outputs build results to GITHUB_OUTPUT
export def output-build-results [
    binary_name: string
    version: string
    target: string
    artifact: string
    artifact_path: string
    checksums: record<sha256: string, sha512: string, b2: string>
] {
    output "artifact" $artifact
    output "artifact_path" $artifact_path
    output "sha256" $checksums.sha256
    output "sha512" $checksums.sha512
    output "b2" $checksums.b2

    let summary = build-summary $binary_name $version $target $artifact $artifact_path $checksums
    output-multiline "summary" $summary
}

# Generates a JSON summary of the build
export def build-summary [
    binary_name: string
    version: string
    target: string
    artifact: string
    artifact_path: string
    checksums: record<sha256: string, sha512: string, b2: string>
]: nothing -> string {
    let summary = {
        binary_name: $binary_name
        version: $version
        target: $target
        artifact: $artifact
        artifact_path: $artifact_path
        sha256: $checksums.sha256
        sha512: $checksums.sha512
        b2: $checksums.b2
    }
    $summary | to json
}
