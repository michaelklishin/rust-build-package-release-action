use crate::error::{Error, Result};

/// Detect platform from artifact filename (for collect-artifacts).
/// Returns short platform identifiers like "macos-arm64", "linux-x64", etc.
pub fn detect_platform_short(filename: &str) -> &'static str {
    let f = filename.to_lowercase();
    if regex_matches(
        &f,
        r"darwin.*arm64|aarch64.*apple|apple.*aarch64|macos.*arm64",
    ) {
        "macos-arm64"
    } else if regex_matches(
        &f,
        r"darwin.*x86_64|x86_64.*apple|apple.*x86_64|macos.*x64|macos.*x86_64",
    ) {
        "macos-x64"
    } else if regex_matches(&f, r"linux.*aarch64|aarch64.*linux|linux.*arm64") {
        "linux-arm64"
    } else if regex_matches(&f, r"linux.*x86_64|x86_64.*linux|linux.*x64|linux.*amd64") {
        "linux-x64"
    } else if regex_matches(
        &f,
        r"windows.*x86_64|x86_64.*windows|windows.*x64|pc-windows.*x86_64",
    ) {
        "windows-x64"
    } else if regex_matches(&f, r"windows.*aarch64|aarch64.*windows|windows.*arm64") {
        "windows-arm64"
    } else if f.ends_with(".deb") {
        "linux-deb"
    } else if f.ends_with(".rpm") {
        "linux-rpm"
    } else if f.ends_with(".apk") {
        "linux-apk"
    } else if f.ends_with(".dmg") {
        "macos-dmg"
    } else if f.ends_with(".msi") {
        "windows-msi"
    } else {
        "unknown"
    }
}

/// Detect platform from artifact name (for format-release).
/// Returns human-readable names like "macOS (Apple Silicon)", "Linux (x64)", etc.
pub fn detect_platform_display(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if regex_matches(&n, r"darwin|macos|osx") {
        if regex_matches(&n, r"arm64|aarch64") {
            "macOS (Apple Silicon)"
        } else {
            "macOS (Intel)"
        }
    } else if regex_matches(&n, r"windows|win") {
        if regex_matches(&n, r"arm64|aarch64") {
            "Windows (ARM64)"
        } else {
            "Windows (x64)"
        }
    } else if regex_matches(&n, r"linux") {
        if n.contains("musl") {
            if regex_matches(&n, r"arm64|aarch64") {
                "Linux (ARM64, musl)"
            } else {
                "Linux (x64, musl)"
            }
        } else if regex_matches(&n, r"arm64|aarch64") {
            "Linux (ARM64)"
        } else if n.contains("armv7") {
            "Linux (ARMv7)"
        } else {
            "Linux (x64)"
        }
    } else if n.ends_with(".deb") {
        "Debian/Ubuntu"
    } else if n.ends_with(".rpm") {
        "RHEL/Fedora"
    } else if n.ends_with(".apk") {
        "Alpine Linux"
    } else if n.ends_with(".dmg") {
        "macOS Installer"
    } else if n.ends_with(".msi") {
        "Windows Installer"
    } else if n.ends_with(".pkg.tar.zst") {
        "Arch Linux"
    } else {
        "Other"
    }
}

fn regex_matches(text: &str, pattern: &str) -> bool {
    regex::Regex::new(pattern)
        .map(|re| re.is_match(text))
        .unwrap_or(false)
}

/// Convert Rust target triple to Debian architecture name.
pub fn target_to_deb_arch(target: &str) -> Result<&'static str> {
    match target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => Ok("amd64"),
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => Ok("arm64"),
        "armv7-unknown-linux-gnueabihf" => Ok("armhf"),
        "i686-unknown-linux-gnu" | "i686-unknown-linux-musl" => Ok("i386"),
        _ => {
            if target.contains("x86_64") {
                Ok("amd64")
            } else if target.contains("aarch64") {
                Ok("arm64")
            } else if target.contains("armv7") {
                Ok("armhf")
            } else if target.contains("i686") {
                Ok("i386")
            } else {
                Err(Error::User(format!(
                    "unsupported target for .deb: {target}"
                )))
            }
        }
    }
}

/// Convert Rust target triple to RPM architecture name.
pub fn target_to_rpm_arch(target: &str) -> Result<&'static str> {
    match target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => Ok("x86_64"),
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => Ok("aarch64"),
        "armv7-unknown-linux-gnueabihf" => Ok("armv7hl"),
        "i686-unknown-linux-gnu" | "i686-unknown-linux-musl" => Ok("i686"),
        _ => {
            if target.contains("x86_64") {
                Ok("x86_64")
            } else if target.contains("aarch64") {
                Ok("aarch64")
            } else if target.contains("armv7") {
                Ok("armv7hl")
            } else if target.contains("i686") {
                Ok("i686")
            } else {
                Err(Error::User(format!(
                    "unsupported target for .rpm: {target}"
                )))
            }
        }
    }
}

/// Convert Rust target triple to Alpine APK architecture name.
pub fn target_to_apk_arch(target: &str) -> Result<&'static str> {
    match target {
        "x86_64-unknown-linux-gnu" | "x86_64-unknown-linux-musl" => Ok("x86_64"),
        "aarch64-unknown-linux-gnu" | "aarch64-unknown-linux-musl" => Ok("aarch64"),
        "armv7-unknown-linux-gnueabihf" | "armv7-unknown-linux-musleabihf" => Ok("armv7"),
        "i686-unknown-linux-gnu" | "i686-unknown-linux-musl" => Ok("x86"),
        _ => {
            if target.contains("x86_64") {
                Ok("x86_64")
            } else if target.contains("aarch64") {
                Ok("aarch64")
            } else if target.contains("armv7") {
                Ok("armv7")
            } else if target.contains("i686") {
                Ok("x86")
            } else {
                Err(Error::User(format!(
                    "unsupported target for .apk: {target}"
                )))
            }
        }
    }
}
