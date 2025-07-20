use std::io;

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

    start_dir.fill_start_dir();

    let printer = Printer::new(start_dir, config);
    printer.print();
    Ok(())
}
