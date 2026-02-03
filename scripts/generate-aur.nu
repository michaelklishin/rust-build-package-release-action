#!/usr/bin/env nu

use common.nu [get-cargo-info, output, output-multiline, hr-line, error]

def main [] {
    let info = get-cargo-info
    let default_name = $env.BINARY_NAME? | default $info.name
    let pkg_name = $env.AUR_PACKAGE_NAME? | default $default_name
    let version = $env.VERSION? | default $info.version

    if $pkg_name == "" {
        error "could not determine package name - set 'aur-name' or 'binary-name'"
    }
    if $version == "" {
        error "could not determine version - set the 'version' input"
    }

    let description = $env.PKG_DESCRIPTION? | default $"($pkg_name) - built with rust-release-action"
    let homepage = $env.PKG_HOMEPAGE? | default ""
    let license = $env.PKG_LICENSE? | default "MIT"
    let maintainer = $env.AUR_MAINTAINER? | default ""
    let source_url = $env.AUR_SOURCE_URL? | default ""
    let source_sha256 = $env.AUR_SOURCE_SHA256? | default ""
    let depends = $env.PKG_DEPENDS? | default ""
    let makedepends = $env.AUR_MAKEDEPENDS? | default "cargo"
    let optdepends = $env.AUR_OPTDEPENDS? | default ""
    let provides = $env.PKG_PROVIDES? | default ""
    let conflicts = $env.PKG_CONFLICTS? | default ""
    let binary_name = $env.BINARY_NAME? | default $pkg_name

    if $source_url != "" and $source_sha256 == "" {
        print $"(ansi yellow)Warning:(ansi reset) source URL provided without SHA256, PKGBUILD will use SKIP"
    }

    print $"(ansi green)Generating AUR PKGBUILD:(ansi reset) ($pkg_name) v($version)"

    let pkgbuild = generate-pkgbuild {
        pkgname: $pkg_name
        pkgver: $version
        pkgdesc: $description
        url: $homepage
        license: $license
        maintainer: $maintainer
        source_url: $source_url
        source_sha256: $source_sha256
        depends: $depends
        makedepends: $makedepends
        optdepends: $optdepends
        provides: $provides
        conflicts: $conflicts
        binary_name: $binary_name
    }

    let output_dir = $env.AUR_OUTPUT_DIR? | default "target/aur"
    mkdir $output_dir

    let pkgbuild_path = $"($output_dir)/PKGBUILD"
    $pkgbuild | save -f $pkgbuild_path

    # Generate .SRCINFO
    let srcinfo = generate-srcinfo {
        pkgname: $pkg_name
        pkgver: $version
        pkgdesc: $description
        url: $homepage
        license: $license
        source_url: $source_url
        source_sha256: $source_sha256
        depends: $depends
        makedepends: $makedepends
        optdepends: $optdepends
        provides: $provides
        conflicts: $conflicts
    }

    let srcinfo_path = $"($output_dir)/.SRCINFO"
    $srcinfo | save -f $srcinfo_path

    print $"(char nl)(ansi green)PKGBUILD:(ansi reset)"
    hr-line
    print $pkgbuild
    hr-line

    output "pkgbuild_path" $pkgbuild_path
    output "srcinfo_path" $srcinfo_path
    output-multiline "pkgbuild" $pkgbuild
}

