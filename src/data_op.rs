use crate::{
    command::{Config, RecursionOptions, SortType},
    files::FileSystemEntry,
    json::Serializer,
    term,
};

#[derive(Debug, Clone)]
pub struct DataProcessor {
    entries: Vec<FileSystemEntry>,
    config: Config,
}
impl PartialEq for DataProcessor {
    fn eq(&self, other: &Self) -> bool {
        self.entries
            .iter()
            .zip(other.entries.iter())
            .all(|(fse1, fse2)| fse1.get_styled_name().eq(&fse2.get_styled_name()))
    }
}

impl DataProcessor {
    pub fn new(entries: Vec<FileSystemEntry>, config: Config) -> Self {
        Self { entries, config }
    }

    pub fn data_len(&self) -> usize {
        self.entries.len()
    }

    pub fn get_entries(&self) -> &[FileSystemEntry] {
        self.entries.as_slice()
    }

    pub fn filter(mut self) -> Self {
        self.entries
            .retain(|fse| self.config.all || !fse.is_hidden());
        self.entries.retain(|fse| {
            self.config.ignore.is_none()
                || !self.config.ignore.as_ref().unwrap().contains(fse.name())
        });

        self
    }

    pub fn sort(mut self) -> Self {
        if let Some(sort_type) = self.config.sort_type.as_ref() {
            match sort_type {
                SortType::Time => self
                    .entries
                    .sort_by_key(|fse| fse.metadata().modified_at.clone()),
                SortType::Size => self.entries.sort_by_key(|fse| fse.metadata().size),
                SortType::Name => self.entries.sort_by_key(|fse| fse.cname()),
            }
        }
        if self.config.reverse {
            self.entries.reverse();
        }
        self
    }

    pub fn prepare(self) -> PreparedData {
        PreparedData::new(self.entries, self.config)
    }
}

pub struct PreparedData {
    names: Vec<String>,
}

impl PreparedData {
    pub fn new(entries: Vec<FileSystemEntry>, config: Config) -> Self {
        let names = if config.long {
            Self::prepare_long(&entries, &config)
        } else {
            Self::prepare_short(&entries, &config)
        };

        let names = if config.numeric {
            Self::add_numbers(names)
        } else {
            names
        };
        Self { names }
    }
    pub fn get_names(&self) -> &[String] {
        self.names.as_slice()
    }

    fn prepare_short(entries: &[FileSystemEntry], _config: &Config) -> Vec<String> {
        entries.iter().map(|fse| fse.to_string_short()).collect()
    }

    fn prepare_long(entries: &[FileSystemEntry], config: &Config) -> Vec<String> {
        let max_time = entries
            .iter()
            .map(|fse| fse.metadata().modified_at.format().to_string().len())
            .max()
            .unwrap_or(0);

        let max_size = entries
            .iter()
            .map(|fse| {
                if config.humanable {
                    fse.metadata().human_size.len()
                } else {
                    fse.metadata().size.to_string().len()
                }
            })
            .max()
            .unwrap_or(0);

        entries
            .iter()
            .map(|fse| {
                #[cfg(unix)]
                return fse.to_string_long(config.humanable, config.inode, max_size, max_time);
                #[cfg(windows)]
                return fse.to_string_long(config.humanable, max_size, max_time);
            })
            .collect()
    }
    fn add_numbers(names: Vec<String>) -> Vec<String> {
        names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{}. {}", i + 1, n))
            .collect()
    }
}

pub trait OutputFormatter {
    fn format(&self) -> String;
}

