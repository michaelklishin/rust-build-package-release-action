#!/usr/bin/env nu

use common.nu [get-cargo-info, output, output-multiline, hr-line, error]

def main [] {
    let info = get-cargo-info
    let binary_name = $env.BINARY_NAME? | default $info.name
    let version = $env.VERSION? | default $info.version

    if $binary_name == "" {
        error "could not determine binary name"
    }
    if $version == "" {
        error "could not determine version"
    }

    let publisher = $env.WINGET_PUBLISHER? | default ""
    let publisher_id = $env.WINGET_PUBLISHER_ID? | default ($publisher | str replace -a " " "")
    let package_id = $env.WINGET_PACKAGE_ID? | default $binary_name
    let description = $env.PKG_DESCRIPTION? | default $"($binary_name) - built with rust-release-action"
    let homepage = $env.PKG_HOMEPAGE? | default ""
    let license = $env.PKG_LICENSE? | default "MIT"
    let license_url = $env.WINGET_LICENSE_URL? | default ""
    let copyright = $env.WINGET_COPYRIGHT? | default ""
    let tags = $env.WINGET_TAGS? | default ""
    let x64_url = $env.WINGET_X64_URL? | default ""
    let x64_sha256 = $env.WINGET_X64_SHA256? | default ""
    let arm64_url = $env.WINGET_ARM64_URL? | default ""
    let arm64_sha256 = $env.WINGET_ARM64_SHA256? | default ""

    if $publisher == "" {
        error "WINGET_PUBLISHER is required"
    }

    print $"(ansi green)Generating Winget manifest:(ansi reset) ($publisher_id).($package_id) v($version)"

    let manifest_id = $"($publisher_id).($package_id)"
    let output_dir = $env.WINGET_OUTPUT_DIR? | default "target/winget"
    let manifest_dir = $"($output_dir)/manifests/($publisher_id | split chars | first | str downcase)/($publisher_id)/($package_id)/($version)"
    mkdir $manifest_dir

    let version_manifest = generate-version-manifest $manifest_id $version
    let version_path = $"($manifest_dir)/($manifest_id).yaml"
    $version_manifest | save -f $version_path

    let locale_manifest = generate-locale-manifest {
        id: $manifest_id
        version: $version
        publisher: $publisher
        name: $binary_name
        description: $description
        homepage: $homepage
        license: $license
        license_url: $license_url
        copyright: $copyright
        tags: $tags
    }
    let locale_path = $"($manifest_dir)/($manifest_id).locale.en-US.yaml"
    $locale_manifest | save -f $locale_path

    let installer_manifest = generate-installer-manifest {
        id: $manifest_id
        version: $version
        x64_url: $x64_url
        x64_sha256: $x64_sha256
        arm64_url: $arm64_url
        arm64_sha256: $arm64_sha256
    }
    let installer_path = $"($manifest_dir)/($manifest_id).installer.yaml"
    $installer_manifest | save -f $installer_path

    print $"(char nl)(ansi green)Manifest files:(ansi reset)"
    hr-line
    print $"Version: ($version_path)"
    print $"Locale:  ($locale_path)"
    print $"Installer: ($installer_path)"
    hr-line

    print $"(char nl)(ansi green)Version manifest:(ansi reset)"
    print $version_manifest

    output "manifest_dir" $manifest_dir
    output "manifest_id" $manifest_id
    output "version_manifest" $version_path
    output "locale_manifest" $locale_path
    output "installer_manifest" $installer_path
}

export def generate-version-manifest [id: string, version: string]: nothing -> string {
    ["# yaml-language-server: $schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json"
     $"PackageIdentifier: ($id)"
     $"PackageVersion: ($version)"
     "DefaultLocale: en-US"
     "ManifestType: version"
     "ManifestVersion: 1.6.0"
    ] | str join "\n"
}

export def generate-locale-manifest [config: record]: nothing -> string {
    mut lines = [
        "# yaml-language-server: $schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json"
        $"PackageIdentifier: ($config.id)"
        $"PackageVersion: ($config.version)"
        "PackageLocale: en-US"
        $"Publisher: ($config.publisher)"
        $"PackageName: ($config.name)"
        $"License: ($config.license)"
        $"ShortDescription: ($config.description)"
    ]

    if $config.homepage != "" {
        $lines = ($lines | append $"PackageUrl: ($config.homepage)")
        $lines = ($lines | append $"PublisherUrl: ($config.homepage)")
    }

    if $config.license_url != "" {
        $lines = ($lines | append $"LicenseUrl: ($config.license_url)")
    }

    if $config.copyright != "" {
        $lines = ($lines | append $"Copyright: ($config.copyright)")
    }

    let tags = parse-tags $config.tags
    if ($tags | is-not-empty) {
        $lines = ($lines | append "Tags:")
        for tag in $tags {
            $lines = ($lines | append $"  - ($tag)")
        }
    }

    $lines = ($lines | append "ManifestType: defaultLocale")
    $lines = ($lines | append "ManifestVersion: 1.6.0")
    $lines | str join "\n"
}

export def generate-installer-manifest [config: record]: nothing -> string {
    let cmd_name = $config.id | split row "." | last
    mut lines = [
        "# yaml-language-server: $schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json"
        $"PackageIdentifier: ($config.id)"
        $"PackageVersion: ($config.version)"
        "InstallerType: portable"
        "Commands:"
        $"  - ($cmd_name)"
        "Installers:"
    ]

    if $config.x64_url != "" {
        $lines = ($lines | append "  - Architecture: x64")
        $lines = ($lines | append $"    InstallerUrl: ($config.x64_url)")
        if $config.x64_sha256 != "" {
            $lines = ($lines | append $"    InstallerSha256: ($config.x64_sha256 | str upcase)")
        }
    }

    if $config.arm64_url != "" {
        $lines = ($lines | append "  - Architecture: arm64")
        $lines = ($lines | append $"    InstallerUrl: ($config.arm64_url)")
        if $config.arm64_sha256 != "" {
            $lines = ($lines | append $"    InstallerSha256: ($config.arm64_sha256 | str upcase)")
        }
    }

    $lines = ($lines | append "ManifestType: installer")
    $lines = ($lines | append "ManifestVersion: 1.6.0")
    $lines | str join "\n"
}

export def parse-tags [raw: string]: nothing -> list<string> {
    if $raw == "" {
        return []
    }
    $raw | split row "," | each {|t| $t | str trim } | where {|t| $t != ""}
}
