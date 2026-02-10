use rust_release_action::platform::{detect_platform_display, detect_platform_short};

#[test]
fn short_platform_macos_arm64() {
    assert_eq!(
        detect_platform_short("myapp-1.0.0-aarch64-apple-darwin.tar.gz"),
        "macos-arm64"
    );
    assert_eq!(
        detect_platform_short("myapp-macos-arm64.tar.gz"),
        "macos-arm64"
    );
}

#[test]
fn short_platform_macos_x64() {
    assert_eq!(
        detect_platform_short("myapp-1.0.0-x86_64-apple-darwin.tar.gz"),
        "macos-x64"
    );
    assert_eq!(detect_platform_short("myapp-macos-x64.tar.gz"), "macos-x64");
}

#[test]
fn short_platform_linux_arm64() {
    assert_eq!(
        detect_platform_short("myapp-1.0.0-aarch64-unknown-linux-gnu.tar.gz"),
        "linux-arm64"
    );
    assert_eq!(
        detect_platform_short("myapp-linux-arm64.tar.gz"),
        "linux-arm64"
    );
}

#[test]
fn short_platform_linux_x64() {
    assert_eq!(
        detect_platform_short("myapp-1.0.0-x86_64-unknown-linux-gnu.tar.gz"),
        "linux-x64"
    );
    assert_eq!(detect_platform_short("myapp-linux-x64.tar.gz"), "linux-x64");
    assert_eq!(
        detect_platform_short("myapp-linux-amd64.tar.gz"),
        "linux-x64"
    );
}

#[test]
fn short_platform_windows_x64() {
    assert_eq!(
        detect_platform_short("myapp-1.0.0-x86_64-pc-windows-msvc.zip"),
        "windows-x64"
    );
    assert_eq!(
        detect_platform_short("myapp-windows-x64.zip"),
        "windows-x64"
    );
}

#[test]
fn short_platform_windows_arm64() {
    assert_eq!(
        detect_platform_short("myapp-1.0.0-aarch64-pc-windows-msvc.zip"),
        "windows-arm64"
    );
}

#[test]
fn short_platform_package_formats() {
    assert_eq!(detect_platform_short("myapp_1.0.0_amd64.deb"), "linux-deb");
    assert_eq!(
        detect_platform_short("myapp-1.0.0-1.x86_64.rpm"),
        "linux-rpm"
    );
    assert_eq!(detect_platform_short("myapp-1.0.0-r0.apk"), "linux-apk");
    assert_eq!(detect_platform_short("myapp-1.0.0.dmg"), "macos-dmg");
    assert_eq!(detect_platform_short("myapp-1.0.0.msi"), "windows-msi");
}

#[test]
fn short_platform_unknown() {
    assert_eq!(detect_platform_short("README.md"), "unknown");
    assert_eq!(detect_platform_short("somefile.txt"), "unknown");
}

#[test]
fn display_platform_macos() {
    assert_eq!(
        detect_platform_display("myapp-aarch64-apple-darwin.tar.gz"),
        "macOS (Apple Silicon)"
    );
    assert_eq!(
        detect_platform_display("myapp-x86_64-apple-darwin.tar.gz"),
        "macOS (Intel)"
    );
}

#[test]
fn display_platform_windows() {
    assert_eq!(
        detect_platform_display("myapp-x86_64-pc-windows-msvc.zip"),
        "Windows (x64)"
    );
    assert_eq!(
        detect_platform_display("myapp-aarch64-pc-windows-msvc.zip"),
        "Windows (ARM64)"
    );
}

#[test]
fn display_platform_linux() {
    assert_eq!(
        detect_platform_display("myapp-x86_64-unknown-linux-gnu.tar.gz"),
        "Linux (x64)"
    );
    assert_eq!(
        detect_platform_display("myapp-aarch64-unknown-linux-gnu.tar.gz"),
        "Linux (ARM64)"
    );
    assert_eq!(
        detect_platform_display("myapp-x86_64-unknown-linux-musl.tar.gz"),
        "Linux (x64, musl)"
    );
    assert_eq!(
        detect_platform_display("myapp-aarch64-unknown-linux-musl.tar.gz"),
        "Linux (ARM64, musl)"
    );
    assert_eq!(
        detect_platform_display("myapp-armv7-unknown-linux-gnueabihf.tar.gz"),
        "Linux (ARMv7)"
    );
}

#[test]
fn display_platform_packages() {
    assert_eq!(detect_platform_display("myapp.deb"), "Debian/Ubuntu");
    assert_eq!(detect_platform_display("myapp.rpm"), "RHEL/Fedora");
    assert_eq!(detect_platform_display("myapp.apk"), "Alpine Linux");
    assert_eq!(detect_platform_display("myapp.dmg"), "macOS Installer");
    assert_eq!(detect_platform_display("myapp.msi"), "Windows Installer");
    assert_eq!(detect_platform_display("myapp.pkg.tar.zst"), "Arch Linux");
}

#[test]
fn display_platform_other() {
    assert_eq!(detect_platform_display("README.md"), "Other");
}
