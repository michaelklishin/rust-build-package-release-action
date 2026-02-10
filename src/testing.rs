use crate::checksum::verify_checksum;
use crate::download;
use crate::env_or;
use crate::error::{Error, Result};
use crate::output::output;
use crate::tools::command_exists;
use crate::tools::run_command;
use std::path::Path;
use std::process::Command;

/// Gets version output from a binary using common flags.
fn get_version_output(binary: &str) -> String {
    let flags = ["--version", "-V", "version", "help"];
    for flag in &flags {
        if let Ok(result) = Command::new(binary).arg(flag).output() {
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout).trim().to_string();
                if !stdout.is_empty() {
                    return stdout;
                }
                let stderr = String::from_utf8_lossy(&result.stderr).trim().to_string();
                if !stderr.is_empty() {
                    return stderr;
                }
            }
        }
    }

    // Debug output
    if let Ok(result) = Command::new(binary).arg("--version").output() {
        eprintln!(
            "  Debug: exit={} stdout={} stderr={}",
            result.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&result.stdout).trim(),
            String::from_utf8_lossy(&result.stderr).trim()
        );
    }
    String::new()
}

/// Verifies an installed binary exists and reports correct version.
fn verify_installed_binary(bin_path: &str, expected_version: &str) -> Result<()> {
    println!("\x1b[32mVerifying binary...\x1b[0m");

    if !Path::new(bin_path).exists() {
        return Err(Error::User(format!("binary not found at {bin_path}")));
    }

    let version_output = get_version_output(bin_path);
    if !version_output.contains(expected_version) {
        return Err(Error::User(format!(
            "version mismatch: expected {expected_version} in output: {version_output}"
        )));
    }
    println!("  Version {expected_version} \u{2713}");
    Ok(())
}

/// Ensure sudo is available (for minimal containers).
fn ensure_sudo_deb() {
    if command_exists("sudo") {
        return;
    }
    println!("  sudo not found, installing...");
    let _ = Command::new("apt-get").args(["update", "-qq"]).status();
    let _ = Command::new("apt-get")
        .args(["install", "-y", "-qq", "sudo"])
        .status();
}

fn ensure_sudo_rpm() {
    if command_exists("sudo") {
        return;
    }
    println!("  sudo not found, installing...");
    let _ = Command::new("dnf")
        .args(["install", "-y", "-q", "sudo"])
        .status();
}

pub fn run_test_deb() -> Result<()> {
    ensure_sudo_deb();
    let binary_name = env_or("BINARY_NAME", "");
    let version = env_or("VERSION", "");

    if binary_name.is_empty() {
        return Err(Error::User("binary-name is required".into()));
    }
    if version.is_empty() {
        return Err(Error::User("version is required".into()));
    }

    let artifact_path = if env_or("DOWNLOAD_FROM_RELEASE", "false") == "true" {
        let arch = env_or("ARCH", "");
        if arch.is_empty() {
            return Err(Error::User(
                "arch is required when download-from-release is true".into(),
            ));
        }
        let valid_archs = ["amd64", "arm64", "i386", "armhf"];
        if !valid_archs.contains(&arch.as_str()) {
            return Err(Error::User(format!(
                "invalid arch '{arch}': must be one of {}",
                valid_archs.join(", ")
            )));
        }
        download::download_artifact(&binary_name, &version, &arch, "deb")?
    } else {
        let path = env_or("ARTIFACT_PATH", "");
        if path.is_empty() {
            return Err(Error::User(
                "artifact is required when download-from-release is false".into(),
            ));
        }
        if !Path::new(&path).exists() {
            return Err(Error::User(format!("artifact not found: {path}")));
        }
        let checksum_file = env_or("CHECKSUM_FILE", "");
        if !checksum_file.is_empty() {
            verify_checksum(Path::new(&path), Path::new(&checksum_file))?;
        }
        path
    };

    println!("\x1b[32mTesting Debian package:\x1b[0m {artifact_path}");
    println!("\x1b[32mExpected version:\x1b[0m {version}");

    // Install
    println!("\x1b[32mInstalling package...\x1b[0m");
    run_command("sudo", &["dpkg", "-i", &artifact_path])?;
    println!("  Package installed \u{2713}");

    // Verify
    verify_installed_binary(&format!("/usr/bin/{binary_name}"), &version)?;

    // Uninstall
    println!("\x1b[32mUninstalling package...\x1b[0m");
    run_command("sudo", &["dpkg", "-r", &binary_name])?;
    println!("  Package uninstalled \u{2713}");

    println!("\x1b[32mAll tests passed\x1b[0m");
    output("result", "success");
    Ok(())
}

