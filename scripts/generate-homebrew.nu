#!/usr/bin/env nu

use common.nu [get-cargo-info, output, output-multiline, hr-line, error]

def main [] {
    let info = get-cargo-info
    let binary_name = $env.BINARY_NAME? | default $info.name
    let version = $env.VERSION? | default $info.version

    if $binary_name == "" {
        error "could not determine binary name"
    }
    if $version == "" {
        error "could not determine version"
    }

    let formula_class = $env.HOMEBREW_FORMULA_CLASS? | default (to-class-name $binary_name)
    let description = $env.PKG_DESCRIPTION? | default $"($binary_name) - built with rust-build-package-release-action"
    let homepage = $env.PKG_HOMEPAGE? | default ""
    let license = $env.PKG_LICENSE? | default ""
    let copyright = $env.HOMEBREW_COPYRIGHT? | default ""

    let macos_arm64_url = $env.HOMEBREW_MACOS_ARM64_URL? | default ""
    let macos_arm64_sha256 = $env.HOMEBREW_MACOS_ARM64_SHA256? | default ""
    let macos_x64_url = $env.HOMEBREW_MACOS_X64_URL? | default ""
    let macos_x64_sha256 = $env.HOMEBREW_MACOS_X64_SHA256? | default ""
    let linux_arm64_url = $env.HOMEBREW_LINUX_ARM64_URL? | default ""
    let linux_arm64_sha256 = $env.HOMEBREW_LINUX_ARM64_SHA256? | default ""
    let linux_x64_url = $env.HOMEBREW_LINUX_X64_URL? | default ""
    let linux_x64_sha256 = $env.HOMEBREW_LINUX_X64_SHA256? | default ""

    print $"(ansi green)Generating Homebrew formula:(ansi reset) ($formula_class)"

    let formula = generate-formula {
        class: $formula_class
        binary_name: $binary_name
        version: $version
        description: $description
        homepage: $homepage
        license: $license
        copyright: $copyright
        macos_arm64_url: $macos_arm64_url
        macos_arm64_sha256: $macos_arm64_sha256
        macos_x64_url: $macos_x64_url
        macos_x64_sha256: $macos_x64_sha256
        linux_arm64_url: $linux_arm64_url
        linux_arm64_sha256: $linux_arm64_sha256
        linux_x64_url: $linux_x64_url
        linux_x64_sha256: $linux_x64_sha256
    }

    let output_dir = $env.HOMEBREW_OUTPUT_DIR? | default "target/homebrew"
    mkdir $output_dir

    let formula_file = $"($output_dir)/($binary_name).rb"
    $formula | save -f $formula_file

    print $"(char nl)(ansi green)Formula file:(ansi reset)"
    hr-line
    print $formula
    hr-line

    output "formula_file" $formula_file
    output "formula_class" $formula_class
    output-multiline "formula" $formula
}

# Converts a binary name to a Ruby class name (e.g., my-tool -> MyTool)
export def to-class-name [name: string]: nothing -> string {
    $name
    | split row "-"
    | each {|part|
        let chars = $part | split chars
        let first = $chars | first | str upcase
        let rest = $chars | skip 1 | str join ""
        $first + $rest
    }
    | str join ""
}

# Formats a license string: "X OR Y" -> any_of: ["X", "Y"], single -> "X"
def format-license [license: string]: nothing -> string {
    if ($license | str contains " OR ") {
        let parts = $license | split row " OR " | each { str trim } | each { $"\"($in)\"" }
        $"any_of: [($parts | str join ', ')]"
    } else {
        $"\"($license)\""
    }
}

