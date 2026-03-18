use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use tracing::{debug, trace};

use super::error::{BackupError, Result};
use super::progress::{ProgressAction, ProgressTracker};

/// Information about a file to be backed up
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileInfo {
    /// Relative path from source root
    pub relative_path: PathBuf,
    /// Absolute path to the source file
    pub source_path: PathBuf,
    /// Absolute path to the destination file
    pub dest_path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// File metadata
    pub metadata: Metadata,
}

impl FileInfo {
    /// Create a new FileInfo
    #[allow(dead_code)]
    pub fn new(
        relative_path: PathBuf,
        source_path: PathBuf,
        dest_path: PathBuf,
        metadata: Metadata,
    ) -> Self {
        Self {
            relative_path,
            source_path,
            dest_path,
            size: metadata.len(),
            metadata,
        }
    }
}

/// Check if a path matches any of the exclude patterns
pub fn is_excluded(path: &Path, exclude_patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    let file_name = path.file_name().map(|n| n.to_string_lossy());

    for pattern in exclude_patterns {
        // Check if pattern matches the full path
        if path_str.contains(pattern.as_str()) {
            trace!(
                "🚫 Excluded (path match): {} matches {}",
                path.display(),
                pattern
            );
            return true;
        }

        // Check if pattern matches the filename (supports glob-like patterns)
        if let Some(ref name) = file_name
            && pattern_matches(name, pattern)
        {
            trace!("🚫 Excluded (pattern match): {} matches {}", name, pattern);
            return true;
        }
    }

    false
}

/// Simple pattern matching (supports * wildcard)
fn pattern_matches(text: &str, pattern: &str) -> bool {
    if pattern == text {
        return true;
    }

    // Handle *.ext patterns
    if pattern.starts_with("*.") {
        let ext = &pattern[1..];
        return text.ends_with(ext);
    }

    // Handle * patterns
    if pattern == "*" {
        return true;
    }

    // Handle prefix* patterns
    if let Some(prefix) = pattern.strip_suffix('*') {
        return text.starts_with(prefix);
    }

    // Handle *suffix patterns
    if let Some(suffix) = pattern.strip_prefix('*') {
        return text.ends_with(suffix);
    }

    false
}

/// Copy a file from source to destination, creating parent directories as needed
pub fn copy_file(source: &Path, dest: &Path, progress: &ProgressTracker) -> Result<()> {
    // Create parent directories if they don't exist
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| BackupError::CreateDirFailed {
            path: parent.display().to_string(),
            source: e,
        })?;
        debug!("📁 Created directory: {}", parent.display());
    }

    // Copy the file
    fs::copy(source, dest).map_err(|e| BackupError::CopyFailed {
        source: source.display().to_string(),
        dest: dest.display().to_string(),
        source_error: e,
    })?;

    debug!("✅ Copied: {} → {}", source.display(), dest.display());
    progress.file_processed(0, ProgressAction::Copied);

    Ok(())
}

/// Delete a file or directory
pub fn delete_path(path: &Path, progress: &ProgressTracker) -> Result<()> {
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| BackupError::RemoveFailed {
            path: path.display().to_string(),
            source: e,
        })?;
        debug!("🗑️ Removed directory: {}", path.display());
    } else {
        fs::remove_file(path).map_err(|e| BackupError::RemoveFailed {
            path: path.display().to_string(),
            source: e,
        })?;
        debug!("🗑️ Removed file: {}", path.display());
    }

    progress.file_processed(0, ProgressAction::Deleted);

    Ok(())
}

/// Check if a file needs to be copied based on modification time
pub fn needs_update(source: &Path, dest: &Path) -> Result<bool> {
    if !dest.exists() {
        return Ok(true);
    }

    let source_meta = fs::metadata(source).map_err(|e| BackupError::MetadataFailed {
        path: source.display().to_string(),
        source: e,
    })?;
    let dest_meta = fs::metadata(dest).map_err(|e| BackupError::MetadataFailed {
        path: dest.display().to_string(),
        source: e,
    })?;

    let source_time = source_meta
        .modified()
        .map_err(|e| BackupError::ModifiedTimeFailed(format!("{}: {}", source.display(), e)))?;
    let dest_time = dest_meta
        .modified()
        .map_err(|e| BackupError::ModifiedTimeFailed(format!("{}: {}", dest.display(), e)))?;

    let needs = source_time > dest_time;
    if needs {
        debug!("📝 File needs update: {}", source.display());
    }
    Ok(needs)
}

