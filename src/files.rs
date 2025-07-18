use std::{
    env,
    fs::{self, DirEntry, Metadata},
    path::PathBuf,
};

use chrono::{DateTime, Local};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FileColor {
    // for links with no-exist target
    Red,
    // for normal links
    Aqua,
    // for directories
    Blue,
    // for executable
    Green,
    // default,
    #[default]
    White,
    // TODO: for files by file extension (white by default)
    Other,
}

impl FileColor {
    pub fn get_code(&self) -> &str {
        match self {
            FileColor::Red => "\x1b[31m",
            FileColor::Green => "\x1b[32m",
            FileColor::Blue => "\x1b[34m",
            FileColor::Aqua => "\x1b[36m",
            FileColor::Other | FileColor::White => "\x1b[37m",
        }
    }
    pub fn reset(&self) -> &str {
        "\x1b[0m"
    }
}

#[cfg(unix)]
#[derive(Debug, Clone)]
pub struct MetaData {
    pub size: u64,
    pub human_size: String,

    pub inode: u64,

    pub mode: u32,
    pub mode_str: String,
    pub executable: bool,

    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
}

#[cfg(windows)]
#[derive(Debug, Clone, Default)]
pub struct MetaData {
    pub size: u64,
    pub human_size: String,

    // windows mode
    pub mode_str: String,
    pub attributes: [bool; 6],

    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
}

