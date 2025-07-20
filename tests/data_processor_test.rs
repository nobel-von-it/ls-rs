use std::{fs, io};

struct DpTestConfig {
    count_files: usize,
    add_hidden: bool,
    // usize is end of range 0..content_size
    add_some_content: Option<usize>,
}
impl DpTestConfig {
    fn new(count_files: usize, add_hidden: bool, add_some_content: Option<usize>) -> Self {
        Self {
            count_files,
            add_hidden,
            add_some_content,
        }
    }
    fn setup_config_and_start_dir(&self, args: &[&str]) -> io::Result<(FileSystemEntry, Config)> {
        let matches = command::ls_command().get_matches_from(args);
        let config = command::Config::clap_parse(&matches);

        let dir = TempDir::new()?;
        if self.add_hidden {
            for i in 0..self.count_files / 2 {
                fs::File::create(dir.path().join(format!("file{i}.txt")))?;
                fs::File::create(
                    dir.path()
                        .join(format!(".file{}.txt", (self.count_files / 2) + i)),
                )?;
            }
        } else {
            for i in 0..self.count_files {
                fs::File::create(dir.path().join(format!("file{i}.txt")))?;
            }
        }

        let fse = FileSystemEntry::new_with_config(&config).unwrap();

        Ok((fse, config))
    }
}

use ls_rs::{
    command::{self, Config},
    data_op::DataProcessor,
    files::FileSystemEntry,
};
use tempfile::TempDir;

#[test]
fn dp_filter_test() {
    let dp_config = DpTestConfig::new(10, true, None);
    let file_count = 10;
    let (fse, config) = dp_config.setup_config_and_start_dir(&["ls_rs"]).unwrap();
    assert!(!config.all);

    let mut dp = DataProcessor::new(fse.get_dir_entries().unwrap(), config);

    let start_len = dp.data_len();
    assert_eq!(start_len, file_count as usize);

    dp = dp.filter();
    let new_len = dp.data_len();

    assert_ne!(start_len, new_len);
    assert_eq!(new_len, file_count as usize / 2);
}
