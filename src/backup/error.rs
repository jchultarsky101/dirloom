use thiserror::Error;

/// Backup operation errors
#[derive(Error, Debug)]
pub enum BackupError {
    #[error("📁 Source directory does not exist: {0}")]
    SourceNotFound(String),

    #[error("📁 Source is not a directory: {0}")]
    SourceNotDirectory(String),

    #[error("📁 Failed to create destination directory: {0}")]
    DestinationCreateFailed(String),

    #[error("📄 Failed to read directory entry: {0}")]
    ReadEntryFailed(String),

    #[error("📄 Failed to read metadata for {path}: {source}")]
    MetadataFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("📄 Failed to create directory {path}: {source}")]
    CreateDirFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("📋 Failed to copy {source} to {dest}: {source_error}")]
    CopyFailed {
        source: String,
        dest: String,
        #[source]
        source_error: std::io::Error,
    },

    #[error("🗑️  Failed to remove {path}: {source}")]
    RemoveFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("📖 Failed to read file {path}: {source}")]
    ReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("⏰ Failed to get modification time: {0}")]
    ModifiedTimeFailed(String),
}

pub type Result<T> = std::result::Result<T, BackupError>;
