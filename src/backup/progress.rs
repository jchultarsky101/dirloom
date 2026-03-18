use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

/// Progress information for backup operations
#[derive(Debug, Clone, Default)]
pub struct Progress {
    /// Total number of files to process
    pub total_files: u64,
    /// Number of files processed so far
    pub processed_files: u64,
    /// Total bytes to transfer
    pub total_bytes: u64,
    /// Bytes transferred so far
    pub processed_bytes: u64,
    /// Current file being processed
    pub current_file: Option<String>,
    /// Number of files copied
    pub copied: u64,
    /// Number of files deleted
    pub deleted: u64,
    /// Number of files skipped
    pub skipped: u64,
    /// Number of errors encountered
    pub errors: u64,
    /// Whether the operation is complete
    pub is_complete: bool,
}

impl Progress {
    /// Create a new Progress instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate the overall progress percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            return 100.0;
        }
        (self.processed_bytes as f64 / self.total_bytes as f64) * 100.0
    }

    /// Calculate the file progress percentage (0-100)
    pub fn file_percentage(&self) -> f64 {
        if self.total_files == 0 {
            return 100.0;
        }
        (self.processed_files as f64 / self.total_files as f64) * 100.0
    }

    /// Get the current status message
    pub fn status_message(&self) -> String {
        if self.is_complete {
            return "✨ Complete".to_string();
        }

        if let Some(ref current) = self.current_file {
            // Truncate long filenames
            let display_name = if current.len() > 50 {
                format!("…{}", &current[current.len() - 47..])
            } else {
                current.clone()
            };
            format!("📄 Processing: {}", display_name)
        } else {
            "🔄 Initializing…".to_string()
        }
    }

    /// Get a summary of operations
    pub fn summary(&self) -> String {
        format!(
            "✅ Copied: {} | 🗑️ Deleted: {} | ⏭️ Skipped: {} | ❌ Errors: {}",
            self.copied, self.deleted, self.skipped, self.errors
        )
    }
}

/// Thread-safe progress tracker
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    progress: Arc<Mutex<Progress>>,
}

impl ProgressTracker {
    /// Create a new ProgressTracker
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(Progress::new())),
        }
    }

    /// Get a clone of the current progress
    pub fn get_progress(&self) -> Progress {
        self.progress.lock().map(|p| p.clone()).unwrap_or_default()
    }

    /// Update the total files count
    pub fn set_total(&self, total_files: u64, total_bytes: u64) {
        if let Ok(mut p) = self.progress.lock() {
            p.total_files = total_files;
            p.total_bytes = total_bytes;
            debug!("📊 Total: {} files, {} bytes", total_files, total_bytes);
        }
    }

    /// Mark a file as started processing
    pub fn start_file(&self, path: &str) {
        if let Ok(mut p) = self.progress.lock() {
            p.current_file = Some(path.to_string());
        }
        trace_file_action("📄", "Processing", path);
    }

    /// Mark a file as processed
    pub fn file_processed(&self, bytes: u64, action: ProgressAction) {
        if let Ok(mut p) = self.progress.lock() {
            p.processed_files += 1;
            p.processed_bytes += bytes;
            match action {
                ProgressAction::Copied => {
                    p.copied += 1;
                    debug!("✅ Copied ({} bytes)", bytes);
                }
                ProgressAction::Deleted => {
                    p.deleted += 1;
                    debug!("🗑️ Deleted");
                }
                ProgressAction::Skipped => {
                    p.skipped += 1;
                    debug!("⏭️ Skipped");
                }
            }
        }
    }

    /// Record an error
    pub fn record_error(&self) {
        if let Ok(mut p) = self.progress.lock() {
            p.errors += 1;
            error!("❌ Error occurred");
        }
    }

    /// Mark the operation as complete
    pub fn complete(&self) {
        if let Ok(mut p) = self.progress.lock() {
            p.is_complete = true;
            p.current_file = None;
            info!(
                "✨ Backup complete: {} copied, {} deleted, {} skipped, {} errors",
                p.copied, p.deleted, p.skipped, p.errors
            );
        }
    }

    /// Reset the progress tracker
    pub fn reset(&self) {
        if let Ok(mut p) = self.progress.lock() {
            *p = Progress::new();
            debug!("🔄 Progress reset");
        }
    }
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Action taken on a file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressAction {
    Copied,
    Deleted,
    Skipped,
}

/// Trace a file action with emoji
fn trace_file_action(emoji: &str, action: &str, path: &str) {
    let display_path = if path.len() > 60 {
        format!("…{}", &path[path.len() - 57..])
    } else {
        path.to_string()
    };
    debug!("{} {} {}", emoji, action, display_path);
}
