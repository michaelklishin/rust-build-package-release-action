use proptest::prelude::*;
use rust_release_action::changelog::{extract_changelog_section, validate_changelog_entry};

proptest! {
    #[test]
    fn extracted_section_contains_version_header(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
    ) {
        let version = format!("{major}.{minor}.{patch}");
        let changelog = format!("# Changelog\n\n## v{version}\n\n- Some change\n\n## v0.0.0\n\n- Old\n");
        let section = extract_changelog_section(&changelog, &version).unwrap();
        let expected_header = format!("## v{version}");
        prop_assert!(section.contains(&expected_header));
        prop_assert!(section.contains("Some change"));
        prop_assert!(!section.contains("## v0.0.0"));
    }

    #[test]
    fn validate_finds_version_with_v_prefix(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
    ) {
        let version = format!("{major}.{minor}.{patch}");
        let changelog = format!("## v{version}\n\n- Change\n");
        prop_assert!(validate_changelog_entry(&changelog, &version));
    }

    #[test]
    fn validate_finds_version_without_v_prefix(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
    ) {
        let version = format!("{major}.{minor}.{patch}");
        let changelog = format!("## {version}\n\n- Change\n");
        prop_assert!(validate_changelog_entry(&changelog, &version));
    }

    #[test]
    fn missing_version_not_validated(
        major in 1u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
    ) {
        let version = format!("{major}.{minor}.{patch}");
        let changelog = "## v0.0.0\n\n- Old change\n";
        prop_assert!(!validate_changelog_entry(changelog, &version));
    }
}
