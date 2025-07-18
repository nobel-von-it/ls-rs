use std::{fs, io};

use ls_rs::{command, data_op::Printer, files::FileSystemEntry};

fn main() -> io::Result<()> {
    let matches = command::ls_command().get_matches();
    let config = command::Config::clap_parse(&matches);

    let mut start_dir = if let Some(dir) = FileSystemEntry::from_path(&config.path) {
        dir
    } else {
        println!("Path does not exist");
        return Ok(());
    };

    for entry in fs::read_dir(&config.path)? {
        if let Ok(entry) = entry {
            if let Some(entry) = FileSystemEntry::from_dir_entry(entry) {
                start_dir.push_to_dir(entry);
            }
        }
    }

    let printer = Printer::new(start_dir, config);
    printer.print();
    Ok(())
}
