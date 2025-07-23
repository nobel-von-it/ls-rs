use ls_rs::{files::FileSystemEntry, json::Serializer};
use tempfile::{NamedTempFile, TempDir};

fn is_valid_json(json: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(json).is_ok()
}

#[test]
fn is_valid_json_test() {
    let valid_json = r#" {"valid": "json"} "#;
    assert!(is_valid_json(valid_json));

    let bruh_json = r#" {"bruh: "json"} "#;
    assert!(!is_valid_json(bruh_json))
}

#[test]
fn json_fse_file_test() {
    let file = NamedTempFile::new().unwrap();

    let fse = FileSystemEntry::from_path(file.path().to_string_lossy());
    assert!(fse.is_ok());

    let fse = fse.unwrap();

    let json = fse.short_json();
    println!("{}", &json);
    assert!(is_valid_json(&json));
}

#[test]
fn json_fse_dir_test() {
    let dir = TempDir::new().unwrap();

    let fse = FileSystemEntry::from_path(dir.path().to_string_lossy());
    assert!(fse.is_ok());
    let fse = fse.unwrap();

    let json = fse.short_json();
    println!("{}", &json);
    assert!(is_valid_json(&json));
}

#[cfg(unix)]
#[test]
fn json_fse_link_test() {
    use std::os::unix::fs::symlink;

    let file = NamedTempFile::new().unwrap();
    let symlink_path = file.path().with_extension("symlink");
    symlink(file.path(), &symlink_path).unwrap();

    let fse = FileSystemEntry::from_path(symlink_path.display().to_string());
    assert!(fse.is_some());
    let fse = fse.unwrap();

    let json = fse.short_json();
    println!("{}", &json);
    assert!(is_valid_json(&json));
}
