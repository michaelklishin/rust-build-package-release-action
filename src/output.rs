use std::env;
use std::fs::OpenOptions;
use std::io::Write;

pub fn output(key: &str, value: &str) {
    if let Ok(path) = env::var("GITHUB_OUTPUT") {
        if !path.is_empty() {
            if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(&path) {
                let _ = writeln!(f, "{key}={value}");
            }
        }
    }
}

pub fn output_multiline(key: &str, value: &str) {
    if let Ok(path) = env::var("GITHUB_OUTPUT") {
        if !path.is_empty() {
            if let Ok(mut f) = OpenOptions::new().append(true).create(true).open(&path) {
                let delimiter = "EOF_RUST_RELEASE_ACTION";
                let _ = writeln!(f, "{key}<<{delimiter}\n{value}\n{delimiter}");
            }
        }
    }
}

pub fn print_hr() {
    println!(
        "\x1b[32m---------------------------------------------------------------------------->\x1b[0m"
    );
}