pub struct TextFormatter {
    names: Vec<String>,
    long: bool,
    cols: Option<usize>,
}
impl OutputFormatter for TextFormatter {
    fn format(&self) -> String {
        if self.long {
            self.format_long()
        } else {
            self.format_short()
        }
    }
}
impl TextFormatter {
    fn new(names: Vec<String>, long: bool, cols: Option<usize>) -> Self {
        Self { names, long, cols }
    }
    fn format_with_cols(&self, cols: usize) -> String {
        if self.names.is_empty() {
            return String::new();
        }

        if self.names.len() <= cols {
            return self.names.join(" ");
        }

        let max_width = self.names.iter().map(|n| n.len()).max().unwrap_or(1);
        let col_width = max_width + 2;
        let total_items = self.names.len();
        let rows = total_items.div_ceil(cols);

        let mut output = String::new();
        for row in 0..rows {
            let mut line = String::new();

            for col in 0..cols {
                let idx = col * rows + row;
                if idx < total_items {
                    let name = &self.names[idx];
                    let temp_col_width = col_width - if name.starts_with("\x1b") { 0 } else { 9 };
                    line.push_str(&format!("{name:<temp_col_width$}"));
                }
            }

            output.push_str(line.trim_end());
            output.push('\n');
        }

        output.trim_end().to_string()
    }
    fn format_with_terminal_width(&self) -> String {
        if self.names.is_empty() {
            return String::new();
        }

        let (term_cols, _) = term::terminal_size().unwrap_or((80, 24));
        let term_cols = term_cols as usize;

        let total_width = self.names.iter().map(|n| n.len()).sum::<usize>() + self.names.len() - 1;
        if total_width <= term_cols {
            return self.names.join(" ");
        }

        let max_width = self.names.iter().map(|n| n.len()).max().unwrap_or(1);
        let col_width = max_width + 2;
        let max_cols = (term_cols / col_width).max(1);
        let total_items = self.names.len();
        let rows = total_items.div_ceil(max_cols);

        let mut output = String::new();
        for row in 0..rows {
            let mut line = String::new();

            for col in 0..max_cols {
                let idx = col * rows + row;
                if idx < total_items {
                    let name = &self.names[idx];
                    line.push_str(&format!("{name:<col_width$}"));
                }
            }

            output.push_str(line.trim_end());
            output.push('\n');
        }

        output.trim_end().to_string()
    }
    fn format_short(&self) -> String {
        if let Some(cols) = self.cols {
            self.format_with_cols(cols)
        } else {
            self.format_with_terminal_width()
        }
    }
    fn format_long(&self) -> String {
        self.names.join("\n")
    }
}

pub struct JsonFormatter {
    // for serializing
    entry: FileSystemEntry,
    mini: bool,
}
impl JsonFormatter {
    pub fn new(entry: FileSystemEntry, mini: bool) -> Self {
        Self { entry, mini }
    }
}
impl OutputFormatter for JsonFormatter {
    fn format(&self) -> String {
        if self.mini {
            self.entry.short_json()
        } else {
            self.entry.long_json()
        }
    }
}

pub struct RecursiveFormatter {
    entry: FileSystemEntry,
    depth: usize,
    max_depth: Option<usize>,
    ignore: Option<String>,
}
impl OutputFormatter for RecursiveFormatter {
    fn format(&self) -> String {
        self.format_recursive(&self.entry, self.depth)
    }
}

impl RecursiveFormatter {
    fn new(entry: FileSystemEntry, config: &Config) -> Self {
        Self {
            entry,
            depth: 0,
            max_depth: match config.recursive.as_ref().unwrap() {
                RecursionOptions::Depth(depth) => Some(*depth),
                RecursionOptions::Unlimited => None,
                RecursionOptions::No => Some(0),
            },
            ignore: config.ignore.clone(),
        }
    }
    fn format_recursive(&self, entry: &FileSystemEntry, current_depth: usize) -> String {
        if let Some(ignore) = self.ignore.as_ref() {
            if ignore.contains(entry.name()) {
                return String::new();
            }
        }
        let mut output = String::new();
        let indent = "  ".repeat(current_depth);

        output.push_str(&format!("{}{}\n", indent, entry.get_styled_name()));

        if entry.is_dir() {
            let should_expand = if let Some(max_depth) = self.max_depth {
                current_depth < max_depth
            } else {
                true
            };

            if should_expand {
                if let Some(dir_entries) = entry.get_dir_entries() {
                    for fse in dir_entries {
                        output.push_str(&self.format_recursive(&fse, current_depth + 1));
                    }
                }
            }
        }

        output
    }
}

pub struct Printer {
    formatter: Box<dyn OutputFormatter>,
}

impl Printer {
    pub fn new(start_dir: FileSystemEntry, config: Config) -> Self {
        let formatter: Box<dyn OutputFormatter> = match (
            config.json_mini,
            config.json_big,
            config.recursive.is_some(),
        ) {
            (true, _, _) => Box::new(JsonFormatter::new(start_dir, true)),
            (_, true, _) => Box::new(JsonFormatter::new(start_dir, false)),
            (_, _, true) => Box::new(RecursiveFormatter::new(start_dir, &config)),
            _ => {
                let long = config.long;
                let cols = config.cols;

                let processor =
                    DataProcessor::new(start_dir.get_dir_entries().unwrap_or_default(), config);

                let prepared_data = processor.filter().sort().prepare();
                Box::new(TextFormatter::new(prepared_data.names, long, cols))
            }
        };

        Self { formatter }
    }
    pub fn print(&self) {
        println!("{}", self.formatter.format())
    }
}
