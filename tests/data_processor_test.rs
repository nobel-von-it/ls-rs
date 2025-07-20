use std::{fs, io};

use ls_rs::{
    command::{self, Config},
    data_op::DataProcessor,
    files::FileSystemEntry,
};
use tempfile::TempDir;

fn dp_setup_config_and_start_dir(
    args: &[&str],
    count_files: u8,
    add_hidden: bool,
) -> io::Result<(FileSystemEntry, Config)> {
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    let dir = TempDir::new()?;
    if add_hidden {
        for i in 0..count_files / 2 {
            fs::File::create(dir.path().join(format!("file{}.txt", i)))?;
            fs::File::create(dir.path().join(format!(".file{}.txt", i * 2)))?;
        }
    } else {
        for i in 0..count_files {
            fs::File::create(dir.path().join(format!("file{}.txt", i)))?;
        }
    }

    let mut fse = FileSystemEntry::from_path(dir.path().to_string_lossy()).unwrap();
    fse.fill_start_dir();

    Ok((fse, config))
}

#[test]
fn dp_filter_test() {
    let file_count = 10;
    let (fse, config) = dp_setup_config_and_start_dir(&["ls_rs"], file_count, true).unwrap();
    assert!(!config.all);

    let mut dp = DataProcessor::new(fse.get_dir_entries().unwrap(), config);

    let start_len = dp.data_len();
    assert_eq!(start_len, file_count as usize);

    dp = dp.filter();
    let new_len = dp.data_len();

    assert_ne!(start_len, new_len);
    assert_eq!(new_len, file_count as usize / 2);
}
