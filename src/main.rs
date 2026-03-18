mod backup;
mod cli;
mod tui;

use std::io;
use std::process::ExitCode;

use clap::Parser;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use backup::sync::SyncMode as BackupSyncMode;
use cli::{Args, SyncMode};
use tui::App;

fn main() -> ExitCode {
    // Initialize tracing with RUST_LOG environment variable support
    init_tracing();

    let args = Args::parse();

    info!("🚀 Dirloom starting");
    info!("📁 Source: {}", args.source.display());
    info!("📁 Destination: {}", args.destination.display());
    info!("🔄 Mode: {:?}", args.mode);

    // Validate paths
    if !args.source.exists() {
        eprintln!(
            "❌ Error: Source directory does not exist: {}",
            args.source.display()
        );
        return ExitCode::FAILURE;
    }

    if !args.source.is_dir() {
        eprintln!(
            "❌ Error: Source is not a directory: {}",
            args.source.display()
        );
        return ExitCode::FAILURE;
    }

    // Convert CLI sync mode to backup sync mode
    let mode = match args.mode {
        SyncMode::Mirror => BackupSyncMode::Mirror,
        SyncMode::Incremental => BackupSyncMode::Incremental,
        SyncMode::Update => BackupSyncMode::Update,
    };

    // Create and run the TUI application
    let mut app = App::new(
        args.source,
        args.destination,
        mode,
        args.exclude,
        args.dry_run,
    );

    match run_tui(&mut app) {
        Ok(_) => {
            info!("👋 Dirloom exiting");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
            ExitCode::FAILURE
        }
    }
}

/// Initialize tracing subscriber with RUST_LOG environment variable support
fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info").add_directive("dirloom=debug".parse().unwrap()));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .with_thread_names(false)
                .pretty()
                .with_ansi(true),
        )
        .init();
}

/// Run the TUI application with proper terminal setup
fn run_tui(app: &mut App) -> backup::Result<()> {
    // Setup terminal
    terminal::enable_raw_mode().map_err(|e| backup::BackupError::ReadEntryFailed(e.to_string()))?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)
        .map_err(|e| backup::BackupError::ReadEntryFailed(e.to_string()))?;
    let backend = CrosstermBackend::new(stdout);
    let terminal =
        Terminal::new(backend).map_err(|e| backup::BackupError::ReadEntryFailed(e.to_string()))?;

    // Run app and capture result
    let result = app.run(terminal);

    // Restore terminal
    terminal::disable_raw_mode()
        .map_err(|e| backup::BackupError::ReadEntryFailed(e.to_string()))?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)
        .map_err(|e| backup::BackupError::ReadEntryFailed(e.to_string()))?;

    result.map_err(|e| backup::BackupError::ReadEntryFailed(e.to_string()))
}
