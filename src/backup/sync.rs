use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rayon::prelude::*;
use tracing::{debug, error, info, trace, warn};

use super::core::{copy_file, delete_path, files_are_identical, is_excluded, needs_update};
use super::error::{BackupError, Result};
use super::progress::{ProgressAction, ProgressTracker};

/// Backup synchronization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    /// Mirror sync: destination matches source exactly (deletes extra files)
    #[default]
    Mirror,
    /// Incremental: only copy changed or new files
    Incremental,
    /// Update: copy newer files, don't delete anything
    Update,
    /// Force: overwrite all files regardless of existing content
    Force,
}

/// Result of a backup operation
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct BackupResult {
    /// Number of files copied
    pub copied: u64,
    /// Number of files deleted
    pub deleted: u64,
    /// Number of files skipped
    pub skipped: u64,
    /// Number of errors
    pub errors: u64,
    /// Total bytes transferred
    #[allow(dead_code)]
    pub bytes_transferred: u64,
}

/// Scan source directory and collect file information
pub fn scan_source(source: &Path, exclude_patterns: &[String]) -> Result<Vec<PathBuf>> {
    info!("🔍 Scanning source directory: {}", source.display());
    let mut files = Vec::new();

    for entry in fs::read_dir(source).map_err(|e| BackupError::ReadEntryFailed(e.to_string()))? {
        let entry = entry.map_err(|e| BackupError::ReadEntryFailed(e.to_string()))?;
        scan_directory_recursive(entry.path(), source, exclude_patterns, &mut files)?;
    }

    debug!("📊 Found {} files to backup", files.len());
    Ok(files)
}

/// Recursively scan a directory
fn scan_directory_recursive(
    dir: PathBuf,
    source: &Path,
    exclude_patterns: &[String],
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if is_excluded(&dir, exclude_patterns) {
        trace!("⏭️ Skipping excluded directory: {}", dir.display());
        return Ok(());
    }

    if dir.is_file() {
        let relative = dir.strip_prefix(source).map_err(|e| {
            BackupError::ReadEntryFailed(format!(
                "Failed to strip prefix from {}: {}",
                dir.display(),
                e
            ))
        })?;
        files.push(relative.to_path_buf());
        trace!("📄 Found file: {}", relative.display());
        return Ok(());
    }

    if dir.is_dir() {
        for entry in fs::read_dir(&dir).map_err(|e| BackupError::ReadEntryFailed(e.to_string()))? {
            let entry = entry.map_err(|e| BackupError::ReadEntryFailed(e.to_string()))?;
            scan_directory_recursive(entry.path(), source, exclude_patterns, files)?;
        }
    }

    Ok(())
}

/// Get list of files in destination directory
pub fn scan_destination(dest: &Path, exclude_patterns: &[String]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if !dest.exists() {
        return Ok(files);
    }

    for entry in fs::read_dir(dest).map_err(|e| BackupError::ReadEntryFailed(e.to_string()))? {
        let entry = entry.map_err(|e| BackupError::ReadEntryFailed(e.to_string()))?;
        scan_dest_directory_recursive(entry.path(), dest, exclude_patterns, &mut files)?;
    }

    Ok(files)
}

/// Recursively scan destination directory
fn scan_dest_directory_recursive(
    dir: PathBuf,
    dest: &Path,
    exclude_patterns: &[String],
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    if is_excluded(&dir, exclude_patterns) {
        return Ok(());
    }

    if dir.is_file() {
        let relative = dir.strip_prefix(dest).map_err(|e| {
            BackupError::ReadEntryFailed(format!(
                "Failed to strip prefix from {}: {}",
                dir.display(),
                e
            ))
        })?;
        files.push(relative.to_path_buf());
        return Ok(());
    }

    if dir.is_dir() {
        for entry in fs::read_dir(&dir).map_err(|e| BackupError::ReadEntryFailed(e.to_string()))? {
            let entry = entry.map_err(|e| BackupError::ReadEntryFailed(e.to_string()))?;
            scan_dest_directory_recursive(entry.path(), dest, exclude_patterns, files)?;
        }
    }

    Ok(())
}

