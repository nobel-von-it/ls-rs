use terminal_size::{Height, Width, terminal_size};

use crate::{command::Config, files::FileSystemEntry};

pub struct Printer {
    config: Config,
    fses: Vec<FileSystemEntry>,
    names: Vec<String>,
}

impl Printer {
    pub fn new(config: Config, fses: FileSystemEntry) -> Self {
        let fses = match fses {
            FileSystemEntry::Directory { entries, .. } => entries,
            _ => vec![],
        };
        Self {
            config,
            fses,
            names: vec![],
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
        // if provided both time and size sort, sort by time first and then by size
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
    pub fn prepare(&mut self) -> &mut Self {
        self.fses.iter().for_each(|fse| {
            self.names.push(fse.to_string_short());
        });
        self
    }
    pub fn print_advance_short_by_config_cols(&self) {
        let cols = if let Some(cols) = self.config.cols {
            cols
        } else {
            return;
        };
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
    pub fn print(&self) {
        if self.config.cols.is_some() {
            self.print_advance_short_by_config_cols();
        } else {
            self.print_advance_short_by_terminal_width();
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
