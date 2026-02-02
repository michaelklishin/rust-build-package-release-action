#!/usr/bin/env nu

use common.nu [get-cargo-info, output, copy-docs, copy-includes, ensure-lockfile, cargo-build, hr-line, error, check-rust-toolchain, generate-checksums, output-build-results, run-pre-build-hook]

def main [] {
    let skip_build = $env.SKIP_BUILD? | default "" | $in == "true"
    let custom_binary_path = $env.BINARY_PATH? | default ""

    if not $skip_build {
        check-rust-toolchain
    }

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
    let binary_path = if $skip_build and $custom_binary_path != "" {
        $custom_binary_path
    } else {
        $"($release_dir)/($binary_name)"
    }

    if not ($binary_path | path exists) {
        if $skip_build {
            error $"binary not found: ($binary_path)"
        }
        print $"(ansi yellow)Binary not found, building...(ansi reset)"
        rm -rf $release_dir
        mkdir $release_dir
        ensure-lockfile
        run-pre-build-hook
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

    # Create install and uninstall scripts
    create-install-script $dmg_dir $binary_name
    create-uninstall-script $dmg_dir $binary_name

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

def create-install-script [dir: string, binary_name: string] {
    let script = [
        "#!/bin/bash"
        $"# Install ($binary_name) to /usr/local/bin"
        "set -e"
        ""
        "INSTALL_DIR=\"/usr/local/bin\""
        $"BINARY=\"($binary_name)\""
        "SCRIPT_DIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\""
        ""
        "if [ ! -f \"$SCRIPT_DIR/$BINARY\" ]; then"
        "    echo \"Error: $BINARY not found in $SCRIPT_DIR\""
        "    exit 1"
        "fi"
        ""
        "echo \"Installing $BINARY to $INSTALL_DIR...\""
        "sudo mkdir -p \"$INSTALL_DIR\""
        "sudo cp \"$SCRIPT_DIR/$BINARY\" \"$INSTALL_DIR/$BINARY\""
        "sudo chmod +x \"$INSTALL_DIR/$BINARY\""
        "echo \"Done. Run '$BINARY --help' to get started.\""
    ] | str join "\n"
    $script | save -f $"($dir)/install.sh"
    chmod +x $"($dir)/install.sh"
}

def create-uninstall-script [dir: string, binary_name: string] {
    let script = [
        "#!/bin/bash"
        $"# Uninstall ($binary_name) from /usr/local/bin"
        "set -e"
        ""
        "INSTALL_DIR=\"/usr/local/bin\""
        $"BINARY=\"($binary_name)\""
        ""
        "if [ -f \"$INSTALL_DIR/$BINARY\" ]; then"
        "    echo \"Removing $BINARY from $INSTALL_DIR...\""
        "    sudo rm -f \"$INSTALL_DIR/$BINARY\""
        "    echo \"Done. $BINARY has been uninstalled.\""
        "else"
        "    echo \"$BINARY is not installed in $INSTALL_DIR\""
        "fi"
    ] | str join "\n"
    $script | save -f $"($dir)/uninstall.sh"
    chmod +x $"($dir)/uninstall.sh"
}
