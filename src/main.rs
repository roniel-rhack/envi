mod app;
mod env;
mod ui;

use app::{App, AppMode};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "envi")]
#[command(about = "A TUI for managing .env files — diff, scan, edit, encrypt")]
#[command(version)]
struct Cli {
    /// Directory to scan for .env files (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let project_dir = if cli.path.is_absolute() {
        cli.path
    } else {
        std::env::current_dir()?.join(&cli.path)
    };

    if !project_dir.is_dir() {
        eprintln!("Error: {} is not a directory", project_dir.display());
        std::process::exit(1);
    }

    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(project_dir);

    if app.env_files.is_empty() {
        // Restore terminal before printing error
        terminal::disable_raw_mode()?;
        io::stdout().execute(LeaveAlternateScreen)?;
        eprintln!("No .env files found in the specified directory.");
        eprintln!("Create a .env file and try again.");
        std::process::exit(1);
    }

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    terminal::disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    while app.running {
        // Clear expired status messages
        if app.should_clear_status() {
            app.status_message = None;
        }

        terminal.draw(|frame| {
            ui::layout::render(frame, app);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key(app, key.code, key.modifiers);
            }
        }
    }

    Ok(())
}

fn handle_key(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match app.mode {
        AppMode::Normal => handle_normal_key(app, key, modifiers),
        AppMode::Editing => handle_edit_key(app, key),
        AppMode::DiffView => handle_diff_key(app, key),
        AppMode::ScanView => handle_scan_key(app, key),
        AppMode::Help => handle_help_key(app, key),
        AppMode::Search => handle_search_key(app, key),
        AppMode::Confirm => handle_confirm_key(app, key),
    }
}

fn handle_normal_key(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    match key {
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char('?') => app.toggle_help(),

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            if app.active_panel == Panel::Variables {
                app.next_var();
            } else if app.active_panel == Panel::Profiles {
                app.next_profile();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.active_panel == Panel::Variables {
                app.prev_var();
            } else if app.active_panel == Panel::Profiles {
                app.prev_profile();
            }
        }
        KeyCode::Char('h') | KeyCode::Left => app.prev_profile(),
        KeyCode::Char('l') | KeyCode::Right => app.next_profile(),
        KeyCode::PageDown => app.page_down(),
        KeyCode::PageUp => app.page_up(),
        KeyCode::Home => {
            app.var_index = 0;
        }
        KeyCode::End => {
            let count = app.current_entry_count();
            if count > 0 {
                app.var_index = count - 1;
            }
        }

        // Panel switching
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                app.prev_panel();
            } else {
                app.next_panel();
            }
        }
        KeyCode::BackTab => app.prev_panel(),

        // Actions
        KeyCode::Char('e') | KeyCode::Enter => app.start_edit(),
        KeyCode::Char('a') => app.add_variable(),
        KeyCode::Char('x') => app.delete_variable(),
        KeyCode::Char('w') => app.save_current(),
        KeyCode::Char('r') => app.reload(),

        // Views
        KeyCode::Char('d') => app.toggle_diff(),
        KeyCode::Char('s') => app.toggle_scan(),
        KeyCode::Char('/') => app.start_search(),
        KeyCode::Char('n') => app.next_search_match(),

        KeyCode::Esc => app.running = false,
        _ => {}
    }
}

fn handle_edit_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter => app.confirm_edit(),
        KeyCode::Esc => app.cancel_edit(),
        KeyCode::Char(c) => app.edit_insert(c),
        KeyCode::Backspace => app.edit_backspace(),
        KeyCode::Delete => app.edit_delete(),
        KeyCode::Left => app.edit_left(),
        KeyCode::Right => app.edit_right(),
        KeyCode::Home => app.edit_home(),
        KeyCode::End => app.edit_end(),
        _ => {}
    }
}

fn handle_diff_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Char('d') => {
            app.mode = AppMode::Normal;
            app.diff_result = None;
        }
        KeyCode::Tab => app.cycle_diff_target(),
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char('j') | KeyCode::Down => app.diff_scroll += 1,
        KeyCode::Char('k') | KeyCode::Up => {
            app.diff_scroll = app.diff_scroll.saturating_sub(1);
        }
        _ => {}
    }
}

fn handle_scan_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Char('s') => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char('j') | KeyCode::Down => app.scan_scroll += 1,
        KeyCode::Char('k') | KeyCode::Up => {
            app.scan_scroll = app.scan_scroll.saturating_sub(1);
        }
        _ => {}
    }
}

fn handle_help_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
}

fn handle_search_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter => app.confirm_search(),
        KeyCode::Esc => {
            app.mode = AppMode::Normal;
            app.search_query.clear();
            app.search_matches.clear();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
            app.update_search();
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.update_search();
        }
        _ => {}
    }
}

fn handle_confirm_key(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('y') | KeyCode::Enter => app.confirm_yes(),
        KeyCode::Char('n') | KeyCode::Esc => app.confirm_no(),
        _ => {}
    }
}

use app::Panel;
