#!/usr/bin/env nu

# Reads Cargo.toml and returns package name and version
export def get-cargo-info []: nothing -> record<name: string, version: string> {
    let cargo = open Cargo.toml
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

# Copies LICENSE-* and README.md to the destination
export def copy-docs [dest: string] {
    glob LICENSE-* | each {|f| cp $f $dest }
    if ("README.md" | path exists) { cp README.md $dest }
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

# Builds with cargo rustc using the environment configuration
export def cargo-build [target: string, binary_name: string] {
    let package = $env.PACKAGE? | default ""
    let no_default_features = $env.NO_DEFAULT_FEATURES? | default "" | $in == "true"
    let target_rustflags = $env.TARGET_RUSTFLAGS? | default ""

    if $target_rustflags != "" {
        $env.RUSTFLAGS = $target_rustflags
    }

    # For musl targets, enables static linking
    if ($target =~ "musl") and ($env.RUSTFLAGS? | default "" | is-empty) {
        $env.RUSTFLAGS = "-C target-feature=+crt-static"
    }

    mut args = ["rustc" "--release" "--target" $target "-q"]

    if $package != "" {
        $args = ($args | append ["--package" $package])
    }

    if $binary_name != "" {
        $args = ($args | append ["--bin" $binary_name])
    }

    if $no_default_features {
        $args = ($args | append "--no-default-features")
    }

    cargo ...$args
}
