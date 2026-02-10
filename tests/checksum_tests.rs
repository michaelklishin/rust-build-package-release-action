mod test_helpers;

use rust_release_action::checksum::{
    blake2_file, compute_checksum, detect_checksum_type, generate_checksums, parse_checksum_file,
    sha256_bytes, sha256_file, sha512_file, verify_checksum,
};
use std::env;
use std::fs;
use std::sync::{LazyLock, Mutex};
use test_helpers::create_temp_file;

#[test]
fn sha256_known_value() {
    let f = create_temp_file(b"hello world\n");
    let hash = sha256_file(f.path()).unwrap();
    assert_eq!(
        hash,
        "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447"
    );
}

#[test]
fn sha256_empty_file() {
    let f = create_temp_file(b"");
    let hash = sha256_file(f.path()).unwrap();
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn sha512_known_value() {
    let f = create_temp_file(b"hello world\n");
    let hash = sha512_file(f.path()).unwrap();
    assert_eq!(
        hash,
        "db3974a97f2407b7cae1ae637c0030687a11913274d578492558e39c16c017de84eacdc8c62fe34ee4e12b4b1428817f09b6a2760c3f8a664ceae94d2434a593"
    );
}

#[test]
fn blake2_known_value() {
    let f = create_temp_file(b"hello world\n");
    let hash = blake2_file(f.path()).unwrap();
    assert_eq!(
        hash,
        "fec91c70284c72d0d4e3684788a90de9338a5b2f47f01fedbe203cafd68708718ae5672d10eca804a8121904047d40d1d6cf11e7a76419357a9469af41f22d01"
    );
}

#[test]
fn sha256_bytes_known_value() {
    let hash = sha256_bytes(b"hello");
    assert_eq!(
        hash,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn detect_checksum_type_sha256() {
    assert_eq!(detect_checksum_type("file.sha256"), "sha256");
}

#[test]
fn detect_checksum_type_sha512() {
    assert_eq!(detect_checksum_type("file.sha512"), "sha512");
}

#[test]
fn detect_checksum_type_b2() {
    assert_eq!(detect_checksum_type("file.b2"), "b2");
}

#[test]
fn detect_checksum_type_default() {
    assert_eq!(detect_checksum_type("file.txt"), "sha256");
}

#[test]
fn parse_checksum_file_standard_format() {
    let f = create_temp_file(b"abc123  filename.tar.gz\n");
    let hash = parse_checksum_file(f.path()).unwrap();
    assert_eq!(hash, "abc123");
}

#[test]
fn parse_checksum_file_hash_only() {
    let f = create_temp_file(b"abc123\n");
    let hash = parse_checksum_file(f.path()).unwrap();
    assert_eq!(hash, "abc123");
}

#[test]
fn parse_checksum_file_empty() {
    let f = create_temp_file(b"");
    let result = parse_checksum_file(f.path());
    assert!(result.is_err());
}

#[test]
fn compute_checksum_sha256() {
    let f = create_temp_file(b"test");
    let hash = compute_checksum(f.path(), "sha256").unwrap();
    assert_eq!(
        hash,
        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    );
}

#[test]
fn compute_checksum_sha512() {
    let f = create_temp_file(b"test");
    let hash = compute_checksum(f.path(), "sha512").unwrap();
    assert!(hash.len() == 128);
}

#[test]
fn compute_checksum_b2() {
    let f = create_temp_file(b"test");
    let hash = compute_checksum(f.path(), "b2").unwrap();
    assert!(hash.len() == 128);
}

#[test]
fn compute_checksum_unsupported() {
    let f = create_temp_file(b"test");
    let result = compute_checksum(f.path(), "md5");
    assert!(result.is_err());
}

#[test]
fn verify_checksum_valid() {
    let artifact = create_temp_file(b"test data");
    let hash = sha256_file(artifact.path()).unwrap();
    let checksum_content = format!("{hash}  artifact.tar.gz\n");
    let checksum_file = create_temp_file(checksum_content.as_bytes());

    verify_checksum(artifact.path(), checksum_file.path()).unwrap();
}

#[test]
fn verify_checksum_mismatch() {
    let artifact = create_temp_file(b"test data");
    let checksum_file = create_temp_file(
        b"0000000000000000000000000000000000000000000000000000000000000000  artifact.tar.gz\n",
    );

    let result = verify_checksum(artifact.path(), checksum_file.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("mismatch"));
}

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[test]
fn generate_checksums_sha256_only() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("artifact.tar.gz");
    fs::write(&file_path, b"test data").unwrap();

    // Safety: serialised by ENV_LOCK
    unsafe { env::set_var("CHECKSUM", "sha256") };
    let checksums = generate_checksums(&file_path).unwrap();
    unsafe { env::remove_var("CHECKSUM") };

    assert!(!checksums.sha256.is_empty());
    assert!(checksums.sha512.is_empty());
    assert!(checksums.b2.is_empty());

    let sidecar = dir.path().join("artifact.tar.gz.sha256");
    assert!(sidecar.exists());
    let content = fs::read_to_string(sidecar).unwrap();
    assert!(content.contains(&checksums.sha256));
    assert!(content.contains("artifact.tar.gz"));
}

#[test]
fn generate_checksums_all_types() {
    let _lock = ENV_LOCK.lock().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("binary");
    fs::write(&file_path, b"binary content").unwrap();

    unsafe { env::set_var("CHECKSUM", "sha256,sha512,b2") };
    let checksums = generate_checksums(&file_path).unwrap();
    unsafe { env::remove_var("CHECKSUM") };

    assert!(!checksums.sha256.is_empty());
    assert!(!checksums.sha512.is_empty());
    assert!(!checksums.b2.is_empty());

    assert!(dir.path().join("binary.sha256").exists());
    assert!(dir.path().join("binary.sha512").exists());
    assert!(dir.path().join("binary.b2").exists());
}
