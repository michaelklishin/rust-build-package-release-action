use rust_release_action::platform::detect_platform_short;

#[test]
fn artifact_classification_tar_gz() {
    assert_eq!(
        detect_platform_short("myapp-1.0.0-x86_64-unknown-linux-gnu.tar.gz"),
        "linux-x64"
    );
    assert_eq!(
        detect_platform_short("myapp-1.0.0-aarch64-apple-darwin.tar.gz"),
        "macos-arm64"
    );
}

#[test]
fn artifact_classification_zip() {
    assert_eq!(
        detect_platform_short("myapp-1.0.0-x86_64-pc-windows-msvc.zip"),
        "windows-x64"
    );
}

#[test]
fn artifact_classification_packages() {
    assert_eq!(detect_platform_short("myapp_1.0.0_amd64.deb"), "linux-deb");
    assert_eq!(
        detect_platform_short("myapp-1.0.0-1.x86_64.rpm"),
        "linux-rpm"
    );
    assert_eq!(detect_platform_short("myapp-1.0.0.apk"), "linux-apk");
    assert_eq!(detect_platform_short("myapp-1.0.0.dmg"), "macos-dmg");
    assert_eq!(detect_platform_short("myapp-1.0.0.msi"), "windows-msi");
}

#[test]
fn artifact_classification_unknown() {
    assert_eq!(detect_platform_short("checksums.txt"), "unknown");
    assert_eq!(detect_platform_short("SBOM.spdx.json"), "unknown");
}
