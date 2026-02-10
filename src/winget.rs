use crate::cargo_info::get_cargo_info;
use crate::env_or;
use crate::error::{Error, Result};
use crate::output::{output, print_hr};
use crate::parse_comma_list;
use std::fs;

/// Generate version manifest YAML.
pub fn generate_version_manifest(id: &str, version: &str) -> String {
    [
        "# yaml-language-server: $schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json",
        &format!("PackageIdentifier: {id}"),
        &format!("PackageVersion: {version}"),
        "DefaultLocale: en-US",
        "ManifestType: version",
        "ManifestVersion: 1.6.0",
    ]
    .join("\n")
}

pub struct LocaleConfig {
    pub id: String,
    pub version: String,
    pub publisher: String,
    pub name: String,
    pub description: String,
    pub homepage: String,
    pub license: String,
    pub license_url: String,
    pub copyright: String,
    pub tags: String,
}

/// Generate locale manifest YAML.
pub fn generate_locale_manifest(config: &LocaleConfig) -> String {
    let mut lines = vec![
        "# yaml-language-server: $schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json".to_string(),
        format!("PackageIdentifier: {}", config.id),
        format!("PackageVersion: {}", config.version),
        "PackageLocale: en-US".to_string(),
        format!("Publisher: {}", config.publisher),
        format!("PackageName: {}", config.name),
        format!("License: {}", config.license),
        format!("ShortDescription: {}", config.description),
    ];

    if !config.homepage.is_empty() {
        lines.push(format!("PackageUrl: {}", config.homepage));
        lines.push(format!("PublisherUrl: {}", config.homepage));
    }

    if !config.license_url.is_empty() {
        lines.push(format!("LicenseUrl: {}", config.license_url));
    }

    if !config.copyright.is_empty() {
        lines.push(format!("Copyright: {}", config.copyright));
    }

    let tags = parse_comma_list(&config.tags);
    if !tags.is_empty() {
        lines.push("Tags:".to_string());
        for tag in &tags {
            lines.push(format!("  - {tag}"));
        }
    }

    lines.push("ManifestType: defaultLocale".to_string());
    lines.push("ManifestVersion: 1.6.0".to_string());
    lines.join("\n")
}

pub struct InstallerConfig {
    pub id: String,
    pub version: String,
    pub x64_url: String,
    pub x64_sha256: String,
    pub arm64_url: String,
    pub arm64_sha256: String,
}

/// Generate installer manifest YAML.
pub fn generate_installer_manifest(config: &InstallerConfig) -> String {
    let cmd_name = config.id.split('.').next_back().unwrap_or(&config.id);
    let mut lines = vec![
        "# yaml-language-server: $schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json".to_string(),
        format!("PackageIdentifier: {}", config.id),
        format!("PackageVersion: {}", config.version),
        "InstallerType: portable".to_string(),
        "Commands:".to_string(),
        format!("  - {cmd_name}"),
        "Installers:".to_string(),
    ];

    if !config.x64_url.is_empty() {
        lines.push("  - Architecture: x64".to_string());
        lines.push(format!("    InstallerUrl: {}", config.x64_url));
        if !config.x64_sha256.is_empty() {
            lines.push(format!(
                "    InstallerSha256: {}",
                config.x64_sha256.to_uppercase()
            ));
        }
    }

    if !config.arm64_url.is_empty() {
        lines.push("  - Architecture: arm64".to_string());
        lines.push(format!("    InstallerUrl: {}", config.arm64_url));
        if !config.arm64_sha256.is_empty() {
            lines.push(format!(
                "    InstallerSha256: {}",
                config.arm64_sha256.to_uppercase()
            ));
        }
    }

    lines.push("ManifestType: installer".to_string());
    lines.push("ManifestVersion: 1.6.0".to_string());
    lines.join("\n")
}

pub fn run_generate_winget() -> Result<()> {
    let info = get_cargo_info()?;
    let binary_name = env_or("BINARY_NAME", &info.name);
    let version = env_or("VERSION", &info.version);

    if binary_name.is_empty() {
        return Err(Error::User("could not determine binary name".into()));
    }
    if version.is_empty() {
        return Err(Error::User("could not determine version".into()));
    }

    let publisher = env_or("WINGET_PUBLISHER", "");
    if publisher.is_empty() {
        return Err(Error::User("WINGET_PUBLISHER is required".into()));
    }

    let publisher_id = env_or("WINGET_PUBLISHER_ID", &publisher.replace(' ', ""));
    let package_id = env_or("WINGET_PACKAGE_ID", &binary_name);
    let manifest_id = format!("{publisher_id}.{package_id}");

    println!("\x1b[32mGenerating Winget manifest:\x1b[0m {manifest_id} v{version}");

    let output_dir = env_or("WINGET_OUTPUT_DIR", "target/winget");
    let first_char = publisher_id
        .chars()
        .next()
        .unwrap_or('x')
        .to_lowercase()
        .to_string();
    let manifest_dir =
        format!("{output_dir}/manifests/{first_char}/{publisher_id}/{package_id}/{version}");
    fs::create_dir_all(&manifest_dir)?;

    let version_manifest = generate_version_manifest(&manifest_id, &version);
    let version_path = format!("{manifest_dir}/{manifest_id}.yaml");
    fs::write(&version_path, &version_manifest)?;

    let locale_config = LocaleConfig {
        id: manifest_id.clone(),
        version: version.clone(),
        publisher: publisher.clone(),
        name: binary_name.clone(),
        description: env_or(
            "PKG_DESCRIPTION",
            &format!("{binary_name} - built with rust-build-package-release-action"),
        ),
        homepage: env_or("PKG_HOMEPAGE", ""),
        license: env_or("PKG_LICENSE", "MIT"),
        license_url: env_or("WINGET_LICENSE_URL", ""),
        copyright: env_or("WINGET_COPYRIGHT", ""),
        tags: env_or("WINGET_TAGS", ""),
    };
    let locale_manifest = generate_locale_manifest(&locale_config);
    let locale_path = format!("{manifest_dir}/{manifest_id}.locale.en-US.yaml");
    fs::write(&locale_path, &locale_manifest)?;

    let installer_config = InstallerConfig {
        id: manifest_id.clone(),
        version,
        x64_url: env_or("WINGET_X64_URL", ""),
        x64_sha256: env_or("WINGET_X64_SHA256", ""),
        arm64_url: env_or("WINGET_ARM64_URL", ""),
        arm64_sha256: env_or("WINGET_ARM64_SHA256", ""),
    };
    let installer_manifest = generate_installer_manifest(&installer_config);
    let installer_path = format!("{manifest_dir}/{manifest_id}.installer.yaml");
    fs::write(&installer_path, &installer_manifest)?;

    println!();
    println!("\x1b[32mManifest files:\x1b[0m");
    print_hr();
    println!("Version: {version_path}");
    println!("Locale:  {locale_path}");
    println!("Installer: {installer_path}");
    print_hr();

    println!();
    println!("\x1b[32mVersion manifest:\x1b[0m");
    println!("{version_manifest}");

    output("manifest_dir", &manifest_dir);
    output("manifest_id", &manifest_id);
    output("version_manifest", &version_path);
    output("locale_manifest", &locale_path);
    output("installer_manifest", &installer_path);
    Ok(())
}
