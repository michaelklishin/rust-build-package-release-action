use rust_release_action::changelog::{extract_changelog_section, validate_changelog_entry};

const SAMPLE_CHANGELOG: &str = "\
# Changelog

## v2.0.0

### Breaking Changes

- Removed deprecated API

### New Features

- Added new parser

## v1.1.0

### Bug Fixes

- Fixed crash on empty input

## v1.0.0

- Initial release
";

#[test]
fn extract_first_version_section() {
    let section = extract_changelog_section(SAMPLE_CHANGELOG, "2.0.0").unwrap();
    assert!(section.contains("## v2.0.0"));
    assert!(section.contains("Removed deprecated API"));
    assert!(section.contains("Added new parser"));
    assert!(!section.contains("## v1.1.0"));
}

#[test]
fn extract_middle_version_section() {
    let section = extract_changelog_section(SAMPLE_CHANGELOG, "1.1.0").unwrap();
    assert!(section.contains("## v1.1.0"));
    assert!(section.contains("Fixed crash on empty input"));
    assert!(!section.contains("## v2.0.0"));
    assert!(!section.contains("## v1.0.0"));
}

#[test]
fn extract_last_version_section() {
    let section = extract_changelog_section(SAMPLE_CHANGELOG, "1.0.0").unwrap();
    assert!(section.contains("## v1.0.0"));
    assert!(section.contains("Initial release"));
}

#[test]
fn extract_missing_version_returns_error() {
    let result = extract_changelog_section(SAMPLE_CHANGELOG, "3.0.0");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("3.0.0"));
}

#[test]
fn extract_version_without_v_prefix() {
    let changelog = "# Changelog\n\n## 1.5.0\n\n- Some change\n";
    let section = extract_changelog_section(changelog, "1.5.0").unwrap();
    assert!(section.contains("## 1.5.0"));
    assert!(section.contains("Some change"));
}

#[test]
fn validate_existing_version() {
    assert!(validate_changelog_entry(SAMPLE_CHANGELOG, "2.0.0"));
    assert!(validate_changelog_entry(SAMPLE_CHANGELOG, "1.1.0"));
    assert!(validate_changelog_entry(SAMPLE_CHANGELOG, "1.0.0"));
}

#[test]
fn validate_missing_version() {
    assert!(!validate_changelog_entry(SAMPLE_CHANGELOG, "3.0.0"));
    assert!(!validate_changelog_entry(SAMPLE_CHANGELOG, "0.1.0"));
}

#[test]
fn validate_without_v_prefix_in_changelog() {
    let changelog = "## 1.0.0\n\n- Change\n";
    assert!(validate_changelog_entry(changelog, "1.0.0"));
}

#[test]
fn extract_empty_changelog_returns_error() {
    let result = extract_changelog_section("", "1.0.0");
    assert!(result.is_err());
}

#[test]
fn extract_changelog_with_only_header() {
    let changelog = "# Changelog\n";
    let result = extract_changelog_section(changelog, "1.0.0");
    assert!(result.is_err());
}
