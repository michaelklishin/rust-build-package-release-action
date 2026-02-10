use rust_release_action::build::build_summary;
use rust_release_action::checksum::Checksums;

#[test]
fn build_summary_contains_all_fields() {
    let checksums = Checksums {
        sha256: "abc123".into(),
        sha512: "def456".into(),
        b2: "ghi789".into(),
    };
    let result = build_summary(
        "myapp",
        "1.0.0",
        "x86_64-unknown-linux-gnu",
        "myapp.tar.gz",
        "/tmp/myapp.tar.gz",
        &checksums,
    );
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["binary_name"], "myapp");
    assert_eq!(parsed["version"], "1.0.0");
    assert_eq!(parsed["target"], "x86_64-unknown-linux-gnu");
    assert_eq!(parsed["artifact"], "myapp.tar.gz");
    assert_eq!(parsed["artifact_path"], "/tmp/myapp.tar.gz");
    assert_eq!(parsed["sha256"], "abc123");
    assert_eq!(parsed["sha512"], "def456");
    assert_eq!(parsed["b2"], "ghi789");
}

#[test]
fn build_summary_with_empty_checksums() {
    let checksums = Checksums::default();
    let result = build_summary(
        "app",
        "2.0.0",
        "aarch64-apple-darwin",
        "app.zip",
        "/out/app.zip",
        &checksums,
    );
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert_eq!(parsed["sha256"], "");
    assert_eq!(parsed["sha512"], "");
    assert_eq!(parsed["b2"], "");
}