# Generates a Homebrew formula
export def generate-formula [config: record]: nothing -> string {
    let has_macos_arm64 = ($config.macos_arm64_url != "") and ($config.macos_arm64_sha256 != "")
    let has_macos_x64 = ($config.macos_x64_url != "") and ($config.macos_x64_sha256 != "")
    let has_linux_arm64 = ($config.linux_arm64_url != "") and ($config.linux_arm64_sha256 != "")
    let has_linux_x64 = ($config.linux_x64_url != "") and ($config.linux_x64_sha256 != "")

    mut formula = ""

    if ($config.copyright? | default "") != "" {
        $formula = $formula + "# MIT License\n"
        $formula = $formula + "#\n"
        $formula = $formula + $"# Copyright \(c\) ($config.copyright)\n"
        $formula = $formula + "#\n"
        $formula = $formula + "# Permission is hereby granted, free of charge, to any person obtaining a copy\n"
        $formula = $formula + "# of this software and associated documentation files (the \"Software\"), to deal\n"
        $formula = $formula + "# in the Software without restriction, including without limitation the rights\n"
        $formula = $formula + "# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell\n"
        $formula = $formula + "# copies of the Software, and to permit persons to whom the Software is\n"
        $formula = $formula + "# furnished to do so, subject to the following conditions:\n"
        $formula = $formula + "#\n"
        $formula = $formula + "# The above copyright notice and this permission notice shall be included in all\n"
        $formula = $formula + "# copies or substantial portions of the Software.\n"
        $formula = $formula + "#\n"
        $formula = $formula + "# THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR\n"
        $formula = $formula + "# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,\n"
        $formula = $formula + "# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE\n"
        $formula = $formula + "# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER\n"
        $formula = $formula + "# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,\n"
        $formula = $formula + "# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE\n"
        $formula = $formula + "# SOFTWARE.\n"
        $formula = $formula + "\n"
    }

    $formula = $formula + $"class ($config.class) < Formula\n"
    $formula = $formula + $"  desc \"($config.description)\"\n"

    if $config.homepage != "" {
        $formula = $formula + $"  homepage \"($config.homepage)\"\n"
    }

    $formula = $formula + $"  version \"($config.version)\"\n"

    if $config.license != "" {
        $formula = $formula + $"  license (format-license $config.license)\n"
    }

    $formula = $formula + "\n"

    if $has_macos_arm64 or $has_macos_x64 {
        $formula = $formula + "  on_macos do\n"
        if $has_macos_arm64 {
            $formula = $formula + "    on_arm do\n"
            $formula = $formula + $"      url \"($config.macos_arm64_url)\"\n"
            $formula = $formula + $"      sha256 \"($config.macos_arm64_sha256)\"\n"
            $formula = $formula + "    end\n"
        }
        if $has_macos_x64 {
            $formula = $formula + "    on_intel do\n"
            $formula = $formula + $"      url \"($config.macos_x64_url)\"\n"
            $formula = $formula + $"      sha256 \"($config.macos_x64_sha256)\"\n"
            $formula = $formula + "    end\n"
        }
        $formula = $formula + "  end\n\n"
    }

    if $has_linux_arm64 or $has_linux_x64 {
        $formula = $formula + "  on_linux do\n"
        if $has_linux_arm64 {
            $formula = $formula + "    on_arm do\n"
            $formula = $formula + $"      url \"($config.linux_arm64_url)\"\n"
            $formula = $formula + $"      sha256 \"($config.linux_arm64_sha256)\"\n"
            $formula = $formula + "    end\n"
        }
        if $has_linux_x64 {
            $formula = $formula + "    on_intel do\n"
            $formula = $formula + $"      url \"($config.linux_x64_url)\"\n"
            $formula = $formula + $"      sha256 \"($config.linux_x64_sha256)\"\n"
            $formula = $formula + "    end\n"
        }
        $formula = $formula + "  end\n\n"
    }

    $formula = $formula + "  def install\n"
    $formula = $formula + $"    bin.install \"($config.binary_name)\"\n"
    $formula = $formula + "  end\n\n"

    $formula = $formula + "  test do\n"
    $formula = $formula + $"    system \"#\{bin}/($config.binary_name)\", \"--version\"\n"
    $formula = $formula + "  end\n"
    $formula = $formula + "end\n"

    $formula
}
