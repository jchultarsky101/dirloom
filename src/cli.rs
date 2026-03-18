use std::path::PathBuf;

use clap::{Parser, ValueEnum};

/// A CLI TUI for backing up directories to another location
#[derive(Parser, Debug)]
#[command(name = "dirloom")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Source directory to backup
    #[arg(value_name = "SOURCE")]
    pub source: PathBuf,

    /// Destination directory for backup
    #[arg(value_name = "DESTINATION")]
    pub destination: PathBuf,

    /// Backup synchronization mode
    #[arg(short = 'm', long, value_enum, default_value_t = SyncMode::Mirror)]
    pub mode: SyncMode,

    /// Exclude patterns (can be used multiple times)
    #[arg(short = 'e', long, value_name = "PAT")]
    pub exclude: Vec<String>,

    /// Show what would be done without making changes
    #[arg(short = 'n', long, default_value_t = false)]
    pub dry_run: bool,

    /// Enable verbose output
    #[arg(short = 'v', long, default_value_t = false)]
    pub verbose: bool,
}

/// Backup synchronization modes
#[derive(ValueEnum, Clone, Debug, Default, PartialEq)]
pub enum SyncMode {
    /// Mirror sync: destination matches source exactly (deletes extra files)
    #[default]
    Mirror,

    /// Incremental: only copy changed or new files
    Incremental,

    /// Update: copy newer files, don't delete anything
    Update,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_arguments() {
        let args = Args::parse_from(["dirloom", "/source", "/dest"]);
        assert_eq!(args.source, PathBuf::from("/source"));
        assert_eq!(args.destination, PathBuf::from("/dest"));
        assert_eq!(args.mode, SyncMode::Mirror);
        assert!(!args.dry_run);
        assert!(!args.verbose);
        assert!(args.exclude.is_empty());
    }

    #[test]
    fn test_mirror_mode() {
        let args = Args::parse_from(["dirloom", "/source", "/dest", "--mode", "mirror"]);
        assert_eq!(args.mode, SyncMode::Mirror);
    }

    #[test]
    fn test_incremental_mode() {
        let args = Args::parse_from(["dirloom", "/source", "/dest", "--mode", "incremental"]);
        assert_eq!(args.mode, SyncMode::Incremental);
    }

    #[test]
    fn test_update_mode() {
        let args = Args::parse_from(["dirloom", "/source", "/dest", "--mode", "update"]);
        assert_eq!(args.mode, SyncMode::Update);
    }

    #[test]
    fn test_short_mode_flag() {
        let args = Args::parse_from(["dirloom", "/source", "/dest", "-m", "incremental"]);
        assert_eq!(args.mode, SyncMode::Incremental);
    }

    #[test]
    fn test_exclude_patterns() {
        let args = Args::parse_from([
            "dirloom",
            "/source",
            "/dest",
            "--exclude",
            "*.tmp",
            "--exclude",
            "node_modules",
        ]);
        assert_eq!(args.exclude, vec!["*.tmp", "node_modules"]);
    }

    #[test]
    fn test_short_exclude_flag() {
        let args = Args::parse_from(["dirloom", "/source", "/dest", "-e", "*.log"]);
        assert_eq!(args.exclude, vec!["*.log"]);
    }

    #[test]
    fn test_dry_run_flag() {
        let args = Args::parse_from(["dirloom", "/source", "/dest", "--dry-run"]);
        assert!(args.dry_run);
    }

    #[test]
    fn test_short_dry_run_flag() {
        let args = Args::parse_from(["dirloom", "/source", "/dest", "-n"]);
        assert!(args.dry_run);
    }

    #[test]
    fn test_verbose_flag() {
        let args = Args::parse_from(["dirloom", "/source", "/dest", "--verbose"]);
        assert!(args.verbose);
    }

    #[test]
    fn test_short_verbose_flag() {
        let args = Args::parse_from(["dirloom", "/source", "/dest", "-v"]);
        assert!(args.verbose);
    }

    #[test]
    fn test_combined_flags() {
        let args = Args::parse_from([
            "dirloom",
            "/source",
            "/dest",
            "-m",
            "incremental",
            "-e",
            "*.tmp",
            "-n",
            "-v",
        ]);
        assert_eq!(args.mode, SyncMode::Incremental);
        assert_eq!(args.exclude, vec!["*.tmp"]);
        assert!(args.dry_run);
        assert!(args.verbose);
    }

    #[test]
    fn test_relative_paths() {
        let args = Args::parse_from(["dirloom", "./src", "../backup"]);
        assert_eq!(args.source, PathBuf::from("./src"));
        assert_eq!(args.destination, PathBuf::from("../backup"));
    }

    #[test]
    fn test_home_directory_paths() {
        let args = Args::parse_from(["dirloom", "~/documents", "~/backups"]);
        assert_eq!(args.source, PathBuf::from("~/documents"));
        assert_eq!(args.destination, PathBuf::from("~/backups"));
    }
}
