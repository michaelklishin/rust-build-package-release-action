use rust_release_action::{env_or, parse_comma_list};
use std::env;
use std::sync::{LazyLock, Mutex};

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
fn env_or_returns_env_value() {
    let _lock = ENV_LOCK.lock().unwrap();
    // Safety: serialised by ENV_LOCK
    unsafe { env::set_var("TEST_ENV_OR_KEY", "found") };
    assert_eq!(env_or("TEST_ENV_OR_KEY", "fallback"), "found");
    unsafe { env::remove_var("TEST_ENV_OR_KEY") };
}

#[test]
fn env_or_returns_default_when_unset() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { env::remove_var("TEST_ENV_OR_MISSING") };
    assert_eq!(env_or("TEST_ENV_OR_MISSING", "default_val"), "default_val");
}

#[test]
fn env_or_returns_empty_string_when_set_empty() {
    let _lock = ENV_LOCK.lock().unwrap();
    unsafe { env::set_var("TEST_ENV_OR_EMPTY", "") };
    assert_eq!(env_or("TEST_ENV_OR_EMPTY", "fallback"), "");
    unsafe { env::remove_var("TEST_ENV_OR_EMPTY") };
}

#[test]
fn parse_comma_list_empty() {
    assert!(parse_comma_list("").is_empty());
}

#[test]
fn parse_comma_list_single() {
    assert_eq!(parse_comma_list("foo"), vec!["foo"]);
}

#[test]
fn parse_comma_list_multiple() {
    assert_eq!(parse_comma_list("a,b,c"), vec!["a", "b", "c"]);
}

#[test]
fn parse_comma_list_trims_whitespace() {
    assert_eq!(parse_comma_list(" a , b , c "), vec!["a", "b", "c"]);
}

#[test]
fn parse_comma_list_skips_empty_segments() {
    assert_eq!(parse_comma_list("a,,b,"), vec!["a", "b"]);
}

#[test]
fn parse_comma_list_all_whitespace_segments() {
    assert!(parse_comma_list(", , ,").is_empty());
}
