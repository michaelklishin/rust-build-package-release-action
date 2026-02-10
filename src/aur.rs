use crate::cargo_info::get_cargo_info;
use crate::env_or;
use crate::error::{Error, Result};
use crate::output::{output, output_multiline, print_hr};
use crate::parse_comma_list;
use std::fs;

/// Formats a PKGBUILD array field like `depends=('foo' 'bar')` if non-empty.
fn pkgbuild_array(output: &mut String, key: &str, raw: &str) {
    let items = parse_comma_list(raw);
    if !items.is_empty() {
        let quoted: String = items
            .iter()
            .map(|d| format!("'{d}'"))
            .collect::<Vec<_>>()
            .join(" ");
        output.push_str(&format!("{key}=({quoted})\n"));
    }
}

pub struct PkgbuildConfig {
    pub pkgname: String,
    pub pkgver: String,
    pub pkgdesc: String,
    pub url: String,
    pub license: String,
    pub maintainer: String,
    pub source_url: String,
    pub source_sha256: String,
    pub depends: String,
    pub makedepends: String,
    pub optdepends: String,
    pub provides: String,
    pub conflicts: String,
    pub binary_name: String,
}

/// Generates PKGBUILD content.
pub fn generate_pkgbuild(config: &PkgbuildConfig) -> String {
    let mut pkgbuild = String::new();

    if !config.maintainer.is_empty() {
        pkgbuild.push_str(&format!("# Maintainer: {}\n", config.maintainer));
    }

    pkgbuild.push_str(&format!("pkgname={}\n", config.pkgname));
    pkgbuild.push_str(&format!("pkgver={}\n", config.pkgver));
    pkgbuild.push_str("pkgrel=1\n");
    pkgbuild.push_str(&format!("pkgdesc=\"{}\"\n", config.pkgdesc));
    pkgbuild.push_str("arch=('x86_64' 'aarch64')\n");

    if !config.url.is_empty() {
        pkgbuild.push_str(&format!("url=\"{}\"\n", config.url));
    }

    pkgbuild.push_str(&format!("license=('{}')\n", config.license));

    pkgbuild_array(&mut pkgbuild, "depends", &config.depends);
    pkgbuild_array(&mut pkgbuild, "makedepends", &config.makedepends);
    pkgbuild_array(&mut pkgbuild, "optdepends", &config.optdepends);
    pkgbuild_array(&mut pkgbuild, "provides", &config.provides);
    pkgbuild_array(&mut pkgbuild, "conflicts", &config.conflicts);

    if !config.source_url.is_empty() {
        pkgbuild.push_str(&format!("source=(\"{}\")\n", config.source_url));
        if !config.source_sha256.is_empty() {
            pkgbuild.push_str(&format!("sha256sums=('{}')\n", config.source_sha256));
        } else {
            pkgbuild.push_str("sha256sums=('SKIP')\n");
        }
    }

    pkgbuild.push_str("\nbuild() {\n");
    pkgbuild.push_str("  cd \"$srcdir/$pkgname-$pkgver\"\n");
    pkgbuild.push_str("  cargo build --release --locked\n");
    pkgbuild.push_str("}\n");

    pkgbuild.push_str("\npackage() {\n");
    pkgbuild.push_str("  cd \"$srcdir/$pkgname-$pkgver\"\n");
    pkgbuild.push_str(&format!(
        "  install -Dm755 \"target/release/{}\" \"$pkgdir/usr/bin/{}\"\n",
        config.binary_name, config.binary_name
    ));
    pkgbuild.push_str(
        "  install -Dm644 LICENSE* -t \"$pkgdir/usr/share/licenses/$pkgname/\" 2>/dev/null || true\n",
    );
    pkgbuild.push_str(
        "  install -Dm644 README.md \"$pkgdir/usr/share/doc/$pkgname/README.md\" 2>/dev/null || true\n",
    );
    pkgbuild.push_str("}\n");

    pkgbuild
}

pub struct SrcinfoConfig {
    pub pkgname: String,
    pub pkgver: String,
    pub pkgdesc: String,
    pub url: String,
    pub license: String,
    pub source_url: String,
    pub source_sha256: String,
    pub depends: String,
    pub makedepends: String,
    pub optdepends: String,
    pub provides: String,
    pub conflicts: String,
}

