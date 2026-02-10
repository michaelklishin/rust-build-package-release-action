use crate::archive::{copy_docs, copy_includes, list_archivable_files};
use crate::build::{cargo_build, output_build_results};
use crate::cargo_info::get_cargo_info;
use crate::checksum::generate_checksums;
use crate::env_or;
use crate::error::{Error, Result};
use crate::nfpm::{nfpm_base_config, nfpm_contents_section, nfpm_dependencies_section};
use crate::output::{output, print_hr};
use crate::platform::{target_to_apk_arch, target_to_deb_arch, target_to_rpm_arch};
use crate::tools::{
    check_nfpm, check_rust_toolchain, command_exists, ensure_lockfile, install_linux_cross_deps,
    run_command, run_command_inherit, run_pre_build_hook,
};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

/// Common build logic for release commands that produce a binary.
struct BuildContext {
    binary_name: String,
    version: String,
    target: String,
    release_dir: String,
    skip_build: bool,
    create_archive: bool,
}

fn setup_build_context(default_target: &str) -> Result<BuildContext> {
    let skip_build = env_or("SKIP_BUILD", "") == "true";

    if !skip_build {
        check_rust_toolchain()?;
    }

    let target = env_or("TARGET", default_target);
    let info = get_cargo_info()?;
    let binary_name = env_or("BINARY_NAME", &info.name);
    let version = info.version;
    let create_archive = env_or("ARCHIVE", "") == "true";

    if binary_name.is_empty() {
        return Err(Error::User("could not determine binary name".into()));
    }
    if version.is_empty() {
        return Err(Error::User("could not determine version".into()));
    }

    let release_dir = format!("target/{target}/release");

    Ok(BuildContext {
        binary_name,
        version,
        target,
        release_dir,
        skip_build,
        create_archive,
    })
}

pub fn run_release() -> Result<()> {
    let target = env_or("TARGET", "");
    if target.is_empty() {
        return Err(Error::User(
            "TARGET is required for the unified release command".into(),
        ));
    }

    println!("\x1b[32mAuto-selected:\x1b[0m platform for target {target}");

    if target.contains("linux") {
        run_release_linux()
    } else if target.contains("darwin") || target.contains("apple") {
        run_release_macos()
    } else if target.contains("windows") {
        run_release_windows()
    } else {
        Err(Error::User(format!(
            "Cannot determine platform from target: {target}. Use release-linux, release-macos, or release-windows directly."
        )))
    }
}

pub fn run_release_linux() -> Result<()> {
    let ctx = setup_build_context("x86_64-unknown-linux-gnu")?;

    if ctx.skip_build {
        let custom = env_or("BINARY_PATH", "");
        if custom.is_empty() {
            return Err(Error::User(
                "binary-path is required when skip-build is true".into(),
            ));
        }
        if !Path::new(&custom).exists() {
            return Err(Error::User(format!("binary not found: {custom}")));
        }
        println!(
            "\x1b[32mPackaging\x1b[0m {} v{} for {} (skip-build)",
            ctx.binary_name, ctx.version, ctx.target
        );
        fs::create_dir_all(&ctx.release_dir)?;
        fs::copy(&custom, format!("{}/{}", ctx.release_dir, ctx.binary_name))?;
        run_command_inherit(
            "chmod",
            &["+x", &format!("{}/{}", ctx.release_dir, ctx.binary_name)],
        )?;
    } else {
        println!(
            "\x1b[32mBuilding\x1b[0m {} v{} for {}",
            ctx.binary_name, ctx.version, ctx.target
        );
        let _ = fs::remove_dir_all(&ctx.release_dir);
        fs::create_dir_all(&ctx.release_dir)?;
        ensure_lockfile()?;
        run_pre_build_hook()?;
        install_linux_cross_deps(&ctx.target)?;
        cargo_build(&ctx.target, &ctx.binary_name)?;
    }

    let binary_path = format!("{}/{}", ctx.release_dir, ctx.binary_name);
    if !Path::new(&binary_path).exists() {
        return Err(Error::User(format!("binary not found: {binary_path}")));
    }

    copy_docs(Path::new(&ctx.release_dir))?;
    copy_includes(Path::new(&ctx.release_dir))?;

    let artifact_base = format!("{}-{}-{}", ctx.binary_name, ctx.version, ctx.target);

    output("version", &ctx.version);
    output("binary_name", &ctx.binary_name);
    output("target", &ctx.target);
    output("binary_path", &binary_path);

    // Always create bare binary artifact
    let bare_artifact = &artifact_base;
    let bare_artifact_path = format!("{}/{bare_artifact}", ctx.release_dir);
    fs::copy(&binary_path, &bare_artifact_path)?;
    run_command_inherit("chmod", &["+x", &bare_artifact_path])?;
    output("bare_artifact", bare_artifact);
    output("bare_artifact_path", &bare_artifact_path);

    if ctx.create_archive {
        let artifact = format!("{artifact_base}.tar.gz");
        let artifact_path = format!("{}/{artifact}", ctx.release_dir);
        println!("\x1b[32mCreating archive:\x1b[0m {artifact}");
        let files = list_archivable_files(Path::new(&ctx.release_dir));
        let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
        let mut tar_args = vec!["-C", &ctx.release_dir, "-czf", &artifact_path];
        tar_args.extend_from_slice(&file_refs);
        run_command_inherit("tar", &tar_args)?;

        generate_checksums(Path::new(&bare_artifact_path))?;
        let checksums = generate_checksums(Path::new(&artifact_path))?;
        println!();
        println!("\x1b[32mBuild artifacts:\x1b[0m");
        print_hr();
        println!("\x1b[32mCreated:\x1b[0m {bare_artifact}, {artifact}");
        output_build_results(
            &ctx.binary_name,
            &ctx.version,
            &ctx.target,
            &artifact,
            &artifact_path,
            &checksums,
        );
    } else {
        let checksums = generate_checksums(Path::new(&bare_artifact_path))?;
        println!();
        println!("\x1b[32mBuild artifacts:\x1b[0m");
        print_hr();
        println!("\x1b[32mCreated:\x1b[0m {bare_artifact}");
        output_build_results(
            &ctx.binary_name,
            &ctx.version,
            &ctx.target,
            bare_artifact,
            &bare_artifact_path,
            &checksums,
        );
    }

    Ok(())
}

