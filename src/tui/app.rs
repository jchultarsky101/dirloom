use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use ratatui::DefaultTerminal;
use tracing::{error, info};

use crate::backup::{Progress, ProgressTracker, SyncMode, run_backup};

use super::dir_picker::DirPicker;
use super::log_buffer::LogBuffer;

/// Application states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Initial state, ready to start backup
    Ready,
    /// Backup is running
    Running,
    /// Backup completed successfully
    Complete,
    /// Backup was cancelled
    Cancelled,
    /// Backup encountered errors
    Error,
    /// Directory picker is active
    PickingDirectory,
}

/// Which field is being edited
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Source,
    Destination,
}

/// Application state and logic
pub struct App {
    /// Source directory path
    pub source: PathBuf,
    /// Destination directory path
    pub destination: PathBuf,
    /// Sync mode
    pub mode: SyncMode,
    /// Exclude patterns
    pub exclude: Vec<String>,
    /// Dry run mode
    pub dry_run: bool,
    /// Current application state
    pub state: AppState,
    /// Progress tracker
    pub progress_tracker: ProgressTracker,
    /// Current progress snapshot
    pub progress: Progress,
    /// Error message if any
    pub error_message: Option<String>,
    /// Signal to cancel operation
    pub cancel_flag: Arc<AtomicBool>,
    /// Whether to exit the application
    pub should_exit: bool,
    /// Log buffer for TUI display
    pub log_buffer: LogBuffer,
    /// Directory picker dialog
    pub dir_picker: Option<DirPicker>,
    /// Which field is being picked (Source or Destination)
    pub edit_field: Option<EditField>,
    /// Whether help dialog is shown
    pub show_help: bool,
}

impl App {
    /// Create a new App instance
    pub fn new(
        source: PathBuf,
        destination: PathBuf,
        mode: SyncMode,
        exclude: Vec<String>,
        dry_run: bool,
    ) -> Self {
        Self {
            source,
            destination,
            mode,
            exclude,
            dry_run,
            state: AppState::Ready,
            progress_tracker: ProgressTracker::new(),
            progress: Progress::default(),
            error_message: None,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            should_exit: false,
            log_buffer: LogBuffer::new(),
            dir_picker: None,
            edit_field: None,
            show_help: false,
        }
    }

    /// Open directory picker for the specified field
    pub fn open_dir_picker(&mut self, field: EditField) {
        let start_path = match field {
            EditField::Source => self.source.clone(),
            EditField::Destination => self.destination.clone(),
        };
        self.dir_picker = Some(DirPicker::new(start_path));
        self.edit_field = Some(field);
        self.state = AppState::PickingDirectory;
    }

    /// Close directory picker and apply result
    pub fn close_dir_picker(&mut self) {
        if let Some(picker) = self.dir_picker.take()
            && let Some(result) = picker.result
        {
            match self.edit_field {
                Some(EditField::Source) => self.source = result,
                Some(EditField::Destination) => self.destination = result,
                None => {}
            }
        }
        self.edit_field = None;
        self.state = AppState::Ready;
    }

    /// Start the backup operation in a background thread
    pub fn start_backup(&mut self) {
        self.state = AppState::Running;
        self.progress_tracker.reset();
        self.cancel_flag.store(false, Ordering::Relaxed);
        self.error_message = None;

        let source = self.source.clone();
        let destination = self.destination.clone();
        let mode = self.mode;
        let exclude = self.exclude.clone();
        let dry_run = self.dry_run;
        let progress_tracker = self.progress_tracker.clone();

        thread::spawn(move || {
            info!("🔄 Backup thread started");
            let result = run_backup(
                &source,
                &destination,
                mode,
                &exclude,
                &progress_tracker,
                dry_run,
            );

            match result {
                Ok(_) => {
                    info!("✅ Backup thread completed successfully");
                }
                Err(e) => {
                    error!("❌ Backup thread error: {}", e);
                    // Store error message - we'll check this in the main thread
                }
            }
        });
    }

    /// Cancel the running backup
    pub fn cancel(&mut self) {
        if self.state == AppState::Running {
            self.cancel_flag.store(true, Ordering::Relaxed);
            self.state = AppState::Cancelled;
            self.progress_tracker.complete();
            info!("⏹️ Backup cancelled by user");
        }
    }

    /// Update progress from the tracker
    pub fn update_progress(&mut self) {
        self.progress = self.progress_tracker.get_progress();

        // Check if backup is complete
        if self.state == AppState::Running && self.progress.is_complete {
            if self.progress.errors > 0 {
                self.state = AppState::Error;
                self.error_message = Some(format!(
                    "Backup completed with {} error(s)",
                    self.progress.errors
                ));
            } else {
                self.state = AppState::Complete;
            }
        }
    }

