//! Integration tests for dirloom backup functionality

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use tempfile::TempDir;

use dirloom::backup::{
    progress::ProgressTracker,
    sync::{SyncMode, run_backup},
};

/// Setup test directories with some files
fn setup_test_environment() -> (TempDir, TempDir, PathBuf, PathBuf) {
    let source_dir = TempDir::new().expect("Failed to create source temp dir");
    let dest_dir = TempDir::new().expect("Failed to create dest temp dir");
    let source = source_dir.path().to_path_buf();
    let dest = dest_dir.path().to_path_buf();
    (source_dir, dest_dir, source, dest)
}

/// Create a test file with given content
fn create_test_file(parent: &PathBuf, path: &str, content: &str) {
    let full_path = parent.join(path);
    if let Some(parent_dir) = full_path.parent() {
        fs::create_dir_all(parent_dir).expect("Failed to create parent dirs");
    }
    let mut file = File::create(&full_path).expect("Failed to create file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to file");
}

/// Check if a file exists and has expected content
fn assert_file_content(path: &PathBuf, expected_content: &str) {
    assert!(path.exists(), "File should exist: {}", path.display());
    let content = fs::read_to_string(path).expect("Failed to read file");
    assert_eq!(
        content,
        expected_content,
        "File content mismatch: {}",
        path.display()
    );
}

#[test]
fn test_mirror_sync_basic() {
    let (_source_dir, _dest_dir, source, dest) = setup_test_environment();

    // Create test files
    create_test_file(&source, "file1.txt", "content1");
    create_test_file(&source, "subdir/file2.txt", "content2");
    create_test_file(&source, "deep/nested/file3.txt", "content3");

    // Run mirror sync
    let progress = ProgressTracker::new();
    let result = run_backup(&source, &dest, SyncMode::Mirror, &[], &progress, false)
        .expect("Backup should succeed");

    // Verify results
    assert_eq!(result.copied, 3, "Should copy 3 files");
    assert_eq!(result.deleted, 0, "Should delete 0 files");
    assert_eq!(result.skipped, 0, "Should skip 0 files on first run");
    assert_eq!(result.errors, 0, "Should have no errors");

    // Verify files exist in destination
    assert_file_content(&dest.join("file1.txt"), "content1");
    assert_file_content(&dest.join("subdir/file2.txt"), "content2");
    assert_file_content(&dest.join("deep/nested/file3.txt"), "content3");
}

#[test]
fn test_mirror_sync_deletes_extra_files() {
    let (_source_dir, _dest_dir, source, dest) = setup_test_environment();

    // Create source files
    create_test_file(&source, "keep.txt", "keep");

    // Create extra file in destination
    create_test_file(&dest, "extra.txt", "extra");

    // Run mirror sync
    let progress = ProgressTracker::new();
    let result = run_backup(&source, &dest, SyncMode::Mirror, &[], &progress, false)
        .expect("Backup should succeed");

    // Verify extra file was deleted
    assert_eq!(result.deleted, 1, "Should delete 1 extra file");
    assert!(
        !dest.join("extra.txt").exists(),
        "Extra file should be deleted"
    );
    assert_file_content(&dest.join("keep.txt"), "keep");
}

#[test]
fn test_incremental_sync_no_deletes() {
    let (_source_dir, _dest_dir, source, dest) = setup_test_environment();

    // Create source files
    create_test_file(&source, "file1.txt", "content1");

    // Create extra file in destination (should be kept in incremental mode)
    create_test_file(&dest, "extra.txt", "extra");

    // Run incremental sync
    let progress = ProgressTracker::new();
    let result = run_backup(&source, &dest, SyncMode::Incremental, &[], &progress, false)
        .expect("Backup should succeed");

    // Verify extra file was NOT deleted
    assert_eq!(result.deleted, 0, "Incremental should not delete files");
    assert!(dest.join("extra.txt").exists(), "Extra file should be kept");
    assert_file_content(&dest.join("file1.txt"), "content1");
}

