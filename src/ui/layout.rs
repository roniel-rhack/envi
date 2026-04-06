use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;

use crate::app::{App, AppMode};

use super::details::render_details;
use super::dialogs::{render_confirm, render_help, render_search};
use super::profiles::render_profiles;
use super::variables::render_variables;

pub fn render(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout: top area + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(size);

    // Three-panel layout
    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(22),
            Constraint::Percentage(50),
            Constraint::Min(30),
        ])
        .split(main_chunks[0]);

    render_profiles(frame, app, panels[0]);
    render_variables(frame, app, panels[1]);
    render_details(frame, app, panels[2]);
    render_status_bar(frame, app, main_chunks[1]);

    // Overlays
    match app.mode {
        AppMode::Help => render_help(frame, size),
        AppMode::Search => render_search(frame, app, size),
        AppMode::Confirm => render_confirm(frame, app, size),
        _ => {}
    }
}

fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::Paragraph;

    let mode_str = match app.mode {
        AppMode::Normal => "NORMAL",
        AppMode::Editing => "EDITING",
        AppMode::DiffView => "DIFF",
        AppMode::ScanView => "SCAN",
        AppMode::Help => "HELP",
        AppMode::Search => "SEARCH",
        AppMode::Confirm => "CONFIRM",
    };

    let mode_color = match app.mode {
        AppMode::Normal => Color::Blue,
        AppMode::Editing => Color::Green,
        AppMode::DiffView => Color::Magenta,
        AppMode::ScanView => Color::Cyan,
        _ => Color::Yellow,
    };

    let dirty_indicator = if app.dirty { " [+]" } else { "" };

    let file_name = app
        .current_file()
        .map(|f| f.name().to_string())
        .unwrap_or_else(|| "No file".to_string());

    let status_msg = app
        .status_message
        .as_ref()
        .map(|(msg, _)| msg.as_str())
        .unwrap_or("");

    let shortcuts = match app.mode {
        AppMode::Normal => "[e]dit [d]iff [s]can [/]search [a]dd [x]del [w]rite [?]help [q]uit",
        AppMode::Editing => "[Enter]confirm [Esc]cancel",
        AppMode::DiffView => "[Tab]cycle target [Esc]close",
        AppMode::ScanView => "[Esc]close",
        _ => "[Esc]close",
    };

    let line = Line::from(vec![
        Span::styled(
            format!(" {} ", mode_str),
            Style::default()
                .fg(Color::Black)
                .bg(mode_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}{} ", file_name, dirty_indicator),
            Style::default().fg(Color::White),
        ),
        Span::styled(
            format!(" {} ", status_msg),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled(
            format!(" {} ", shortcuts),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let bar = Paragraph::new(line).style(Style::default().bg(Color::Rgb(30, 30, 40)));
    frame.render_widget(bar, area);
}