    /// Handle key events
    pub fn handle_key(&mut self, key: KeyCode) {
        // Handle directory picker input
        if self.state == AppState::PickingDirectory {
            if let Some(picker) = self.dir_picker.as_mut() {
                match key {
                    KeyCode::Up | KeyCode::Char('k') => picker.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => picker.move_down(),
                    KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                        // Enter navigates into directory
                        picker.enter_directory();
                    }
                    KeyCode::Tab | KeyCode::Char(' ') => {
                        // Tab or Space selects current directory
                        picker.select_current();
                        self.close_dir_picker();
                    }
                    KeyCode::Backspace | KeyCode::Left | KeyCode::Char('h') => {
                        picker.go_up();
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        picker.cancel();
                        self.close_dir_picker();
                    }
                    _ => {}
                }
            }
            return;
        }

        // Handle normal app input
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                if self.state == AppState::Running {
                    self.cancel();
                } else {
                    self.should_exit = true;
                }
            }
            KeyCode::Char('c') | KeyCode::Char(' ') => {
                if self.state == AppState::Ready || self.state == AppState::Complete {
                    self.start_backup();
                }
            }
            KeyCode::Char('r') => {
                if self.state == AppState::Complete
                    || self.state == AppState::Cancelled
                    || self.state == AppState::Error
                {
                    self.state = AppState::Ready;
                    self.progress_tracker.reset();
                    self.progress = Progress::default();
                    self.error_message = None;
                }
            }
            // Open directory picker for source (1) or destination (2)
            KeyCode::Char('1') => {
                if self.state == AppState::Ready {
                    self.open_dir_picker(EditField::Source);
                }
            }
            KeyCode::Char('2') => {
                if self.state == AppState::Ready {
                    self.open_dir_picker(EditField::Destination);
                }
            }
            KeyCode::Char('s') => {
                // Shortcut for source picker
                if self.state == AppState::Ready {
                    self.open_dir_picker(EditField::Source);
                }
            }
            KeyCode::Char('d') => {
                // Shortcut for destination picker
                if self.state == AppState::Ready {
                    self.open_dir_picker(EditField::Destination);
                }
            }
            KeyCode::Char('m') => {
                // Cycle through backup modes: Mirror → Incremental → Update → Force → Mirror
                if self.state == AppState::Ready {
                    self.mode = match self.mode {
                        SyncMode::Mirror => SyncMode::Incremental,
                        SyncMode::Incremental => SyncMode::Update,
                        SyncMode::Update => SyncMode::Force,
                        SyncMode::Force => SyncMode::Mirror,
                    };
                }
            }
            KeyCode::Char('?') | KeyCode::Char('h') => {
                // Toggle help dialog
                if self.state == AppState::Ready {
                    self.show_help = !self.show_help;
                }
            }
            _ => {}
        }
    }

    /// Run the TUI application
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize TUI tracing with log buffer
        super::log_buffer::init_tui_tracing(self.log_buffer.clone());

        loop {
            terminal.draw(|frame| {
                crate::tui::ui::draw(frame, self);
            })?;

            // Update progress
            self.update_progress();

            // Check for exit conditions
            if self.should_exit {
                break;
            }

            // Handle events with timeout for smooth progress updates
            if event::poll(std::time::Duration::from_millis(100))?
                && let Ok(Event::Key(key)) = event::read()
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key.code);
            }
        }

        Ok(())
    }

    /// Get the title for the current state
    pub fn title(&self) -> &'static str {
        match self.state {
            AppState::Ready => "🌻 Dirloom - Ready",
            AppState::Running => "🌻 Dirloom - Backing up…",
            AppState::Complete => "🌻 Dirloom - Complete ✨",
            AppState::Cancelled => "🌻 Dirloom - Cancelled",
            AppState::Error => "🌻 Dirloom - Error",
            AppState::PickingDirectory => "🌻 Dirloom - Select Directory",
        }
    }

    /// Get status text for the current state
    #[allow(dead_code)]
    pub fn status_text(&self) -> &'static str {
        match self.state {
            AppState::Ready => "Press [Space] to start backup, [q] to quit",
            AppState::Running => "Press [q] to cancel, [Space] to restart when complete",
            AppState::Complete => "Press [r] to reset, [q] to quit",
            AppState::Cancelled => "Press [r] to reset, [q] to quit",
            AppState::Error => "Press [r] to reset, [q] to quit",
            AppState::PickingDirectory => "Navigate with ↑/↓, Enter to select, Esc to cancel",
        }
    }
}

// Re-export crossterm event types for convenience
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
