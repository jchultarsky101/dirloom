use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
};

use super::app::{App, AppState};

/// Draw the main UI
pub fn draw(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // Title
        Constraint::Min(8),    // Main content
        Constraint::Length(3), // Progress gauge
        Constraint::Length(1), // File progress
        Constraint::Length(4), // Statistics
        Constraint::Length(3), // Help
    ])
    .split(frame.area());

    draw_title(frame, app, chunks[0]);
    draw_main_content(frame, app, chunks[1]);
    draw_progress(frame, app, chunks[2]);
    draw_file_progress(frame, app, chunks[3]);
    draw_summary(frame, app, chunks[4]);
    draw_help(frame, app, chunks[5]);
}

/// Draw the title bar with gradient-style styling
fn draw_title(frame: &mut Frame, app: &App, area: Rect) {
    let title = app.title();

    let state_color = match app.state {
        AppState::Ready => Color::Yellow,
        AppState::Running => Color::Cyan,
        AppState::Complete => Color::Green,
        AppState::Cancelled => Color::Yellow,
        AppState::Error => Color::Red,
    };

    let state_indicator = match app.state {
        AppState::Ready => "●",
        AppState::Running => "◐",
        AppState::Complete => "●",
        AppState::Cancelled => "○",
        AppState::Error => "●",
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
            .style(Style::default().bg(Color::Black)),
    );

    frame.render_widget(paragraph, area);
}

/// Draw the main content area with configuration and status
fn draw_main_content(frame: &mut Frame, app: &App, area: Rect) {
    let source_display = app.source.display().to_string();
    let dest_display = app.destination.display().to_string();
    let mode_text = format!("{:?}", app.mode);
    let dry_run_text = if app.dry_run {
        " 🏃‍♂️ Dry Run"
    } else {
        ""
    };

    let mode_style = match app.mode {
        crate::backup::SyncMode::Mirror => Color::Magenta,
        crate::backup::SyncMode::Incremental => Color::Blue,
        crate::backup::SyncMode::Update => Color::Cyan,
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "📁 Source:      ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(&source_display),
        ]),
        Line::from(vec![
            Span::styled(
                "📁 Destination: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(&dest_display),
        ]),
        Line::from(vec![
            Span::styled(
                "🔄 Mode:        ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::styled(&mode_text, Style::default().fg(mode_style)),
            Span::raw(dry_run_text),
        ]),
    ];

    if !app.exclude.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(
                "🚫 Excluded:    ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(app.exclude.join(", ")),
        ]));
    }

    lines.push(Line::from(""));

    match app.state {
        AppState::Ready => {
            lines.push(Line::from(vec![
                Span::styled("💡 ", Style::default().fg(Color::Yellow)),
                Span::raw("Ready to start backup…"),
            ]));
        }
        AppState::Running => {
            lines.push(Line::from(vec![
                Span::styled("⏳ ", Style::default().fg(Color::Cyan)),
                Span::raw(app.progress.status_message()),
            ]));
        }
        AppState::Complete => {
            lines.push(Line::from(vec![
                Span::styled("✅ ", Style::default().fg(Color::Green)),
                Span::raw("Backup completed successfully!"),
            ]));
            lines.push(Line::from(""));
            lines.push(Line::from(app.progress.summary()));
        }
        AppState::Cancelled => {
            lines.push(Line::from(vec![
                Span::styled("⏹️ ", Style::default().fg(Color::Yellow)),
                Span::raw("Backup was cancelled."),
            ]));
        }
        AppState::Error => {
            lines.push(Line::from(vec![
                Span::styled("❌ ", Style::default().fg(Color::Red)),
                Span::raw(app.error_message.as_deref().unwrap_or("An error occurred.")),
            ]));
        }
    }

    let border_color = match app.state {
        AppState::Ready => Color::Yellow,
        AppState::Running => Color::Cyan,
        AppState::Complete => Color::Green,
        AppState::Cancelled => Color::Yellow,
        AppState::Error => Color::Red,
    };

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::default()
            .title(" 📋 Configuration ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(paragraph, area);
}

/// Draw the main progress gauge
fn draw_progress(frame: &mut Frame, app: &App, area: Rect) {
    let percentage = app.progress.percentage();

    let label = if app.progress.total_files > 0 {
        format!(
            "{:.1}% ({}/{})",
            percentage, app.progress.processed_files, app.progress.total_files
        )
    } else if app.progress.is_complete {
        "✨ Complete!".to_string()
    } else {
        format!("{:.1}%", percentage)
    };

    let gauge_color = if app.progress.percentage() >= 100.0 {
        Color::Green
    } else {
        Color::Cyan
    };

    let gauge = Gauge::default()
        .gauge_style(Style::default().fg(gauge_color))
        .label(label)
        .ratio((percentage / 100.0).min(1.0))
        .block(
            Block::default()
                .title(" 📊 Progress ")
                .borders(Borders::ALL),
        );

    frame.render_widget(gauge, area);
}

/// Draw the file progress line
fn draw_file_progress(frame: &mut Frame, app: &App, area: Rect) {
    let file_pct = app.progress.file_percentage();
    let byte_pct = if app.progress.total_bytes > 0 {
        (app.progress.processed_bytes as f64 / app.progress.total_bytes as f64) * 100.0
    } else {
        100.0
    };

    let progress_text = format!("📄 Files: {:.1}%  |  💾 Data: {:.1}%", file_pct, byte_pct);

    let file_progress = Paragraph::new(progress_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(file_progress, area);
}

/// Draw the summary statistics
fn draw_summary(frame: &mut Frame, app: &App, area: Rect) {
    let items = vec![
        ListItem::new(Line::from(vec![
            Span::styled("✅ ", Style::default().fg(Color::Green)),
            Span::raw(format!("Copied:   {}", app.progress.copied)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("🗑️  ", Style::default().fg(Color::Red)),
            Span::raw(format!("Deleted:  {}", app.progress.deleted)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("⏭️  ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("Skipped:  {}", app.progress.skipped)),
        ])),
        ListItem::new(Line::from(vec![
            Span::styled("❌ ", Style::default().fg(Color::Red)),
            Span::raw(format!("Errors:   {}", app.progress.errors)),
        ])),
    ];

    let border_color = if app.progress.errors > 0 {
        Color::Red
    } else {
        Color::Green
    };

    let list = List::new(items).block(
        Block::default()
            .title(" 📈 Statistics ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color)),
    );

    frame.render_widget(list, area);
}

/// Draw the help bar
fn draw_help(frame: &mut Frame, _app: &App, area: Rect) {
    let key_hints = vec![
        Span::styled("[Space]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Start  "),
        Span::styled("[q]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Quit  "),
        Span::styled("[r]", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" Reset"),
    ];

    let paragraph = Paragraph::new(Line::from(key_hints))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}