pub fn run_release_macos() -> Result<()> {
    let ctx = setup_build_context("aarch64-apple-darwin")?;

    if ctx.skip_build {
        let custom = env_or("BINARY_PATH", "");
        if custom.is_empty() {
            return Err(Error::User(
                "binary-path is required when skip-build is true".into(),
            ));
        }
        if !Path::new(&custom).exists() {
            return Err(Error::User(format!("binary not found: {custom}")));
        }
        println!(
            "\x1b[32mPackaging\x1b[0m {} v{} for {} (skip-build)",
            ctx.binary_name, ctx.version, ctx.target
        );
        fs::create_dir_all(&ctx.release_dir)?;
        fs::copy(&custom, format!("{}/{}", ctx.release_dir, ctx.binary_name))?;
        run_command_inherit(
            "chmod",
            &["+x", &format!("{}/{}", ctx.release_dir, ctx.binary_name)],
        )?;
    } else {
        println!(
            "\x1b[32mBuilding\x1b[0m {} v{} for {}",
            ctx.binary_name, ctx.version, ctx.target
        );
        let _ = fs::remove_dir_all(&ctx.release_dir);
        fs::create_dir_all(&ctx.release_dir)?;
        ensure_lockfile()?;
        run_pre_build_hook()?;
        run_command_inherit("rustup", &["target", "add", &ctx.target])?;
        cargo_build(&ctx.target, &ctx.binary_name)?;
    }

    let binary_path = format!("{}/{}", ctx.release_dir, ctx.binary_name);
    if !Path::new(&binary_path).exists() {
        return Err(Error::User(format!("binary not found: {binary_path}")));
    }

    copy_docs(Path::new(&ctx.release_dir))?;
    copy_includes(Path::new(&ctx.release_dir))?;

    let artifact_base = format!("{}-{}-{}", ctx.binary_name, ctx.version, ctx.target);

    output("version", &ctx.version);
    output("binary_name", &ctx.binary_name);
    output("target", &ctx.target);
    output("binary_path", &binary_path);

    let bare_artifact = &artifact_base;
    let bare_artifact_path = format!("{}/{bare_artifact}", ctx.release_dir);
    fs::copy(&binary_path, &bare_artifact_path)?;
    run_command_inherit("chmod", &["+x", &bare_artifact_path])?;
    output("bare_artifact", bare_artifact);
    output("bare_artifact_path", &bare_artifact_path);

    if ctx.create_archive {
        let artifact = format!("{artifact_base}.tar.gz");
        let artifact_path = format!("{}/{artifact}", ctx.release_dir);
        println!("\x1b[32mCreating archive:\x1b[0m {artifact}");
        let files = list_archivable_files(Path::new(&ctx.release_dir));
        let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
        let mut tar_args = vec!["-C", &ctx.release_dir, "-czf", &artifact_path];
        tar_args.extend_from_slice(&file_refs);
        run_command_inherit("tar", &tar_args)?;

        generate_checksums(Path::new(&bare_artifact_path))?;
        let checksums = generate_checksums(Path::new(&artifact_path))?;
        println!();
        println!("\x1b[32mBuild artifacts:\x1b[0m");
        print_hr();
        println!("\x1b[32mCreated:\x1b[0m {bare_artifact}, {artifact}");
        output_build_results(
            &ctx.binary_name,
            &ctx.version,
            &ctx.target,
            &artifact,
            &artifact_path,
            &checksums,
        );
    } else {
        let checksums = generate_checksums(Path::new(&bare_artifact_path))?;
        println!();
        println!("\x1b[32mBuild artifacts:\x1b[0m");
        print_hr();
        println!("\x1b[32mCreated:\x1b[0m {bare_artifact}");
        output_build_results(
            &ctx.binary_name,
            &ctx.version,
            &ctx.target,
            bare_artifact,
            &bare_artifact_path,
            &checksums,
        );
    }

    Ok(())
}

