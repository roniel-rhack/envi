use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, ConfirmAction};

pub fn render_help(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(60, 70, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Help — envi ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let help_text = vec![
        Line::from(Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  j/↓        Move down"),
        Line::from("  k/↑        Move up"),
        Line::from("  PgDn/PgUp  Page down/up"),
        Line::from("  Tab        Next panel"),
        Line::from("  Shift+Tab  Previous panel"),
        Line::from("  h/l        Previous/next profile"),
        Line::from(""),
        Line::from(Span::styled(
            "Actions",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  e          Edit selected value"),
        Line::from("  a          Add new variable"),
        Line::from("  x          Delete selected variable"),
        Line::from("  w          Save current file"),
        Line::from("  r          Reload all files"),
        Line::from(""),
        Line::from(Span::styled(
            "Views",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  d          Toggle diff view"),
        Line::from("  s          Toggle code scan"),
        Line::from("  /          Search variables"),
        Line::from("  n          Next search match"),
        Line::from(""),
        Line::from(Span::styled(
            "Other",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  ?          Toggle this help"),
        Line::from("  q/Esc      Quit / close overlay"),
        Line::from(""),
        Line::from(Span::styled(
            "  Press Esc or ? to close",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, popup);
}

pub fn render_search(frame: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(50, 10, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Search ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let match_info = if app.search_matches.is_empty() {
        if app.search_query.is_empty() {
            "Type to search...".to_string()
        } else {
            "No matches".to_string()
        }
    } else {
        format!(
            "Match {}/{}",
            app.search_match_index + 1,
            app.search_matches.len()
        )
    };

    let lines = vec![
        Line::from(vec![
            Span::styled("  > ", Style::default().fg(Color::Yellow)),
            Span::styled(&app.search_query, Style::default().fg(Color::White)),
            Span::styled("_", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", match_info),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, popup);
}

pub fn render_confirm(frame: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(40, 15, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

    let message = match app.confirm_action {
        Some(ConfirmAction::DeleteVar) => "Delete this variable?",
        Some(ConfirmAction::SaveFile) => "Save changes?",
        Some(ConfirmAction::QuitWithoutSave) => "Quit without saving?",
        None => "Confirm action?",
    };

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("  {}", message),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [y]", Style::default().fg(Color::Green)),
            Span::styled(" Yes  ", Style::default().fg(Color::Gray)),
            Span::styled("[n]", Style::default().fg(Color::Red)),
            Span::styled(" No", Style::default().fg(Color::Gray)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, popup);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
