use ls_rs::command;

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
    let matches = command::ls_command().get_matches_from(&args);
    let config = command::Config::clap_parse(&matches);

    assert!(config.long);
    assert!(config.all);

    assert_eq!(config.cols, None);
}

#[test]
fn flag_full_in_one_test() {
    let args = ["ls-rs", "-alnHrNTSRoijJ"];
    let matches = command::ls_command().get_matches_from(&args);
    let config = command::Config::clap_parse(&matches);

    assert!(config.all);
    assert!(config.long);
    assert!(config.numeric);
    assert!(config.humanable);
    assert!(config.reverse);
    assert!(config.name_sort);
    assert!(config.time_sort);
    assert!(config.size_sort);
    assert!(config.recursive);
    assert!(config.one_col);
    assert!(config.inode);
    assert!(config.json_mini);
    assert!(config.json_big);

    assert_eq!(config.cols, None);
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
        "--sort",
        "--time",
        "--size",
        "--recursive",
        "--one",
        "--inode",
        "--json",
        "--JSON",
    ];
    let matches = command::ls_command().get_matches_from(&args);
    let config = command::Config::clap_parse(&matches);

    assert!(config.all);
    assert!(config.long);
    assert!(config.numeric);
    assert!(config.humanable);
    assert!(config.reverse);
    assert!(config.name_sort);
    assert!(config.time_sort);
    assert!(config.size_sort);
    assert!(config.recursive);
    assert!(config.one_col);
    assert!(config.inode);
    assert!(config.json_mini);
    assert!(config.json_big);

    assert_eq!(config.cols, None);
}

#[test]
fn flag_full_short_test() {
    let args = [
        "ls-rs", "-a", "-l", "-n", "-H", "-r", "-N", "-T", "-S", "-R", "-o", "-i", "-j", "-J",
    ];
    let matches = command::ls_command().get_matches_from(&args);
    let config = command::Config::clap_parse(&matches);

    assert!(config.all);
    assert!(config.long);
    assert!(config.numeric);
    assert!(config.humanable);
    assert!(config.reverse);
    assert!(config.name_sort);
    assert!(config.time_sort);
    assert!(config.size_sort);
    assert!(config.recursive);
    assert!(config.one_col);
    assert!(config.inode);
    assert!(config.json_mini);
    assert!(config.json_big);

    assert_eq!(config.cols, None);
}

#[test]
fn flag_valuable_test() {
    const PATH: &str = "sldkjfskdfj";
    const COLS: usize = 10;

    let args = ["ls-rs", "-C", &COLS.to_string(), PATH];
    let matches = command::ls_command().get_matches_from(&args);
    let config = command::Config::clap_parse(&matches);

    assert!(!config.all);
    assert!(!config.long);
    assert!(!config.numeric);
    assert!(!config.humanable);
    assert!(!config.reverse);
    assert!(!config.name_sort);
    assert!(!config.time_sort);
    assert!(!config.size_sort);
    assert!(!config.recursive);
    assert!(!config.one_col);
    assert!(!config.inode);
    assert!(!config.json_mini);
    assert!(!config.json_big);

    assert_eq!(config.path, PATH);
    assert_eq!(config.cols, Some(COLS));
}
