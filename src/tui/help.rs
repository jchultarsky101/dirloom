//! Help screen modal dialog

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

/// Warm color palette for help dialog
mod colors {
    use ratatui::style::Color;

    pub const PEACH: Color = Color::Rgb(255, 170, 128);
    pub const AMBER: Color = Color::Rgb(255, 184, 66);
    pub const ROSE: Color = Color::Rgb(255, 121, 121);
    pub const SUCCESS: Color = Color::Rgb(255, 184, 108);
    pub const PINK: Color = Color::Rgb(255, 121, 121);
    pub const BG_DARKEST: Color = Color::Rgb(30, 25, 30);
    pub const FG_PRIMARY: Color = Color::Rgb(230, 220, 220);
    pub const FG_SECONDARY: Color = Color::Rgb(160, 145, 150);
}

/// Draw the help modal dialog
pub fn draw_help_dialog(frame: &mut Frame, area: Rect) {
    // Create centered modal area (70% width, 70% height)
    let modal_area = centered_rect(70, 70, area);

    // First, clear the area to prevent background bleed-through
    frame.render_widget(Clear, modal_area);

    // Create the modal block with solid background
    let modal_block = Block::default()
        .title(" ❓ Dirloom Help ")
        .title_style(
            Style::default()
                .fg(colors::AMBER)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::FG_SECONDARY))
        .style(Style::default().bg(colors::BG_DARKEST));

    // Calculate inner area (excluding borders)
    let inner_area = modal_block.inner(modal_area);

    // Render the modal background first
    frame.render_widget(modal_block, modal_area);

    // Split inner area into sections
    let chunks = Layout::vertical([
        Constraint::Length(6),  // Welcome/intro
        Constraint::Length(8),  // Keyboard shortcuts
        Constraint::Length(10), // Backup modes
        Constraint::Length(3),  // Close hint
    ])
    .margin(1)
    .split(inner_area);

    // Welcome section
    let welcome = Paragraph::new(vec![
        Line::from(Span::styled(
            "Dirloom - Directory Backup TUI",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(colors::PEACH),
        )),
        Line::from(""),
        Line::from("A terminal-based backup tool with real-time progress tracking."),
    ])
    .style(
        Style::default()
            .fg(colors::FG_PRIMARY)
            .bg(colors::BG_DARKEST),
    )
    .wrap(Wrap { trim: true });
    frame.render_widget(welcome, chunks[0]);

    // Keyboard shortcuts section
    let shortcuts_title = Line::from(Span::styled(
        "Keyboard Shortcuts",
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(colors::AMBER),
    ));

    let shortcuts = vec![
        Line::from(vec![
            Span::styled("[Space] ", Style::default().fg(colors::SUCCESS)),
            Span::raw("Start backup  "),
            Span::styled("[m] ", Style::default().fg(colors::PEACH)),
            Span::raw("Cycle mode"),
        ]),
        Line::from(vec![
            Span::styled("[1] ", Style::default().fg(colors::PEACH)),
            Span::raw("Pick source  "),
            Span::styled("[2] ", Style::default().fg(colors::PEACH)),
            Span::raw("Pick destination"),
        ]),
        Line::from(vec![
            Span::styled("[r] ", Style::default().fg(colors::AMBER)),
            Span::raw("Reset  "),
            Span::styled("[q/Esc] ", Style::default().fg(colors::ROSE)),
            Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled("[?] ", Style::default().fg(colors::AMBER)),
            Span::raw("Show this help"),
        ]),
    ];

    let shortcuts_section = Paragraph::new(shortcuts)
        .style(
            Style::default()
                .fg(colors::FG_PRIMARY)
                .bg(colors::BG_DARKEST),
        )
        .block(
            Block::default()
                .borders(Borders::NONE)
                .title(shortcuts_title),
        );
    frame.render_widget(shortcuts_section, chunks[1]);

    // Directory picker shortcuts
    let picker_title = Line::from(Span::styled(
        "Directory Picker",
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(colors::AMBER),
    ));

    let picker_shortcuts = vec![
        Line::from(vec![
            Span::styled("[↑/↓] ", Style::default().fg(colors::PEACH)),
            Span::raw("Navigate  "),
            Span::styled("[Enter] ", Style::default().fg(colors::PEACH)),
            Span::raw("Open directory"),
        ]),
        Line::from(vec![
            Span::styled("[Tab/Space] ", Style::default().fg(colors::SUCCESS)),
            Span::raw("Select  "),
            Span::styled("[Esc] ", Style::default().fg(colors::ROSE)),
            Span::raw("Cancel"),
        ]),
    ];

    let picker_section = Paragraph::new(picker_shortcuts)
        .style(
            Style::default()
                .fg(colors::FG_PRIMARY)
                .bg(colors::BG_DARKEST),
        )
        .block(Block::default().borders(Borders::NONE).title(picker_title));
    frame.render_widget(picker_section, chunks[2]);

    // Backup modes section
    let modes_title = Line::from(Span::styled(
        "Backup Modes",
        Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(colors::AMBER),
    ));

    let modes = vec![
        Line::from(vec![
            Span::styled("🪞 Mirror ", Style::default().fg(colors::PINK)),
            Span::raw("- Exact copy, deletes extra files"),
        ]),
        Line::from(vec![
            Span::styled("📈 Incremental ", Style::default().fg(colors::PEACH)),
            Span::raw("- Only changed/new files"),
        ]),
        Line::from(vec![
            Span::styled("🔄 Update ", Style::default().fg(colors::AMBER)),
            Span::raw("- Copy newer files only"),
        ]),
        Line::from(vec![
            Span::styled("⚡ Force ", Style::default().fg(colors::ROSE)),
            Span::raw("- Overwrite ALL files"),
        ]),
    ];

    let modes_section = Paragraph::new(modes)
        .style(
            Style::default()
                .fg(colors::FG_PRIMARY)
                .bg(colors::BG_DARKEST),
        )
        .block(Block::default().borders(Borders::NONE).title(modes_title));
    frame.render_widget(modes_section, chunks[3]);
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
