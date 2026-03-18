//! Directory picker dialog for selecting source and destination directories.
//!
//! Best practices implemented:
//! - Directories-first listing with alphabetical sorting
//! - Clear visual feedback for selected item
//! - Modal overlay with solid background
//! - Keyboard navigation (j/k, Enter, Backspace, Esc)
//! - Current path display at top

use std::fs;
use std::path::{Path, PathBuf};

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Directory picker dialog state
#[derive(Debug, Clone)]
pub struct DirPicker {
    /// Current directory being browsed
    current_dir: PathBuf,
    /// Selected index in the list
    selected: usize,
    /// List of entries (directories first, then files)
    entries: Vec<DirEntry>,
    /// Whether the picker is active
    pub active: bool,
    /// Result path if confirmed (None if cancelled)
    pub result: Option<PathBuf>,
}

/// Directory entry for display
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_parent: bool,
}

impl DirPicker {
    /// Create a new directory picker starting at the given path
    pub fn new(start_path: PathBuf) -> Self {
        let mut picker = Self {
            current_dir: start_path.clone(),
            selected: 0,
            entries: Vec::new(),
            active: true,
            result: None,
        };
        picker.refresh_entries();
        picker
    }

    /// Refresh the directory entries
    fn refresh_entries(&mut self) {
        self.entries.clear();

        // Add parent directory entry if not at root
        if self.current_dir.parent().is_some() {
            self.entries.push(DirEntry {
                name: "..".to_string(),
                path: self.current_dir.parent().unwrap().to_path_buf(),
                is_dir: true,
                is_parent: true,
            });
        }

        // Read directory contents
        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            let mut dirs: Vec<DirEntry> = Vec::new();
            let mut files: Vec<DirEntry> = Vec::new();

            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path();
                let is_dir = path.is_dir();

                // Skip hidden files (starting with .)
                if name.starts_with('.') && !is_dir {
                    continue;
                }

                let dir_entry = DirEntry {
                    name,
                    path,
                    is_dir,
                    is_parent: false,
                };

                if is_dir {
                    dirs.push(dir_entry);
                } else {
                    files.push(dir_entry);
                }
            }

            // Sort directories first (alphabetically), then files
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            self.entries.extend(dirs);
            self.entries.extend(files);
        }

        // Reset selection if out of bounds
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
    }

    /// Navigate up to parent directory
    pub fn go_up(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.refresh_entries();
            self.selected = 0;
        }
    }

    /// Navigate into selected directory
    pub fn enter_directory(&mut self) {
        if let Some(entry) = self.entries.get(self.selected)
            && entry.is_dir
        {
            self.current_dir = entry.path.clone();
            self.refresh_entries();
            self.selected = 0;
        }
    }

    /// Select current directory as the result
    pub fn select_current(&mut self) {
        self.result = Some(self.current_dir.clone());
        self.active = false;
    }

    /// Navigate into selected directory or select file
    #[allow(dead_code)]
    pub fn select(&mut self) -> bool {
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                self.current_dir = entry.path.clone();
                self.refresh_entries();
                self.selected = 0;
                false // Continue browsing
            } else {
                // Select file (for future use, currently directories only)
                self.result = Some(entry.path.clone());
                self.active = false;
                true // Done
            }
        } else {
            false
        }
    }

    /// Confirm current directory as selection
    #[allow(dead_code)]
    pub fn confirm(&mut self) {
        self.result = Some(self.current_dir.clone());
        self.active = false;
    }

    /// Cancel selection
    pub fn cancel(&mut self) {
        self.result = None;
        self.active = false;
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected < self.entries.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Get the current directory path
    pub fn current_path(&self) -> &Path {
        &self.current_dir
    }

    /// Get the selected entry
    #[allow(dead_code)]
    pub fn selected_entry(&self) -> Option<&DirEntry> {
        self.entries.get(self.selected)
    }
}

/// Warm color palette for directory picker
mod colors {
    use ratatui::style::Color;

    pub const PEACH: Color = Color::Rgb(255, 170, 128);
    pub const AMBER: Color = Color::Rgb(255, 184, 66);
    pub const ROSE: Color = Color::Rgb(255, 121, 121);
    pub const SUCCESS: Color = Color::Rgb(255, 184, 108);
    pub const BG_DARKEST: Color = Color::Rgb(30, 25, 30);
    pub const FG_SECONDARY: Color = Color::Rgb(160, 145, 150);
}

/// Draw the directory picker modal dialog
pub fn draw_dir_picker(frame: &mut Frame, picker: &DirPicker, area: Rect) {
    // Create centered modal area (80% width, 80% height)
    let modal_area = centered_rect(80, 80, area);

    // First, clear the area to prevent background bleed-through
    frame.render_widget(Clear, modal_area);

    // Create the modal block with solid background
    let modal_block = Block::default()
        .title(" 📁 Select Directory ")
        .title_style(
            Style::default()
                .fg(colors::AMBER)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::FG_SECONDARY))
        .style(Style::default().bg(colors::BG_DARKEST));

    // Render the modal background first
    frame.render_widget(modal_block.clone(), modal_area);

    // Calculate inner area (excluding borders)
    let inner_area = modal_block.inner(modal_area);

    // Split inner area into sections
    let chunks = Layout::vertical([
        Constraint::Length(2), // Title/path
        Constraint::Min(5),    // Directory list
        Constraint::Length(2), // Help
    ])
    .margin(1)
    .split(inner_area);

    // Draw title with current path
    let path_str = picker.current_path().display().to_string();
    let title = Paragraph::new(Line::from(vec![
        Span::styled("📁 ", Style::default().fg(colors::AMBER)),
        Span::styled("Select: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(&path_str),
    ]))
    .style(Style::default().bg(colors::BG_DARKEST))
    .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(title, chunks[0]);

    // Draw directory list
    let items: Vec<ListItem> = picker
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == picker.selected {
                Style::default()
                    .bg(colors::PEACH)
                    .fg(colors::BG_DARKEST)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_dir {
                Style::default().fg(colors::SUCCESS).bg(colors::BG_DARKEST)
            } else {
                Style::default()
                    .fg(colors::FG_SECONDARY)
                    .bg(colors::BG_DARKEST)
            };

            let icon = if entry.is_parent {
                "📂"
            } else if entry.is_dir {
                "📁"
            } else {
                "📄"
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", icon), style),
                Span::styled(&entry.name, style),
            ]))
        })
        .collect();

    let list = List::new(items).style(Style::default().bg(colors::BG_DARKEST));
    frame.render_widget(list, chunks[1]);

    // Draw help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled(
            "↑/↓",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::PEACH),
        ),
        Span::raw(" Navigate  "),
        Span::styled(
            "Enter",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::PEACH),
        ),
        Span::raw(" Open  "),
        Span::styled(
            "Tab/Space",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::SUCCESS),
        ),
        Span::raw(" Select  "),
        Span::styled(
            "Backspace",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::PEACH),
        ),
        Span::raw(" Up  "),
        Span::styled(
            "Esc",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::ROSE),
        ),
        Span::raw(" Cancel"),
    ]))
    .alignment(Alignment::Center)
    .style(
        Style::default()
            .fg(colors::FG_SECONDARY)
            .bg(colors::BG_DARKEST),
    );
    frame.render_widget(help, chunks[2]);
}

/// Create a centered rectangle with given percentage of area
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    let popup_layout = Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1]);

    popup_layout[1]
}