#[test]
fn test_incremental_sync_skips_unchanged() {
    let (_source_dir, _dest_dir, source, dest) = setup_test_environment();

    // Create identical files in source and destination
    create_test_file(&source, "file1.txt", "content1");
    create_test_file(&dest, "file1.txt", "content1");

    // Run incremental sync
    let progress = ProgressTracker::new();
    let result = run_backup(&source, &dest, SyncMode::Incremental, &[], &progress, false)
        .expect("Backup should succeed");

    // Verify file was skipped (already identical)
    assert_eq!(result.copied, 0, "Should skip unchanged files");
    assert_eq!(result.skipped, 1, "Should skip 1 file");
}

#[test]
fn test_update_sync_only_newer() {
    let (_source_dir, _dest_dir, source, dest) = setup_test_environment();

    // Create newer file in source
    create_test_file(&source, "newer.txt", "newer content");

    // Create older file in destination with different content
    create_test_file(&dest, "older.txt", "older content");

    // Run update sync
    let progress = ProgressTracker::new();
    let result = run_backup(&source, &dest, SyncMode::Update, &[], &progress, false)
        .expect("Backup should succeed");

    // Verify only newer file was copied
    assert_eq!(result.copied, 1, "Should copy 1 file");
    assert!(dest.join("newer.txt").exists());
    assert!(dest.join("older.txt").exists()); // Should still exist
}

#[test]
fn test_exclude_patterns() {
    let (_source_dir, _dest_dir, source, dest) = setup_test_environment();

    // Create test files including some to exclude
    create_test_file(&source, "include.txt", "include");
    create_test_file(&source, "exclude.tmp", "exclude");
    create_test_file(&source, "node_modules/pkg.js", "pkg");
    create_test_file(&source, "src/main.rs", "main");

    // Run backup with exclude patterns
    let progress = ProgressTracker::new();
    let exclude = vec!["*.tmp".to_string(), "node_modules".to_string()];
    let result = run_backup(&source, &dest, SyncMode::Mirror, &exclude, &progress, false)
        .expect("Backup should succeed");

    // Verify excluded files were not copied
    assert_eq!(result.copied, 2, "Should copy 2 files (excluding patterns)");
    assert!(dest.join("include.txt").exists());
    assert!(dest.join("src/main.rs").exists());
    assert!(!dest.join("exclude.tmp").exists());
    assert!(!dest.join("node_modules/pkg.js").exists());
}

#[test]
fn test_dry_run_no_changes() {
    let (_source_dir, _dest_dir, source, dest) = setup_test_environment();

    // Create test files
    create_test_file(&source, "file1.txt", "content1");

    // Run dry run
    let progress = ProgressTracker::new();
    let result = run_backup(&source, &dest, SyncMode::Mirror, &[], &progress, true)
        .expect("Dry run should succeed");

    // Verify no files were actually copied
    assert_eq!(result.copied, 1, "Should report 1 file to copy");
    assert!(
        !dest.join("file1.txt").exists(),
        "Dry run should not create files"
    );
}

#[test]
fn test_empty_source_directory() {
    let (_source_dir, _dest_dir, source, dest) = setup_test_environment();

    // Run backup on empty source
    let progress = ProgressTracker::new();
    let result = run_backup(&source, &dest, SyncMode::Mirror, &[], &progress, false)
        .expect("Backup should succeed");

    // Verify nothing was copied
    assert_eq!(result.copied, 0);
    assert_eq!(result.deleted, 0);
    assert_eq!(result.skipped, 0);
}

#[test]
fn test_nonexistent_source_fails() {
    let (_source_dir, dest_dir, _source, _dest) = setup_test_environment();
    let nonexistent = dest_dir.path().join("does_not_exist");

    let progress = ProgressTracker::new();
    let result = run_backup(
        &nonexistent,
        dest_dir.path(),
        SyncMode::Mirror,
        &[],
        &progress,
        false,
    );

    assert!(result.is_err(), "Should fail with nonexistent source");
}
