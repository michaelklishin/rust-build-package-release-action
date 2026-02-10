use rust_release_action::output::{output, output_multiline};
use std::env;
use std::fs;
use std::sync::{LazyLock, Mutex};
use tempfile::NamedTempFile;

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
fn output_writes_key_value() {
    let _lock = ENV_LOCK.lock().unwrap();
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_str().unwrap().to_string();

    // Safety: serialised by ENV_LOCK
    unsafe { env::set_var("GITHUB_OUTPUT", &path) };
    output("my_key", "my_value");
    unsafe { env::remove_var("GITHUB_OUTPUT") };

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "my_key=my_value\n");
}

#[test]
fn output_appends_to_existing() {
    let _lock = ENV_LOCK.lock().unwrap();
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_str().unwrap().to_string();
    fs::write(&path, "existing=data\n").unwrap();

    unsafe { env::set_var("GITHUB_OUTPUT", &path) };
    output("second", "val");
    unsafe { env::remove_var("GITHUB_OUTPUT") };

    let content = fs::read_to_string(&path).unwrap();
    assert_eq!(content, "existing=data\nsecond=val\n");
}

#[test]
fn output_multiline_uses_delimiter() {
    let _lock = ENV_LOCK.lock().unwrap();
    let file = NamedTempFile::new().unwrap();
    let path = file.path().to_str().unwrap().to_string();

    unsafe { env::set_var("GITHUB_OUTPUT", &path) };
    output_multiline("body", "line1\nline2");
    unsafe { env::remove_var("GITHUB_OUTPUT") };

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.starts_with("body<<EOF_RUST_RELEASE_ACTION\n"));
    assert!(content.contains("line1\nline2"));
    assert!(content.ends_with("EOF_RUST_RELEASE_ACTION\n"));
}

#[test]
fn output_noop_without_env() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { env::remove_var("GITHUB_OUTPUT") };
    output("key", "value");
    output_multiline("key", "value");
}

#[test]
fn output_noop_with_empty_env() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { env::set_var("GITHUB_OUTPUT", "") };
    output("key", "value");
    output_multiline("key", "value");
    unsafe { env::remove_var("GITHUB_OUTPUT") };
}
