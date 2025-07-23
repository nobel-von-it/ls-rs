use std::io;

#[derive(Debug, thiserror::Error)]
pub enum LsError {
    #[error("IOError {0}")]
    IOError(#[from] io::Error),
    #[error("Trying to unwrap None value {0}")]
    NoneValue(String),
    #[error("Function cannot determine the type of file {0}")]
    UnknownTypeOfFile(String),
}

impl LsError {
    pub fn none_from<S: AsRef<str>>(s: S) -> Self {
        LsError::NoneValue(s.as_ref().to_string())
    }
}

pub type LsResult<T> = Result<T, LsError>;