/// Calculate total size of files to be backed up
pub fn calculate_total_size(source: &Path, files: &[PathBuf]) -> u64 {
    files
        .par_iter()
        .filter_map(|relative| {
            let path = source.join(relative);
            fs::metadata(&path).ok().map(|m| m.len())
        })
        .sum()
}

/// Process a single file for backup
fn process_file(
    relative: &PathBuf,
    source: &Path,
    dest: &Path,
    progress: &ProgressTracker,
    dry_run: bool,
    check_fn: impl Fn(&Path, &Path) -> Result<bool>,
) -> (bool, bool) {
    let source_path = source.join(relative);
    let dest_path = dest.join(relative);

    progress.start_file(&relative.display().to_string());

    match check_fn(&source_path, &dest_path) {
        Ok(should_copy) if should_copy => {
            if dry_run {
                progress.file_processed(0, ProgressAction::Copied);
                (true, false)
            } else {
                match copy_file(&source_path, &dest_path, progress) {
                    Ok(()) => (true, false),
                    Err(e) => {
                        error!("❌ Error copying {}: {}", relative.display(), e);
                        progress.record_error();
                        (false, true)
                    }
                }
            }
        }
        Ok(_) => {
            progress.file_processed(0, ProgressAction::Skipped);
            (false, false)
        }
        Err(e) => {
            error!("❌ Error processing {}: {}", relative.display(), e);
            progress.record_error();
            (false, true)
        }
    }
}

/// Perform mirror sync: destination matches source exactly
pub fn mirror_sync(
    source: &Path,
    dest: &Path,
    exclude_patterns: &[String],
    progress: &ProgressTracker,
    dry_run: bool,
) -> Result<BackupResult> {
    info!(
        "🪞 Starting mirror sync: {} → {}",
        source.display(),
        dest.display()
    );
    if dry_run {
        warn!("🔍 Dry run mode - no changes will be made");
    }

    let mut result = BackupResult::default();

    // Scan source files
    progress.start_file("🔍 Scanning source…");
    let source_files = scan_source(source, exclude_patterns)?;
    let total_size = calculate_total_size(source, &source_files);
    progress.set_total(source_files.len() as u64, total_size);

    // Process files in parallel with limited concurrency
    let source_files_arc = source_files.clone();
    let results: Vec<(bool, bool)> = source_files_arc
        .par_iter()
        .map_with(
            (source, dest, progress, dry_run),
            |(src, dst, prog, dry), relative| {
                process_file(relative, src, dst, prog, *dry, |s, d| {
                    if d.exists() {
                        files_are_identical(s, d).map(|identical| !identical)
                    } else {
                        Ok(true)
                    }
                })
            },
        )
        .collect();

    // Aggregate results
    for (copied, error) in results {
        if error {
            result.errors += 1;
        } else if copied {
            result.copied += 1;
        } else {
            result.skipped += 1;
        }
    }

    // Delete files in destination that don't exist in source
    progress.start_file("🗑️ Checking for extra files…");
    let dest_files = scan_destination(dest, exclude_patterns)?;
    let source_files_set: HashSet<_> = source_files.iter().collect();

    let delete_results: Vec<bool> = dest_files
        .par_iter()
        .filter(|relative| !source_files_set.contains(relative))
        .map(|relative| {
            let dest_path = dest.join(relative);
            progress.start_file(&format!("🗑️ Deleting: {}", relative.display()));

            if dry_run {
                progress.file_processed(0, ProgressAction::Deleted);
                true
            } else {
                match delete_path(&dest_path, progress) {
                    Ok(()) => true,
                    Err(e) => {
                        error!("❌ Error deleting {}: {}", relative.display(), e);
                        progress.record_error();
                        false
                    }
                }
            }
        })
        .collect();

    for success in delete_results {
        if success {
            result.deleted += 1;
        } else {
            result.errors += 1;
        }
    }

    progress.complete();
    Ok(result)
}

