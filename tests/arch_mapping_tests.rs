use rust_release_action::platform::{target_to_apk_arch, target_to_deb_arch, target_to_rpm_arch};

#[test]
fn deb_arch_x86_64_gnu() {
    assert_eq!(
        target_to_deb_arch("x86_64-unknown-linux-gnu").unwrap(),
        "amd64"
    );
}

#[test]
fn deb_arch_x86_64_musl() {
    assert_eq!(
        target_to_deb_arch("x86_64-unknown-linux-musl").unwrap(),
        "amd64"
    );
}

#[test]
fn deb_arch_aarch64_gnu() {
    assert_eq!(
        target_to_deb_arch("aarch64-unknown-linux-gnu").unwrap(),
        "arm64"
    );
}

#[test]
fn deb_arch_aarch64_musl() {
    assert_eq!(
        target_to_deb_arch("aarch64-unknown-linux-musl").unwrap(),
        "arm64"
    );
}

#[test]
fn deb_arch_armv7() {
    assert_eq!(
        target_to_deb_arch("armv7-unknown-linux-gnueabihf").unwrap(),
        "armhf"
    );
}

#[test]
fn deb_arch_i686() {
    assert_eq!(
        target_to_deb_arch("i686-unknown-linux-gnu").unwrap(),
        "i386"
    );
    assert_eq!(
        target_to_deb_arch("i686-unknown-linux-musl").unwrap(),
        "i386"
    );
}

#[test]
fn deb_arch_fallback_x86_64() {
    assert_eq!(
        target_to_deb_arch("x86_64-some-other-target").unwrap(),
        "amd64"
    );
}

#[test]
fn deb_arch_unsupported() {
    assert!(target_to_deb_arch("mips-unknown-linux-gnu").is_err());
}

#[test]
fn rpm_arch_x86_64() {
    assert_eq!(
        target_to_rpm_arch("x86_64-unknown-linux-gnu").unwrap(),
        "x86_64"
    );
}

#[test]
fn rpm_arch_aarch64() {
    assert_eq!(
        target_to_rpm_arch("aarch64-unknown-linux-gnu").unwrap(),
        "aarch64"
    );
}

#[test]
fn rpm_arch_armv7() {
    assert_eq!(
        target_to_rpm_arch("armv7-unknown-linux-gnueabihf").unwrap(),
        "armv7hl"
    );
}

#[test]
fn rpm_arch_i686() {
    assert_eq!(
        target_to_rpm_arch("i686-unknown-linux-gnu").unwrap(),
        "i686"
    );
}

#[test]
fn rpm_arch_unsupported() {
    assert!(target_to_rpm_arch("mips-unknown-linux-gnu").is_err());
}

#[test]
fn apk_arch_x86_64() {
    assert_eq!(
        target_to_apk_arch("x86_64-unknown-linux-musl").unwrap(),
        "x86_64"
    );
}

#[test]
fn apk_arch_aarch64() {
    assert_eq!(
        target_to_apk_arch("aarch64-unknown-linux-musl").unwrap(),
        "aarch64"
    );
}

#[test]
fn apk_arch_armv7() {
    assert_eq!(
        target_to_apk_arch("armv7-unknown-linux-gnueabihf").unwrap(),
        "armv7"
    );
    assert_eq!(
        target_to_apk_arch("armv7-unknown-linux-musleabihf").unwrap(),
        "armv7"
    );
}

#[test]
fn apk_arch_i686() {
    assert_eq!(target_to_apk_arch("i686-unknown-linux-gnu").unwrap(), "x86");
    assert_eq!(
        target_to_apk_arch("i686-unknown-linux-musl").unwrap(),
        "x86"
    );
}

#[test]
fn apk_arch_unsupported() {
    assert!(target_to_apk_arch("mips-unknown-linux-gnu").is_err());
}
