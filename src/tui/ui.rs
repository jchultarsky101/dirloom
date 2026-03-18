use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

use super::app::{App, AppState, EditField};

/// Warm color palette for dark theme TUI
/// Based on best practices: https://github.com/catppuccin/catppuccin, https://github.com/rose-pine/rose-pine-theme
mod colors {
    use ratatui::style::Color;

    // Warm accent colors
    pub const PEACH: Color = Color::Rgb(255, 170, 128); // #FFAA80 - Primary accent
    pub const ORANGE: Color = Color::Rgb(255, 153, 51); // #FF9933 - Secondary accent
    pub const AMBER: Color = Color::Rgb(255, 184, 66); // #FFB842 - Warnings/highlights
    pub const ROSE: Color = Color::Rgb(255, 121, 121); // #FF7979 - Errors
    pub const PINK: Color = Color::Rgb(255, 158, 158); // #FF9E9E - Alternative error

    // Neutral colors for dark background
    pub const BG_DARKEST: Color = Color::Rgb(30, 25, 30); // #1E191E - Main background
    pub const FG_PRIMARY: Color = Color::Rgb(230, 220, 220); // #E6DCDC - Primary text
    pub const FG_SECONDARY: Color = Color::Rgb(160, 145, 150); // #A09196 - Secondary text

    // Status colors (warm variants)
    pub const SUCCESS: Color = Color::Rgb(255, 184, 108); // #FFB86C - Warm green alternative
    pub const WARNING: Color = Color::Rgb(255, 200, 100); // #FFC864 - Warm yellow
    pub const RUNNING: Color = Color::Rgb(255, 170, 128); // #FFAA80 - Warm cyan alternative
}

/// Draw the main UI
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Length(5), // Compact backup config
        Constraint::Length(3), // Progress gauge
        Constraint::Length(5), // Statistics (expanded to include file counts)
        Constraint::Min(5),    // Log panel
        Constraint::Length(3), // Help
    ])
    .split(frame.area());

    draw_title(frame, app, chunks[0]);
    draw_backup_config(frame, app, chunks[1]);
    draw_progress(frame, app, chunks[2]);
    draw_summary(frame, app, chunks[3]);
    draw_log_panel(frame, app, chunks[4]);
    draw_help(frame, app, chunks[5]);

    // Draw directory picker modal if active
    if app.state == AppState::PickingDirectory
        && let Some(picker) = &app.dir_picker
    {
        super::dir_picker::draw_dir_picker(frame, picker, frame.area());
    }

    // Draw help dialog modal if active
    if app.show_help {
        super::help::draw_help_dialog(frame, frame.area());
    }
}

/// Draw the title bar with state indicator
fn draw_title(frame: &mut Frame, app: &App, area: Rect) {
    let title = app.title();

    let state_color = match app.state {
        AppState::Ready => colors::AMBER,
        AppState::Running => colors::RUNNING,
        AppState::Complete => colors::SUCCESS,
        AppState::Cancelled => colors::WARNING,
        AppState::Error => colors::ROSE,
        AppState::PickingDirectory => colors::PEACH,
    };

    let state_indicator = match app.state {
        AppState::Ready => "●",
        AppState::Running => "◐",
        AppState::Complete => "●",
        AppState::Cancelled => "○",
        AppState::Error => "●",
        AppState::PickingDirectory => "◉",
    };

    let paragraph = Paragraph::new(Line::from(vec![
        Span::styled(state_indicator, Style::default().fg(state_color)),
        Span::raw(" "),
        Span::raw(title),
    ]))
    .style(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .add_modifier(Modifier::ITALIC),
    )
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(state_color))
            .style(Style::default().bg(colors::BG_DARKEST)),
    );

    frame.render_widget(paragraph, area);
}