/// Generates .SRCINFO content.
pub fn generate_srcinfo(config: &SrcinfoConfig) -> String {
    let mut srcinfo = format!("pkgbase = {}\n", config.pkgname);
    srcinfo.push_str(&format!("\tpkgdesc = {}\n", config.pkgdesc));
    srcinfo.push_str(&format!("\tpkgver = {}\n", config.pkgver));
    srcinfo.push_str("\tpkgrel = 1\n");

    if !config.url.is_empty() {
        srcinfo.push_str(&format!("\turl = {}\n", config.url));
    }

    srcinfo.push_str("\tarch = x86_64\n");
    srcinfo.push_str("\tarch = aarch64\n");
    srcinfo.push_str(&format!("\tlicense = {}\n", config.license));

    for dep in parse_comma_list(&config.depends) {
        srcinfo.push_str(&format!("\tdepends = {dep}\n"));
    }
    for dep in parse_comma_list(&config.makedepends) {
        srcinfo.push_str(&format!("\tmakedepends = {dep}\n"));
    }
    for dep in parse_comma_list(&config.optdepends) {
        srcinfo.push_str(&format!("\toptdepends = {dep}\n"));
    }
    for prov in parse_comma_list(&config.provides) {
        srcinfo.push_str(&format!("\tprovides = {prov}\n"));
    }
    for conf in parse_comma_list(&config.conflicts) {
        srcinfo.push_str(&format!("\tconflicts = {conf}\n"));
    }

    if !config.source_url.is_empty() {
        srcinfo.push_str(&format!("\tsource = {}\n", config.source_url));
        if !config.source_sha256.is_empty() {
            srcinfo.push_str(&format!("\tsha256sums = {}\n", config.source_sha256));
        } else {
            srcinfo.push_str("\tsha256sums = SKIP\n");
        }
    }

    srcinfo.push_str(&format!("\npkgname = {}\n", config.pkgname));
    srcinfo
}

pub fn run_generate_aur() -> Result<()> {
    let info = get_cargo_info()?;
    let default_name = env_or("BINARY_NAME", &info.name);
    let pkg_name = env_or("AUR_PACKAGE_NAME", &default_name);
    let version = env_or("VERSION", &info.version);

    if pkg_name.is_empty() {
        return Err(Error::User(
            "could not determine package name - set 'aur-name' or 'binary-name'".into(),
        ));
    }
    if version.is_empty() {
        return Err(Error::User(
            "could not determine version - set the 'version' input".into(),
        ));
    }

    let description = env_or(
        "PKG_DESCRIPTION",
        &format!("{pkg_name} - built with rust-build-package-release-action"),
    );
    let license = env_or("PKG_LICENSE", "MIT");
    let binary_name = env_or("BINARY_NAME", &pkg_name);
    let source_url = env_or("AUR_SOURCE_URL", "");
    let source_sha256 = env_or("AUR_SOURCE_SHA256", "");

    if !source_url.is_empty() && source_sha256.is_empty() {
        println!(
            "\x1b[33mWarning:\x1b[0m source URL provided without SHA256, PKGBUILD will use SKIP"
        );
    }

    println!("\x1b[32mGenerating AUR PKGBUILD:\x1b[0m {pkg_name} v{version}");

    let pkgbuild_config = PkgbuildConfig {
        pkgname: pkg_name.clone(),
        pkgver: version.clone(),
        pkgdesc: description.clone(),
        url: env_or("PKG_HOMEPAGE", ""),
        license: license.clone(),
        maintainer: env_or("AUR_MAINTAINER", ""),
        source_url: source_url.clone(),
        source_sha256: source_sha256.clone(),
        depends: env_or("PKG_DEPENDS", ""),
        makedepends: env_or("AUR_MAKEDEPENDS", "cargo"),
        optdepends: env_or("AUR_OPTDEPENDS", ""),
        provides: env_or("PKG_PROVIDES", ""),
        conflicts: env_or("PKG_CONFLICTS", ""),
        binary_name,
    };

    let pkgbuild = generate_pkgbuild(&pkgbuild_config);

    let output_dir = env_or("AUR_OUTPUT_DIR", "target/aur");
    fs::create_dir_all(&output_dir)?;

    let pkgbuild_path = format!("{output_dir}/PKGBUILD");
    fs::write(&pkgbuild_path, &pkgbuild)?;

    let srcinfo_config = SrcinfoConfig {
        pkgname: pkg_name,
        pkgver: version,
        pkgdesc: description,
        url: env_or("PKG_HOMEPAGE", ""),
        license,
        source_url,
        source_sha256,
        depends: env_or("PKG_DEPENDS", ""),
        makedepends: env_or("AUR_MAKEDEPENDS", "cargo"),
        optdepends: env_or("AUR_OPTDEPENDS", ""),
        provides: env_or("PKG_PROVIDES", ""),
        conflicts: env_or("PKG_CONFLICTS", ""),
    };

    let srcinfo = generate_srcinfo(&srcinfo_config);
    let srcinfo_path = format!("{output_dir}/.SRCINFO");
    fs::write(&srcinfo_path, &srcinfo)?;

    println!();
    println!("\x1b[32mPKGBUILD:\x1b[0m");
    print_hr();
    print!("{pkgbuild}");
    print_hr();

    output("pkgbuild_path", &pkgbuild_path);
    output("srcinfo_path", &srcinfo_path);
    output_multiline("pkgbuild", &pkgbuild);
    Ok(())
}
