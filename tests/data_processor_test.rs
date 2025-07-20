use std::{
    fs,
    io::{self, Write},
};

struct DpTestConfig {
    count_default_files: usize,
    count_hidden_files: usize,
    // usize is end of range 0..content_size
    size_multiplier: Option<usize>,
}

const DEFAULT_MESSAGE: &str = "LIMP";

impl DpTestConfig {
    fn new(
        count_default_files: usize,
        count_hidden_files: usize,
        add_some_content: Option<usize>,
    ) -> Self {
        Self {
            count_default_files,
            count_hidden_files,
            size_multiplier: add_some_content,
        }
    }
    fn setup_config_and_start_dir(&self, args: &[&str]) -> io::Result<(FileSystemEntry, Config)> {
        let matches = command::ls_command().get_matches_from(args);
        let config = command::Config::clap_parse(&matches);

        let dir = TempDir::new()?;
        for i in 0..self.count_default_files {
            let mut file = fs::File::create(dir.path().join(format!("file{i}.txt")))?;
            if let Some(mul) = self.size_multiplier {
                file.write_all(
                    DEFAULT_MESSAGE
                        .repeat((self.count_default_files - i) * mul)
                        .as_bytes(),
                )?;
            }
        }
        for i in 0..self.count_hidden_files {
            let mut file = fs::File::create(
                dir.path()
                    .join(format!(".hidden-file{}.txt", self.count_default_files + i)),
            )?;
            if let Some(mul) = self.size_multiplier {
                file.write_all(
                    DEFAULT_MESSAGE
                        .repeat((self.count_hidden_files + self.count_default_files - i) * mul)
                        .as_bytes(),
                )?;
            }
        }

        println!("{}", fs::read_dir(dir.path()).unwrap().count());
        fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .for_each(|de| println!("{}", de.file_name().display()));
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
    let count_files = 10;
    let dp_config = DpTestConfig::new(count_files, count_files, None);
    let (fse, config) = dp_config.setup_config_and_start_dir(&["ls_rs"]).unwrap();
    assert!(!config.all);

    let mut dp = DataProcessor::new(fse.get_dir_entries().unwrap(), config);

    let start_len = dp.data_len();
    assert_eq!(start_len, count_files * 2);

    dp = dp.filter();
    let new_len = dp.data_len();

    assert_ne!(start_len, new_len);
    assert_eq!(new_len, count_files);
}

#[test]
fn dp_sort_size_test() {
    let count_files = 10;
    let dp_config = DpTestConfig::new(count_files, count_files, Some(3));
    let (fse, config) = dp_config
        .setup_config_and_start_dir(&["ls_rs", "-S"])
        .unwrap();
    assert!(config.size_sort);

    let dp = DataProcessor::new(fse.get_dir_entries().unwrap(), config);
    assert_eq!(dp.data_len(), count_files * 2);
    let dp_sorted = dp.clone().filter().sort();
    assert_ne!(dp, dp_sorted);
}
