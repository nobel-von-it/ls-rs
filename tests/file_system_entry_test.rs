use tempfile::{NamedTempFile, TempDir};

use ls_rs::{
    error::LsError,
    files::{FileColor, FileSystemEntry, FileType},
};

#[test]
fn fse_from_file_path_test() {
    let file = NamedTempFile::new().unwrap();
    let res = FileSystemEntry::from_path(file.path().to_string_lossy());
    assert!(res.is_ok());
    let fse = res.unwrap();

    let ft = FileType::from(&fse);
    assert!(ft.is_file());
}

#[test]
fn fse_from_dir_path_test() {
    let dir = TempDir::new().unwrap();
    let res = FileSystemEntry::from_path(dir.path().to_string_lossy());
    assert!(res.is_ok());
    let fse = res.unwrap();

    let ft = FileType::from(&fse);
    assert!(ft.is_directory());
}

#[cfg(unix)]
#[test]
fn fse_from_link_path_test() {
    use std::fs;

    let dir = TempDir::new().unwrap();
    let original = dir.path().join("original.rs");
    fs::File::create(&original).unwrap();
    let link = dir.path().join("link.link");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&original, &link).unwrap();

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&original, &link).unwrap();

    let path = link.display().to_string();
    println!("link path {}", &path);

    let res = FileSystemEntry::from_path(path);
    assert!(res.is_ok());
    let fse = res.unwrap();
    assert_eq!(fse.name(), "link.link");

    let ft = FileType::from(&fse);
    assert!(ft.is_link());
}

#[test]
fn fse_from_invalid_path_test() {
    let res = FileSystemEntry::from_path("invalid_path");
    assert!(res.is_err());
    assert!(matches!(res, Err(LsError::IOError(_))));
}

#[test]
fn fse_from_empty_path_test() {
    let res = FileSystemEntry::from_path("");
    assert!(res.is_err());
    assert!(matches!(res, Err(LsError::IOError(_))));
}

#[test]
fn fse_from_root_path_test() {
    let res = FileSystemEntry::from_path("/");
    assert!(res.is_err());
    // I don't know how it works.
    assert!(matches!(res, Err(LsError::NoneValue(_))));
}

#[test]
fn fse_file_styled_name_test() {
    let file = NamedTempFile::new().unwrap();
    let fse = FileSystemEntry::from_path(file.path().to_string_lossy()).unwrap();
    assert_eq!(
        fse.name(),
        file.path().file_name().unwrap().to_string_lossy()
    );
    let style = fse.style();
    assert!(style.suffix.is_none());
    assert_eq!(style.color, FileColor::White);

    let styled = fse.get_styled_name();
    // Until extension is not supported file is displayed without styles
    assert!(styled.starts_with("\x1b"));
    assert!(styled.contains(fse.name()));
    assert_ne!(styled, fse.name());
}

#[cfg(unix)]
#[test]
fn fse_file_executable_styled_name_test() {
    use std::os::unix::fs::PermissionsExt;

    let file = NamedTempFile::new().unwrap();
    let md = file.as_file().metadata().unwrap();

    let mut perms = md.permissions();
    perms.set_mode(0o755);
    file.as_file().set_permissions(perms).unwrap();

    let fse = FileSystemEntry::from_path(file.path().to_string_lossy()).unwrap();

    let style = fse.style();
    assert!(style.suffix.is_none());
    assert_eq!(style.color, FileColor::Green);

    let styled = fse.get_styled_name();
    assert!(styled.starts_with("\x1b[32m"));
    assert!(styled.contains(fse.name()));
}

#[test]
fn fse_dir_styled_name_test() {
    let dir = TempDir::new().unwrap();
    let fse = FileSystemEntry::from_path(dir.path().to_string_lossy()).unwrap();
    assert_eq!(
        fse.name(),
        dir.path().file_name().unwrap().to_string_lossy()
    );
    let style = fse.style();
    assert!(style.suffix.is_some());
    let suffix = style.suffix.unwrap();
    assert_eq!(suffix, '/');
    assert_eq!(style.color, FileColor::Blue);

    let styled = fse.get_styled_name();
    assert!(styled.starts_with("\x1b[34m"));
    assert!(styled.contains(fse.name()));
}

#[cfg(unix)]
#[test]
fn fse_link_styled_name_test() {
    let dir = TempDir::new().unwrap();
    let original = dir.path().join("original.rs");
    std::fs::File::create(&original).unwrap();
    let link = dir.path().join("link.link");

    #[cfg(unix)]
    std::os::unix::fs::symlink(&original, &link).unwrap();

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(&original, &link).unwrap();

    let fse = FileSystemEntry::from_path(link.to_string_lossy()).unwrap();
    assert_eq!(fse.name(), "link.link");

    let style = fse.style();
    assert!(style.suffix.is_some());
    let suffix = style.suffix.unwrap();
    assert_eq!(suffix, '@');
    assert_eq!(style.color, FileColor::Aqua);

    let styled = fse.get_styled_name();
    assert!(styled.starts_with("\x1b[36m"));
    assert!(styled.contains(fse.name()));
}
