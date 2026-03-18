//! Backup module for directory synchronization operations.
//!
//! This module provides functionality for backing up directories with different
//! synchronization modes: mirror, incremental, and update.

pub mod core;
pub mod error;
pub mod progress;
pub mod sync;

pub use error::{BackupError, Result};
pub use progress::{Progress, ProgressTracker};
pub use sync::{SyncMode, run_backup};