/// Perform incremental sync: only copy changed or new files
pub fn incremental_sync(
    source: &Path,
    dest: &Path,
    exclude_patterns: &[String],
    progress: &ProgressTracker,
    dry_run: bool,
) -> Result<BackupResult> {
    info!(
        "📈 Starting incremental sync: {} → {}",
        source.display(),
        dest.display()
    );
    if dry_run {
        warn!("🔍 Dry run mode - no changes will be made");
    }

    let mut result = BackupResult::default();

    // Scan source files
    progress.start_file("🔍 Scanning source…");
    let source_files = scan_source(source, exclude_patterns)?;
    let total_size = calculate_total_size(source, &source_files);
    progress.set_total(source_files.len() as u64, total_size);

    // Process files in parallel with limited concurrency
    let results: Vec<(bool, bool)> = source_files
        .par_iter()
        .map_with(
            (source, dest, progress, dry_run),
            |(src, dst, prog, dry), relative| {
                process_file(relative, src, dst, prog, *dry, |s, d| {
                    if d.exists() {
                        files_are_identical(s, d).map(|identical| !identical)
                    } else {
                        Ok(true)
                    }
                })
            },
        )
        .collect();

    // Aggregate results
    for (copied, error) in results {
        if error {
            result.errors += 1;
        } else if copied {
            result.copied += 1;
        } else {
            result.skipped += 1;
        }
    }

    // Don't delete files in incremental mode
    progress.complete();
    Ok(result)
}

/// Perform update sync: copy newer files, don't delete
pub fn update_sync(
    source: &Path,
    dest: &Path,
    exclude_patterns: &[String],
    progress: &ProgressTracker,
    dry_run: bool,
) -> Result<BackupResult> {
    info!(
        "🔄 Starting update sync: {} → {}",
        source.display(),
        dest.display()
    );
    if dry_run {
        warn!("🔍 Dry run mode - no changes will be made");
    }

    let mut result = BackupResult::default();

    // Scan source files
    progress.start_file("🔍 Scanning source…");
    let source_files = scan_source(source, exclude_patterns)?;
    let total_size = calculate_total_size(source, &source_files);
    progress.set_total(source_files.len() as u64, total_size);

    // Process files in parallel with limited concurrency
    let results: Vec<(bool, bool)> = source_files
        .par_iter()
        .map_with(
            (source, dest, progress, dry_run),
            |(src, dst, prog, dry), relative| {
                process_file(relative, src, dst, prog, *dry, |s, d| {
                    if d.exists() {
                        needs_update(s, d)
                    } else {
                        Ok(true)
                    }
                })
            },
        )
        .collect();

    // Aggregate results
    for (copied, error) in results {
        if error {
            result.errors += 1;
        } else if copied {
            result.copied += 1;
        } else {
            result.skipped += 1;
        }
    }

    // Don't delete files in update mode
    progress.complete();
    Ok(result)
}

/// Perform force sync: overwrite all files regardless of existing content
pub fn force_sync(
    source: &Path,
    dest: &Path,
    exclude_patterns: &[String],
    progress: &ProgressTracker,
    dry_run: bool,
) -> Result<BackupResult> {
    info!(
        "⚡ Starting force sync: {} → {}",
        source.display(),
        dest.display()
    );
    if dry_run {
        warn!("🔍 Dry run mode - no changes will be made");
    }

    let mut result = BackupResult::default();

    // Scan source files
    progress.start_file("🔍 Scanning source…");
    let source_files = scan_source(source, exclude_patterns)?;
    let total_size = calculate_total_size(source, &source_files);
    progress.set_total(source_files.len() as u64, total_size);

    // Process files in parallel - copy ALL files (no skipping)
    let results: Vec<(bool, bool)> = source_files
        .par_iter()
        .map_with(
            (source, dest, progress, dry_run),
            |(src, dst, prog, dry), relative| {
                process_file(relative, src, dst, prog, *dry, |_s, _d| Ok(true))
            },
        )
        .collect();

    // Aggregate results
    for (copied, error) in results {
        if error {
            result.errors += 1;
        } else if copied {
            result.copied += 1;
        }
    }

    // Don't delete files in force mode (only overwrite)
    progress.complete();
    Ok(result)
}

