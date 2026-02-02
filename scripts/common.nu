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

# Lists files in a directory, excluding archives, checksums, signatures, and SBOMs
export def list-archivable-files [dir: string]: nothing -> list<string> {
    let exclude = '\.(tar\.gz|zip|sha256|sha512|b2|sig|pem|sigstore\.json|spdx\.json|cdx\.json)$'
    ls $dir
        | where type == file
        | where { |f| not ($f.name =~ $exclude) }
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
        let result = if (which sha512sum | is-not-empty) {
            sha512sum $file_path | split row " " | first
        } else if (which shasum | is-not-empty) {
            shasum -a 512 $file_path | split row " " | first
        } else { "" }

        if $result != "" {
            let checksum_file = $"($file_path).sha512"
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

# Runs the pre-build hook command if PRE_BUILD is set
export def run-pre-build-hook [] {
    let pre_build = $env.PRE_BUILD? | default ""
    if $pre_build != "" {
        print $"(ansi green)Running pre-build hook...(ansi reset)"
        let result = do { bash -c $pre_build } | complete
        if $result.exit_code != 0 {
            error $"pre-build hook failed: ($result.stderr)"
        }
        if $result.stdout != "" {
            print $result.stdout
        }
    }
}

# Checks if a Cargo feature exists in the project
def has-cargo-feature [feature: string]: nothing -> bool {
    let package = $env.PACKAGE? | default ""
    let args = if $package != "" {
        ["metadata" "--format-version" "1" "--no-deps" "--package" $package]
    } else {
        ["metadata" "--format-version" "1" "--no-deps"]
    }
    let metadata = do { cargo ...$args } | complete
    if $metadata.exit_code != 0 {
        return false
    }
    let data = $metadata.stdout | from json
    let pkg = if $package != "" {
        $data.packages | where name == $package | first
    } else {
        $data.packages | first
    }
    $feature in ($pkg.features | columns)
}

# Builds with cargo rustc using the environment configuration
export def cargo-build [target: string, binary_name: string] {
    let package = $env.PACKAGE? | default ""
    let no_default_features = $env.NO_DEFAULT_FEATURES? | default "" | $in == "true"
    mut features = $env.FEATURES? | default ""
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

    # For musl targets, auto-enable mimalloc feature if available (improves performance)
    if ($target =~ "musl") and (not ($features =~ "mimalloc")) and (has-cargo-feature "mimalloc") {
        print $"(ansi green)Enabling mimalloc feature for musl build(ansi reset)"
        $features = if $features == "" { "mimalloc" } else { $"($features),mimalloc" }
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

# Checks that nfpm is available, installs if missing
export def check-nfpm [] {
    if (which nfpm | is-empty) {
        print $"(ansi yellow)nfpm not found, installing...(ansi reset)"
        let arch = if (^uname -m | str trim) == "aarch64" { "arm64" } else { "x86_64" }
        let nfpm_version = "2.44.2"
        let url = $"https://github.com/goreleaser/nfpm/releases/download/v($nfpm_version)/nfpm_($nfpm_version)_Linux_($arch).tar.gz"
        http get $url | tar xz -C /tmp nfpm
        sudo mv /tmp/nfpm /usr/local/bin/nfpm
    }
}

# Formats a list of dependencies for nfpm YAML
export def format-dependency-list [key: string, raw: string]: nothing -> string {
    if $raw == "" {
        return ""
    }
    let items = $raw | split row "," | each {|i| $i | str trim } | where {|i| $i != ""}
    if ($items | is-empty) {
        return ""
    }
    mut result = $"($key):\n"
    for item in $items {
        $result = $result + $"  - \"($item)\"\n"
    }
    $result
}

# Generates base nfpm config header
export def nfpm-base-config [
    binary_name: string
    version: string
    arch: string
]: nothing -> string {
    let description = $env.PKG_DESCRIPTION? | default $"($binary_name) - built with rust-release-action"
    let maintainer = $env.PKG_MAINTAINER? | default "Unknown <unknown@example.com>"
    let homepage = $env.PKG_HOMEPAGE? | default ""
    let license = $env.PKG_LICENSE? | default ""
    let vendor = $env.PKG_VENDOR? | default ""

    mut config = $"name: \"($binary_name)\"
arch: \"($arch)\"
platform: linux
version: \"($version)\"
maintainer: \"($maintainer)\"
description: \"($description)\"
"
    if $homepage != "" { $config = $config + $"homepage: \"($homepage)\"\n" }
    if $license != "" { $config = $config + $"license: \"($license)\"\n" }
    if $vendor != "" { $config = $config + $"vendor: \"($vendor)\"\n" }
    $config
}

# Generates nfpm contents section for binary and docs
export def nfpm-contents-section [binary_name: string, binary_path: string]: nothing -> string {
    mut config = $"
contents:
  - src: \"($binary_path)\"
    dst: \"/usr/bin/($binary_name)\"
    file_info:
      mode: 0755
"
    let licenses = glob LICENSE* | each {|f| $f | path expand }
    for lic in $licenses {
        let basename = $lic | path basename
        $config = $config + $"  - src: \"($lic)\"
    dst: \"/usr/share/doc/($binary_name)/($basename)\"
    file_info:
      mode: 0644
"
    }

    if ("README.md" | path exists) {
        let readme = "README.md" | path expand
        $config = $config + $"  - src: \"($readme)\"
    dst: \"/usr/share/doc/($binary_name)/README.md\"
    file_info:
      mode: 0644
"
    }

    let includes_raw = $env.PKG_CONTENTS? | default ""
    if $includes_raw != "" {
        let includes = $includes_raw | split row "," | each {|i| $i | str trim } | where {|i| $i != ""}
        for inc in $includes {
            let parts = $inc | split row ":"
            if ($parts | length) == 2 {
                let src = $parts | first | path expand
                let dst = $parts | last
                $config = $config + $"  - src: \"($src)\"
    dst: \"($dst)\"
"
            }
        }
    }
    $config
}

# Generates nfpm dependency sections
export def nfpm-dependencies-section []: nothing -> string {
    let depends = $env.PKG_DEPENDS? | default ""
    let recommends = $env.PKG_RECOMMENDS? | default ""
    let suggests = $env.PKG_SUGGESTS? | default ""
    let conflicts = $env.PKG_CONFLICTS? | default ""
    let replaces = $env.PKG_REPLACES? | default ""
    let provides = $env.PKG_PROVIDES? | default ""

    mut config = ""
    $config = $config + (format-dependency-list "depends" $depends)
    $config = $config + (format-dependency-list "recommends" $recommends)
    $config = $config + (format-dependency-list "suggests" $suggests)
    $config = $config + (format-dependency-list "conflicts" $conflicts)
    $config = $config + (format-dependency-list "replaces" $replaces)
    $config = $config + (format-dependency-list "provides" $provides)
    $config
}

# Installs cross-compilation dependencies for Linux targets
export def install-linux-cross-deps [target: string] {
    let is_ubuntu = (which apt-get | is-not-empty)
    let is_fedora = (which dnf | is-not-empty)

    if $target =~ "musl" {
        if $is_ubuntu {
            sudo apt-get update -qq
            sudo apt-get install -y -qq musl-tools
        }
    } else if $target == "aarch64-unknown-linux-gnu" {
        let arch = (^uname -m | str trim)
        if $arch != "aarch64" {
            if $is_ubuntu {
                sudo apt-get update -qq
                sudo apt-get install -y -qq gcc-aarch64-linux-gnu
            } else if $is_fedora {
                sudo dnf install -y gcc-aarch64-linux-gnu
            }
            $env.CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER = "aarch64-linux-gnu-gcc"
        }
    } else if $target == "armv7-unknown-linux-gnueabihf" {
        if $is_ubuntu {
            sudo apt-get update -qq
            sudo apt-get install -y -qq pkg-config gcc-arm-linux-gnueabihf
        } else if $is_fedora {
            sudo dnf install -y pkg-config gcc-arm-linux-gnueabihf
        }
        $env.CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER = "arm-linux-gnueabihf-gcc"
    }

    rustup target add $target
}

# Checks that cargo-sbom is available, installs if missing
export def check-cargo-sbom [] {
    if (which cargo-sbom | is-empty) {
        print $"(ansi yellow)cargo-sbom not found, installing...(ansi reset)"
        cargo install cargo-sbom
    }
}

# Generates SPDX and CycloneDX SBOMs
export def generate-sbom [output_dir: string, binary_name: string, version: string]: nothing -> record<spdx: string, cyclonedx: string> {
    let spdx_path = $"($output_dir)/($binary_name)-($version).spdx.json"
    let cyclonedx_path = $"($output_dir)/($binary_name)-($version).cdx.json"

    print $"(ansi green)Generating SPDX SBOM...(ansi reset)"
    let result = do { cargo sbom --output-format spdx_json_2_3 } | complete
    if $result.exit_code != 0 {
        error $"cargo-sbom SPDX generation failed: ($result.stderr)"
    }
    $result.stdout | save -f $spdx_path

    print $"(ansi green)Generating CycloneDX SBOM...(ansi reset)"
    let result = do { cargo sbom --output-format cyclone_dx_json_1_4 } | complete
    if $result.exit_code != 0 {
        error $"cargo-sbom CycloneDX generation failed: ($result.stderr)"
    }
    $result.stdout | save -f $cyclonedx_path

    if not ($spdx_path | path exists) {
        error $"SPDX SBOM was not created: ($spdx_path)"
    }
    if not ($cyclonedx_path | path exists) {
        error $"CycloneDX SBOM was not created: ($cyclonedx_path)"
    }

    { spdx: $spdx_path, cyclonedx: $cyclonedx_path }
}