pub fn run_release_windows() -> Result<()> {
    let ctx = setup_build_context("x86_64-pc-windows-msvc")?;

    if ctx.skip_build {
        let custom = env_or("BINARY_PATH", "");
        if custom.is_empty() {
            return Err(Error::User(
                "binary-path is required when skip-build is true".into(),
            ));
        }
        if !Path::new(&custom).exists() {
            return Err(Error::User(format!("binary not found: {custom}")));
        }
        println!(
            "\x1b[32mPackaging\x1b[0m {} v{} for {} (skip-build)",
            ctx.binary_name, ctx.version, ctx.target
        );
        fs::create_dir_all(&ctx.release_dir)?;
        fs::copy(
            &custom,
            format!("{}/{}.exe", ctx.release_dir, ctx.binary_name),
        )?;
    } else {
        println!(
            "\x1b[32mBuilding\x1b[0m {} v{} for {}",
            ctx.binary_name, ctx.version, ctx.target
        );
        let _ = fs::remove_dir_all(&ctx.release_dir);
        fs::create_dir_all(&ctx.release_dir)?;
        ensure_lockfile()?;
        run_pre_build_hook()?;
        run_command_inherit("rustup", &["target", "add", &ctx.target])?;
        cargo_build(&ctx.target, &ctx.binary_name)?;
    }

    let binary_path = format!("{}/{}.exe", ctx.release_dir, ctx.binary_name);
    if !Path::new(&binary_path).exists() {
        return Err(Error::User(format!("binary not found: {binary_path}")));
    }

    copy_docs(Path::new(&ctx.release_dir))?;
    copy_includes(Path::new(&ctx.release_dir))?;

    let artifact_base = format!("{}-{}-{}", ctx.binary_name, ctx.version, ctx.target);

    output("version", &ctx.version);
    output("binary_name", &ctx.binary_name);
    output("target", &ctx.target);
    output("binary_path", &binary_path.replace('\\', "/"));

    let bare_artifact = format!("{artifact_base}.exe");
    let bare_artifact_path = format!("{}/{bare_artifact}", ctx.release_dir);
    fs::copy(&binary_path, &bare_artifact_path)?;
    output("bare_artifact", &bare_artifact);
    output("bare_artifact_path", &bare_artifact_path.replace('\\', "/"));

    if ctx.create_archive {
        let artifact = format!("{artifact_base}.zip");
        let artifact_path = format!("{}/{artifact}", ctx.release_dir);
        println!("\x1b[32mCreating archive:\x1b[0m {artifact}");
        let files = list_archivable_files(Path::new(&ctx.release_dir));
        let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
        // Use 7z on Windows
        let original_dir = env::current_dir()?;
        env::set_current_dir(&ctx.release_dir)?;
        let mut zip_args = vec!["a", &artifact];
        zip_args.extend_from_slice(&file_refs);
        let zip_result = run_command_inherit("7z", &zip_args);
        env::set_current_dir(original_dir)?;
        zip_result?;

        generate_checksums(Path::new(&bare_artifact_path))?;
        let checksums = generate_checksums(Path::new(&artifact_path))?;
        let normalised_path = artifact_path.replace('\\', "/");
        println!();
        println!("\x1b[32mBuild artifacts:\x1b[0m");
        print_hr();
        println!("\x1b[32mCreated:\x1b[0m {bare_artifact}, {artifact}");
        output_build_results(
            &ctx.binary_name,
            &ctx.version,
            &ctx.target,
            &artifact,
            &normalised_path,
            &checksums,
        );
    } else {
        let checksums = generate_checksums(Path::new(&bare_artifact_path))?;
        let normalised_path = bare_artifact_path.replace('\\', "/");
        println!();
        println!("\x1b[32mBuild artifacts:\x1b[0m");
        print_hr();
        println!("\x1b[32mCreated:\x1b[0m {bare_artifact}");
        output_build_results(
            &ctx.binary_name,
            &ctx.version,
            &ctx.target,
            &bare_artifact,
            &normalised_path,
            &checksums,
        );
    }

    Ok(())
}

