use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::app::{App, Panel};

pub fn render_profiles(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Profiles;

    let border_color = if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Profiles ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let items: Vec<ListItem> = app
        .env_files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let name = f.name().to_string();
            let count = f.entries.len();
            let errors = f.errors.len();

            let indicator = if i == app.profile_index { "▸ " } else { "  " };

            let mut spans = vec![
                Span::styled(
                    indicator,
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    name,
                    if i == app.profile_index {
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Gray)
                    },
                ),
                Span::styled(
                    format!(" ({})", count),
                    Style::default().fg(Color::DarkGray),
                ),
            ];

            if errors > 0 {
                spans.push(Span::styled(
                    format!(" !{}", errors),
                    Style::default().fg(Color::Red),
                ));
            }

            ListItem::new(Line::from(spans))
        })
        .collect();

    if items.is_empty() {
        let empty = List::new(vec![ListItem::new(Line::from(Span::styled(
            "  No .env files",
            Style::default().fg(Color::DarkGray),
        )))])
        .block(block);
        frame.render_widget(empty, area);
    } else {
        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}
