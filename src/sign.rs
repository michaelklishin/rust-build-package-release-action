use crate::error::{Error, Result};
use crate::output::{output, print_hr};
use crate::tools::{command_exists, run_command};
use std::env;
#[cfg(target_os = "windows")]
use std::fs;
use std::path::Path;
use std::process::Command;

/// Get cosign path, installing if missing.
fn get_cosign_path() -> Result<String> {
    if command_exists("cosign") {
        return Ok("cosign".to_string());
    }

    println!("\x1b[33mcosign not found, installing...\x1b[0m");
    let cosign_version = "3.0.4";

    #[cfg(target_os = "windows")]
    {
        let url = format!(
            "https://github.com/sigstore/cosign/releases/download/v{cosign_version}/cosign-windows-amd64.exe"
        );
        let user_profile = env::var("USERPROFILE").unwrap_or_default();
        let dest_dir = format!("{user_profile}/.local/bin");
        fs::create_dir_all(&dest_dir)?;
        let dest_path = format!("{dest_dir}/cosign.exe");
        run_command("curl", &["-fsSL", &url, "-o", &dest_path])?;
        return Ok(dest_path);
    }

    #[cfg(not(target_os = "windows"))]
    {
        let arch_output = Command::new("uname").arg("-m").output();
        let arch = match arch_output {
            Ok(ref o) => {
                let a = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if a == "arm64" || a == "aarch64" {
                    "arm64"
                } else {
                    "amd64"
                }
            }
            Err(_) => "amd64",
        };

        let os_output = Command::new("uname").arg("-s").output();
        let os = match os_output {
            Ok(ref o) => {
                let s = String::from_utf8_lossy(&o.stdout).trim().to_lowercase();
                if s == "darwin" { "darwin" } else { "linux" }
            }
            Err(_) => "linux",
        };

        let url = format!(
            "https://github.com/sigstore/cosign/releases/download/v{cosign_version}/cosign-{os}-{arch}"
        );
        run_command("curl", &["-fsSL", &url, "-o", "/tmp/cosign"])?;
        run_command("chmod", &["+x", "/tmp/cosign"])?;
        if command_exists("sudo") {
            run_command("sudo", &["mv", "/tmp/cosign", "/usr/local/bin/cosign"])?;
        } else {
            run_command("mv", &["/tmp/cosign", "/usr/local/bin/cosign"])?;
        }
        Ok("/usr/local/bin/cosign".to_string())
    }
}

pub fn run_sign_artifact() -> Result<()> {
    let cosign_path = get_cosign_path()?;

    let artifact_path = env::var("ARTIFACT_PATH").unwrap_or_default();
    if artifact_path.is_empty() {
        return Err(Error::User("ARTIFACT_PATH is required".into()));
    }

    if !Path::new(&artifact_path).exists() {
        return Err(Error::User(format!("artifact not found: {artifact_path}")));
    }

    println!("\x1b[32mSigning artifact:\x1b[0m {artifact_path}");

    let sig_path = format!("{artifact_path}.sig");
    let cert_path = format!("{artifact_path}.pem");
    let bundle_path = format!("{artifact_path}.sigstore.json");

    let result = Command::new(&cosign_path)
        .args([
            "sign-blob",
            "--yes",
            "--output-signature",
            &sig_path,
            "--output-certificate",
            &cert_path,
            "--bundle",
            &bundle_path,
            &artifact_path,
        ])
        .output()
        .map_err(|e| Error::User(format!("cosign failed: {e}")))?;

    if !result.status.success() {
        eprintln!("\x1b[31mcosign output:\x1b[0m");
        eprintln!("{}", String::from_utf8_lossy(&result.stderr));
        return Err(Error::User("cosign signing failed".into()));
    }

    println!();
    println!("\x1b[32mSignature files:\x1b[0m");
    print_hr();

    if Path::new(&sig_path).exists() {
        println!("\x1b[32mSignature:\x1b[0m {sig_path}");
        output("signature_path", &sig_path);
    }

    if Path::new(&cert_path).exists() {
        println!("\x1b[32mCertificate:\x1b[0m {cert_path}");
        output("certificate_path", &cert_path);
    }

    if Path::new(&bundle_path).exists() {
        println!("\x1b[32mBundle:\x1b[0m {bundle_path}");
        output("bundle_path", &bundle_path);
    }

    output("artifact_path", &artifact_path);
    Ok(())
}