pub fn run_release_linux_deb() -> Result<()> {
    let skip_build = env_or("SKIP_BUILD", "") == "true";
    let custom_binary_path = env_or("BINARY_PATH", "");

    if !skip_build {
        check_rust_toolchain()?;
    }
    check_nfpm()?;

    let target = env_or("TARGET", "x86_64-unknown-linux-gnu");
    let info = get_cargo_info()?;
    let binary_name = env_or("BINARY_NAME", &info.name);
    let version = info.version;

    if binary_name.is_empty() {
        return Err(Error::User("could not determine binary name".into()));
    }
    if version.is_empty() {
        return Err(Error::User("could not determine version".into()));
    }

    let arch = target_to_deb_arch(&target)?;
    println!("\x1b[32mBuilding .deb package:\x1b[0m {binary_name} v{version} for {arch}");

    let release_dir = format!("target/{target}/release");
    let binary_path = if skip_build && !custom_binary_path.is_empty() {
        custom_binary_path
    } else {
        format!("{release_dir}/{binary_name}")
    };

    if !Path::new(&binary_path).exists() {
        if skip_build {
            return Err(Error::User(format!("binary not found: {binary_path}")));
        }
        println!("\x1b[33mBinary not found, building...\x1b[0m");
        let _ = fs::remove_dir_all(&release_dir);
        fs::create_dir_all(&release_dir)?;
        ensure_lockfile()?;
        run_pre_build_hook()?;
        install_linux_cross_deps(&target)?;
        cargo_build(&target, &binary_name)?;
    }

    if !Path::new(&binary_path).exists() {
        return Err(Error::User(format!("binary not found: {binary_path}")));
    }

    let pkg_dir = "target/pkg-deb";
    let _ = fs::remove_dir_all(pkg_dir);
    fs::create_dir_all(pkg_dir)?;

    let abs_binary_path = fs::canonicalize(&binary_path)?;
    let abs_str = abs_binary_path.to_string_lossy().to_string();

    let section = env_or("PKG_SECTION", "utils");
    let priority = env_or("PKG_PRIORITY", "optional");

    let mut nfpm_config = nfpm_base_config(&binary_name, &version, arch);
    nfpm_config.push_str(&format!("section: \"{section}\"\n"));
    nfpm_config.push_str(&format!("priority: \"{priority}\"\n"));
    nfpm_config.push_str(&nfpm_contents_section(&binary_name, &abs_str));
    nfpm_config.push_str(&nfpm_dependencies_section());

    let config_path = format!("{pkg_dir}/nfpm.yaml");
    fs::write(&config_path, &nfpm_config)?;

    let artifact = format!("{binary_name}_{version}_{arch}.deb");
    let artifact_path = format!("{release_dir}/{artifact}");

    println!("\x1b[32mRunning nfpm...\x1b[0m");
    run_command_inherit(
        "nfpm",
        &[
            "package",
            "--config",
            &config_path,
            "--packager",
            "deb",
            "--target",
            &artifact_path,
        ],
    )?;

    if !Path::new(&artifact_path).exists() {
        return Err(Error::User(format!(
            "failed to create package: {artifact_path}"
        )));
    }

    let checksums = generate_checksums(Path::new(&artifact_path))?;
    println!();
    println!("\x1b[32mBuild artifacts:\x1b[0m");
    print_hr();
    println!("\x1b[32mCreated:\x1b[0m {artifact}");

    output("version", &version);
    output("binary_name", &binary_name);
    output("target", &target);
    output("binary_path", &binary_path);
    output_build_results(
        &binary_name,
        &version,
        &target,
        &artifact,
        &artifact_path,
        &checksums,
    );
    Ok(())
}

