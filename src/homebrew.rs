use crate::cargo_info::get_cargo_info;
use crate::env_or;
use crate::error::{Error, Result};
use crate::output::{output, output_multiline, print_hr};
use std::fs;

/// Converts a binary name to a Ruby class name (e.g., my-tool -> MyTool).
pub fn to_class_name(name: &str) -> String {
    name.split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(c) => {
                    let mut s = c.to_uppercase().to_string();
                    s.push_str(&chars.collect::<String>());
                    s
                }
                None => String::new(),
            }
        })
        .collect()
}

/// Configuration for generating a Homebrew formula.
pub struct FormulaConfig {
    pub class: String,
    pub binary_name: String,
    pub version: String,
    pub description: String,
    pub homepage: String,
    pub license: String,
    pub macos_arm64_url: String,
    pub macos_arm64_sha256: String,
    pub macos_x64_url: String,
    pub macos_x64_sha256: String,
    pub linux_arm64_url: String,
    pub linux_arm64_sha256: String,
    pub linux_x64_url: String,
    pub linux_x64_sha256: String,
}

/// Generates a Homebrew formula string.
pub fn generate_formula(config: &FormulaConfig) -> String {
    let has_macos_arm64 =
        !config.macos_arm64_url.is_empty() && !config.macos_arm64_sha256.is_empty();
    let has_macos_x64 = !config.macos_x64_url.is_empty() && !config.macos_x64_sha256.is_empty();
    let has_linux_arm64 =
        !config.linux_arm64_url.is_empty() && !config.linux_arm64_sha256.is_empty();
    let has_linux_x64 = !config.linux_x64_url.is_empty() && !config.linux_x64_sha256.is_empty();

    let mut formula = format!("class {} < Formula\n", config.class);
    formula.push_str(&format!("  desc \"{}\"\n", config.description));

    if !config.homepage.is_empty() {
        formula.push_str(&format!("  homepage \"{}\"\n", config.homepage));
    }

    formula.push_str(&format!("  version \"{}\"\n", config.version));

    if !config.license.is_empty() {
        formula.push_str(&format!("  license \"{}\"\n", config.license));
    }

    formula.push('\n');

    // macOS section
    if has_macos_arm64 || has_macos_x64 {
        if has_macos_arm64 && has_macos_x64 {
            formula.push_str("  on_macos do\n");
            formula.push_str("    if Hardware::CPU.arm?\n");
            formula.push_str(&format!("      url \"{}\"\n", config.macos_arm64_url));
            formula.push_str(&format!("      sha256 \"{}\"\n", config.macos_arm64_sha256));
            formula.push_str("    else\n");
            formula.push_str(&format!("      url \"{}\"\n", config.macos_x64_url));
            formula.push_str(&format!("      sha256 \"{}\"\n", config.macos_x64_sha256));
            formula.push_str("    end\n");
            formula.push_str("  end\n\n");
        } else if has_macos_arm64 {
            formula.push_str("  on_macos do\n");
            formula.push_str("    on_arm do\n");
            formula.push_str(&format!("      url \"{}\"\n", config.macos_arm64_url));
            formula.push_str(&format!("      sha256 \"{}\"\n", config.macos_arm64_sha256));
            formula.push_str("    end\n");
            formula.push_str("  end\n\n");
        } else {
            formula.push_str("  on_macos do\n");
            formula.push_str("    on_intel do\n");
            formula.push_str(&format!("      url \"{}\"\n", config.macos_x64_url));
            formula.push_str(&format!("      sha256 \"{}\"\n", config.macos_x64_sha256));
            formula.push_str("    end\n");
            formula.push_str("  end\n\n");
        }
    }

    // Linux section
    if has_linux_arm64 || has_linux_x64 {
        if has_linux_arm64 && has_linux_x64 {
            formula.push_str("  on_linux do\n");
            formula.push_str("    if Hardware::CPU.arm?\n");
            formula.push_str(&format!("      url \"{}\"\n", config.linux_arm64_url));
            formula.push_str(&format!("      sha256 \"{}\"\n", config.linux_arm64_sha256));
            formula.push_str("    else\n");
            formula.push_str(&format!("      url \"{}\"\n", config.linux_x64_url));
            formula.push_str(&format!("      sha256 \"{}\"\n", config.linux_x64_sha256));
            formula.push_str("    end\n");
            formula.push_str("  end\n\n");
        } else if has_linux_arm64 {
            formula.push_str("  on_linux do\n");
            formula.push_str("    on_arm do\n");
            formula.push_str(&format!("      url \"{}\"\n", config.linux_arm64_url));
            formula.push_str(&format!("      sha256 \"{}\"\n", config.linux_arm64_sha256));
            formula.push_str("    end\n");
            formula.push_str("  end\n\n");
        } else {
            formula.push_str("  on_linux do\n");
            formula.push_str("    on_intel do\n");
            formula.push_str(&format!("      url \"{}\"\n", config.linux_x64_url));
            formula.push_str(&format!("      sha256 \"{}\"\n", config.linux_x64_sha256));
            formula.push_str("    end\n");
            formula.push_str("  end\n\n");
        }
    }

    formula.push_str("  def install\n");
    formula.push_str(&format!("    bin.install \"{}\"\n", config.binary_name));
    formula.push_str("  end\n\n");

    formula.push_str("  test do\n");
    formula.push_str(&format!(
        "    system \"#{{bin}}/{}\", \"--version\"\n",
        config.binary_name
    ));
    formula.push_str("  end\n");
    formula.push_str("end\n");

    formula
}

