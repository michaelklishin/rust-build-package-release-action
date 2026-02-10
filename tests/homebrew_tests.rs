use rust_release_action::homebrew::{FormulaConfig, generate_formula, to_class_name};

#[test]
fn class_name_simple() {
    assert_eq!(to_class_name("mytool"), "Mytool");
}

#[test]
fn class_name_hyphenated() {
    assert_eq!(to_class_name("my-tool"), "MyTool");
}

#[test]
fn class_name_multiple_hyphens() {
    assert_eq!(to_class_name("my-great-tool"), "MyGreatTool");
}

#[test]
fn class_name_single_char() {
    assert_eq!(to_class_name("x"), "X");
}

#[test]
fn class_name_empty_string() {
    assert_eq!(to_class_name(""), "");
}

#[test]
fn formula_all_platforms() {
    let config = FormulaConfig {
        class: "MyTool".into(),
        binary_name: "my-tool".into(),
        version: "1.0.0".into(),
        description: "A great tool".into(),
        homepage: "https://example.com".into(),
        license: "MIT".into(),
        macos_arm64_url: "https://example.com/macos-arm64.tar.gz".into(),
        macos_arm64_sha256: "abc123".into(),
        macos_x64_url: "https://example.com/macos-x64.tar.gz".into(),
        macos_x64_sha256: "def456".into(),
        linux_arm64_url: "https://example.com/linux-arm64.tar.gz".into(),
        linux_arm64_sha256: "ghi789".into(),
        linux_x64_url: "https://example.com/linux-x64.tar.gz".into(),
        linux_x64_sha256: "jkl012".into(),
    };

    let formula = generate_formula(&config);

    assert!(formula.starts_with("class MyTool < Formula\n"));
    assert!(formula.contains("desc \"A great tool\""));
    assert!(formula.contains("homepage \"https://example.com\""));
    assert!(formula.contains("version \"1.0.0\""));
    assert!(formula.contains("license \"MIT\""));
    assert!(formula.contains("on_macos do"));
    assert!(formula.contains("on_linux do"));
    assert!(formula.contains("Hardware::CPU.arm?"));
    assert!(formula.contains("bin.install \"my-tool\""));
    assert!(formula.contains("system \"#{bin}/my-tool\", \"--version\""));
    assert!(formula.ends_with("end\n"));
}

#[test]
fn formula_macos_arm64_only() {
    let config = FormulaConfig {
        class: "Tool".into(),
        binary_name: "tool".into(),
        version: "0.1.0".into(),
        description: "desc".into(),
        homepage: String::new(),
        license: String::new(),
        macos_arm64_url: "https://example.com/arm64.tar.gz".into(),
        macos_arm64_sha256: "abc123".into(),
        macos_x64_url: String::new(),
        macos_x64_sha256: String::new(),
        linux_arm64_url: String::new(),
        linux_arm64_sha256: String::new(),
        linux_x64_url: String::new(),
        linux_x64_sha256: String::new(),
    };

    let formula = generate_formula(&config);

    assert!(formula.contains("on_macos do"));
    assert!(formula.contains("on_arm do"));
    assert!(!formula.contains("on_linux do"));
    assert!(!formula.contains("homepage"));
    assert!(!formula.contains("license"));
}

#[test]
fn formula_linux_x64_only() {
    let config = FormulaConfig {
        class: "Tool".into(),
        binary_name: "tool".into(),
        version: "0.1.0".into(),
        description: "desc".into(),
        homepage: String::new(),
        license: String::new(),
        macos_arm64_url: String::new(),
        macos_arm64_sha256: String::new(),
        macos_x64_url: String::new(),
        macos_x64_sha256: String::new(),
        linux_arm64_url: String::new(),
        linux_arm64_sha256: String::new(),
        linux_x64_url: "https://example.com/linux-x64.tar.gz".into(),
        linux_x64_sha256: "hash".into(),
    };

    let formula = generate_formula(&config);

    assert!(!formula.contains("on_macos do"));
    assert!(formula.contains("on_linux do"));
    assert!(formula.contains("on_intel do"));
}

#[test]
fn formula_no_platforms() {
    let config = FormulaConfig {
        class: "Tool".into(),
        binary_name: "tool".into(),
        version: "0.1.0".into(),
        description: "desc".into(),
        homepage: String::new(),
        license: String::new(),
        macos_arm64_url: String::new(),
        macos_arm64_sha256: String::new(),
        macos_x64_url: String::new(),
        macos_x64_sha256: String::new(),
        linux_arm64_url: String::new(),
        linux_arm64_sha256: String::new(),
        linux_x64_url: String::new(),
        linux_x64_sha256: String::new(),
    };

    let formula = generate_formula(&config);

    assert!(!formula.contains("on_macos"));
    assert!(!formula.contains("on_linux"));
    assert!(formula.contains("bin.install \"tool\""));
}
