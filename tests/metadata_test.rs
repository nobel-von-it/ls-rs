use chrono::{DateTime, Local};
use ls_rs::files::MetaData;
use std::io::{self, Write};
use tempfile::NamedTempFile;

// #[derive(Debug, Clone)]
// pub struct MetaData {
//     pub size: u64,
//     pub human_size: String,
//
//     pub inode: u64,
//
//     pub mode: u32,
//     pub mode_str: String,
//
//     pub created_at: DateTime<Local>,
//     pub modified_at: DateTime<Local>,
// }

#[test]
fn metadata_file_size_test() -> io::Result<()> {
    let mut file = NamedTempFile::new()?;
    let message = "Hello world";
    file.write_all(message.as_bytes())?;

    let md = MetaData::try_from(&file.as_file().metadata()?);
    assert!(md.is_some());
    let md = md.unwrap();

    assert_eq!(md.size, message.len() as u64);
    assert_eq!(md.human_size, format!("{}B", message.len()));

    Ok(())
}

#[cfg(unix)]
#[test]
fn medatada_file_mode_test() -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let file = NamedTempFile::new()?;
    let md = file.as_file().metadata()?;

    let mut perms = md.permissions();
    perms.set_mode(0o755);
    file.as_file().set_permissions(perms)?;

    let md = MetaData::try_from(&file.as_file().metadata()?);
    assert!(md.is_some());
    let md = md.unwrap();

    assert_eq!(md.mode, 0o100755); // flags + mode
    assert_eq!(md.mode_str, "-rwxr-xr-x");

    Ok(())
}

#[cfg(windows)]
#[test]
fn medatada_file_readonly_mode_test() -> io::Result<()> {
    let file = NamedTempFile::new()?;
    let md = file.as_file().metadata()?;

    let mut perms = md.permissions();
    println!("readonly {}", perms.readonly());
    assert!(!perms.readonly());
    perms.set_readonly(true);
    assert!(perms.readonly());
    file.as_file().set_permissions(perms)?;
    assert!(file.as_file().metadata()?.permissions().readonly());

    let md = MetaData::try_from(&file.as_file().metadata()?);
    assert!(md.is_some());
    let md = md.unwrap();

    md.attributes
        .iter()
        .enumerate()
        .for_each(|(i, a)| println!("attrs[{i}] = {a}"));
    assert!(md.attributes[2]); // flags + mode
    assert_eq!(md.mode_str, "-ar---");

    Ok(())
}

// #[cfg(windows)]
// #[test]
// fn medatada_dir_mode_test() -> io::Result<()> {
//     let file = NamedTempFile::new()?;
//     let md = file.as_file().metadata()?;

//     let mut perms = md.permissions();
//     perms.set_readonly(!perms.readonly());

//     let md = MetaData::try_from(&file.as_file().metadata()?);
//     assert!(md.is_some());
//     let md = md.unwrap();

//     assert!(md.attributes[2]); // flags + mode
//     assert_eq!(md.mode_str, "-r--r--r--");

//     Ok(())
// }

#[cfg(unix)]
#[test]
fn metadata_inode_or_file_index_test() -> io::Result<()> {
    let file = NamedTempFile::new()?;
    let metadata = file.as_file().metadata()?;

    let md = MetaData::try_from(&metadata);
    assert!(md.is_some());
    let md = md.unwrap();

    assert!(md.inode > 0);

    Ok(())
}

#[test]
fn metadata_timestamps_test() -> io::Result<()> {
    use std::{thread::sleep, time::Duration};

    let file = NamedTempFile::new()?;
    let created_before = file.as_file().metadata()?.created()?;
    sleep(Duration::from_secs(1));

    file.as_file().set_len(0)?;
    let modified_before = file.as_file().metadata()?.modified()?;
    sleep(Duration::from_secs(1));

    let md = MetaData::try_from(&file.as_file().metadata()?);
    assert!(md.is_some());
    let md = md.unwrap();

    let created_before: DateTime<Local> = created_before.into();
    let modified_before: DateTime<Local> = modified_before.into();

    assert!(md.created_at <= created_before);
    assert!(md.modified_at >= modified_before);

    Ok(())
}

#[cfg(unix)]
#[test]
fn metadata_symlink_test() -> io::Result<()> {
    use std::os::unix::fs::symlink;
    let file = NamedTempFile::new()?;
    let symlink_path = file.path().with_extension("symlink");
    symlink(file.path(), &symlink_path)?;

    let md = MetaData::try_from(&std::fs::symlink_metadata(&symlink_path)?);
    assert!(md.is_some());
    let md = md.unwrap();

    assert!(md.mode_str.starts_with('l')); // "lrwxr-xr-x"

    Ok(())
}