pub fn run_generate_homebrew() -> Result<()> {
    let info = get_cargo_info()?;
    let binary_name = env_or("BINARY_NAME", &info.name);
    let version = env_or("VERSION", &info.version);

    if binary_name.is_empty() {
        return Err(Error::User("could not determine binary name".into()));
    }
    if version.is_empty() {
        return Err(Error::User("could not determine version".into()));
    }

    let formula_class = env_or("HOMEBREW_FORMULA_CLASS", &to_class_name(&binary_name));
    let description = env_or(
        "PKG_DESCRIPTION",
        &format!("{binary_name} - built with rust-build-package-release-action"),
    );

    let config = FormulaConfig {
        class: formula_class.clone(),
        binary_name: binary_name.clone(),
        version,
        description,
        homepage: env_or("PKG_HOMEPAGE", ""),
        license: env_or("PKG_LICENSE", ""),
        macos_arm64_url: env_or("HOMEBREW_MACOS_ARM64_URL", ""),
        macos_arm64_sha256: env_or("HOMEBREW_MACOS_ARM64_SHA256", ""),
        macos_x64_url: env_or("HOMEBREW_MACOS_X64_URL", ""),
        macos_x64_sha256: env_or("HOMEBREW_MACOS_X64_SHA256", ""),
        linux_arm64_url: env_or("HOMEBREW_LINUX_ARM64_URL", ""),
        linux_arm64_sha256: env_or("HOMEBREW_LINUX_ARM64_SHA256", ""),
        linux_x64_url: env_or("HOMEBREW_LINUX_X64_URL", ""),
        linux_x64_sha256: env_or("HOMEBREW_LINUX_X64_SHA256", ""),
    };

    println!("\x1b[32mGenerating Homebrew formula:\x1b[0m {formula_class}");

    let formula = generate_formula(&config);

    let output_dir = env_or("HOMEBREW_OUTPUT_DIR", "target/homebrew");
    fs::create_dir_all(&output_dir)?;

    let formula_file = format!("{output_dir}/{binary_name}.rb");
    fs::write(&formula_file, &formula)?;

    println!();
    println!("\x1b[32mFormula file:\x1b[0m");
    print_hr();
    print!("{formula}");
    print_hr();

    output("formula_file", &formula_file);
    output("formula_class", &formula_class);
    output_multiline("formula", &formula);
    Ok(())
}