impl MetaData {
    #[cfg(unix)]
    pub fn try_from(metadata: &Metadata) -> Option<Self> {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        Some(MetaData {
            size: metadata.len(),
            human_size: get_human_readable_size(metadata.len()),
            inode: metadata.ino(),
            mode: metadata.mode(),
            mode_str: get_file_mode_formated(&metadata),
            executable: metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
            created_at: DateTime::from(metadata.created().ok()?),
            modified_at: DateTime::from(metadata.modified().ok()?),
        })
    }
    #[cfg(windows)]
    pub fn try_from(metadata: &Metadata) -> Option<Self> {
        Some(MetaData {
            size: metadata.len(),
            human_size: get_human_readable_size(metadata.len()),

            attributes: get_based_file_attributes(metadata),
            mode_str: get_file_mode_formated(metadata),

            created_at: DateTime::from(metadata.created().ok()?),
            modified_at: DateTime::from(metadata.modified().ok()?),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct FileStyle {
    pub suffix: Option<char>,
    pub color: FileColor,
}

#[derive(Debug, Clone)]
pub struct BaseInfo {
    pub name: String,
    pub style: FileStyle,

    pub path: PathBuf,
}

pub enum FileType {
    File,
    Directory,
    Link,
}

impl From<&FileSystemEntry> for FileType {
    fn from(entry: &FileSystemEntry) -> Self {
        match entry {
            FileSystemEntry::File { .. } => FileType::File,
            FileSystemEntry::Directory { .. } => FileType::Directory,
            FileSystemEntry::Link { .. } => FileType::Link,
        }
    }
}

impl FileType {
    pub fn is_file(&self) -> bool {
        match self {
            FileType::File => true,
            _ => false,
        }
    }
    pub fn is_directory(&self) -> bool {
        match self {
            FileType::Directory => true,
            _ => false,
        }
    }
    pub fn is_link(&self) -> bool {
        match self {
            FileType::Link => true,
            _ => false,
        }
    }
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

#[cfg(unix)]
fn get_file_mode_formated(md: &Metadata) -> String {
    use std::os::unix::fs::PermissionsExt;

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

    builder
}

#[cfg(windows)]
fn get_based_file_attributes(md: &Metadata) -> [bool; 6] {
    use std::os::windows::fs::MetadataExt;
    use winapi::um::winnt;

    let attrs = md.file_attributes();

    [
        (attrs & winnt::FILE_ATTRIBUTE_DIRECTORY) != 0,
        (attrs & winnt::FILE_ATTRIBUTE_ARCHIVE) != 0,
        (attrs & winnt::FILE_ATTRIBUTE_READONLY) != 0,
        (attrs & winnt::FILE_ATTRIBUTE_HIDDEN) != 0,
        (attrs & winnt::FILE_ATTRIBUTE_SYSTEM) != 0,
        (attrs & winnt::FILE_ATTRIBUTE_REPARSE_POINT) != 0,
    ]
}
#[cfg(windows)]
fn get_file_mode_formated(md: &Metadata) -> String {
    let attrs = get_based_file_attributes(md);
    let mut mode = String::with_capacity(6);

    for flag in attrs {
        mode.push(if flag {
            "darhsl".chars().nth(mode.len()).unwrap()
        } else {
            '-'
        });
    }
    mode
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
    #[cfg(unix)]
    pub fn new_from_values(name: String, path: PathBuf, metadata: Metadata) -> Option<Self> {
        let meta_data = MetaData::try_from(&metadata)?;

        if metadata.is_file() {
            Some(FileSystemEntry::File {
                extension: path
                    .extension()
                    .and_then(|s| s.to_str().map(|s| s.to_string())),
                base_info: BaseInfo {
                    name,
                    style: if meta_data.executable {
                        FileStyle {
                            suffix: None,
                            color: FileColor::Green,
                        }
                    } else {
                        FileStyle::default()
                    },
                    path,
                },
                metadata: meta_data,
            })
        } else if metadata.is_dir() {
            Some(FileSystemEntry::Directory {
                base_info: BaseInfo {
                    name,
                    style: FileStyle {
                        suffix: Some('/'),
                        color: FileColor::Blue,
                    },
                    path,
                },
                metadata: meta_data,
                entries: vec![],
            })
        } else if metadata.is_symlink() {
            let target = fs::read_link(&path).ok()?;
            Some(FileSystemEntry::Link {
                base_info: BaseInfo {
                    name,
                    style: FileStyle {
                        suffix: Some('@'),
                        color: FileColor::Aqua,
                    },
                    path,
                },
                metadata: meta_data,
                target,
            })
        } else {
            None
        }
    }
    #[cfg(windows)]
    pub fn new_from_values(name: String, path: PathBuf, metadata: Metadata) -> Option<Self> {
        let meta_data = MetaData::try_from(&metadata)?;

        if metadata.is_file() {
            Some(FileSystemEntry::File {
                extension: path
                    .extension()
                    .and_then(|s| s.to_str().map(|s| s.to_string())),
                base_info: BaseInfo {
                    name,
                    style: FileStyle::default(),
                    path,
                },
                metadata: meta_data,
            })
        } else if metadata.is_dir() {
            Some(FileSystemEntry::Directory {
                base_info: BaseInfo {
                    name,
                    style: FileStyle {
                        suffix: Some('/'),
                        color: FileColor::Blue,
                    },
                    path,
                },
                metadata: meta_data,
                entries: vec![],
            })
        } else if metadata.is_symlink() {
            let target = fs::read_link(&path).ok()?;
            Some(FileSystemEntry::Link {
                base_info: BaseInfo {
                    name,
                    style: FileStyle {
                        suffix: Some('@'),
                        color: FileColor::Aqua,
                    },
                    path,
                },
                metadata: meta_data,
                target,
            })
        } else {
            None
        }
    }
    pub fn get_dir_entries(&self) -> Option<Vec<FileSystemEntry>> {
        match self {
            FileSystemEntry::Directory { entries, .. } => Some(entries.clone()),
            _ => None,
        }
    }
    pub fn get_styled_name_by_info(&self, info: &BaseInfo) -> String {
        let suffix = if let Some(suffix) = info.style.suffix {
            suffix.to_string()
        } else {
            String::new()
        };
        format!(
            "{}{}{}{}",
            info.style.color.get_code(),
            info.name,
            info.style.color.reset(),
            suffix
        )
    }
    pub fn get_styled_name(&self) -> String {
        match self {
            FileSystemEntry::File { base_info, .. } => self.get_styled_name_by_info(&base_info),
            FileSystemEntry::Directory { base_info, .. } => {
                self.get_styled_name_by_info(&base_info)
            }
            FileSystemEntry::Link { base_info, .. } => self.get_styled_name_by_info(&base_info),
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
        let metadata = fs::symlink_metadata(&path).ok()?;

        let name = path.file_name()?.to_string_lossy().to_string();

        FileSystemEntry::new_from_values(name, path, metadata)
    }
    pub fn from_dir_entry(entry: DirEntry) -> Option<Self> {
        let path = entry.path();
        let metadata = entry.metadata().ok()?;

        let name = entry.file_name().to_string_lossy().to_string();
        FileSystemEntry::new_from_values(name, path, metadata)
    }
    pub fn name(&self) -> &str {
        match self {
            FileSystemEntry::File { base_info, .. } => &base_info.name,
            FileSystemEntry::Directory { base_info, .. } => &base_info.name,
            FileSystemEntry::Link { base_info, .. } => &base_info.name,
        }
    }
    pub fn style(&self) -> &FileStyle {
        match self {
            FileSystemEntry::File { base_info, .. } => &base_info.style,
            FileSystemEntry::Directory { base_info, .. } => &base_info.style,
            FileSystemEntry::Link { base_info, .. } => &base_info.style,
        }
    }
    pub fn to_string_short(&self) -> String {
        self.get_styled_name()
    }
    #[cfg(unix)]
    pub fn to_string_long(
        &self,
        human_size: bool,
        inode: bool,
        max_size: usize,
        max_time: usize,
    ) -> String {
        let styled_name = self.get_styled_name();
        let md = self.metadata();
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
            styled_name,
            size_width = max_size,
            time_width = max_time,
        )
    }
    #[cfg(windows)]
    pub fn to_string_long(&self, human_size: bool, max_size: usize, max_time: usize) -> String {
        let styled_name = self.get_styled_name();
        let md = self.metadata();
        let date_str = md.modified_at.format("%b %e %R");
        format!(
            "{} {:<size_width$} {:>time_width$} {}",
            md.mode_str,
            if human_size {
                md.human_size.to_string()
            } else {
                md.size.to_string()
            },
            date_str,
            styled_name,
            size_width = max_size,
            time_width = max_time,
        )
    }

    fn _get_name_and_metadata(&self) -> (&str, &MetaData) {
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
    fn _get_info_and_metadata(&self) -> (&BaseInfo, &MetaData) {
        match self {
            FileSystemEntry::File {
                base_info,
                metadata,
                ..
            } => (base_info, metadata),
            FileSystemEntry::Directory {
                base_info,
                metadata,
                ..
            } => (base_info, metadata),
            FileSystemEntry::Link {
                base_info,
                metadata,
                ..
            } => (base_info, metadata),
        }
    }
}
