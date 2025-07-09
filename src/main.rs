use std::{
    fs::{self, DirEntry},
    io,
    os::unix::fs::MetadataExt,
    path::PathBuf,
};

use chrono::{DateTime, Local};
use ls::command;

#[derive(Debug, Clone)]
struct MetaData {
    size: u64,
    mode: u32,
    mode_str: String,

    created_at: DateTime<Local>,
    modified_at: DateTime<Local>,
}

#[derive(Debug)]
enum FileSystemEntry {
    File {
        name: String,
        path: PathBuf,
        metadata: MetaData,
    },
    Directory {
        name: String,
        path: PathBuf,
        metadata: MetaData,
        entries: Vec<FileSystemEntry>,
    },
    Link {
        name: String,
        path: PathBuf,
        metadata: MetaData,
        target: PathBuf,
    },
}

impl FileSystemEntry {
    fn push_to_dir(&mut self, entry: FileSystemEntry) {
        if let FileSystemEntry::Directory { entries, .. } = self {
            entries.push(entry);
        }
    }
    fn from_dir_entry(entry: DirEntry) -> Option<Self> {
        None
    }
}

fn main() -> io::Result<()> {
    let matches = command::ls_command().get_matches();
    let config = command::Config::clap_parse(&matches);

    let path = PathBuf::from(&config.path);
    let metadata = fs::metadata(&path)?;

    let mut start_dir = FileSystemEntry::Directory {
        name: path.file_name().unwrap().to_string_lossy().to_string(),
        path,
        metadata: MetaData {
            size: metadata.len(),
            mode: metadata.mode(),
            mode_str: metadata.mode().to_string(),
            created_at: DateTime::from(metadata.created().unwrap()),
            modified_at: DateTime::from(metadata.modified().unwrap()),
        },
        entries: vec![],
    };

    for entry in fs::read_dir(&config.path)? {
        if let Ok(entry) = entry {
            if let Some(entry) = FileSystemEntry::from_dir_entry(entry) {
                start_dir.push_to_dir(entry);
            }
        }
    }

    println!("{:?}", config);
    Ok(())
}
