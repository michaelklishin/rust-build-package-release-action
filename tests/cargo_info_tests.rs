mod test_helpers;

use rust_release_action::cargo_info::get_cargo_info_from_path;
use test_helpers::create_temp_text_file;

#[test]
fn reads_package_name_and_version() {
    let f = create_temp_text_file(
        r#"
[package]
name = "my-tool"
version = "1.2.3"
"#,
    );
    let info = get_cargo_info_from_path(f.path().to_str().unwrap()).unwrap();
    assert_eq!(info.name, "my-tool");
    assert_eq!(info.version, "1.2.3");
}

#[test]
fn reads_workspace_version_fallback() {
    let f = create_temp_text_file(
        r#"
[package]
name = "my-tool"

[workspace.package]
version = "2.0.0"
"#,
    );
    let info = get_cargo_info_from_path(f.path().to_str().unwrap()).unwrap();
    assert_eq!(info.name, "my-tool");
    assert_eq!(info.version, "2.0.0");
}

#[test]
fn package_version_takes_precedence() {
    let f = create_temp_text_file(
        r#"
[package]
name = "my-tool"
version = "1.0.0"

[workspace.package]
version = "2.0.0"
"#,
    );
    let info = get_cargo_info_from_path(f.path().to_str().unwrap()).unwrap();
    assert_eq!(info.version, "1.0.0");
}

#[test]
fn missing_name_returns_empty() {
    let f = create_temp_text_file(
        r#"
[package]
version = "1.0.0"
"#,
    );
    let info = get_cargo_info_from_path(f.path().to_str().unwrap()).unwrap();
    assert_eq!(info.name, "");
    assert_eq!(info.version, "1.0.0");
}

#[test]
fn missing_version_returns_empty() {
    let f = create_temp_text_file(
        r#"
[package]
name = "tool"
"#,
    );
    let info = get_cargo_info_from_path(f.path().to_str().unwrap()).unwrap();
    assert_eq!(info.name, "tool");
    assert_eq!(info.version, "");
}

#[test]
fn missing_file_returns_error() {
    let result = get_cargo_info_from_path("/nonexistent/Cargo.toml");
    assert!(result.is_err());
}

#[test]
fn invalid_toml_returns_error() {
    let f = create_temp_text_file("this is not valid toml [[[");
    let result = get_cargo_info_from_path(f.path().to_str().unwrap());
    assert!(result.is_err());
}

#[test]
fn empty_file_returns_empty_fields() {
    let f = create_temp_text_file("");
    let info = get_cargo_info_from_path(f.path().to_str().unwrap()).unwrap();
    assert_eq!(info.name, "");
    assert_eq!(info.version, "");
}

#[test]
fn with_additional_fields() {
    let f = create_temp_text_file(
        r#"
[package]
name = "complex-tool"
version = "0.5.0"
edition = "2024"
authors = ["Test"]
description = "A test"
license = "MIT"
"#,
    );
    let info = get_cargo_info_from_path(f.path().to_str().unwrap()).unwrap();
    assert_eq!(info.name, "complex-tool");
    assert_eq!(info.version, "0.5.0");
}
