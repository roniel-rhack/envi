use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::{App, Panel};

pub fn render_details(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Details;

    let border_color = if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .title(" Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let file = match app.current_file() {
        Some(f) => f,
        None => {
            let p = Paragraph::new("No file selected").block(block);
            frame.render_widget(p, area);
            return;
        }
    };

    let entry = match file.entries.get(app.var_index) {
        Some(e) => e,
        None => {
            let p = Paragraph::new("No variable selected").block(block);
            frame.render_widget(p, area);
            return;
        }
    };

    let mut lines: Vec<Line> = Vec::new();

    // Key name
    lines.push(Line::from(vec![
        Span::styled("Key: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            &entry.key,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines.push(Line::from(""));

    // Value
    lines.push(Line::from(Span::styled(
        "Value:",
        Style::default().fg(Color::DarkGray),
    )));
    if entry.is_encrypted {
        lines.push(Line::from(Span::styled(
            "  [encrypted]",
            Style::default().fg(Color::Magenta),
        )));
    } else if entry.value.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty)",
            Style::default().fg(Color::Red),
        )));
    } else {
        // Word-wrap long values
        for chunk in crate::ui::helpers::char_chunks(&entry.value, 40) {
            lines.push(Line::from(Span::styled(
                format!("  {}", chunk),
                Style::default().fg(Color::White),
            )));
        }
    }

    lines.push(Line::from(""));

    // Comment
    if let Some(comment) = &entry.comment {
        lines.push(Line::from(vec![
            Span::styled("Comment: ", Style::default().fg(Color::DarkGray)),
            Span::styled(comment, Style::default().fg(Color::Gray)),
        ]));
        lines.push(Line::from(""));
    }

    // Line number
    lines.push(Line::from(vec![
        Span::styled("Line: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            entry.line_number.to_string(),
            Style::default().fg(Color::Gray),
        ),
    ]));

    // Encryption status
    lines.push(Line::from(vec![
        Span::styled("Encrypted: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if entry.is_encrypted { "Yes" } else { "No" },
            Style::default().fg(if entry.is_encrypted {
                Color::Magenta
            } else {
                Color::Gray
            }),
        ),
    ]));

    lines.push(Line::from(""));

    // Validation warnings
    let mut warnings: Vec<String> = Vec::new();
    if entry.value.is_empty() && !entry.is_encrypted {
        warnings.push("Empty value".to_string());
    }
    {
        let key_lower = entry.key.to_lowercase();
        if key_lower.contains("password")
            || key_lower.contains("secret")
            || key_lower.contains("token")
            || key_lower.contains("api_key")
            || key_lower.contains("private_key")
            || key_lower.contains("credential")
            || key_lower.contains("auth")
        {
            warnings.push("Possibly sensitive — consider encrypting".to_string());
        }
    }
    if entry.key != entry.key.to_uppercase() {
        warnings.push("Key is not UPPER_SNAKE_CASE".to_string());
    }

    // Check presence in other files
    let other_files: Vec<String> = app
        .env_files
        .iter()
        .filter(|f| f.path != file.path)
        .filter_map(|f| {
            if f.get(&entry.key).is_some() {
                Some(f.name().to_string())
            } else {
                None
            }
        })
        .collect();

    let missing_files: Vec<String> = app
        .env_files
        .iter()
        .filter(|f| f.path != file.path)
        .filter_map(|f| {
            if f.get(&entry.key).is_none() {
                Some(f.name().to_string())
            } else {
                None
            }
        })
        .collect();

    if !other_files.is_empty() {
        lines.push(Line::from(Span::styled(
            "Also in:",
            Style::default().fg(Color::DarkGray),
        )));
        for name in &other_files {
            lines.push(Line::from(Span::styled(
                format!("  {}", name),
                Style::default().fg(Color::Green),
            )));
        }
        lines.push(Line::from(""));
    }

    if !missing_files.is_empty() {
        lines.push(Line::from(Span::styled(
            "Missing from:",
            Style::default().fg(Color::DarkGray),
        )));
        for name in &missing_files {
            lines.push(Line::from(Span::styled(
                format!("  {}", name),
                Style::default().fg(Color::Red),
            )));
        }
        lines.push(Line::from(""));
    }

    if !warnings.is_empty() {
        lines.push(Line::from(Span::styled(
            "Warnings:",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        for w in &warnings {
            lines.push(Line::from(Span::styled(
                format!("  ⚠ {}", w),
                Style::default().fg(Color::Yellow),
            )));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}