pub fn run_release_linux_rpm() -> Result<()> {
    let skip_build = env_or("SKIP_BUILD", "") == "true";
    let custom_binary_path = env_or("BINARY_PATH", "");

    if !skip_build {
        check_rust_toolchain()?;
    }
    check_nfpm()?;

    let target = env_or("TARGET", "x86_64-unknown-linux-gnu");
    let info = get_cargo_info()?;
    let binary_name = env_or("BINARY_NAME", &info.name);
    let version = info.version;

    if binary_name.is_empty() {
        return Err(Error::User("could not determine binary name".into()));
    }
    if version.is_empty() {
        return Err(Error::User("could not determine version".into()));
    }

    let arch = target_to_rpm_arch(&target)?;
    println!("\x1b[32mBuilding .rpm package:\x1b[0m {binary_name} v{version} for {arch}");

    let release_dir = format!("target/{target}/release");
    let binary_path = if skip_build && !custom_binary_path.is_empty() {
        custom_binary_path
    } else {
        format!("{release_dir}/{binary_name}")
    };

    if !Path::new(&binary_path).exists() {
        if skip_build {
            return Err(Error::User(format!("binary not found: {binary_path}")));
        }
        println!("\x1b[33mBinary not found, building...\x1b[0m");
        let _ = fs::remove_dir_all(&release_dir);
        fs::create_dir_all(&release_dir)?;
        ensure_lockfile()?;
        run_pre_build_hook()?;
        install_linux_cross_deps(&target)?;
        cargo_build(&target, &binary_name)?;
    }

    if !Path::new(&binary_path).exists() {
        return Err(Error::User(format!("binary not found: {binary_path}")));
    }

    let pkg_dir = "target/pkg-rpm";
    let _ = fs::remove_dir_all(pkg_dir);
    fs::create_dir_all(pkg_dir)?;

    let abs_binary_path = fs::canonicalize(&binary_path)?;
    let abs_str = abs_binary_path.to_string_lossy().to_string();

    let release_num = env_or("PKG_RELEASE", "1");
    let group = env_or("PKG_GROUP", "Applications/System");
    let description = env_or(
        "PKG_DESCRIPTION",
        &format!("{binary_name} - built with rust-build-package-release-action"),
    );
    let summary = env_or("PKG_SUMMARY", &description);

    let mut nfpm_config = nfpm_base_config(&binary_name, &version, arch);
    nfpm_config.push_str(&format!("release: \"{release_num}\"\n"));
    nfpm_config.push_str(&nfpm_contents_section(&binary_name, &abs_str));
    nfpm_config.push_str(&format!(
        "\nrpm:\n  group: \"{group}\"\n  summary: \"{summary}\"\n  compression: gzip\n"
    ));
    nfpm_config.push_str(&nfpm_dependencies_section());

    let config_path = format!("{pkg_dir}/nfpm.yaml");
    fs::write(&config_path, &nfpm_config)?;

    let artifact = format!("{binary_name}-{version}-{release_num}.{arch}.rpm");
    let artifact_path = format!("{release_dir}/{artifact}");

    println!("\x1b[32mRunning nfpm...\x1b[0m");
    run_command_inherit(
        "nfpm",
        &[
            "package",
            "--config",
            &config_path,
            "--packager",
            "rpm",
            "--target",
            &artifact_path,
        ],
    )?;

    if !Path::new(&artifact_path).exists() {
        return Err(Error::User(format!(
            "failed to create package: {artifact_path}"
        )));
    }

    let checksums = generate_checksums(Path::new(&artifact_path))?;
    println!();
    println!("\x1b[32mBuild artifacts:\x1b[0m");
    print_hr();
    println!("\x1b[32mCreated:\x1b[0m {artifact}");

    output("version", &version);
    output("binary_name", &binary_name);
    output("target", &target);
    output("binary_path", &binary_path);
    output_build_results(
        &binary_name,
        &version,
        &target,
        &artifact,
        &artifact_path,
        &checksums,
    );
    Ok(())
}

