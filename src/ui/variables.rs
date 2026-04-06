use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::{App, AppMode, Panel};
use crate::env::diff::DiffKind;

pub fn render_variables(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_panel == Panel::Variables;

    let border_color = if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let title = match app.mode {
        AppMode::DiffView => {
            if let Some(ref diff) = app.diff_result {
                format!(
                    " Diff: {} vs {} ({} missing, {} extra, {} changed) ",
                    diff.source_name,
                    diff.target_name,
                    diff.missing_count(),
                    diff.extra_count(),
                    diff.changed_count()
                )
            } else {
                " Diff ".to_string()
            }
        }
        AppMode::ScanView => " Code Scan Results ".to_string(),
        _ => {
            let file_name = app
                .current_file()
                .map(|f| f.name().to_string())
                .unwrap_or_default();
            format!(" Variables — {} ", file_name)
        }
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    match app.mode {
        AppMode::DiffView => render_diff_view(frame, app, area, block),
        AppMode::ScanView => render_scan_view(frame, app, area, block),
        _ => render_normal_view(frame, app, area, block),
    }
}

fn render_normal_view(frame: &mut Frame, app: &App, area: Rect, block: Block) {
    let file = match app.current_file() {
        Some(f) => f,
        None => {
            let empty = List::new(vec![ListItem::new(Line::from(Span::styled(
                "  No .env files found in this directory",
                Style::default().fg(Color::DarkGray),
            )))])
            .block(block);
            frame.render_widget(empty, area);
            return;
        }
    };

    let items: Vec<ListItem> = file
        .entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let is_selected = i == app.var_index;
            let is_search_match = app.search_matches.contains(&i);

            let value_display = if app.mode == AppMode::Editing && is_selected {
                let buf = &app.edit_buffer;
                let cursor = app.edit_cursor;
                format!("{}|{}", &buf[..cursor], &buf[cursor..])
            } else if entry.is_encrypted {
                "[encrypted]".to_string()
            } else if entry.value.len() > 40 {
                format!("{}...", &entry.value[..37])
            } else {
                entry.value.clone()
            };

            let key_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if is_search_match {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Green)
            };

            let value_style = if app.mode == AppMode::Editing && is_selected {
                Style::default().fg(Color::White).bg(Color::Rgb(40, 40, 60))
            } else if entry.is_encrypted {
                Style::default().fg(Color::Magenta)
            } else if entry.value.is_empty() {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▸ " } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<20}", entry.key), key_style),
                Span::styled(" = ", Style::default().fg(Color::DarkGray)),
                Span::styled(value_display, value_style),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.var_index));

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::Rgb(40, 40, 55)));

    frame.render_stateful_widget(list, area, &mut state);
}

fn render_diff_view(frame: &mut Frame, app: &App, area: Rect, block: Block) {
    let diff = match &app.diff_result {
        Some(d) => d,
        None => {
            let empty = List::new(vec![ListItem::new("No diff available")]).block(block);
            frame.render_widget(empty, area);
            return;
        }
    };

    let items: Vec<ListItem> = diff
        .entries
        .iter()
        .map(|entry| {
            let (prefix, color) = match entry.kind {
                DiffKind::Missing => ("- MISSING  ", Color::Red),
                DiffKind::Extra => ("+ EXTRA    ", Color::Yellow),
                DiffKind::Changed => ("~ CHANGED  ", Color::Magenta),
                DiffKind::Unchanged => ("  OK       ", Color::DarkGray),
            };

            let value_info = match entry.kind {
                DiffKind::Changed => format!(
                    "{} -> {}",
                    entry.source_value.as_deref().unwrap_or(""),
                    entry.target_value.as_deref().unwrap_or("")
                ),
                DiffKind::Missing => entry.source_value.as_deref().unwrap_or("").to_string(),
                DiffKind::Extra => entry.target_value.as_deref().unwrap_or("").to_string(),
                DiffKind::Unchanged => entry.source_value.as_deref().unwrap_or("").to_string(),
            };

            let truncated = if value_info.len() > 35 {
                format!("{}...", &value_info[..32])
            } else {
                value_info
            };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(color)),
                Span::styled(
                    format!("{:<20}", entry.key),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" {}", truncated), Style::default().fg(Color::Gray)),
            ]))
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_scan_view(frame: &mut Frame, app: &App, area: Rect, block: Block) {
    let scan = match &app.scan_result {
        Some(s) => s,
        None => {
            let empty = List::new(vec![ListItem::new("No scan results")]).block(block);
            frame.render_widget(empty, area);
            return;
        }
    };

    let mut items: Vec<ListItem> = Vec::new();

    if !scan.undefined_vars.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            format!(
                "--- Used in code but NOT in .env ({}) ---",
                scan.undefined_vars.len()
            ),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))));
        for var in &scan.undefined_vars {
            let usages = scan.used_vars.get(var).map(|u| u.len()).unwrap_or(0);
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  ! ", Style::default().fg(Color::Red)),
                Span::styled(var.clone(), Style::default().fg(Color::Red)),
                Span::styled(
                    format!(" ({} usages)", usages),
                    Style::default().fg(Color::DarkGray),
                ),
            ])));
        }
    }

    if !scan.unused_vars.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            format!(
                "--- Defined in .env but NOT used in code ({}) ---",
                scan.unused_vars.len()
            ),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))));
        for var in &scan.unused_vars {
            items.push(ListItem::new(Line::from(vec![
                Span::styled("  ? ", Style::default().fg(Color::Yellow)),
                Span::styled(var.clone(), Style::default().fg(Color::Yellow)),
            ])));
        }
    }

    if scan.undefined_vars.is_empty() && scan.unused_vars.is_empty() {
        items.push(ListItem::new(Line::from(Span::styled(
            "  All variables are accounted for!",
            Style::default().fg(Color::Green),
        ))));
    }

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
