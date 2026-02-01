#!/usr/bin/env nu

use common.nu [get-cargo-info, output, copy-docs, copy-includes, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, output-build-results]

def main [] {
    check-rust-toolchain

    let target = $env.TARGET? | default "aarch64-apple-darwin"
    let info = get-cargo-info
    let binary_name = $env.BINARY_NAME? | default $info.name
    let version = $info.version

    if $binary_name == "" {
        error "could not determine binary name"
    }
    if $version == "" {
        error "could not determine version"
    }

    print $"(ansi green)Building .dmg installer:(ansi reset) ($binary_name) v($version) for ($target)"

    let release_dir = $"target/($target)/release"
    let binary_path = $"($release_dir)/($binary_name)"

    if not ($binary_path | path exists) {
        print $"(ansi yellow)Binary not found, building...(ansi reset)"
        rm -rf $release_dir
        mkdir $release_dir
        ensure-lockfile
        rustup target add $target
        cargo-build $target $binary_name
    }

    if not ($binary_path | path exists) {
        error $"binary not found: ($binary_path)"
    }

    let dmg_dir = "target/dmg-contents"
    rm -rf $dmg_dir
    mkdir $dmg_dir

    cp $binary_path $dmg_dir
    chmod +x $"($dmg_dir)/($binary_name)"
    copy-docs $dmg_dir
    copy-includes $dmg_dir

    let vol_name = $"($binary_name)-($version)"
    let artifact = $"($binary_name)-($version)-($target).dmg"
    let artifact_path = $"($release_dir)/($artifact)"

    print $"(ansi green)Creating DMG...(ansi reset)"
    create-dmg $dmg_dir $vol_name $artifact_path

    if not ($artifact_path | path exists) {
        error $"failed to create DMG: ($artifact_path)"
    }

    let checksums = generate-checksums $artifact_path
    print $"(char nl)(ansi green)Build artifacts:(ansi reset)"
    hr-line
    let dmg_pattern = '.dmg$'
    ls $release_dir | where { |f| $f.name =~ $dmg_pattern } | print
    print $"(ansi green)Created:(ansi reset) ($artifact)"

    output "version" $version
    output "binary_name" $binary_name
    output "target" $target
    output "binary_path" $binary_path
    output-build-results $binary_name $version $target $artifact $artifact_path $checksums
}

def create-dmg [src_dir: string, vol_name: string, output_path: string] {
    let temp_dmg = $"($output_path).temp.dmg"

    # Create writable DMG from source folder
    let result = do { hdiutil create -srcfolder $src_dir -volname $vol_name -fs HFS+ -format UDRW -ov $temp_dmg } | complete
    if $result.exit_code != 0 {
        error $"hdiutil create failed: ($result.stderr)"
    }

    # Convert to compressed read-only DMG (UDZO for broad compatibility with macOS 10.6+)
    let result = do { hdiutil convert $temp_dmg -format UDZO -o $output_path } | complete
    if $result.exit_code != 0 {
        rm -f $temp_dmg
        error $"hdiutil convert failed: ($result.stderr)"
    }

    rm -f $temp_dmg
}
