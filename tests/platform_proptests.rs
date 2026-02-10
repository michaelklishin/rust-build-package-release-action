use proptest::prelude::*;
use rust_release_action::platform::detect_platform_short;

proptest! {
    #[test]
    fn apple_darwin_arm64_detected(
        name in "[a-z]{3,10}",
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        let filename = format!("{name}-{version}-aarch64-apple-darwin.tar.gz");
        prop_assert_eq!(detect_platform_short(&filename), "macos-arm64");
    }

    #[test]
    fn apple_darwin_x64_detected(
        name in "[a-z]{3,10}",
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        let filename = format!("{name}-{version}-x86_64-apple-darwin.tar.gz");
        prop_assert_eq!(detect_platform_short(&filename), "macos-x64");
    }

    #[test]
    fn linux_gnu_x64_detected(
        name in "[a-z]{3,10}",
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        let filename = format!("{name}-{version}-x86_64-unknown-linux-gnu.tar.gz");
        prop_assert_eq!(detect_platform_short(&filename), "linux-x64");
    }

    #[test]
    fn linux_gnu_arm64_detected(
        name in "[a-z]{3,10}",
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        let filename = format!("{name}-{version}-aarch64-unknown-linux-gnu.tar.gz");
        prop_assert_eq!(detect_platform_short(&filename), "linux-arm64");
    }

    #[test]
    fn windows_x64_detected(
        name in "[a-z]{3,10}",
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
    ) {
        let filename = format!("{name}-{version}-x86_64-pc-windows-msvc.zip");
        prop_assert_eq!(detect_platform_short(&filename), "windows-x64");
    }
}
