use rust_release_action::nfpm::{
    format_dependency_list, nfpm_base_config, nfpm_contents_section, nfpm_dependencies_section,
};
use std::env;
use std::fs;
use std::sync::{LazyLock, Mutex};

#[test]
fn format_dependency_list_empty() {
    assert_eq!(format_dependency_list("depends", ""), "");
}

#[test]
fn format_dependency_list_single() {
    let result = format_dependency_list("depends", "openssl");
    assert_eq!(result, "depends:\n  - \"openssl\"\n");
}

#[test]
fn format_dependency_list_multiple() {
    let result = format_dependency_list("depends", "openssl,zlib,curl");
    assert_eq!(
        result,
        "depends:\n  - \"openssl\"\n  - \"zlib\"\n  - \"curl\"\n"
    );
}

#[test]
fn format_dependency_list_with_spaces() {
    let result = format_dependency_list("recommends", "foo , bar");
    assert_eq!(result, "recommends:\n  - \"foo\"\n  - \"bar\"\n");
}

#[test]
fn format_dependency_list_trailing_comma() {
    let result = format_dependency_list("suggests", "foo,bar,");
    assert_eq!(result, "suggests:\n  - \"foo\"\n  - \"bar\"\n");
}

#[test]
fn format_dependency_list_different_keys() {
    let keys = [
        "depends",
        "recommends",
        "suggests",
        "conflicts",
        "replaces",
        "provides",
    ];
    for key in keys {
        let result = format_dependency_list(key, "pkg");
        assert!(result.starts_with(&format!("{key}:\n")));
    }
}

#[test]
fn nfpm_base_config_required_fields() {
    let config = nfpm_base_config("myapp", "1.2.3", "amd64");
    assert!(config.contains("name: \"myapp\""));
    assert!(config.contains("version: \"1.2.3\""));
    assert!(config.contains("arch: \"amd64\""));
    assert!(config.contains("platform: linux"));
}

#[test]
fn nfpm_base_config_arm64_arch() {
    let config = nfpm_base_config("tool", "0.1.0", "arm64");
    assert!(config.contains("arch: \"arm64\""));
}

#[test]
fn nfpm_dependencies_section_no_env_vars() {
    // With no PKG_* env vars set, this should return an empty collection
    let result = nfpm_dependencies_section();
    assert!(result.is_empty());
}

static CWD_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
fn nfpm_contents_section_includes_binary() {
    let _lock = CWD_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();

    // Safety: serialised by CWD_LOCK
    unsafe { env::remove_var("PKG_CONTENTS") };
    let result = nfpm_contents_section("myapp", "/usr/src/myapp");
    env::set_current_dir(original_dir).unwrap();

    assert!(result.contains("src: \"/usr/src/myapp\""));
    assert!(result.contains("dst: \"/usr/bin/myapp\""));
    assert!(result.contains("mode: 0755"));
}

#[test]
fn nfpm_contents_section_includes_license_and_readme() {
    let _lock = CWD_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let original_dir = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();

    fs::write(dir.path().join("LICENSE"), "MIT").unwrap();
    fs::write(dir.path().join("README.md"), "# Readme").unwrap();

    unsafe { env::remove_var("PKG_CONTENTS") };
    let result = nfpm_contents_section("myapp", "/usr/src/myapp");
    env::set_current_dir(original_dir).unwrap();

    assert!(result.contains("LICENSE"));
    assert!(result.contains("README.md"));
    assert!(result.contains("/usr/share/doc/myapp/"));
}
