use std::{
    env,
    fs::{self, DirEntry, File, Metadata},
    io,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
pub enum FileColor {
    // for links with no-exist target
    Red,
    // for normal links
    Aqua,
    // for directories
    Blue,
    // TODO: for files by file extension
    Other,
}

#[derive(Debug, Clone)]
pub struct MetaData {
    pub size: u64,
    pub human_size: String,

    pub inode: u64,

    pub mode: u32,
    pub mode_str: String,

    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
}

impl MetaData {
    fn try_from(metadata: &Metadata) -> Option<Self> {
        Some(MetaData {
            size: metadata.len(),
            human_size: get_human_readable_size(metadata.len()),
            inode: metadata.ino(),
            mode: metadata.mode(),
            mode_str: get_file_mode_formated(&metadata).ok()?,
            created_at: DateTime::from(metadata.created().ok()?),
            modified_at: DateTime::from(metadata.modified().ok()?),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FileStyle {
    pub suffix: char,
    pub color: FileColor,
}

#[derive(Debug, Clone)]
pub struct BaseInfo {
    pub name: String,
    pub style: Option<FileStyle>,

    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum FileSystemEntry {
    File {
        base_info: BaseInfo,
        metadata: MetaData,
        extension: Option<String>,
    },
    Directory {
        base_info: BaseInfo,
        metadata: MetaData,
        entries: Vec<FileSystemEntry>,
    },
    Link {
        base_info: BaseInfo,
        metadata: MetaData,
        target: PathBuf,
    },
}

fn get_file_mode_formated(md: &Metadata) -> io::Result<String> {
    let perm = md.permissions();
    let mode = perm.mode();

    let mut builder = String::with_capacity(10);

    builder.push(match md.file_type() {
        ft if ft.is_dir() => 'd',
        ft if ft.is_file() => '-',
        ft if ft.is_symlink() => 'l',
        _ => '?',
    });

    // User permissions
    builder.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    builder.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    builder.push(if mode & 0o100 != 0 {
        if mode & 0o4000 != 0 { 's' } else { 'x' }
    } else {
        if mode & 0o4000 != 0 { 'S' } else { '-' }
    });

    // Group permissions
    builder.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    builder.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    builder.push(if mode & 0o010 != 0 {
        if mode & 0o2000 != 0 { 's' } else { 'x' }
    } else {
        if mode & 0o2000 != 0 { 'S' } else { '-' }
    });

    // Other permissions
    builder.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    builder.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    builder.push(if mode & 0o001 != 0 {
        if mode & 0o1000 != 0 { 't' } else { 'x' }
    } else {
        if mode & 0o1000 != 0 { 'T' } else { '-' }
    });

    Ok(builder)
}

fn get_human_readable_size(size: u64) -> String {
    let mut size = size as f64;
    let mut suffix = "B";
    if size > 1024. {
        size /= 1024.;
        suffix = "K";
    }
    if size > 1024. {
        size /= 1024.;
        suffix = "M";
    }
    if size > 1024. {
        size /= 1024.;
        suffix = "G";
    }

    let rounded = (size * 100.).round() / 100.;

    if rounded.fract() == 0. {
        format!("{}{}", rounded as i64, suffix)
    } else if rounded.fract() * 10. % 1. == 0. {
        format!("{:.1}{}", rounded, suffix)
    } else {
        format!("{:.2}{}", rounded, suffix)
    }
}

impl FileSystemEntry {
    pub fn new_from_values(
        name: String,
        path: PathBuf,
        style: Option<FileStyle>,
        metadata: Metadata,
    ) -> Option<Self> {
        let meta_data = MetaData::try_from(&metadata)?;

        if metadata.is_file() {
            Some(FileSystemEntry::File {
                extension: path
                    .extension()
                    .and_then(|s| s.to_str().map(|s| s.to_string())),
                base_info: BaseInfo { name, style, path },
                metadata: meta_data,
            })
        } else if metadata.is_dir() {
            Some(FileSystemEntry::Directory {
                base_info: BaseInfo { name, style, path },
                metadata: meta_data,
                entries: vec![],
            })
        } else if metadata.is_symlink() {
            let target = fs::read_link(&path).ok()?;
            Some(FileSystemEntry::Link {
                base_info: BaseInfo { name, style, path },
                metadata: meta_data,
                target,
            })
        } else {
            None
        }
    }
    pub fn is_hidden(&self) -> bool {
        match self {
            FileSystemEntry::File { base_info, .. } => base_info.name.starts_with("."),
            FileSystemEntry::Directory { base_info, .. } => base_info.name.starts_with("."),
            FileSystemEntry::Link { base_info, .. } => base_info.name.starts_with("."),
        }
    }
    pub fn metadata(&self) -> &MetaData {
        match self {
            FileSystemEntry::File { metadata, .. } => metadata,
            FileSystemEntry::Directory { metadata, .. } => metadata,
            FileSystemEntry::Link { metadata, .. } => metadata,
        }
    }
    pub fn base_info(&self) -> &BaseInfo {
        match self {
            FileSystemEntry::File { base_info, .. } => base_info,
            FileSystemEntry::Directory { base_info, .. } => base_info,
            FileSystemEntry::Link { base_info, .. } => base_info,
        }
    }
    pub fn push_to_dir(&mut self, entry: FileSystemEntry) {
        if let FileSystemEntry::Directory { entries, .. } = self {
            entries.push(entry);
        }
    }
    pub fn from_path<S: AsRef<str>>(path: S) -> Option<Self> {
        let path = if path.as_ref() == "." {
            env::current_dir().ok()?
        } else {
            PathBuf::from(path.as_ref())
        };
        let metadata = fs::metadata(&path).ok()?;

        let name = path.file_name()?.to_string_lossy().to_string();

        FileSystemEntry::new_from_values(name, path, None, metadata)
    }
    pub fn from_dir_entry(entry: DirEntry) -> Option<Self> {
        let path = entry.path();
        let metadata = entry.metadata().ok()?;

        let name = entry.file_name().to_string_lossy().to_string();
        FileSystemEntry::new_from_values(name, path, None, metadata)
    }
    pub fn to_string_short(&self) -> String {
        match self {
            FileSystemEntry::File { base_info, .. } => base_info.name.clone(),
            FileSystemEntry::Directory { base_info, .. } => format!("{}/", base_info.name),
            FileSystemEntry::Link { base_info, .. } => format!("{}@", base_info.name),
        }
    }

    fn get_name_and_metadata(&self) -> (&str, &MetaData) {
        match self {
            FileSystemEntry::File {
                base_info,
                metadata,
                ..
            } => (&base_info.name, &metadata),
            FileSystemEntry::Directory {
                base_info,
                metadata,
                ..
            } => (&base_info.name, &metadata),
            FileSystemEntry::Link {
                base_info,
                metadata,
                ..
            } => (&base_info.name, &metadata),
        }
    }
    pub fn get_info_for_long(&self, human_size: bool) -> LongFSEString {
        let (name, metadata) = self.get_name_and_metadata();
        let size = if human_size {
            metadata.human_size.to_string()
        } else {
            format!("{}", metadata.size)
        };
        LongFSEString {
            mode: metadata.mode_str.to_string(),
            size,
            modified_at: metadata.modified_at,
            name: name.to_string(),
        }
    }
    pub fn to_string_long(
        &self,
        human_size: bool,
        inode: bool,
        max_size: usize,
        max_time: usize,
    ) -> String {
        let (name, md) = self.get_name_and_metadata();
        let date_str = md.modified_at.format("%b %e %R");
        format!(
            "{}{} {:<size_width$} {:>time_width$} {}",
            if inode {
                format!("{} ", md.inode)
            } else {
                String::new()
            },
            md.mode_str,
            if human_size {
                md.human_size.to_string()
            } else {
                md.size.to_string()
            },
            date_str,
            name,
            size_width = max_size,
            time_width = max_time,
        )
    }
}

pub struct LongFSEString {
    pub mode: String,
    pub size: String,
    pub modified_at: DateTime<Local>,
    pub name: String,
}
