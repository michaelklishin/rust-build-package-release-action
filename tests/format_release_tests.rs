use rust_release_action::format_release::format_size;

#[test]
fn format_size_bytes() {
    assert_eq!(format_size(0), "0 B");
    assert_eq!(format_size(500), "500 B");
    assert_eq!(format_size(1023), "1023 B");
}

#[test]
fn format_size_kilobytes() {
    assert_eq!(format_size(1024), "1.0 KB");
    assert_eq!(format_size(1536), "1.5 KB");
    assert_eq!(format_size(10240), "10.0 KB");
}

#[test]
fn format_size_megabytes() {
    assert_eq!(format_size(1048576), "1.0 MB");
    assert_eq!(format_size(5242880), "5.0 MB");
    assert_eq!(format_size(10485760), "10.0 MB");
}

#[test]
fn format_size_boundary_values() {
    assert_eq!(format_size(1), "1 B");
    assert_eq!(format_size(1024 * 1024 - 1), "1024.0 KB");
    assert_eq!(format_size(100 * 1024 * 1024), "100.0 MB");
}
