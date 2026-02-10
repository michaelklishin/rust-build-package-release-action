use rust_release_action::parse_comma_list;
use rust_release_action::winget::{
    InstallerConfig, LocaleConfig, generate_installer_manifest, generate_locale_manifest,
    generate_version_manifest,
};

#[test]
fn parse_tags_empty() {
    assert!(parse_comma_list("").is_empty());
}

#[test]
fn parse_tags_single() {
    assert_eq!(parse_comma_list("cli"), vec!["cli"]);
}

#[test]
fn parse_tags_multiple() {
    assert_eq!(
        parse_comma_list("cli,rust,tool"),
        vec!["cli", "rust", "tool"]
    );
}

#[test]
fn parse_tags_with_spaces() {
    assert_eq!(
        parse_comma_list("cli , rust , tool"),
        vec!["cli", "rust", "tool"]
    );
}

#[test]
fn version_manifest() {
    let manifest = generate_version_manifest("Publisher.MyTool", "1.0.0");

    assert!(manifest.contains("PackageIdentifier: Publisher.MyTool"));
    assert!(manifest.contains("PackageVersion: 1.0.0"));
    assert!(manifest.contains("DefaultLocale: en-US"));
    assert!(manifest.contains("ManifestType: version"));
    assert!(manifest.contains("ManifestVersion: 1.6.0"));
}

#[test]
fn locale_manifest_basic() {
    let config = LocaleConfig {
        id: "Publisher.Tool".into(),
        version: "1.0.0".into(),
        publisher: "Publisher".into(),
        name: "Tool".into(),
        description: "A great tool".into(),
        homepage: "https://example.com".into(),
        license: "MIT".into(),
        license_url: "https://example.com/LICENSE".into(),
        copyright: "Copyright 2024".into(),
        tags: "cli,rust".into(),
    };

    let manifest = generate_locale_manifest(&config);

    assert!(manifest.contains("PackageIdentifier: Publisher.Tool"));
    assert!(manifest.contains("PackageVersion: 1.0.0"));
    assert!(manifest.contains("Publisher: Publisher"));
    assert!(manifest.contains("PackageName: Tool"));
    assert!(manifest.contains("License: MIT"));
    assert!(manifest.contains("ShortDescription: A great tool"));
    assert!(manifest.contains("PackageUrl: https://example.com"));
    assert!(manifest.contains("PublisherUrl: https://example.com"));
    assert!(manifest.contains("LicenseUrl: https://example.com/LICENSE"));
    assert!(manifest.contains("Copyright: Copyright 2024"));
    assert!(manifest.contains("Tags:"));
    assert!(manifest.contains("  - cli"));
    assert!(manifest.contains("  - rust"));
    assert!(manifest.contains("ManifestType: defaultLocale"));
}

#[test]
fn locale_manifest_minimal() {
    let config = LocaleConfig {
        id: "Pub.Tool".into(),
        version: "0.1.0".into(),
        publisher: "Pub".into(),
        name: "Tool".into(),
        description: "desc".into(),
        homepage: String::new(),
        license: "MIT".into(),
        license_url: String::new(),
        copyright: String::new(),
        tags: String::new(),
    };

    let manifest = generate_locale_manifest(&config);

    assert!(!manifest.contains("PackageUrl:"));
    assert!(!manifest.contains("LicenseUrl:"));
    assert!(!manifest.contains("Copyright:"));
    assert!(!manifest.contains("Tags:"));
}

#[test]
fn installer_manifest_both_architectures() {
    let config = InstallerConfig {
        id: "Publisher.Tool".into(),
        version: "1.0.0".into(),
        x64_url: "https://example.com/tool-x64.zip".into(),
        x64_sha256: "abc123".into(),
        arm64_url: "https://example.com/tool-arm64.zip".into(),
        arm64_sha256: "def456".into(),
    };

    let manifest = generate_installer_manifest(&config);

    assert!(manifest.contains("PackageIdentifier: Publisher.Tool"));
    assert!(manifest.contains("InstallerType: portable"));
    assert!(manifest.contains("Commands:"));
    assert!(manifest.contains("  - Tool"));
    assert!(manifest.contains("  - Architecture: x64"));
    assert!(manifest.contains("    InstallerUrl: https://example.com/tool-x64.zip"));
    assert!(manifest.contains("    InstallerSha256: ABC123"));
    assert!(manifest.contains("  - Architecture: arm64"));
    assert!(manifest.contains("    InstallerUrl: https://example.com/tool-arm64.zip"));
    assert!(manifest.contains("    InstallerSha256: DEF456"));
    assert!(manifest.contains("ManifestType: installer"));
}

#[test]
fn installer_manifest_x64_only() {
    let config = InstallerConfig {
        id: "Publisher.Tool".into(),
        version: "1.0.0".into(),
        x64_url: "https://example.com/tool.zip".into(),
        x64_sha256: "hash".into(),
        arm64_url: String::new(),
        arm64_sha256: String::new(),
    };

    let manifest = generate_installer_manifest(&config);

    assert!(manifest.contains("Architecture: x64"));
    assert!(!manifest.contains("Architecture: arm64"));
}

#[test]
fn installer_manifest_command_from_id() {
    let config = InstallerConfig {
        id: "MyCompany.MyTool".into(),
        version: "1.0.0".into(),
        x64_url: "https://example.com/tool.zip".into(),
        x64_sha256: "hash".into(),
        arm64_url: String::new(),
        arm64_sha256: String::new(),
    };

    let manifest = generate_installer_manifest(&config);
    assert!(manifest.contains("  - MyTool"));
}
