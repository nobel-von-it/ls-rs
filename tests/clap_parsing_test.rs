use ls_rs::command::{self, SortType};

// #[derive(Debug)]
// pub struct Config {
//     pub path: String,
//     pub cols: Option<usize>,
//     pub all: bool,
//     pub long: bool,
//     pub numeric: bool,
//     pub humanable: bool,
//     pub reverse: bool,
//     pub name_sort: bool,
//     pub time_sort: bool,
//     pub size_sort: bool,
//     // pub ext_sort: bool,
//     pub recursive: bool,
//     pub one_col: bool,
//     pub inode: bool,
//     pub json_mini: bool,
//     pub json_big: bool,
// }

#[test]
fn flag_la_test() {
    let args = ["ls-rs", "-la"];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(config.long);
    assert!(config.all);

    assert_eq!(config.cols, None);
}

#[test]
fn flag_full_in_one_test() {
    let args = ["ls-rs", "-alnHrNTSoijJ"];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(config.all);
    assert!(config.long);
    assert!(config.numeric);
    assert!(config.humanable);
    assert!(config.reverse);
    assert!(config.sort_type.is_some());
    assert!(matches!(config.sort_type.unwrap(), SortType::Name));
    assert!(config.one_col);
    assert!(config.inode);
    assert!(config.json_mini);
    assert!(config.json_big);

    assert_eq!(config.cols, None);
    assert_eq!(config.recursive, None);
}

#[test]
fn flag_full_full_test() {
    let args = [
        "ls-rs",
        "--all",
        "--long",
        "--numeric",
        "--humanable",
        "--reverse",
        "--name",
        "--time",
        "--size",
        "--one",
        "--inode",
        "--json",
        "--JSON",
    ];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(config.all);
    assert!(config.long);
    assert!(config.numeric);
    assert!(config.humanable);
    assert!(config.reverse);
    assert!(config.sort_type.is_some());
    assert!(matches!(config.sort_type.unwrap(), SortType::Name));
    assert!(config.one_col);
    assert!(config.inode);
    assert!(config.json_mini);
    assert!(config.json_big);

    assert_eq!(config.cols, None);
    assert_eq!(config.recursive, None);
}

#[test]
fn flag_full_short_test() {
    let args = [
        "ls-rs", "-a", "-l", "-n", "-H", "-r", "-N", "-T", "-S", "-o", "-i", "-j", "-J",
    ];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(config.all);
    assert!(config.long);
    assert!(config.numeric);
    assert!(config.humanable);
    assert!(config.reverse);
    assert!(config.sort_type.is_some());
    assert!(matches!(config.sort_type.unwrap(), SortType::Name));
    assert!(config.one_col);
    assert!(config.inode);
    assert!(config.json_mini);
    assert!(config.json_big);

    assert_eq!(config.cols, None);
    assert_eq!(config.recursive, None);
}

#[test]
fn flag_valuable_test() {
    const PATH: &str = "sldkjfskdfj";
    const COLS: usize = 10;
    const REC_DEPTH: usize = 3;

    let args = [
        "ls-rs",
        "-C",
        &COLS.to_string(),
        PATH,
        "-R",
        &REC_DEPTH.to_string(),
    ];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(config.sort_type.is_none());
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert_eq!(config.path, PATH);
    assert_eq!(config.cols, Some(COLS));
    assert_eq!(
        config.recursive,
        Some(command::RecursionOptions::Depth(REC_DEPTH))
    );
}

#[test]
fn flag_recursion_unlim_test() {
    const PATH: &str = "sldkjfskdfj";
    const COLS: usize = 10;
    const REC_MSG: &str = "max";

    let args = ["ls-rs", "-C", &COLS.to_string(), PATH, "-R", REC_MSG];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(config.sort_type.is_none());
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert_eq!(config.path, PATH);
    assert_eq!(config.cols, Some(COLS));
    assert_eq!(config.recursive, Some(command::RecursionOptions::Unlimited));
}

#[test]
fn flag_sort_type_value_name_test() {
    let args = ["ls-rs", "--sort", "name"];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert!(config.sort_type.is_some());
    assert!(matches!(config.sort_type.unwrap(), SortType::Name));
}

#[test]
fn flag_sort_type_flag_name_test() {
    let args = ["ls-rs", "-N"];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert!(config.sort_type.is_some());
    assert!(matches!(config.sort_type.unwrap(), SortType::Name));
}

#[test]
fn flag_sort_type_value_time_test() {
    let args = ["ls-rs", "--sort", "time"];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert!(config.sort_type.is_some());
    assert!(matches!(config.sort_type.unwrap(), SortType::Time));
}

#[test]
fn flag_sort_type_flag_time_test() {
    let args = ["ls-rs", "-T"];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert!(config.sort_type.is_some());
    assert!(matches!(config.sort_type.unwrap(), SortType::Time));
}

#[test]
fn flag_sort_type_value_size_test() {
    let args = ["ls-rs", "--sort", "size"];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert!(config.sort_type.is_some());
    assert!(matches!(config.sort_type.unwrap(), SortType::Size));
}

#[test]
fn flag_sort_type_flag_size_test() {
    let args = ["ls-rs", "-S"];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert!(config.sort_type.is_some());
    assert!(matches!(config.sort_type.unwrap(), SortType::Size));
}

#[test]
fn flag_sort_type_value_wrong_test() {
    let args = ["ls-rs", "--sort", "wrong"];
    let matches = command::ls_command().get_matches_from(args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert!(config.sort_type.is_none());
}
