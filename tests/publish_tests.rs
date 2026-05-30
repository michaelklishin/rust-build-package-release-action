use rust_release_action::publish::{build_publish_args, is_version_tag};
use std::sync::{LazyLock, Mutex};

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

const PUBLISH_VARS: &[&str] = &[
    "PACKAGE",
    "FEATURES",
    "NO_DEFAULT_FEATURES",
    "LOCKED",
    "MANIFEST_PATH",
];

fn clear_publish_env() {
    for var in PUBLISH_VARS {
        // Safety: serialised by ENV_LOCK
        unsafe { std::env::remove_var(var) };
    }
}

#[test]
fn publish_args_minimal() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();

    assert_eq!(build_publish_args(false), vec!["publish"]);
}

#[test]
fn publish_args_dry_run() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();

    assert_eq!(build_publish_args(true), vec!["publish", "--dry-run"]);
}

#[test]
fn publish_args_with_package() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();
    // Safety: serialised by ENV_LOCK
    unsafe { std::env::set_var("PACKAGE", "my-crate") };

    assert_eq!(
        build_publish_args(false),
        vec!["publish", "--package", "my-crate"]
    );
}

#[test]
fn publish_args_with_features() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();
    // Safety: serialised by ENV_LOCK
    unsafe { std::env::set_var("FEATURES", "foo,bar") };

    assert_eq!(
        build_publish_args(false),
        vec!["publish", "--features", "foo,bar"]
    );
}

#[test]
fn publish_args_no_default_features() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();
    // Safety: serialised by ENV_LOCK
    unsafe { std::env::set_var("NO_DEFAULT_FEATURES", "true") };

    assert_eq!(
        build_publish_args(false),
        vec!["publish", "--no-default-features"]
    );
}

#[test]
fn publish_args_locked() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();
    // Safety: serialised by ENV_LOCK
    unsafe { std::env::set_var("LOCKED", "true") };

    assert_eq!(build_publish_args(false), vec!["publish", "--locked"]);
}

#[test]
fn publish_args_custom_manifest() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();
    // Safety: serialised by ENV_LOCK
    unsafe { std::env::set_var("MANIFEST_PATH", "subcrate/Cargo.toml") };

    assert_eq!(
        build_publish_args(false),
        vec!["publish", "--manifest-path", "subcrate/Cargo.toml"]
    );
}

#[test]
fn publish_args_default_manifest_omitted() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();
    // Safety: serialised by ENV_LOCK
    unsafe { std::env::set_var("MANIFEST_PATH", "Cargo.toml") };

    let args = build_publish_args(false);
    assert!(!args.contains(&"--manifest-path".to_string()));
}

#[test]
fn publish_args_all_options() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();
    // Safety: serialised by ENV_LOCK
    unsafe {
        std::env::set_var("PACKAGE", "my-lib");
        std::env::set_var("FEATURES", "serde");
        std::env::set_var("NO_DEFAULT_FEATURES", "true");
        std::env::set_var("LOCKED", "true");
        std::env::set_var("MANIFEST_PATH", "libs/Cargo.toml");
    }

    let args = build_publish_args(true);
    assert_eq!(
        args,
        vec![
            "publish",
            "--dry-run",
            "--package",
            "my-lib",
            "--manifest-path",
            "libs/Cargo.toml",
            "--no-default-features",
            "--features",
            "serde",
            "--locked",
        ]
    );
}

#[test]
fn publish_args_no_default_features_false_is_ignored() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();
    // Safety: serialised by ENV_LOCK
    unsafe { std::env::set_var("NO_DEFAULT_FEATURES", "false") };

    assert_eq!(build_publish_args(false), vec!["publish"]);
}

#[test]
fn publish_args_locked_false_is_ignored() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_publish_env();
    // Safety: serialised by ENV_LOCK
    unsafe { std::env::set_var("LOCKED", "false") };

    assert_eq!(build_publish_args(false), vec!["publish"]);
}

#[test]
fn is_version_tag_accepts_well_formed_tags() {
    assert!(is_version_tag("v1.2.3"));
    assert!(is_version_tag("v0.12.0"));
    assert!(is_version_tag("v10.20.30"));
    assert!(is_version_tag("v1.2.3-rc.1"));
    assert!(is_version_tag("v1.2.3-alpha.2"));
    assert!(is_version_tag("v1.2.3+build.42"));
}

#[test]
fn is_version_tag_rejects_pr_and_branch_refs() {
    assert!(!is_version_tag("14/merge"));
    assert!(!is_version_tag("123/merge"));
    assert!(!is_version_tag("main"));
    assert!(!is_version_tag("dependabot/cargo/foo-1.0.0"));
}

#[test]
fn is_version_tag_rejects_empty_and_malformed() {
    assert!(!is_version_tag(""));
    assert!(!is_version_tag("v"));
    assert!(!is_version_tag("1.2.3"));
    assert!(!is_version_tag("vfoo"));
    assert!(!is_version_tag("v1.2"));
    assert!(!is_version_tag("v1"));
}