/// Run backup based on sync mode
pub fn run_backup(
    source: &Path,
    dest: &Path,
    mode: SyncMode,
    exclude_patterns: &[String],
    progress: &ProgressTracker,
    dry_run: bool,
) -> Result<BackupResult> {
    // Validate source exists
    if !source.exists() {
        error!("❌ Source directory does not exist: {}", source.display());
        return Err(BackupError::SourceNotFound(source.display().to_string()));
    }

    if !source.is_dir() {
        error!("❌ Source is not a directory: {}", source.display());
        return Err(BackupError::SourceNotDirectory(
            source.display().to_string(),
        ));
    }

    info!("🚀 Starting backup operation");
    debug!(
        "📁 Source: {} | Destination: {} | Mode: {:?}",
        source.display(),
        dest.display(),
        mode
    );

    // Create destination if it doesn't exist
    if !dest.exists() && !dry_run {
        fs::create_dir_all(dest).map_err(|e| {
            BackupError::DestinationCreateFailed(format!("{}: {}", dest.display(), e))
        })?;
        debug!("📁 Created destination directory: {}", dest.display());
    }

    let result = match mode {
        SyncMode::Mirror => mirror_sync(source, dest, exclude_patterns, progress, dry_run),
        SyncMode::Incremental => {
            incremental_sync(source, dest, exclude_patterns, progress, dry_run)
        }
        SyncMode::Update => update_sync(source, dest, exclude_patterns, progress, dry_run),
        SyncMode::Force => force_sync(source, dest, exclude_patterns, progress, dry_run),
    };

    match &result {
        Ok(r) => {
            info!(
                "✨ Backup completed: {} copied, {} deleted, {} skipped, {} errors",
                r.copied, r.deleted, r.skipped, r.errors
            );
        }
        Err(e) => {
            error!("❌ Backup failed: {}", e);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_dirs() -> (TempDir, TempDir, PathBuf, PathBuf) {
        let source_dir = TempDir::new().unwrap();
        let dest_dir = TempDir::new().unwrap();
        let source = source_dir.path().to_path_buf();
        let dest = dest_dir.path().to_path_buf();
        (source_dir, dest_dir, source, dest)
    }

    fn create_test_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_scan_source() {
        let (_source_dir, _dest_dir, source, _dest) = setup_test_dirs();
        create_test_file(&source.join("file1.txt"), "content1");
        create_test_file(&source.join("subdir/file2.txt"), "content2");
        create_test_file(&source.join("node_modules/pkg/index.js"), "pkg");

        let exclude = vec!["node_modules".to_string()];
        let files = scan_source(&source, &exclude).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.contains(&PathBuf::from("file1.txt")));
        assert!(files.contains(&PathBuf::from("subdir/file2.txt")));
    }

    #[test]
    fn test_calculate_total_size() {
        let (_source_dir, _dest_dir, source, _dest) = setup_test_dirs();
        create_test_file(&source.join("file1.txt"), "12345");
        create_test_file(&source.join("file2.txt"), "1234567890");

        let files = vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")];
        let total = calculate_total_size(&source, &files);

        assert_eq!(total, 15);
    }

    #[test]
    fn test_files_are_identical_same_content() {
        let (_source_dir, _dest_dir, source, dest) = setup_test_dirs();
        let source_file = source.join("test.txt");
        let dest_file = dest.join("test.txt");

        create_test_file(&source_file, "same content");
        create_test_file(&dest_file, "same content");

        assert!(files_are_identical(&source_file, &dest_file).unwrap());
    }

    #[test]
    fn test_files_are_identical_different_content() {
        let (_source_dir, _dest_dir, source, dest) = setup_test_dirs();
        let source_file = source.join("test.txt");
        let dest_file = dest.join("test.txt");

        create_test_file(&source_file, "content1");
        create_test_file(&dest_file, "content2");

        assert!(!files_are_identical(&source_file, &dest_file).unwrap());
    }

    #[test]
    fn test_files_are_identical_dest_not_exists() {
        let (_source_dir, _dest_dir, source, dest) = setup_test_dirs();
        let source_file = source.join("test.txt");
        let dest_file = dest.join("test.txt");

        create_test_file(&source_file, "content");
        // dest_file doesn't exist

        assert!(!files_are_identical(&source_file, &dest_file).unwrap());
    }
}
