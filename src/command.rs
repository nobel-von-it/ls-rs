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
        .arg(arg_flag_t(
            "ignore",
            false,
            "Comma separated ignore list",
            'I',
        ))
        .arg(arg_bool("all", false, "Show hidden files", false))
        .arg(arg_bool("long", false, "Long format", false))
        .arg(arg_bool("numeric", false, "Numbers in left", false))
        .arg(arg_bool_t("humanable", false, "Human readable", 'H', false))
        .arg(arg_base("sort", false, "Sort by value").long("sort"))
        .arg(arg_bool("reverse", false, "Reverse order", false))
        .arg(arg_bool_t("name", false, "Sort by name", 'N', false))
        .arg(arg_bool_t("time", false, "Sort by time", 'T', false))
        .arg(arg_bool_t("size", false, "Sort by size", 'S', false))
        // .arg(arg_bool_t("ext", false, "Sort by extension", 'X', false))
        .arg(arg_flag_t("recursive", false, "Recursive", 'R'))
        .arg(arg_bool("one", false, "One line input", false))
        .arg(arg_bool("inode", false, "Add inode info to output", false))
        .arg(arg_bool_t("json", false, "Short json output", 'j', false))
        .arg(arg_bool_t(
            "JSON",
            false,
            "Long (full) json output",
            'J',
            false,
        ))
}

#[derive(Debug, Clone)]
pub struct Config {
    pub path: String,
    pub cols: Option<usize>,
    // comma separated
    pub ignore: Option<String>,
    pub all: bool,
    pub long: bool,
    pub numeric: bool,
    pub humanable: bool,
    pub reverse: bool,
    pub sort_type: Option<SortType>,
    // pub ext_sort: bool,
    pub recursive: Option<RecursionOptions>,
    pub one_col: bool,
    pub inode: bool,
    pub json_mini: bool,
    pub json_big: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecursionOptions {
    Depth(usize),
    Unlimited,
    No,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortType {
    Name,
    Size,
    Time,
}

impl Config {
    pub fn clap_parse(matches: &ArgMatches) -> Self {
        let sort_type = if *matches.get_one("name").unwrap() {
            Some(SortType::Name)
        } else if *matches.get_one("size").unwrap() {
            Some(SortType::Size)
        } else if *matches.get_one("time").unwrap() {
            Some(SortType::Time)
        } else {
            let sort = matches.get_one::<String>("sort");
            if let Some(sort) = sort {
                match sort.as_str() {
                    "name" => Some(SortType::Name),
                    "size" => Some(SortType::Size),
                    "time" => Some(SortType::Time),
                    _ => None,
                }
            } else {
                None
            }
        };

        Self {
            path: matches
                .get_one::<String>("path")
                .unwrap_or(&".".to_string())
                .to_string(),
            cols: matches
                .get_one::<String>("cols")
                .map(|s| s.parse().unwrap_or(0)),
            ignore: matches.get_one::<String>("ignore").cloned(),
            all: *matches.get_one("all").unwrap(),
            long: *matches.get_one("long").unwrap(),
            numeric: *matches.get_one("numeric").unwrap(),
            humanable: *matches.get_one("humanable").unwrap(),
            reverse: *matches.get_one("reverse").unwrap(),
            sort_type,
            // ext_sort: *matches.get_one("ext").unwrap(),
            recursive: matches.get_one::<String>("recursive").map(|depth| {
                let depth = depth.to_lowercase();
                if depth == "max" || depth.contains("unlim") {
                    RecursionOptions::Unlimited
                } else if let Ok(depth) = depth.parse() {
                    RecursionOptions::Depth(depth)
                } else {
                    RecursionOptions::No
                }
            }),
            one_col: *matches.get_one("one").unwrap(),
            inode: *matches.get_one("inode").unwrap(),
            json_mini: *matches.get_one("json").unwrap(),
            json_big: *matches.get_one("JSON").unwrap(),
        }
    }
}
