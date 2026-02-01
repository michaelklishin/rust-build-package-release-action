#!/usr/bin/env nu

use common.nu [get-cargo-info, output, hr-line, error, check-rust-toolchain, check-cargo-sbom, generate-sbom]

def main [] {
    check-rust-toolchain
    check-cargo-sbom

    let info = get-cargo-info
    let binary_name = $env.BINARY_NAME? | default $info.name
    let version = $info.version

    if $binary_name == "" {
        error "could not determine binary name"
    }
    if $version == "" {
        error "could not determine version"
    }

    print $"(ansi green)Generating SBOM:(ansi reset) ($binary_name) v($version)"

    let output_dir = $env.SBOM_OUTPUT_DIR? | default "target/sbom"
    mkdir $output_dir

    let sbom_paths = generate-sbom $output_dir $binary_name $version

    print $"(char nl)(ansi green)SBOM files:(ansi reset)"
    hr-line
    let sbom_pattern = '.spdx.json$|.cdx.json$'
    ls $output_dir | where { |f| $f.name =~ $sbom_pattern } | print

    output "version" $version
    output "binary_name" $binary_name
    output "sbom_spdx" $sbom_paths.spdx
    output "sbom_cyclonedx" $sbom_paths.cyclonedx
}
