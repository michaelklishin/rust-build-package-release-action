use rust_release_action::aur::{
    PkgbuildConfig, SrcinfoConfig, generate_pkgbuild, generate_srcinfo,
};
use rust_release_action::parse_comma_list;

#[test]
fn parse_list_empty() {
    assert!(parse_comma_list("").is_empty());
}

#[test]
fn parse_list_single() {
    assert_eq!(parse_comma_list("foo"), vec!["foo"]);
}

#[test]
fn parse_list_multiple() {
    assert_eq!(parse_comma_list("foo,bar,baz"), vec!["foo", "bar", "baz"]);
}

#[test]
fn parse_list_with_spaces() {
    assert_eq!(
        parse_comma_list("foo , bar , baz"),
        vec!["foo", "bar", "baz"]
    );
}

#[test]
fn parse_list_trailing_comma() {
    assert_eq!(parse_comma_list("foo,bar,"), vec!["foo", "bar"]);
}

#[test]
fn pkgbuild_basic() {
    let config = PkgbuildConfig {
        pkgname: "mytool".into(),
        pkgver: "1.0.0".into(),
        pkgdesc: "A tool".into(),
        url: "https://example.com".into(),
        license: "MIT".into(),
        maintainer: "Test <test@example.com>".into(),
        source_url: "https://example.com/src.tar.gz".into(),
        source_sha256: "abc123".into(),
        depends: String::new(),
        makedepends: "cargo".into(),
        optdepends: String::new(),
        provides: String::new(),
        conflicts: String::new(),
        binary_name: "mytool".into(),
    };

    let pkgbuild = generate_pkgbuild(&config);

    assert!(pkgbuild.contains("# Maintainer: Test <test@example.com>"));
    assert!(pkgbuild.contains("pkgname=mytool"));
    assert!(pkgbuild.contains("pkgver=1.0.0"));
    assert!(pkgbuild.contains("pkgrel=1"));
    assert!(pkgbuild.contains("pkgdesc=\"A tool\""));
    assert!(pkgbuild.contains("arch=('x86_64' 'aarch64')"));
    assert!(pkgbuild.contains("url=\"https://example.com\""));
    assert!(pkgbuild.contains("license=('MIT')"));
    assert!(pkgbuild.contains("makedepends=('cargo')"));
    assert!(pkgbuild.contains("source=(\"https://example.com/src.tar.gz\")"));
    assert!(pkgbuild.contains("sha256sums=('abc123')"));
    assert!(pkgbuild.contains("cargo build --release --locked"));
    assert!(
        pkgbuild.contains("install -Dm755 \"target/release/mytool\" \"$pkgdir/usr/bin/mytool\"")
    );
}

#[test]
fn pkgbuild_no_maintainer() {
    let config = PkgbuildConfig {
        pkgname: "tool".into(),
        pkgver: "1.0.0".into(),
        pkgdesc: "desc".into(),
        url: String::new(),
        license: "MIT".into(),
        maintainer: String::new(),
        source_url: String::new(),
        source_sha256: String::new(),
        depends: String::new(),
        makedepends: String::new(),
        optdepends: String::new(),
        provides: String::new(),
        conflicts: String::new(),
        binary_name: "tool".into(),
    };

    let pkgbuild = generate_pkgbuild(&config);

    assert!(!pkgbuild.contains("# Maintainer:"));
    assert!(!pkgbuild.contains("url="));
    assert!(!pkgbuild.contains("source="));
}

#[test]
fn pkgbuild_with_dependencies() {
    let config = PkgbuildConfig {
        pkgname: "tool".into(),
        pkgver: "1.0.0".into(),
        pkgdesc: "desc".into(),
        url: String::new(),
        license: "MIT".into(),
        maintainer: String::new(),
        source_url: String::new(),
        source_sha256: String::new(),
        depends: "openssl,zlib".into(),
        makedepends: "cargo,cmake".into(),
        optdepends: "bash-completion".into(),
        provides: "tool-bin".into(),
        conflicts: "tool-git".into(),
        binary_name: "tool".into(),
    };

    let pkgbuild = generate_pkgbuild(&config);

    assert!(pkgbuild.contains("depends=('openssl' 'zlib')"));
    assert!(pkgbuild.contains("makedepends=('cargo' 'cmake')"));
    assert!(pkgbuild.contains("optdepends=('bash-completion')"));
    assert!(pkgbuild.contains("provides=('tool-bin')"));
    assert!(pkgbuild.contains("conflicts=('tool-git')"));
}

#[test]
fn pkgbuild_source_without_sha256() {
    let config = PkgbuildConfig {
        pkgname: "tool".into(),
        pkgver: "1.0.0".into(),
        pkgdesc: "desc".into(),
        url: String::new(),
        license: "MIT".into(),
        maintainer: String::new(),
        source_url: "https://example.com/src.tar.gz".into(),
        source_sha256: String::new(),
        depends: String::new(),
        makedepends: String::new(),
        optdepends: String::new(),
        provides: String::new(),
        conflicts: String::new(),
        binary_name: "tool".into(),
    };

    let pkgbuild = generate_pkgbuild(&config);

    assert!(pkgbuild.contains("sha256sums=('SKIP')"));
}

#[test]
fn srcinfo_basic() {
    let config = SrcinfoConfig {
        pkgname: "mytool".into(),
        pkgver: "1.0.0".into(),
        pkgdesc: "A tool".into(),
        url: "https://example.com".into(),
        license: "MIT".into(),
        source_url: "https://example.com/src.tar.gz".into(),
        source_sha256: "abc123".into(),
        depends: String::new(),
        makedepends: "cargo".into(),
        optdepends: String::new(),
        provides: String::new(),
        conflicts: String::new(),
    };

    let srcinfo = generate_srcinfo(&config);

    assert!(srcinfo.starts_with("pkgbase = mytool\n"));
    assert!(srcinfo.contains("\tpkgdesc = A tool\n"));
    assert!(srcinfo.contains("\tpkgver = 1.0.0\n"));
    assert!(srcinfo.contains("\tpkgrel = 1\n"));
    assert!(srcinfo.contains("\turl = https://example.com\n"));
    assert!(srcinfo.contains("\tarch = x86_64\n"));
    assert!(srcinfo.contains("\tarch = aarch64\n"));
    assert!(srcinfo.contains("\tlicense = MIT\n"));
    assert!(srcinfo.contains("\tmakedepends = cargo\n"));
    assert!(srcinfo.contains("\tsource = https://example.com/src.tar.gz\n"));
    assert!(srcinfo.contains("\tsha256sums = abc123\n"));
    assert!(srcinfo.contains("\npkgname = mytool\n"));
}

#[test]
fn srcinfo_with_dependencies() {
    let config = SrcinfoConfig {
        pkgname: "tool".into(),
        pkgver: "1.0.0".into(),
        pkgdesc: "desc".into(),
        url: String::new(),
        license: "MIT".into(),
        source_url: String::new(),
        source_sha256: String::new(),
        depends: "openssl,zlib".into(),
        makedepends: String::new(),
        optdepends: "bash".into(),
        provides: "tool-bin".into(),
        conflicts: "tool-git".into(),
    };

    let srcinfo = generate_srcinfo(&config);

    assert!(srcinfo.contains("\tdepends = openssl\n"));
    assert!(srcinfo.contains("\tdepends = zlib\n"));
    assert!(srcinfo.contains("\toptdepends = bash\n"));
    assert!(srcinfo.contains("\tprovides = tool-bin\n"));
    assert!(srcinfo.contains("\tconflicts = tool-git\n"));
    assert!(!srcinfo.contains("\turl ="));
}