/// Check if two files are identical (same size, modification time, and content)
pub fn files_are_identical(source: &Path, dest: &Path) -> Result<bool> {
    if !dest.exists() {
        return Ok(false);
    }

    let source_meta = fs::metadata(source).map_err(|e| BackupError::MetadataFailed {
        path: source.display().to_string(),
        source: e,
    })?;
    let dest_meta = fs::metadata(dest).map_err(|e| BackupError::MetadataFailed {
        path: dest.display().to_string(),
        source: e,
    })?;

    // Compare size first (quick check)
    if source_meta.len() != dest_meta.len() {
        trace!(
            "📊 Size differs: {} vs {}",
            source_meta.len(),
            dest_meta.len()
        );
        return Ok(false);
    }

    // Compare modification time
    match (source_meta.modified(), dest_meta.modified()) {
        (Ok(source_time), Ok(dest_time)) => {
            // Allow small time differences (filesystem precision)
            if let Ok(diff) = source_time.duration_since(dest_time)
                && diff.as_secs() >= 2
            {
                return Ok(false);
            }
            if let Ok(diff) = dest_time.duration_since(source_time)
                && diff.as_secs() >= 2
            {
                return Ok(false);
            }
        }
        _ => return Ok(false),
    }

    // Compare content (definitive check)
    let source_content = fs::read(source).map_err(|e| BackupError::ReadFailed {
        path: source.display().to_string(),
        source: e,
    })?;
    let dest_content = fs::read(dest).map_err(|e| BackupError::ReadFailed {
        path: dest.display().to_string(),
        source: e,
    })?;

    let identical = source_content == dest_content;
    if identical {
        trace!("✅ Files identical: {}", source.display());
    } else {
        trace!("📝 Files differ: {}", source.display());
    }
    Ok(identical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_excluded_exact_match() {
        let patterns = vec!["node_modules".to_string()];
        assert!(is_excluded(Path::new("node_modules"), &patterns));
        assert!(!is_excluded(Path::new("src"), &patterns));
    }

    #[test]
    fn test_is_excluded_path_contains() {
        let patterns = vec!["node_modules".to_string()];
        assert!(is_excluded(Path::new("project/node_modules"), &patterns));
        assert!(is_excluded(
            Path::new("project/node_modules/package"),
            &patterns
        ));
    }

    #[test]
    fn test_is_excluded_glob_pattern() {
        let patterns = vec!["*.tmp".to_string(), "*.log".to_string()];
        assert!(is_excluded(Path::new("file.tmp"), &patterns));
        assert!(is_excluded(Path::new("debug.log"), &patterns));
        assert!(!is_excluded(Path::new("file.txt"), &patterns));
    }

    #[test]
    fn test_is_excluded_multiple_patterns() {
        let patterns = vec![
            "*.tmp".to_string(),
            "node_modules".to_string(),
            ".git".to_string(),
        ];
        assert!(is_excluded(Path::new("cache.tmp"), &patterns));
        assert!(is_excluded(Path::new("node_modules"), &patterns));
        assert!(is_excluded(Path::new(".git"), &patterns));
        assert!(!is_excluded(Path::new("src/main.rs"), &patterns));
    }

    #[test]
    fn test_pattern_matches() {
        assert!(pattern_matches("file.tmp", "*.tmp"));
        assert!(pattern_matches("test.log", "*.log"));
        assert!(!pattern_matches("file.txt", "*.tmp"));
        assert!(pattern_matches("anything", "*"));
        assert!(pattern_matches("prefix_test", "prefix*"));
        assert!(pattern_matches("test_suffix", "*suffix"));
    }
}
