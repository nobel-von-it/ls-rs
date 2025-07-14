use terminal_size::{Height, Width, terminal_size};

use crate::{
    command::Config,
    files::{FileSystemEntry, LongFSEString},
    json::Serializer,
};

pub struct Printer {
    config: Config,
    start_dir: FileSystemEntry,
    fses: Vec<FileSystemEntry>,
    names: Vec<String>,
}

impl Printer {
    pub fn new(config: Config, start_dir: FileSystemEntry) -> Self {
        let fses = match start_dir.clone() {
            FileSystemEntry::Directory { entries, .. } => entries,
            _ => vec![],
        };
        Self {
            config,
            start_dir,
            fses,
            names: vec![],
        }
    }
    pub fn json_checker(&mut self) -> Option<&mut Self> {
        if self.config.json_mini || self.config.json_big {
            self.json_finalizer();
            return None;
        }
        Some(self)
    }
    fn json_finalizer(&self) {
        if self.config.json_mini {
            println!("{}", self.start_dir.short_json())
        } else if self.config.json_big {
            println!("{}", self.start_dir.long_json())
        }
    }
    pub fn filter(&mut self) -> &mut Self {
        self.fses = self
            .fses
            .iter()
            .filter(|fse| self.config.all || !fse.is_hidden())
            .cloned()
            .collect();
        self
    }
    pub fn sort(&mut self) -> &mut Self {
        // if provided, sort by time first and then by size and then by name
        if self.config.time_sort {
            self.fses.sort_by_key(|fse| fse.metadata().modified_at);
        }
        if self.config.size_sort {
            self.fses.sort_by_key(|fse| fse.metadata().size);
        }
        if self.config.name_sort {
            self.fses.sort_by_key(|fse| fse.to_string_short());
        }

        if self.config.reverse {
            self.fses.reverse();
        }

        self
    }
    pub fn prepare_short(&mut self) -> &mut Self {
        self.fses.iter().for_each(|fse| {
            self.names.push(fse.to_string_short());
        });

        self
    }
    pub fn prepare_long(&mut self) -> &mut Self {
        let max_time = self
            .fses
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
        let max_size = self
            .fses
            .iter()
            .map(|fse| {
                if self.config.humanable {
                    fse.metadata().human_size.len()
                } else {
                    fse.metadata().size.to_string().len()
                }
            })
            .max()
            .unwrap_or(0);

        self.fses.iter().for_each(|fse| {
            self.names.push(fse.to_string_long(
                self.config.humanable,
                self.config.inode,
                max_size,
                max_time,
            ))
        });

        self
    }
    pub fn prepare(&mut self) -> &mut Self {
        if self.config.long {
            self.prepare_long();
        } else {
            self.prepare_short();
        }
        if self.config.numeric {
            self.add_numbers_to_names();
        }
        self
    }
    pub fn add_numbers_to_names(&mut self) -> &mut Self {
        self.names = self
            .names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{}. {}", i + 1, n))
            .collect();
        self
    }
    pub fn get_config_cols_value(&self) -> Option<usize> {
        if let Some(cols) = self.config.cols {
            Some(cols)
        } else if self.config.one_col {
            Some(1)
        } else {
            None
        }
    }
    pub fn print_advance_short_by(&self) {
        if let Some(cols) = self.get_config_cols_value() {
            self.print_advance_short_by_config_cols(cols);
        } else {
            self.print_advance_short_by_terminal_width();
        }
    }
    pub fn print_advance_short_by_config_cols(&self, cols: usize) {
        if self.names.is_empty() {
            return;
        }

        if self.names.len() <= cols {
            println!("{}", self.names.join(" "));
            return;
        }

        let max_width = self.names.iter().map(|n| n.len()).max().unwrap_or(0);

        let col_width = max_width + 2;
        let total_items = self.names.len();
        let rows = (total_items + cols - 1) / cols;

        for row in 0..rows {
            let mut line = String::new();

            for col in 0..cols {
                let idx = col * rows + row;
                if idx < total_items {
                    let name = &self.names[idx];

                    line.push_str(&format!("{name:<col_width$}"));
                }
            }

            println!("{}", line.trim_end());
        }
    }
    pub fn print_advance_short_by_terminal_width(&self) {
        if self.names.is_empty() {
            return;
        }

        let (Width(term_cols), _) = terminal_size().unwrap_or((Width(80), Height(24)));
        let term_cols = term_cols as usize;

        let total_width = self.names.iter().map(|n| n.len()).sum::<usize>() + self.names.len() - 1;
        if total_width <= term_cols {
            println!("{}", self.names.join(" "));
            return;
        }

        let max_width = self.names.iter().map(|n| n.len()).max().unwrap_or(0);
        let col_width = max_width + 2;

        let max_cols = (term_cols / col_width).max(1);
        let rows = (self.names.len() + max_cols - 1) / max_cols;

        for row in 0..rows {
            let mut line = String::new();

            for col in 0..max_cols {
                let idx = col * rows + row;
                if let Some(name) = self.names.get(idx) {
                    line.push_str(&format!("{name:<col_width$}"));
                }
            }

            println!("{}", line.trim_end());
        }
    }
    pub fn print_long(&self) {
        self.names.iter().for_each(|n| println!("{}", n));
    }
    pub fn print(&self) {
        if self.config.long {
            self.print_long();
        } else {
            self.print_advance_short_by();
        }
        // self.fses.iter().for_each(|fse| {
        //     if self.config.long {
        //         fse.to_string_long()
        //     } else {
        //         fse.to_string_short()
        //     }
        // });
    }
    // pub fn map(&mut self) -> &mut Self {
    //     self.fses.iter().map(|fse| {
    //         if self.config.long {
    //             fse.to_string_long()
    //         } else {
    //             fse.to_string_short()
    //         }
    //     })
    // }
}
