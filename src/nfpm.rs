use crate::env_or;
use crate::parse_comma_list;
use std::fs;
use std::path::Path;

/// Formats a list of dependencies for nfpm YAML.
pub fn format_dependency_list(key: &str, raw: &str) -> String {
    let items = parse_comma_list(raw);
    if items.is_empty() {
        return String::new();
    }
    let mut result = format!("{key}:\n");
    for item in &items {
        result.push_str(&format!("  - \"{item}\"\n"));
    }
    result
}

/// Generates base nfpm config header.
pub fn nfpm_base_config(binary_name: &str, version: &str, arch: &str) -> String {
    let description = env_or(
        "PKG_DESCRIPTION",
        &format!("{binary_name} - built with rust-build-package-release-action"),
    );
    let maintainer = env_or("PKG_MAINTAINER", "Unknown <unknown@example.com>");
    let homepage = env_or("PKG_HOMEPAGE", "");
    let license = env_or("PKG_LICENSE", "");
    let vendor = env_or("PKG_VENDOR", "");

    let mut config = format!(
        "name: \"{binary_name}\"\n\
         arch: \"{arch}\"\n\
         platform: linux\n\
         version: \"{version}\"\n\
         maintainer: \"{maintainer}\"\n\
         description: \"{description}\"\n"
    );
    if !homepage.is_empty() {
        config.push_str(&format!("homepage: \"{homepage}\"\n"));
    }
    if !license.is_empty() {
        config.push_str(&format!("license: \"{license}\"\n"));
    }
    if !vendor.is_empty() {
        config.push_str(&format!("vendor: \"{vendor}\"\n"));
    }
    config
}

/// Generates nfpm contents section for binary and docs.
pub fn nfpm_contents_section(binary_name: &str, binary_path: &str) -> String {
    let mut config = format!(
        "\ncontents:\n  - src: \"{binary_path}\"\n    dst: \"/usr/bin/{binary_name}\"\n    file_info:\n      mode: 0755\n"
    );

    // Add LICENSE files
    let licenses: Vec<_> = glob::glob("LICENSE*")
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    for lic in &licenses {
        let abs = fs::canonicalize(lic).unwrap_or_else(|_| lic.clone());
        let basename = lic.file_name().unwrap_or_default().to_string_lossy();
        config.push_str(&format!(
            "  - src: \"{}\"\n    dst: \"/usr/share/doc/{binary_name}/{basename}\"\n    file_info:\n      mode: 0644\n",
            abs.display()
        ));
    }

    if Path::new("README.md").exists() {
        let readme = fs::canonicalize("README.md").unwrap_or_else(|_| "README.md".into());
        config.push_str(&format!(
            "  - src: \"{}\"\n    dst: \"/usr/share/doc/{binary_name}/README.md\"\n    file_info:\n      mode: 0644\n",
            readme.display()
        ));
    }

    let includes = parse_comma_list(&env_or("PKG_CONTENTS", ""));
    if !includes.is_empty() {
        for inc in &includes {
            let parts: Vec<&str> = inc.split(':').collect();
            if parts.len() == 2 {
                let src = fs::canonicalize(parts[0]).unwrap_or_else(|_| parts[0].into());
                let dst = parts[1];
                config.push_str(&format!(
                    "  - src: \"{}\"\n    dst: \"{dst}\"\n",
                    src.display()
                ));
            }
        }
    }

    config
}

/// Generates nfpm dependency sections.
pub fn nfpm_dependencies_section() -> String {
    let mut config = String::new();
    config.push_str(&format_dependency_list(
        "depends",
        &env_or("PKG_DEPENDS", ""),
    ));
    config.push_str(&format_dependency_list(
        "recommends",
        &env_or("PKG_RECOMMENDS", ""),
    ));
    config.push_str(&format_dependency_list(
        "suggests",
        &env_or("PKG_SUGGESTS", ""),
    ));
    config.push_str(&format_dependency_list(
        "conflicts",
        &env_or("PKG_CONFLICTS", ""),
    ));
    config.push_str(&format_dependency_list(
        "replaces",
        &env_or("PKG_REPLACES", ""),
    ));
    config.push_str(&format_dependency_list(
        "provides",
        &env_or("PKG_PROVIDES", ""),
    ));
    config
}
