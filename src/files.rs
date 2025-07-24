use std::{
    collections::HashSet,
    env,
    fs::{self, DirEntry, Metadata},
    path::{Path, PathBuf},
};

#[cfg(windows)]
use crate::time::Time;
use crate::{
    command::{Config, RecursionOptions},
    error::{LsError, LsResult},
};

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
    fn get_code(&self) -> &str {
        match self {
            FileColor::Red => "\x1b[31m",
            FileColor::Green => "\x1b[32m",
            FileColor::Blue => "\x1b[34m",
            FileColor::Aqua => "\x1b[36m",
            FileColor::Other | FileColor::White => "\x1b[37m",
        }
    }
    fn reset(&self) -> &str {
        "\x1b[0m"
    }
    pub fn wrap<S: AsRef<str>>(&self, s: S) -> String {
        format!("{}{}{}", self.get_code(), s.as_ref(), self.reset())
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

    pub created_at: Time,
    pub modified_at: Time,
}

impl MetaData {
    #[cfg(unix)]
    pub fn try_from(metadata: &Metadata) -> LsResult<Self> {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        Some(MetaData {
            size: metadata.len(),
            human_size: get_human_readable_size(metadata.len()),
            inode: metadata.ino(),
            mode: metadata.mode(),
            mode_str: get_file_mode_formated(&metadata),
            executable: metadata.is_file() && metadata.permissions().mode() & 0o111 != 0,
            created_at: DateTime::from(metadata.created()?),
            modified_at: DateTime::from(metadata.modified()?),
        })
    }
    #[cfg(windows)]
    pub fn try_from(metadata: &Metadata) -> LsResult<Self> {
        use crate::time::Time;

        Ok(MetaData {
            size: metadata.len(),
            human_size: get_human_readable_size(metadata.len()),

            attributes: get_based_file_attributes(metadata),
            mode_str: get_file_mode_formated(metadata),

            created_at: Time::from(metadata.created()?),
            modified_at: Time::from(metadata.modified()?),
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
        matches!(self, FileType::File)
    }
    pub fn is_directory(&self) -> bool {
        matches!(self, FileType::Directory)
    }
    pub fn is_link(&self) -> bool {
        matches!(self, FileType::Link)
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

fn path_to_string<P: AsRef<Path>>(path: P) -> LsResult<String> {
    Ok(path
        .as_ref()
        .file_name()
        .ok_or(LsError::none_from("incorrect file_name"))?
        .to_str()
        .ok_or(LsError::none_from("non-valid unicode in name"))?
        .to_string())
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
        format!("{rounded:.1}{suffix}")
    } else {
        format!("{rounded:.2}{suffix}")
    }
}

impl FileSystemEntry {
    pub fn new_with_config(config: &Config) -> LsResult<Self> {
        let path = if config.path.eq(".") {
            env::current_dir()?
        } else {
            PathBuf::from(&config.path)
        };
        let name = path_to_string(&path)?;
        let metadata = fs::symlink_metadata(&path)?;
        let mut fse = Self::new_from_values(name, path, metadata)?;

        fse.fill_start_dir(config.recursive.clone().map(|r| match r {
            RecursionOptions::Depth(depth) => depth,
            RecursionOptions::Unlimited => 40,
            RecursionOptions::No => 0,
        }))?;
        Ok(fse)
    }
    pub fn new_from_values(name: String, path: PathBuf, metadata: Metadata) -> LsResult<Self> {
        let meta_data = MetaData::try_from(&metadata)?;

        if metadata.is_file() {
            Ok(FileSystemEntry::File {
                extension: path
                    .extension()
                    .and_then(|s| s.to_str().map(|s| s.to_string())),
                base_info: BaseInfo {
                    name,
                    #[cfg(windows)]
                    style: FileStyle::default(),
                    #[cfg(unix)]
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
            Ok(FileSystemEntry::Directory {
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
            let target = fs::read_link(&path)?;
            Ok(FileSystemEntry::Link {
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
            Err(LsError::UnknownTypeOfFile(name.clone()))
        }
    }
    pub fn fill_start_dir(&mut self, recursive: Option<usize>) -> LsResult<()> {
        if let Some(depth) = recursive {
            let mut visited_paths = HashSet::new();
            self.fill_dir_recursive_safe(depth, 0, &mut visited_paths);
        } else {
            self.fill_dir_non_recursive()?;
        }
        Ok(())
    }
    fn fill_dir_recursive_safe(
        &mut self,
        max_depth: usize,
        current_depth: usize,
        visited_paths: &mut HashSet<PathBuf>,
    ) {
        if let FileSystemEntry::Directory {
            base_info, entries, ..
        } = self
        {
            if current_depth >= max_depth {
                return;
            }

            let canonical_path = match base_info.path.canonicalize() {
                Ok(path) => path,
                Err(_) => return,
            };

            if visited_paths.contains(&canonical_path) {
                return;
            }
            visited_paths.insert(canonical_path.clone());

            let dir_entries = match fs::read_dir(&base_info.path) {
                Ok(entries) => entries,
                Err(_) => return,
            };

            for entry in dir_entries.flatten() {
                if let Ok(mut fse) = FileSystemEntry::from_dir_entry(entry) {
                    if let FileSystemEntry::Directory { .. } = &mut fse {
                        fse.fill_dir_recursive_safe(max_depth, current_depth + 1, visited_paths);
                    }
                    entries.push(fse);
                }
            }

            visited_paths.remove(&canonical_path);
        }
    }
    fn fill_dir_non_recursive(&mut self) -> LsResult<()> {
        if let FileSystemEntry::Directory {
            base_info, entries, ..
        } = self
        {
            for entry in fs::read_dir(&base_info.path)?.flatten() {
                if let Ok(fse) = FileSystemEntry::from_dir_entry(entry) {
                    entries.push(fse)
                }
            }
        }
        Ok(())
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
        format!("{}{}", info.style.color.wrap(&info.name), suffix)
    }
    pub fn get_styled_name(&self) -> String {
        match self {
            FileSystemEntry::File { base_info, .. } => self.get_styled_name_by_info(base_info),
            FileSystemEntry::Directory { base_info, .. } => self.get_styled_name_by_info(base_info),
            FileSystemEntry::Link { base_info, .. } => self.get_styled_name_by_info(base_info),
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
    pub fn from_path<S: AsRef<str>>(path: S) -> LsResult<Self> {
        let path = if path.as_ref() == "." {
            env::current_dir()?
        } else {
            PathBuf::from(path.as_ref())
        };
        let metadata = fs::symlink_metadata(&path)?;

        let name = path_to_string(&path)?;

        FileSystemEntry::new_from_values(name, path, metadata)
    }
    pub fn from_dir_entry(entry: DirEntry) -> LsResult<Self> {
        let path = entry.path();
        let metadata = entry.metadata()?;

        let name = path_to_string(&path)?;
        FileSystemEntry::new_from_values(name, path, metadata)
    }
    pub fn name(&self) -> &str {
        match self {
            FileSystemEntry::File { base_info, .. } => &base_info.name,
            FileSystemEntry::Directory { base_info, .. } => &base_info.name,
            FileSystemEntry::Link { base_info, .. } => &base_info.name,
        }
    }
    pub fn cname(&self) -> String {
        match self {
            FileSystemEntry::File { base_info, .. } => base_info.name.clone(),
            FileSystemEntry::Directory { base_info, .. } => base_info.name.clone(),
            FileSystemEntry::Link { base_info, .. } => base_info.name.clone(),
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
    pub fn is_dir(&self) -> bool {
        matches!(self, FileSystemEntry::Directory { .. })
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
        let date_str = md.modified_at.format();
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
}
