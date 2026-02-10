use proptest::prelude::*;
use rust_release_action::version::is_valid_semver;

proptest! {
    #[test]
    fn valid_semver_always_has_three_parts(
        major in 0u32..1000,
        minor in 0u32..1000,
        patch in 0u32..1000,
    ) {
        let v = format!("{major}.{minor}.{patch}");
        prop_assert!(is_valid_semver(&v), "expected valid: {v}");
    }

    #[test]
    fn valid_semver_with_prerelease(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
        pre in "[a-zA-Z0-9]{1,10}",
    ) {
        let v = format!("{major}.{minor}.{patch}-{pre}");
        prop_assert!(is_valid_semver(&v), "expected valid: {v}");
    }

    #[test]
    fn valid_semver_with_build(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
        build in "[a-zA-Z0-9]{1,10}",
    ) {
        let v = format!("{major}.{minor}.{patch}+{build}");
        prop_assert!(is_valid_semver(&v), "expected valid: {v}");
    }

    #[test]
    fn valid_semver_with_prerelease_and_build(
        major in 0u32..100,
        minor in 0u32..100,
        patch in 0u32..100,
        pre in "[a-zA-Z0-9]{1,10}",
        build in "[a-zA-Z0-9]{1,10}",
    ) {
        let v = format!("{major}.{minor}.{patch}-{pre}+{build}");
        prop_assert!(is_valid_semver(&v), "expected valid: {v}");
    }

    #[test]
    fn two_part_version_is_invalid(major in 0u32..100, minor in 0u32..100) {
        let v = format!("{major}.{minor}");
        prop_assert!(!is_valid_semver(&v), "expected invalid: {v}");
    }

    #[test]
    fn prefixed_v_is_invalid(major in 0u32..100, minor in 0u32..100, patch in 0u32..100) {
        let v = format!("v{major}.{minor}.{patch}");
        prop_assert!(!is_valid_semver(&v), "expected invalid: {v}");
    }
}
