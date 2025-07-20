use std::io;

use ls_rs::{command, data_op::Printer, files::FileSystemEntry};

fn main() -> io::Result<()> {
    let matches = command::ls_command().get_matches();
    let config = command::Config::clap_parse(&matches);

    let start_dir = if let Some(dir) = FileSystemEntry::new_with_config(&config) {
        dir
    } else {
        println!("Path does not exist");
        return Ok(());
    };
    // println!("{start_dir:#?}");

    let printer = Printer::new(start_dir, config);
    printer.print();
    Ok(())
}
