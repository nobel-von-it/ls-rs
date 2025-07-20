use crate::{command::Config, files::FileSystemEntry, json::Serializer, term};

pub struct DataProcessor {
    entries: Vec<FileSystemEntry>,
    config: Config,
}

impl DataProcessor {
    pub fn new(entries: Vec<FileSystemEntry>, config: Config) -> Self {
        Self { entries, config }
    }

    pub fn data_len(&self) -> usize {
        self.entries.len()
    }

    pub fn filter(mut self) -> Self {
        self.entries
            .retain(|fse| self.config.all || !fse.is_hidden());
        self
    }

    pub fn sort(mut self) -> Self {
        // if provided, sort by time first and then by size and then by name
        if self.config.time_sort {
            self.entries.sort_by_key(|fse| fse.metadata().modified_at);
        }
        if self.config.size_sort {
            self.entries.sort_by_key(|fse| fse.metadata().size);
        }
        if self.config.name_sort {
            self.entries.sort_by_key(|fse| fse.to_string_short());
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

    fn prepare_short(entries: &[FileSystemEntry], _config: &Config) -> Vec<String> {
        entries.iter().map(|fse| fse.to_string_short()).collect()
    }

    fn prepare_long(entries: &[FileSystemEntry], config: &Config) -> Vec<String> {
        let max_time = entries
            .iter()
            .map(|fse| {
                fse.metadata()
                    .modified_at
                    .format("%b %e %R")
                    .to_string()
                    .len()
            })
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

pub struct Printer {
    formatter: Box<dyn OutputFormatter>,
}

impl Printer {
    pub fn new(start_dir: FileSystemEntry, config: Config) -> Self {
        let formatter: Box<dyn OutputFormatter> = match (config.json_mini, config.json_big) {
            (true, _) => Box::new(JsonFormatter::new(start_dir, true)),
            (_, true) => Box::new(JsonFormatter::new(start_dir, false)),
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
