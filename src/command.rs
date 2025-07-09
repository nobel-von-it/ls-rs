use clap::{Arg, ArgAction, ArgMatches, Command};

fn arg_base(name: &'static str, req: bool, about: &'static str) -> Arg {
    Arg::new(name).required(req).help(about)
}

fn arg_flag(name: &'static str, req: bool, about: &'static str) -> Arg {
    arg_base(name, req, about)
        .short(name.chars().next().unwrap())
        .long(name)
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
        .arg(arg_flag("sort", false, "Sort by"))
        .arg(arg_flag("depth", false, "Depth of listing"))
        .arg(arg_bool("all", false, "Show hidden files", false))
        .arg(arg_bool("long", false, "Long format", false))
        .arg(arg_bool("reverse", false, "Reverse order", false))
}

#[derive(Debug)]
pub struct Config {
    pub path: String,
    pub sort: Option<String>,
    pub all: bool,
    pub long: bool,
    pub reverse: bool,
    pub depth: Option<u32>,
}

impl Config {
    pub fn clap_parse(matches: &ArgMatches) -> Self {
        Self {
            path: matches
                .get_one::<String>("path")
                .unwrap_or(&".".to_string())
                .to_string(),
            sort: matches.get_one::<String>("sort").map(|s| s.to_string()),
            all: *matches.get_one("all").unwrap(),
            long: *matches.get_one("long").unwrap(),
            reverse: *matches.get_one("reverse").unwrap(),
            depth: matches
                .get_one::<String>("depth")
                .map(|s| s.parse().unwrap()),
        }
    }
}
