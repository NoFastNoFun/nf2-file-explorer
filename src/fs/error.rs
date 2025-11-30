use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileOperationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Path does not exist: {0}")]
    PathNotFound(String),
    
    #[error("Path is not a directory: {0}")]
    NotADirectory(String),
    
    #[error("Path is not a file: {0}")]
    NotAFile(String),
    
    #[error("File already exists: {0}")]
    FileExists(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Operation cancelled")]
    Cancelled,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("{0}")]
    Custom(String),
}

impl From<String> for FileOperationError {
    fn from(s: String) -> Self {
        FileOperationError::Custom(s)
    }
}

pub type FileOperationResult<T> = Result<T, FileOperationError>;