pub fn run_release_linux_apk() -> Result<()> {
    let skip_build = env_or("SKIP_BUILD", "") == "true";
    let custom_binary_path = env_or("BINARY_PATH", "");

    if !skip_build {
        check_rust_toolchain()?;
    }
    check_nfpm()?;

    let target = env_or("TARGET", "x86_64-unknown-linux-musl");
    let info = get_cargo_info()?;
    let binary_name = env_or("BINARY_NAME", &info.name);
    let version = info.version;

    if binary_name.is_empty() {
        return Err(Error::User("could not determine binary name".into()));
    }
    if version.is_empty() {
        return Err(Error::User("could not determine version".into()));
    }

    let arch = target_to_apk_arch(&target)?;
    println!("\x1b[32mBuilding .apk package:\x1b[0m {binary_name} v{version} for {arch}");

    let release_dir = format!("target/{target}/release");
    let binary_path = if skip_build && !custom_binary_path.is_empty() {
        custom_binary_path
    } else {
        format!("{release_dir}/{binary_name}")
    };

    if !Path::new(&binary_path).exists() {
        if skip_build {
            return Err(Error::User(format!("binary not found: {binary_path}")));
        }
        println!("\x1b[33mBinary not found, building...\x1b[0m");
        let _ = fs::remove_dir_all(&release_dir);
        fs::create_dir_all(&release_dir)?;
        ensure_lockfile()?;
        run_pre_build_hook()?;
        install_linux_cross_deps(&target)?;
        cargo_build(&target, &binary_name)?;
    }

    if !Path::new(&binary_path).exists() {
        return Err(Error::User(format!("binary not found: {binary_path}")));
    }

    let pkg_dir = "target/pkg-apk";
    let _ = fs::remove_dir_all(pkg_dir);
    fs::create_dir_all(pkg_dir)?;

    let abs_binary_path = fs::canonicalize(&binary_path)?;
    let abs_str = abs_binary_path.to_string_lossy().to_string();

    let mut nfpm_config = nfpm_base_config(&binary_name, &version, arch);
    nfpm_config.push_str(&nfpm_contents_section(&binary_name, &abs_str));
    nfpm_config.push_str(&nfpm_dependencies_section());

    let config_path = format!("{pkg_dir}/nfpm.yaml");
    fs::write(&config_path, &nfpm_config)?;

    let release_num = env_or("PKG_RELEASE", "0");
    let artifact = format!("{binary_name}-{version}-r{release_num}.apk");
    let artifact_path = format!("{release_dir}/{artifact}");

    println!("\x1b[32mRunning nfpm...\x1b[0m");
    run_command_inherit(
        "nfpm",
        &[
            "package",
            "--config",
            &config_path,
            "--packager",
            "apk",
            "--target",
            &artifact_path,
        ],
    )?;

    if !Path::new(&artifact_path).exists() {
        return Err(Error::User(format!(
            "failed to create package: {artifact_path}"
        )));
    }

    let checksums = generate_checksums(Path::new(&artifact_path))?;
    println!();
    println!("\x1b[32mBuild artifacts:\x1b[0m");
    print_hr();
    println!("\x1b[32mCreated:\x1b[0m {artifact}");

    output("version", &version);
    output("binary_name", &binary_name);
    output("target", &target);
    output("binary_path", &binary_path);
    output_build_results(
        &binary_name,
        &version,
        &target,
        &artifact,
        &artifact_path,
        &checksums,
    );
    Ok(())
}

