mod test_helpers;

use rust_release_action::archive::{copy_docs, copy_includes, list_archivable_files};
use std::fs;
use std::sync::{LazyLock, Mutex};
use tempfile::TempDir;

/// Serializes tests that change the process-wide CWD.
static CWD_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn create_test_dir_with_files(files: &[&str]) -> TempDir {
    let dir = TempDir::new().unwrap();
    for name in files {
        fs::write(dir.path().join(name), "content").unwrap();
    }
    dir
}

#[test]
fn archivable_includes_binary_and_docs() {
    let dir = create_test_dir_with_files(&["myapp", "README.md", "LICENSE"]);
    let files = list_archivable_files(dir.path());
    assert!(files.contains(&"myapp".to_string()));
    assert!(files.contains(&"README.md".to_string()));
    assert!(files.contains(&"LICENSE".to_string()));
}

#[test]
fn archivable_excludes_archives() {
    let dir = create_test_dir_with_files(&["myapp", "myapp.tar.gz", "myapp.zip"]);
    let files = list_archivable_files(dir.path());
    assert!(files.contains(&"myapp".to_string()));
    assert!(!files.contains(&"myapp.tar.gz".to_string()));
    assert!(!files.contains(&"myapp.zip".to_string()));
}

#[test]
fn archivable_excludes_checksums() {
    let dir = create_test_dir_with_files(&["myapp", "myapp.sha256", "myapp.sha512", "myapp.b2"]);
    let files = list_archivable_files(dir.path());
    assert!(files.contains(&"myapp".to_string()));
    assert!(!files.contains(&"myapp.sha256".to_string()));
    assert!(!files.contains(&"myapp.sha512".to_string()));
    assert!(!files.contains(&"myapp.b2".to_string()));
}

#[test]
fn archivable_excludes_signatures() {
    let dir =
        create_test_dir_with_files(&["myapp", "myapp.sig", "myapp.pem", "myapp.sigstore.json"]);
    let files = list_archivable_files(dir.path());
    assert!(files.contains(&"myapp".to_string()));
    assert!(!files.contains(&"myapp.sig".to_string()));
    assert!(!files.contains(&"myapp.pem".to_string()));
    assert!(!files.contains(&"myapp.sigstore.json".to_string()));
}

#[test]
fn archivable_excludes_sbom_files() {
    let dir = create_test_dir_with_files(&["myapp", "myapp.spdx.json", "myapp.cdx.json"]);
    let files = list_archivable_files(dir.path());
    assert!(files.contains(&"myapp".to_string()));
    assert!(!files.contains(&"myapp.spdx.json".to_string()));
    assert!(!files.contains(&"myapp.cdx.json".to_string()));
}

#[test]
fn archivable_returns_sorted() {
    let dir = create_test_dir_with_files(&["zzz", "aaa", "mmm"]);
    let files = list_archivable_files(dir.path());
    assert_eq!(files, vec!["aaa", "mmm", "zzz"]);
}

#[test]
fn archivable_skips_directories() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("myapp"), "binary").unwrap();
    fs::create_dir(dir.path().join("subdir")).unwrap();
    let files = list_archivable_files(dir.path());
    assert_eq!(files, vec!["myapp"]);
}

#[test]
fn copy_docs_copies_license_and_readme() {
    let _lock = CWD_LOCK.lock().unwrap();
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    fs::write(src.path().join("LICENSE"), "MIT").unwrap();
    fs::write(src.path().join("LICENSE-APACHE"), "Apache").unwrap();
    fs::write(src.path().join("README.md"), "Hello").unwrap();

    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(src.path()).unwrap();
    copy_docs(dest.path()).unwrap();
    std::env::set_current_dir(original).unwrap();

    assert!(dest.path().join("LICENSE").exists());
    assert!(dest.path().join("LICENSE-APACHE").exists());
    assert!(dest.path().join("README.md").exists());
}

#[test]
fn copy_docs_no_readme_still_works() {
    let _lock = CWD_LOCK.lock().unwrap();
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    fs::write(src.path().join("LICENSE"), "MIT").unwrap();

    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(src.path()).unwrap();
    copy_docs(dest.path()).unwrap();
    std::env::set_current_dir(original).unwrap();

    assert!(dest.path().join("LICENSE").exists());
    assert!(!dest.path().join("README.md").exists());
}

#[test]
fn copy_includes_copies_matching_files() {
    let _lock = CWD_LOCK.lock().unwrap();
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    fs::write(src.path().join("config.toml"), "key = 1").unwrap();
    fs::write(src.path().join("data.json"), "{}").unwrap();
    fs::write(src.path().join("other.txt"), "skip").unwrap();

    let original = std::env::current_dir().unwrap();
    std::env::set_current_dir(src.path()).unwrap();
    unsafe {
        std::env::set_var("ARCHIVE_INCLUDE", "*.toml, *.json");
    }
    copy_includes(dest.path()).unwrap();
    unsafe {
        std::env::remove_var("ARCHIVE_INCLUDE");
    }
    std::env::set_current_dir(original).unwrap();

    assert!(dest.path().join("config.toml").exists());
    assert!(dest.path().join("data.json").exists());
    assert!(!dest.path().join("other.txt").exists());
}

#[test]
fn copy_includes_empty_is_noop() {
    let _lock = CWD_LOCK.lock().unwrap();
    let dest = TempDir::new().unwrap();

    unsafe {
        std::env::remove_var("ARCHIVE_INCLUDE");
    }
    copy_includes(dest.path()).unwrap();

    assert_eq!(fs::read_dir(dest.path()).unwrap().count(), 0);
}
