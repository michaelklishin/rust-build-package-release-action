use rust_release_action::version::{is_valid_semver, semver_pattern, version_from_tag};

#[test]
fn valid_semver_versions() {
    let cases = [
        "1.0.0",
        "0.1.0",
        "10.20.30",
        "1.2.3-alpha",
        "1.2.3-alpha.1",
        "1.2.3-beta.2",
        "1.0.0-rc.1",
        "2.0.0+build.123",
        "1.0.0-alpha+build",
        "1.0.0-alpha.1+build.456",
        "0.0.0",
        "999.999.999",
        "1.0.0-0",
        "1.0.0-alpha0",
        "1.0.0+20240101",
    ];
    for v in &cases {
        assert!(is_valid_semver(v), "expected valid: {v}");
    }
}

#[test]
fn invalid_semver_versions() {
    let cases = [
        "1.0",
        "1",
        "v1.0.0",
        "1.0.0.",
        ".1.0.0",
        "1..0.0",
        "1.0.0-",
        "1.0.0+",
        "a.b.c",
        "1.0.0--double",
        "",
        " 1.0.0",
        "1.0.0 ",
        "1.0.0-alpha..1",
    ];
    for v in &cases {
        assert!(!is_valid_semver(v), "expected invalid: {v}");
    }
}

#[test]
fn semver_pattern_compiles() {
    let _re = semver_pattern();
}

#[test]
fn version_from_tag_strips_v_prefix() {
    assert_eq!(version_from_tag("v1.2.3"), Some("1.2.3"));
    assert_eq!(version_from_tag("v0.1.0-alpha"), Some("0.1.0-alpha"));
}

#[test]
fn version_from_tag_without_prefix_returns_none() {
    assert_eq!(version_from_tag("1.2.3"), None);
    assert_eq!(version_from_tag(""), None);
}

#[test]
fn version_from_tag_only_v_returns_empty() {
    assert_eq!(version_from_tag("v"), Some(""));
}