pub fn run_release_macos_dmg() -> Result<()> {
    let skip_build = env_or("SKIP_BUILD", "") == "true";
    let custom_binary_path = env_or("BINARY_PATH", "");

    if !skip_build {
        check_rust_toolchain()?;
    }

    let target = env_or("TARGET", "aarch64-apple-darwin");
    let info = get_cargo_info()?;
    let binary_name = env_or("BINARY_NAME", &info.name);
    let version = info.version;

    if binary_name.is_empty() {
        return Err(Error::User("could not determine binary name".into()));
    }
    if version.is_empty() {
        return Err(Error::User("could not determine version".into()));
    }

    println!("\x1b[32mBuilding .dmg installer:\x1b[0m {binary_name} v{version} for {target}");

    let release_dir = format!("target/{target}/release");
    let binary_path = if skip_build && !custom_binary_path.is_empty() {
        custom_binary_path
    } else {
        format!("{release_dir}/{binary_name}")
    };

    if !Path::new(&binary_path).exists() {
        if skip_build {
            return Err(Error::User(format!("binary not found: {binary_path}")));
        }
        println!("\x1b[33mBinary not found, building...\x1b[0m");
        let _ = fs::remove_dir_all(&release_dir);
        fs::create_dir_all(&release_dir)?;
        ensure_lockfile()?;
        run_pre_build_hook()?;
        run_command_inherit("rustup", &["target", "add", &target])?;
        cargo_build(&target, &binary_name)?;
    }

    if !Path::new(&binary_path).exists() {
        return Err(Error::User(format!("binary not found: {binary_path}")));
    }

    let dmg_dir = "target/dmg-contents";
    let _ = fs::remove_dir_all(dmg_dir);
    fs::create_dir_all(dmg_dir)?;

    fs::copy(&binary_path, format!("{dmg_dir}/{binary_name}"))?;
    run_command_inherit("chmod", &["+x", &format!("{dmg_dir}/{binary_name}")])?;
    copy_docs(Path::new(dmg_dir))?;
    copy_includes(Path::new(dmg_dir))?;

    // Create install/uninstall scripts
    create_install_script(dmg_dir, &binary_name)?;
    create_uninstall_script(dmg_dir, &binary_name)?;

    let vol_name = format!("{binary_name}-{version}");
    let artifact = format!("{binary_name}-{version}-{target}.dmg");
    let artifact_path = format!("{release_dir}/{artifact}");

    println!("\x1b[32mCreating DMG...\x1b[0m");
    create_dmg(dmg_dir, &vol_name, &artifact_path)?;

    if !Path::new(&artifact_path).exists() {
        return Err(Error::User(format!(
            "failed to create DMG: {artifact_path}"
        )));
    }

    let checksums = generate_checksums(Path::new(&artifact_path))?;
    println!();
    println!("\x1b[32mBuild artifacts:\x1b[0m");
    print_hr();
    println!("\x1b[32mCreated:\x1b[0m {artifact}");

    output("version", &version);
    output("binary_name", &binary_name);
    output("target", &target);
    output("binary_path", &binary_path);
    output_build_results(
        &binary_name,
        &version,
        &target,
        &artifact,
        &artifact_path,
        &checksums,
    );
    Ok(())
}

fn create_dmg(src_dir: &str, vol_name: &str, output_path: &str) -> Result<()> {
    let temp_dmg = format!("{output_path}.temp.dmg");

    run_command_inherit("sync", &[])?;

    // Retry up to 3 times for "Resource busy" errors
    let mut attempts = 0;
    loop {
        let result = Command::new("hdiutil")
            .args([
                "create",
                "-srcfolder",
                src_dir,
                "-volname",
                vol_name,
                "-fs",
                "HFS+",
                "-format",
                "UDRW",
                "-ov",
                &temp_dmg,
            ])
            .output();
        match result {
            Ok(o) if o.status.success() => break,
            Ok(o) => {
                attempts += 1;
                if attempts >= 3 {
                    return Err(Error::Command {
                        command: "hdiutil create".into(),
                        stderr: String::from_utf8_lossy(&o.stderr).into(),
                    });
                }
                println!("\x1b[33mhdiutil create failed, retrying in 2s...\x1b[0m");
                thread::sleep(Duration::from_secs(2));
            }
            Err(e) => {
                return Err(Error::Command {
                    command: "hdiutil create".into(),
                    stderr: e.to_string(),
                });
            }
        }
    }

    let result = Command::new("hdiutil")
        .args(["convert", &temp_dmg, "-format", "UDZO", "-o", output_path])
        .output()
        .map_err(|e| Error::Command {
            command: "hdiutil convert".into(),
            stderr: e.to_string(),
        })?;
    if !result.status.success() {
        let _ = fs::remove_file(&temp_dmg);
        return Err(Error::Command {
            command: "hdiutil convert".into(),
            stderr: String::from_utf8_lossy(&result.stderr).into(),
        });
    }

    let _ = fs::remove_file(&temp_dmg);
    Ok(())
}

fn create_install_script(dir: &str, binary_name: &str) -> Result<()> {
    let script = format!(
        r#"#!/bin/bash
# Install {binary_name} to /usr/local/bin
set -e

INSTALL_DIR="/usr/local/bin"
BINARY="{binary_name}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if [ ! -f "$SCRIPT_DIR/$BINARY" ]; then
    echo "Error: $BINARY not found in $SCRIPT_DIR"
    exit 1
fi

echo "Installing $BINARY to $INSTALL_DIR..."
sudo mkdir -p "$INSTALL_DIR"
sudo cp "$SCRIPT_DIR/$BINARY" "$INSTALL_DIR/$BINARY"
sudo chmod +x "$INSTALL_DIR/$BINARY"
echo "Done. Run '$BINARY --help' to get started.""#
    );
    let path = format!("{dir}/install.sh");
    fs::write(&path, &script)?;
    run_command_inherit("chmod", &["+x", &path])?;
    Ok(())
}

