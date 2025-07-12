use clap::{Arg, ArgAction, ArgMatches, Command};

fn arg_base(name: &'static str, req: bool, about: &'static str) -> Arg {
    Arg::new(name).required(req).help(about)
}

fn arg_flag_t(name: &'static str, req: bool, about: &'static str, short: char) -> Arg {
    arg_base(name, req, about).short(short).long(name)
}

fn arg_flag(name: &'static str, req: bool, about: &'static str) -> Arg {
    arg_base(name, req, about)
        .short(name.chars().next().unwrap())
        .long(name)
}

fn arg_bool_t(name: &'static str, req: bool, about: &'static str, short: char, value: bool) -> Arg {
    arg_flag_t(name, req, about, short).action(if !value {
        ArgAction::SetTrue
    } else {
        ArgAction::SetFalse
    })
}

fn arg_bool(name: &'static str, req: bool, about: &'static str, value: bool) -> Arg {
    arg_flag(name, req, about).action(if !value {
        ArgAction::SetTrue
    } else {
        ArgAction::SetFalse
    })
}

fn arg_str(name: &'static str, req: bool, about: &'static str) -> Arg {
    arg_base(name, req, about)
}

pub fn ls_command() -> Command {
    Command::new("fls")
        .about("Fast list files")
        .arg(arg_str("path", false, "Path to list"))
        .arg(arg_flag_t("cols", false, "Number of columns", 'C'))
        .arg(arg_bool("all", false, "Show hidden files", false))
        .arg(arg_bool("long", false, "Long format", false))
        .arg(arg_bool("numeric", false, "Numbers in left", false))
        .arg(arg_bool_t("humanable", false, "Human readable", 'H', false))
        .arg(arg_bool("reverse", false, "Reverse order", false))
        .arg(arg_bool_t("sort", false, "Sort by name", 'N', false))
        .arg(arg_bool_t("time", false, "Sort by time", 'T', false))
        .arg(arg_bool_t("size", false, "Sort by size", 'S', false))
        // .arg(arg_bool_t("ext", false, "Sort by extension", 'X', false))
        .arg(arg_bool_t("recursive", false, "Recursive", 'R', false))
        .arg(arg_bool("one", false, "One line input", false))
        .arg(arg_bool("inode", false, "Add inode info to output", false))
}

#[derive(Debug)]
pub struct Config {
    pub path: String,
    pub cols: Option<usize>,
    pub all: bool,
    pub long: bool,
    pub numeric: bool,
    pub humanable: bool,
    pub reverse: bool,
    pub name_sort: bool,
    pub time_sort: bool,
    pub size_sort: bool,
    // pub ext_sort: bool,
    pub recursive: bool,
    pub one_col: bool,
    pub inode: bool,
}

impl Config {
    pub fn clap_parse(matches: &ArgMatches) -> Self {
        Self {
            path: matches
                .get_one::<String>("path")
                .unwrap_or(&".".to_string())
                .to_string(),
            cols: matches
                .get_one::<String>("cols")
                .map(|s| s.parse().unwrap_or(0)),
            all: *matches.get_one("all").unwrap(),
            long: *matches.get_one("long").unwrap(),
            numeric: *matches.get_one("numeric").unwrap(),
            humanable: *matches.get_one("humanable").unwrap(),
            reverse: *matches.get_one("reverse").unwrap(),
            name_sort: *matches.get_one("sort").unwrap(),
            time_sort: *matches.get_one("time").unwrap(),
            size_sort: *matches.get_one("size").unwrap(),
            // ext_sort: *matches.get_one("ext").unwrap(),
            recursive: *matches.get_one("recursive").unwrap(),
            one_col: *matches.get_one("one").unwrap(),
            inode: *matches.get_one("inode").unwrap(),
        }
    }
}