pub fn run_test_rpm() -> Result<()> {
    ensure_sudo_rpm();
    let binary_name = env_or("BINARY_NAME", "");
    let version = env_or("VERSION", "");

    if binary_name.is_empty() {
        return Err(Error::User("binary-name is required".into()));
    }
    if version.is_empty() {
        return Err(Error::User("version is required".into()));
    }

    let artifact_path = if env_or("DOWNLOAD_FROM_RELEASE", "false") == "true" {
        let arch = env_or("ARCH", "");
        if arch.is_empty() {
            return Err(Error::User(
                "arch is required when download-from-release is true".into(),
            ));
        }
        let valid_archs = ["x86_64", "aarch64", "i686", "armv7hl"];
        if !valid_archs.contains(&arch.as_str()) {
            return Err(Error::User(format!(
                "invalid arch '{arch}': must be one of {}",
                valid_archs.join(", ")
            )));
        }
        download::download_artifact(&binary_name, &version, &arch, "rpm")?
    } else {
        let path = env_or("ARTIFACT_PATH", "");
        if path.is_empty() {
            return Err(Error::User(
                "artifact is required when download-from-release is false".into(),
            ));
        }
        if !Path::new(&path).exists() {
            return Err(Error::User(format!("artifact not found: {path}")));
        }
        let checksum_file = env_or("CHECKSUM_FILE", "");
        if !checksum_file.is_empty() {
            verify_checksum(Path::new(&path), Path::new(&checksum_file))?;
        }
        path
    };

    println!("\x1b[32mTesting RPM package:\x1b[0m {artifact_path}");
    println!("\x1b[32mExpected version:\x1b[0m {version}");

    // Install
    println!("\x1b[32mInstalling package...\x1b[0m");
    run_command("sudo", &["rpm", "-i", &artifact_path])?;
    println!("  Package installed \u{2713}");

    // Verify
    verify_installed_binary(&format!("/usr/bin/{binary_name}"), &version)?;

    // Uninstall
    println!("\x1b[32mUninstalling package...\x1b[0m");
    run_command("sudo", &["rpm", "-e", &binary_name])?;
    println!("  Package uninstalled \u{2713}");

    println!("\x1b[32mAll tests passed\x1b[0m");
    output("result", "success");
    Ok(())
}

pub fn run_test_windows() -> Result<()> {
    let binary_name = env_or("BINARY_NAME", "");
    let version = env_or("VERSION", "");

    if binary_name.is_empty() {
        return Err(Error::User("binary-name is required".into()));
    }
    if version.is_empty() {
        return Err(Error::User("version is required".into()));
    }

    let (binary_path, msi_path) = if env_or("DOWNLOAD_FROM_RELEASE", "false") == "true" {
        let downloaded = download::download_windows_artifacts(&binary_name, &version)?;
        (downloaded.binary, downloaded.msi)
    } else {
        let binary_path = env_or("BINARY_PATH", "");
        let msi_path = env_or("MSI_PATH", "");
        let checksum_file = env_or("CHECKSUM_FILE", "");
        let msi_checksum_file = env_or("MSI_CHECKSUM_FILE", "");

        if binary_path.is_empty() && msi_path.is_empty() {
            return Err(Error::User(
                "binary-path or msi-path is required when download-from-release is false".into(),
            ));
        }

        if !binary_path.is_empty() && !checksum_file.is_empty() {
            verify_checksum(Path::new(&binary_path), Path::new(&checksum_file))?;
        }
        if !msi_path.is_empty() && !msi_checksum_file.is_empty() {
            verify_checksum(Path::new(&msi_path), Path::new(&msi_checksum_file))?;
        }

        (binary_path, msi_path)
    };

    println!("\x1b[32mTesting Windows artifacts\x1b[0m");
    println!("\x1b[32mExpected version:\x1b[0m {version}");

    if !binary_path.is_empty() {
        println!("\x1b[32mTesting binary:\x1b[0m {binary_path}");
        if !Path::new(&binary_path).exists() {
            return Err(Error::User(format!("binary not found: {binary_path}")));
        }
        let version_output = get_version_output(&binary_path);
        if !version_output.contains(&version) {
            return Err(Error::User(format!(
                "version mismatch: expected {version} in output: {version_output}"
            )));
        }
        println!("  Version {version} \u{2713}");
    }

    if !msi_path.is_empty() {
        println!("\x1b[32mTesting MSI installer:\x1b[0m {msi_path}");
        if !Path::new(&msi_path).exists() {
            return Err(Error::User(format!("MSI not found: {msi_path}")));
        }

        let user_profile = env_or("USERPROFILE", "C:/Users/runneradmin");
        let install_dir = format!("{user_profile}/{binary_name}-msi");

        // Install
        let win_path = format!("{}\\", install_dir.replace('/', "\\"));
        println!("\x1b[32mInstalling MSI to:\x1b[0m {win_path}");
        run_command(
            "msiexec",
            &[
                "/i",
                &msi_path,
                "/quiet",
                "/norestart",
                &format!("APPLICATIONFOLDER={win_path}"),
            ],
        )?;
        println!("  MSI installed \u{2713}");

        // Verify - check both root and bin subdirectory
        let possible_paths = [
            format!("{install_dir}/{binary_name}.exe"),
            format!("{install_dir}/bin/{binary_name}.exe"),
        ];
        let mut found_path = String::new();
        for p in &possible_paths {
            if Path::new(p).exists() {
                found_path = p.clone();
                break;
            }
        }
        if found_path.is_empty() {
            return Err(Error::User("installed binary not found".into()));
        }
        println!("  Found: {found_path}");
        let version_output = get_version_output(&found_path);
        if !version_output.contains(&version) {
            return Err(Error::User(format!(
                "version mismatch: expected {version} in output: {version_output}"
            )));
        }
        println!("  Version {version} \u{2713}");

        // Uninstall
        println!("\x1b[32mUninstalling MSI...\x1b[0m");
        run_command("msiexec", &["/x", &msi_path, "/quiet", "/norestart"])?;
        println!("  MSI uninstalled \u{2713}");
    }

    println!("\x1b[32mAll tests passed\x1b[0m");
    output("result", "success");
    Ok(())
}
