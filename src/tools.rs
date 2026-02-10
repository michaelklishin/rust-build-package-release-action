use crate::error::{Error, Result};
use std::process::Command;
use std::{env, path::Path, process};

/// Check that a command exists in PATH.
pub fn command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check that Rust toolchain is available.
pub fn check_rust_toolchain() -> Result<()> {
    if !command_exists("cargo") {
        eprintln!("\x1b[31mERROR:\x1b[0m Rust toolchain not found");
        eprintln!();
        eprintln!("Add a Rust setup step before this action:");
        eprintln!("  - uses: dtolnay/rust-toolchain@stable");
        eprintln!();
        eprintln!("Or install manually:");
        eprintln!("  rustup toolchain install stable --profile minimal");
        process::exit(1);
    }
    Ok(())
}

/// Check that nfpm is available, install if missing.
pub fn check_nfpm() -> Result<()> {
    if command_exists("nfpm") {
        return Ok(());
    }

    println!("\x1b[33mnfpm not found, installing...\x1b[0m");
    let arch = get_uname_arch();
    let nfpm_arch = if arch == "aarch64" { "arm64" } else { "x86_64" };
    let nfpm_version = "2.44.2";
    let url = format!(
        "https://github.com/goreleaser/nfpm/releases/download/v{nfpm_version}/nfpm_{nfpm_version}_Linux_{nfpm_arch}.tar.gz"
    );

    run_command(
        "bash",
        &["-c", &format!("curl -fsSL '{url}' | tar xz -C /tmp nfpm")],
    )?;
    run_command("sudo", &["mv", "/tmp/nfpm", "/usr/local/bin/nfpm"])?;
    Ok(())
}

/// Check that cargo-sbom is available, install if missing.
pub fn check_cargo_sbom() -> Result<()> {
    if command_exists("cargo-sbom") {
        return Ok(());
    }
    println!("\x1b[33mcargo-sbom not found, installing...\x1b[0m");
    run_command("cargo", &["install", "cargo-sbom"])?;
    Ok(())
}

/// Check that cargo-zigbuild is available, install if missing.
pub fn check_zigbuild() -> Result<()> {
    if command_exists("cargo-zigbuild") {
        return Ok(());
    }
    println!("\x1b[33mcargo-zigbuild not found, installing...\x1b[0m");
    if command_exists("pip3") {
        run_command("pip3", &["install", "cargo-zigbuild"])?;
    } else if command_exists("pipx") {
        run_command("pipx", &["install", "cargo-zigbuild"])?;
    } else {
        run_command("cargo", &["install", "cargo-zigbuild"])?;
    }
    Ok(())
}

/// Ensure Cargo.lock exists.
pub fn ensure_lockfile() -> Result<()> {
    if !Path::new("Cargo.lock").exists() {
        println!("\x1b[33mGenerating Cargo.lock...\x1b[0m");
        run_command("cargo", &["generate-lockfile"])?;
    }
    Ok(())
}

/// Run pre-build hook command if PRE_BUILD is set.
pub fn run_pre_build_hook() -> Result<()> {
    let pre_build = env::var("PRE_BUILD").unwrap_or_default();
    if pre_build.is_empty() {
        return Ok(());
    }
    println!("\x1b[32mRunning pre-build hook...\x1b[0m");
    let output = Command::new("bash")
        .args(["-c", &pre_build])
        .output()
        .map_err(|e| Error::User(format!("failed to run pre-build hook: {e}")))?;
    if !output.status.success() {
        return Err(Error::User(format!(
            "pre-build hook failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    Ok(())
}

/// Install cross-compilation dependencies for Linux targets.
pub fn install_linux_cross_deps(target: &str) -> Result<()> {
    let is_ubuntu = command_exists("apt-get");
    let is_fedora = command_exists("dnf");

    if target.contains("musl") {
        if is_ubuntu {
            run_command("sudo", &["apt-get", "update", "-qq"])?;
            run_command("sudo", &["apt-get", "install", "-y", "-qq", "musl-tools"])?;
        }
    } else if target == "aarch64-unknown-linux-gnu" {
        let arch = get_uname_arch();
        if arch != "aarch64" {
            if is_ubuntu {
                run_command("sudo", &["apt-get", "update", "-qq"])?;
                run_command(
                    "sudo",
                    &["apt-get", "install", "-y", "-qq", "gcc-aarch64-linux-gnu"],
                )?;
            } else if is_fedora {
                run_command("sudo", &["dnf", "install", "-y", "gcc-aarch64-linux-gnu"])?;
            }
            // Safety: running single-threaded at this point during build setup
            unsafe {
                env::set_var(
                    "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER",
                    "aarch64-linux-gnu-gcc",
                );
            }
        }
    } else if target == "armv7-unknown-linux-gnueabihf" {
        if is_ubuntu {
            run_command("sudo", &["apt-get", "update", "-qq"])?;
            run_command(
                "sudo",
                &[
                    "apt-get",
                    "install",
                    "-y",
                    "-qq",
                    "pkg-config",
                    "gcc-arm-linux-gnueabihf",
                ],
            )?;
        } else if is_fedora {
            run_command(
                "sudo",
                &[
                    "dnf",
                    "install",
                    "-y",
                    "pkg-config",
                    "gcc-arm-linux-gnueabihf",
                ],
            )?;
        }
        // Safety: running single-threaded at this point during build setup
        unsafe {
            env::set_var(
                "CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER",
                "arm-linux-gnueabihf-gcc",
            );
        }
    }

    run_command("rustup", &["target", "add", target])?;
    Ok(())
}

/// Run a command and return an error if it fails.
pub fn run_command(program: &str, args: &[&str]) -> Result<process::Output> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| Error::Command {
            command: format!("{program} {}", args.join(" ")),
            stderr: e.to_string(),
        })?;
    if !output.status.success() {
        return Err(Error::Command {
            command: format!("{program} {}", args.join(" ")),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(output)
}

/// Run a command, inheriting stdio (for commands that need to show output).
pub fn run_command_inherit(program: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| Error::Command {
            command: format!("{program} {}", args.join(" ")),
            stderr: e.to_string(),
        })?;
    if !status.success() {
        return Err(Error::Command {
            command: format!("{program} {}", args.join(" ")),
            stderr: format!("exit code: {}", status.code().unwrap_or(-1)),
        });
    }
    Ok(())
}

fn get_uname_arch() -> String {
    Command::new("uname")
        .arg("-m")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "x86_64".to_string())
}