# Generates PKGBUILD content
export def generate-pkgbuild [config: record]: nothing -> string {
    let pkgname = $config.pkgname
    let pkgver = $config.pkgver
    let pkgdesc = $config.pkgdesc
    let url = $config.url
    let license = $config.license
    let maintainer = $config.maintainer
    let source_url = $config.source_url
    let source_sha256 = $config.source_sha256
    let depends_raw = $config.depends
    let makedepends_raw = $config.makedepends
    let optdepends_raw = $config.optdepends
    let provides_raw = $config.provides
    let conflicts_raw = $config.conflicts
    let binary_name = $config.binary_name

    mut pkgbuild = ""

    if $maintainer != "" {
        $pkgbuild = $pkgbuild + "# Maintainer: " + $maintainer + "\n"
    }

    $pkgbuild = $pkgbuild + "pkgname=" + $pkgname + "\n"
    $pkgbuild = $pkgbuild + "pkgver=" + $pkgver + "\n"
    $pkgbuild = $pkgbuild + "pkgrel=1\n"
    $pkgbuild = $pkgbuild + "pkgdesc=\"" + $pkgdesc + "\"\n"
    $pkgbuild = $pkgbuild + "arch=('x86_64' 'aarch64')\n"

    if $url != "" {
        $pkgbuild = $pkgbuild + "url=\"" + $url + "\"\n"
    }

    $pkgbuild = $pkgbuild + "license=('" + $license + "')\n"

    # Dependencies
    let depends = parse-list $depends_raw
    if ($depends | is-not-empty) {
        let deps_str = $depends | each {|d| "'" + $d + "'" } | str join " "
        $pkgbuild = $pkgbuild + "depends=(" + $deps_str + ")\n"
    }

    let makedepends = parse-list $makedepends_raw
    if ($makedepends | is-not-empty) {
        let deps_str = $makedepends | each {|d| "'" + $d + "'" } | str join " "
        $pkgbuild = $pkgbuild + "makedepends=(" + $deps_str + ")\n"
    }

    let optdepends = parse-list $optdepends_raw
    if ($optdepends | is-not-empty) {
        let deps_str = $optdepends | each {|d| "'" + $d + "'" } | str join " "
        $pkgbuild = $pkgbuild + "optdepends=(" + $deps_str + ")\n"
    }

    let provides = parse-list $provides_raw
    if ($provides | is-not-empty) {
        let prov_str = $provides | each {|d| "'" + $d + "'" } | str join " "
        $pkgbuild = $pkgbuild + "provides=(" + $prov_str + ")\n"
    }

    let conflicts = parse-list $conflicts_raw
    if ($conflicts | is-not-empty) {
        let conf_str = $conflicts | each {|d| "'" + $d + "'" } | str join " "
        $pkgbuild = $pkgbuild + "conflicts=(" + $conf_str + ")\n"
    }

    # Source
    if $source_url != "" {
        $pkgbuild = $pkgbuild + "source=(\"" + $source_url + "\")\n"
        if $source_sha256 != "" {
            $pkgbuild = $pkgbuild + "sha256sums=('" + $source_sha256 + "')\n"
        } else {
            $pkgbuild = $pkgbuild + "sha256sums=('SKIP')\n"
        }
    }

    # Build function
    $pkgbuild = $pkgbuild + "\nbuild() {\n"
    $pkgbuild = $pkgbuild + "  cd \"$srcdir/$pkgname-$pkgver\"\n"
    $pkgbuild = $pkgbuild + "  cargo build --release --locked\n"
    $pkgbuild = $pkgbuild + "}\n"

    # Package function
    $pkgbuild = $pkgbuild + "\npackage() {\n"
    $pkgbuild = $pkgbuild + "  cd \"$srcdir/$pkgname-$pkgver\"\n"
    $pkgbuild = $pkgbuild + "  install -Dm755 \"target/release/" + $binary_name + "\" \"$pkgdir/usr/bin/" + $binary_name + "\"\n"
    $pkgbuild = $pkgbuild + "  install -Dm644 LICENSE* -t \"$pkgdir/usr/share/licenses/$pkgname/\" 2>/dev/null || true\n"
    $pkgbuild = $pkgbuild + "  install -Dm644 README.md \"$pkgdir/usr/share/doc/$pkgname/README.md\" 2>/dev/null || true\n"
    $pkgbuild = $pkgbuild + "}\n"

    $pkgbuild
}

# Generates .SRCINFO content
def generate-srcinfo [config: record]: nothing -> string {
    mut srcinfo = "pkgbase = " + $config.pkgname + "\n"
    $srcinfo = $srcinfo + "\tpkgdesc = " + $config.pkgdesc + "\n"
    $srcinfo = $srcinfo + "\tpkgver = " + $config.pkgver + "\n"
    $srcinfo = $srcinfo + "\tpkgrel = 1\n"

    if $config.url != "" {
        $srcinfo = $srcinfo + "\turl = " + $config.url + "\n"
    }

    $srcinfo = $srcinfo + "\tarch = x86_64\n"
    $srcinfo = $srcinfo + "\tarch = aarch64\n"
    $srcinfo = $srcinfo + "\tlicense = " + $config.license + "\n"

    let depends = parse-list $config.depends
    for dep in $depends {
        $srcinfo = $srcinfo + "\tdepends = " + $dep + "\n"
    }

    let makedepends = parse-list $config.makedepends
    for dep in $makedepends {
        $srcinfo = $srcinfo + "\tmakedepends = " + $dep + "\n"
    }

    let optdepends = parse-list $config.optdepends
    for dep in $optdepends {
        $srcinfo = $srcinfo + "\toptdepends = " + $dep + "\n"
    }

    let provides = parse-list $config.provides
    for prov in $provides {
        $srcinfo = $srcinfo + "\tprovides = " + $prov + "\n"
    }

    let conflicts = parse-list $config.conflicts
    for conf in $conflicts {
        $srcinfo = $srcinfo + "\tconflicts = " + $conf + "\n"
    }

    if $config.source_url != "" {
        $srcinfo = $srcinfo + "\tsource = " + $config.source_url + "\n"
        if $config.source_sha256 != "" {
            $srcinfo = $srcinfo + "\tsha256sums = " + $config.source_sha256 + "\n"
        } else {
            $srcinfo = $srcinfo + "\tsha256sums = SKIP\n"
        }
    }

    $srcinfo = $srcinfo + "\npkgname = " + $config.pkgname + "\n"
    $srcinfo
}

# Parses a comma-separated list
def parse-list [raw: string]: nothing -> list<string> {
    if $raw == "" {
        return []
    }
    $raw | split row "," | each {|i| $i | str trim } | where {|i| $i != ""}
}