fn create_uninstall_script(dir: &str, binary_name: &str) -> Result<()> {
    let script = format!(
        r#"#!/bin/bash
# Uninstall {binary_name} from /usr/local/bin
set -e

INSTALL_DIR="/usr/local/bin"
BINARY="{binary_name}"

if [ -f "$INSTALL_DIR/$BINARY" ]; then
    echo "Removing $BINARY from $INSTALL_DIR..."
    sudo rm -f "$INSTALL_DIR/$BINARY"
    echo "Done. $BINARY has been uninstalled."
else
    echo "$BINARY is not installed in $INSTALL_DIR"
fi"#
    );
    let path = format!("{dir}/uninstall.sh");
    fs::write(&path, &script)?;
    run_command_inherit("chmod", &["+x", &path])?;
    Ok(())
}

pub fn run_release_windows_msi() -> Result<()> {
    let skip_build = env_or("SKIP_BUILD", "") == "true";
    let custom_binary_path = env_or("BINARY_PATH", "");

    if !skip_build {
        check_rust_toolchain()?;
    }

    let target = env_or("TARGET", "x86_64-pc-windows-msvc");
    let info = get_cargo_info()?;
    let package_name = env_or("PACKAGE", &info.name);
    let binary_name = env_or("BINARY_NAME", &info.name);
    let version = info.version;

    if binary_name.is_empty() {
        return Err(Error::User("could not determine binary name".into()));
    }
    if version.is_empty() {
        return Err(Error::User("could not determine version".into()));
    }

    println!("\x1b[32mBuilding\x1b[0m {binary_name} v{version} MSI for {target}");

    let release_dir = format!("target/{target}/release");

    if skip_build {
        if custom_binary_path.is_empty() {
            return Err(Error::User(
                "binary-path is required when skip-build is true".into(),
            ));
        }
        if !Path::new(&custom_binary_path).exists() {
            return Err(Error::User(format!(
                "binary not found: {custom_binary_path}"
            )));
        }
        fs::create_dir_all(&release_dir)?;
        fs::copy(
            &custom_binary_path,
            format!("{release_dir}/{binary_name}.exe"),
        )?;
    } else {
        let _ = fs::remove_dir_all(&release_dir);
        fs::create_dir_all(&release_dir)?;
        ensure_lockfile()?;
        run_pre_build_hook()?;
        run_command_inherit("rustup", &["target", "add", &target])?;
        cargo_build(&target, &binary_name)?;
    }

    let binary_path = format!("{release_dir}/{binary_name}.exe");
    if !Path::new(&binary_path).exists() {
        return Err(Error::User(format!("binary not found: {binary_path}")));
    }

    copy_docs(Path::new(&release_dir))?;
    copy_includes(Path::new(&release_dir))?;

    // Copy to target/release for cargo-wix
    fs::create_dir_all("target/release")?;
    for entry in fs::read_dir(&release_dir)?.flatten() {
        let dest = Path::new("target/release").join(entry.file_name());
        let _ = fs::copy(entry.path(), dest);
    }

    // Check for WiX and cargo-wix
    if !command_exists("cargo-wix") {
        println!("\x1b[33mInstalling cargo-wix...\x1b[0m");
        run_command("cargo", &["install", "cargo-wix", "--version", "0.3.8"])?;
    }

    let msi_path = format!("target/wix/{binary_name}-{version}-{target}.msi");
    println!("\x1b[32mCreating MSI package...\x1b[0m");
    run_command_inherit(
        "cargo",
        &[
            "wix",
            "--no-build",
            "--nocapture",
            "--package",
            &package_name,
            "--output",
            &msi_path,
        ],
    )?;

    if !Path::new(&msi_path).exists() {
        return Err(Error::User(format!("MSI not created: {msi_path}")));
    }

    let checksums = generate_checksums(Path::new(&msi_path))?;
    let artifact_path = msi_path.replace('\\', "/");
    let artifact = Path::new(&artifact_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    output("version", &version);
    output("binary_name", &binary_name);
    output("target", &target);
    output("binary_path", &binary_path.replace('\\', "/"));

    println!();
    println!("\x1b[32mBuild artifacts:\x1b[0m");
    print_hr();
    println!("\x1b[32mCreated:\x1b[0m {artifact}");
    output_build_results(
        &binary_name,
        &version,
        &target,
        &artifact,
        &artifact_path,
        &checksums,
    );
    Ok(())
}