/// Draw compact backup configuration panel
fn draw_backup_config(frame: &mut Frame, app: &App, area: Rect) {
    let source = app.source.display().to_string();
    let dest = app.destination.display().to_string();
    let dry_run = if app.dry_run { " [DRY RUN]" } else { "" };
    let status_msg = app.progress.status_message();

    let mode_color = match app.mode {
        crate::backup::SyncMode::Mirror => colors::PINK,
        crate::backup::SyncMode::Incremental => colors::PEACH,
        crate::backup::SyncMode::Update => colors::ORANGE,
        crate::backup::SyncMode::Force => colors::ROSE,
    };

    // Compact layout: source → dest on one line
    let path_line = Line::from(vec![
        Span::styled("📁 ", Style::default()),
        Span::raw(&source),
        Span::styled(" → ", Style::default().fg(colors::FG_SECONDARY)),
        Span::raw(&dest),
    ]);

    // Mode selector on second line with visual indicator
    let mode_display = match app.mode {
        crate::backup::SyncMode::Mirror => "🪞 Mirror",
        crate::backup::SyncMode::Incremental => "📈 Incremental",
        crate::backup::SyncMode::Update => "🔄 Update",
        crate::backup::SyncMode::Force => "⚡ Force",
    };

    let mut mode_spans = vec![
        Span::styled("🔄 Mode: ", Style::default()),
        Span::styled(
            mode_display,
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(dry_run),
        Span::styled(" [m]", Style::default().fg(colors::FG_SECONDARY)),
    ];

    if !app.exclude.is_empty() {
        mode_spans.push(Span::raw("  "));
        mode_spans.push(Span::styled("🚫 ", Style::default()));
        mode_spans.push(Span::raw(app.exclude.join(", ")));
    }
    let mode_line = Line::from(mode_spans);

    // Status indicator - show picker hint when picking
    let status = match app.state {
        AppState::PickingDirectory => {
            let field_name = match app.edit_field {
                Some(EditField::Source) => "Source",
                Some(EditField::Destination) => "Destination",
                None => "Directory",
            };
            vec![
                Span::styled("📁 ", Style::default().fg(colors::PEACH)),
                Span::raw(format!("Selecting {}... Use ↑/↓ to navigate", field_name)),
            ]
        }
        AppState::Ready => vec![
            Span::styled("💡 ", Style::default().fg(colors::AMBER)),
            Span::raw("Press Space to start"),
        ],
        AppState::Running => vec![
            Span::styled("⏳ ", Style::default().fg(colors::RUNNING)),
            Span::raw(&status_msg),
        ],
        AppState::Complete => vec![
            Span::styled("✅ ", Style::default().fg(colors::SUCCESS)),
            Span::raw("Complete!"),
        ],
        AppState::Cancelled => vec![
            Span::styled("⏹️ ", Style::default().fg(colors::WARNING)),
            Span::raw("Cancelled"),
        ],
        AppState::Error => vec![
            Span::styled("❌ ", Style::default().fg(colors::ROSE)),
            Span::raw(app.error_message.as_deref().unwrap_or("Error")),
        ],
    };

    let border_color = match app.state {
        AppState::Ready => colors::AMBER,
        AppState::Running => colors::RUNNING,
        AppState::Complete => colors::SUCCESS,
        AppState::Cancelled => colors::WARNING,
        AppState::Error => colors::ROSE,
        AppState::PickingDirectory => colors::PEACH,
    };

    let paragraph = Paragraph::new(vec![
        Line::from(""),
        path_line,
        mode_line,
        Line::from(status),
    ])
    .wrap(Wrap { trim: true })
    .block(
        Block::default()
            .title(" 📋 Backup ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(paragraph, area);
}

/// Draw the main progress gauge
fn draw_progress(frame: &mut Frame, app: &App, area: Rect) {
    let percentage = app.progress.percentage();

    // Show clean initial state when backup hasn't started
    let label = if app.state == AppState::Ready && app.progress.total_files == 0 {
        "Waiting to start…".to_string()
    } else if app.progress.total_files > 0 {
        format!(
            "{:.1}% ({}/{})",
            percentage, app.progress.processed_files, app.progress.total_files
        )
    } else if app.progress.is_complete {
        "✨ Complete!".to_string()
    } else {
        format!("{:.1}%", percentage)
    };

    let gauge_color = if app.state == AppState::Ready && app.progress.total_files == 0 {
        colors::FG_SECONDARY
    } else if app.progress.percentage() >= 100.0 {
        colors::SUCCESS
    } else {
        colors::RUNNING
    };

    // Create gauge with dark background for better label contrast
    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color).bg(colors::BG_DARKEST))
        .label(Span::styled(
            label,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .ratio((percentage / 100.0).min(1.0))
        .block(
            Block::default()
                .title(" 📊 Progress ")
                .borders(Borders::ALL),
        );

    frame.render_widget(gauge, area);
}

/// Draw the summary statistics with file counts
fn draw_summary(frame: &mut Frame, app: &App, area: Rect) {
    // Show clean initial state when backup hasn't started
    if app.state == AppState::Ready && app.progress.total_files == 0 {
        let items = vec![ListItem::new(Line::from(vec![Span::styled(
            "Press Space to start backup",
            Style::default().fg(colors::FG_SECONDARY),
        )]))];

        let list = List::new(items).block(
            Block::default()
                .title(" 📈 Statistics ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::FG_SECONDARY)),
        );

        frame.render_widget(list, area);
        return;
    }

    // Two rows: first row has file stats, second row has action stats
    let items = vec![
        // Row 1: File statistics
        ListItem::new(Line::from(vec![
            Span::styled(
                format!("📄 Total: {}  ", app.progress.total_files),
                Style::default()
                    .fg(colors::FG_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("Processed: {}", app.progress.processed_files),
                Style::default().fg(colors::FG_SECONDARY),
            ),
        ])),
        // Row 2: Action statistics
        ListItem::new(Line::from(vec![
            Span::styled("✅ ", Style::default().fg(colors::SUCCESS)),
            Span::raw(format!("Copied: {}  ", app.progress.copied)),
            Span::styled("🗑️ ", Style::default().fg(colors::ROSE)),
            Span::raw(format!("Deleted: {}  ", app.progress.deleted)),
            Span::styled("⏭️ ", Style::default().fg(colors::WARNING)),
            Span::raw(format!("Skipped: {}  ", app.progress.skipped)),
            Span::styled("❌ ", Style::default().fg(colors::ROSE)),
            Span::raw(format!("Errors: {}", app.progress.errors)),
        ])),
    ];

    let border_color = if app.progress.errors > 0 {
        colors::ROSE
    } else if app.progress.is_complete {
        colors::SUCCESS
    } else {
        colors::FG_SECONDARY
    };

    let list = List::new(items).block(
        Block::default()
            .title(" 📈 Statistics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(list, area);
}

/// Draw the log panel
fn draw_log_panel(frame: &mut Frame, app: &App, area: Rect) {
    // Show clean initial state when backup hasn't started
    if app.state == AppState::Ready && app.progress.total_files == 0 {
        let lines = vec![Line::from(vec![Span::styled(
            "Log messages will appear here during backup…",
            Style::default().fg(colors::FG_SECONDARY),
        )])];

        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .title(" 📜 Activity Log ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::FG_SECONDARY)),
        );

        frame.render_widget(paragraph, area);
        return;
    }

    let log_lines = app.log_buffer.get_last(15);

    let lines: Vec<Line> = log_lines
        .iter()
        .map(|line| {
            // Parse emoji prefix for coloring
            if line.starts_with("❌") {
                Line::styled(line.clone(), Style::default().fg(colors::ROSE))
            } else if line.starts_with("⚠️") {
                Line::styled(line.clone(), Style::default().fg(colors::WARNING))
            } else if line.starts_with("✅") {
                Line::styled(line.clone(), Style::default().fg(colors::SUCCESS))
            } else if line.starts_with("🔍") {
                Line::styled(line.clone(), Style::default().fg(colors::PEACH))
            } else if line.starts_with("📝") {
                Line::styled(line.clone(), Style::default().fg(colors::PINK))
            } else if line.starts_with("🚀") {
                Line::styled(line.clone(), Style::default().fg(colors::PEACH))
            } else if line.starts_with("🪞") {
                Line::styled(line.clone(), Style::default().fg(colors::PINK))
            } else if line.starts_with("✨") {
                Line::styled(line.clone(), Style::default().fg(colors::SUCCESS))
            } else if line.starts_with("📁") {
                Line::styled(line.clone(), Style::default().fg(colors::FG_PRIMARY))
            } else if line.starts_with("🔄") {
                Line::styled(line.clone(), Style::default().fg(colors::ORANGE))
            } else if line.starts_with("⏳") {
                Line::styled(line.clone(), Style::default().fg(colors::RUNNING))
            } else if line.starts_with("💡") {
                Line::styled(line.clone(), Style::default().fg(colors::AMBER))
            } else if line.starts_with("⏹️") {
                Line::styled(line.clone(), Style::default().fg(colors::WARNING))
            } else {
                // Default styling for lines without emoji
                Line::raw(line.clone()).style(Style::default().fg(colors::FG_SECONDARY))
            }
        })
        .collect();

    let border_color = match app.state {
        AppState::Ready => colors::FG_SECONDARY,
        AppState::Running => colors::RUNNING,
        AppState::Complete => colors::SUCCESS,
        AppState::Cancelled => colors::WARNING,
        AppState::Error => colors::ROSE,
        AppState::PickingDirectory => colors::PEACH,
    };

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .title(" 📜 Activity Log ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(paragraph, area);
}

/// Draw the help bar
fn draw_help(frame: &mut Frame, app: &App, area: Rect) {
    // Show picker help when picking directory
    if app.state == AppState::PickingDirectory {
        let key_hints = vec![
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
                "Tab",
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
        ];

        let paragraph = Paragraph::new(Line::from(key_hints))
            .style(Style::default().fg(colors::FG_SECONDARY))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);
        return;
    }

    // Normal help text
    let key_hints = vec![
        Span::styled(
            "[Space]",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::PEACH),
        ),
        Span::raw(" Start  "),
        Span::styled(
            "[m]",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::PEACH),
        ),
        Span::raw(" Mode  "),
        Span::styled(
            "[1]",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::PEACH),
        ),
        Span::raw(" Source  "),
        Span::styled(
            "[2]",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::PEACH),
        ),
        Span::raw(" Dest  "),
        Span::styled(
            "[?]",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::AMBER),
        ),
        Span::raw(" Help  "),
        Span::styled(
            "[q]",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::ROSE),
        ),
        Span::raw(" Quit  "),
        Span::styled(
            "[r]",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::AMBER),
        ),
        Span::raw(" Reset"),
    ];

    let paragraph = Paragraph::new(Line::from(key_hints))
        .style(Style::default().fg(colors::FG_SECONDARY))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}
