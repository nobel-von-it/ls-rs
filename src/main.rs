use ls_rs::{command, data_op::Printer, error::LsResult, files::FileSystemEntry};

fn main() -> LsResult<()> {
    let matches = command::ls_command().get_matches();
    let config = command::Config::clap_parse(&matches);

    let start_dir = match FileSystemEntry::new_with_config(&config) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("{e}");
            return Err(e);
        }
    };

    let printer = Printer::new(start_dir, config);
    printer.print();
    Ok(())
}
