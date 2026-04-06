use crate::env::diff::{self, DiffResult};
use crate::env::parser::{self, EnvFile};
use crate::env::scanner::{self, ScanResult};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    Profiles,
    Variables,
    Details,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Editing,
    DiffView,
    ScanView,
    Help,
    Search,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfirmAction {
    DeleteVar,
    #[allow(dead_code)]
    SaveFile,
    QuitWithoutSave,
}

pub struct App {
    pub running: bool,
    pub project_dir: PathBuf,
    pub env_files: Vec<EnvFile>,
    pub active_panel: Panel,
    pub mode: AppMode,

    // Profile panel state
    pub profile_index: usize,

    // Variables panel state
    pub var_index: usize,
    pub var_scroll: usize,

    // Editing state
    pub edit_buffer: String,
    pub edit_cursor: usize,

    // Search
    pub search_query: String,
    pub search_matches: Vec<usize>,
    pub search_match_index: usize,

    // Diff
    pub diff_result: Option<DiffResult>,
    pub diff_target_index: usize,
    pub diff_scroll: usize,

    // Scanner
    pub scan_result: Option<ScanResult>,
    pub scan_scroll: usize,

    // Confirm dialog
    pub confirm_action: Option<ConfirmAction>,

    // Status message
    pub status_message: Option<(String, std::time::Instant)>,

    // Whether file has unsaved changes
    pub dirty: bool,
}

impl App {
    pub fn new(project_dir: PathBuf) -> Self {
        let env_files = Self::load_env_files(&project_dir);
        Self {
            running: true,
            project_dir,
            env_files,
            active_panel: Panel::Variables,
            mode: AppMode::Normal,
            profile_index: 0,
            var_index: 0,
            var_scroll: 0,
            edit_buffer: String::new(),
            edit_cursor: 0,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_match_index: 0,
            diff_result: None,
            diff_target_index: 1,
            diff_scroll: 0,
            scan_result: None,
            scan_scroll: 0,
            confirm_action: None,
            status_message: None,
            dirty: false,
        }
    }

    fn load_env_files(dir: &Path) -> Vec<EnvFile> {
        let paths = parser::discover_env_files(dir);
        paths
            .iter()
            .filter_map(|p| parser::parse_file(p).ok())
            .collect()
    }

    pub fn reload(&mut self) {
        self.env_files = Self::load_env_files(&self.project_dir);
        self.dirty = false;
        self.set_status("Files reloaded");
    }

    pub fn current_file(&self) -> Option<&EnvFile> {
        self.env_files.get(self.profile_index)
    }

    pub fn current_file_mut(&mut self) -> Option<&mut EnvFile> {
        self.env_files.get_mut(self.profile_index)
    }

    pub fn current_entry_count(&self) -> usize {
        self.current_file().map(|f| f.entries.len()).unwrap_or(0)
    }

    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), std::time::Instant::now()));
    }

    // Navigation
    pub fn next_profile(&mut self) {
        if !self.env_files.is_empty() {
            self.profile_index = (self.profile_index + 1) % self.env_files.len();
            self.var_index = 0;
            self.var_scroll = 0;
            self.diff_result = None;
        }
    }

    pub fn prev_profile(&mut self) {
        if !self.env_files.is_empty() {
            self.profile_index = if self.profile_index == 0 {
                self.env_files.len() - 1
            } else {
                self.profile_index - 1
            };
            self.var_index = 0;
            self.var_scroll = 0;
            self.diff_result = None;
        }
    }

    pub fn next_var(&mut self) {
        let count = self.current_entry_count();
        if count > 0 {
            self.var_index = (self.var_index + 1).min(count - 1);
        }
    }

    pub fn prev_var(&mut self) {
        if self.var_index > 0 {
            self.var_index -= 1;
        }
    }

    pub fn page_down(&mut self) {
        let count = self.current_entry_count();
        if count > 0 {
            self.var_index = (self.var_index + 10).min(count - 1);
        }
    }

    pub fn page_up(&mut self) {
        self.var_index = self.var_index.saturating_sub(10);
    }

    // Editing
    pub fn start_edit(&mut self) {
        if let Some(file) = self.current_file() {
            if let Some(entry) = file.entries.get(self.var_index) {
                self.edit_buffer = entry.value.clone();
                self.edit_cursor = self.edit_buffer.chars().count();
                self.mode = AppMode::Editing;
            }
        }
    }

    pub fn confirm_edit(&mut self) {
        let idx = self.var_index;
        let new_value = self.edit_buffer.clone();
        if let Some(file) = self.env_files.get_mut(self.profile_index) {
            if let Some(entry) = file.entries.get_mut(idx) {
                entry.value = new_value;
                self.dirty = true;
            }
        }
        self.mode = AppMode::Normal;
        self.set_status("Value updated (unsaved)");
    }

    pub fn cancel_edit(&mut self) {
        self.mode = AppMode::Normal;
        self.edit_buffer.clear();
    }

    pub fn edit_insert(&mut self, c: char) {
        let byte_pos = self.char_to_byte(self.edit_cursor);
        self.edit_buffer.insert(byte_pos, c);
        self.edit_cursor += 1;
    }

    pub fn edit_backspace(&mut self) {
        if self.edit_cursor > 0 {
            self.edit_cursor -= 1;
            let byte_pos = self.char_to_byte(self.edit_cursor);
            self.edit_buffer.remove(byte_pos);
        }
    }

    pub fn edit_delete(&mut self) {
        let char_count = self.edit_buffer.chars().count();
        if self.edit_cursor < char_count {
            let byte_pos = self.char_to_byte(self.edit_cursor);
            self.edit_buffer.remove(byte_pos);
        }
    }

    pub fn edit_left(&mut self) {
        if self.edit_cursor > 0 {
            self.edit_cursor -= 1;
        }
    }

    pub fn edit_right(&mut self) {
        let char_count = self.edit_buffer.chars().count();
        if self.edit_cursor < char_count {
            self.edit_cursor += 1;
        }
    }

    pub fn edit_home(&mut self) {
        self.edit_cursor = 0;
    }

    pub fn edit_end(&mut self) {
        self.edit_cursor = self.edit_buffer.chars().count();
    }

    /// Convert a char index to a byte offset in edit_buffer.
    fn char_to_byte(&self, char_idx: usize) -> usize {
        self.edit_buffer
            .char_indices()
            .nth(char_idx)
            .map(|(i, _)| i)
            .unwrap_or(self.edit_buffer.len())
    }

    // Save
    pub fn save_current(&mut self) {
        if let Some(file) = self.current_file() {
            match parser::write_env_file(file) {
                Ok(()) => {
                    self.dirty = false;
                    self.set_status("File saved!");
                }
                Err(e) => {
                    self.set_status(&format!("Save failed: {}", e));
                }
            }
        }
    }

    // Diff
    pub fn toggle_diff(&mut self) {
        if self.mode == AppMode::DiffView {
            self.mode = AppMode::Normal;
            self.diff_result = None;
            return;
        }

        if self.env_files.len() < 2 {
            self.set_status("Need at least 2 .env files to diff");
            return;
        }

        // Find a good default target for diffing
        let target = if self.profile_index == 0 && self.env_files.len() > 1 {
            1
        } else {
            0
        };
        self.diff_target_index = target;
        self.compute_diff();
        self.mode = AppMode::DiffView;
        self.diff_scroll = 0;
    }

    pub fn cycle_diff_target(&mut self) {
        if self.env_files.len() < 2 {
            return;
        }
        self.diff_target_index = (self.diff_target_index + 1) % self.env_files.len();
        if self.diff_target_index == self.profile_index {
            self.diff_target_index = (self.diff_target_index + 1) % self.env_files.len();
        }
        self.compute_diff();
    }

    fn compute_diff(&mut self) {
        if let (Some(source), Some(target)) = (
            self.env_files.get(self.profile_index),
            self.env_files.get(self.diff_target_index),
        ) {
            self.diff_result = Some(diff::diff_files(source, target));
        }
    }

    // Scanner
    pub fn toggle_scan(&mut self) {
        if self.mode == AppMode::ScanView {
            self.mode = AppMode::Normal;
            return;
        }

        // Collect all defined keys across all env files
        let key_refs: Vec<&str> = self
            .env_files
            .iter()
            .flat_map(|f| f.entries.iter().map(|e| e.key.as_str()))
            .collect();

        self.scan_result = Some(scanner::scan_project(&self.project_dir, &key_refs));
        self.mode = AppMode::ScanView;
        self.scan_scroll = 0;
        self.set_status("Code scan complete");
    }

    // Search
    pub fn start_search(&mut self) {
        self.search_query.clear();
        self.search_matches.clear();
        self.mode = AppMode::Search;
    }

    pub fn update_search(&mut self) {
        self.search_matches.clear();
        if self.search_query.is_empty() {
            return;
        }
        let query = self.search_query.to_lowercase();
        let matches: Vec<usize> = self
            .env_files
            .get(self.profile_index)
            .map(|file| {
                file.entries
                    .iter()
                    .enumerate()
                    .filter(|(_, entry)| {
                        entry.key.to_lowercase().contains(&query)
                            || entry.value.to_lowercase().contains(&query)
                    })
                    .map(|(i, _)| i)
                    .collect()
            })
            .unwrap_or_default();
        self.search_matches = matches;
        if !self.search_matches.is_empty() {
            self.search_match_index = 0;
            self.var_index = self.search_matches[0];
        }
    }

    pub fn next_search_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.search_match_index = (self.search_match_index + 1) % self.search_matches.len();
            self.var_index = self.search_matches[self.search_match_index];
        }
    }

    pub fn confirm_search(&mut self) {
        self.mode = AppMode::Normal;
    }

    // Add new variable
    pub fn add_variable(&mut self) {
        if let Some(file) = self.current_file_mut() {
            let base = "NEW_VAR";
            let existing_keys: Vec<&str> = file.entries.iter().map(|e| e.key.as_str()).collect();

            let key = if !existing_keys.contains(&base) {
                base.to_string()
            } else {
                let mut n = 1;
                loop {
                    let candidate = format!("{}_{}", base, n);
                    if !existing_keys.contains(&candidate.as_str()) {
                        break candidate;
                    }
                    n += 1;
                }
            };

            let new_entry = parser::EnvEntry {
                key,
                value: String::new(),
                comment: None,
                line_number: file.entries.len() + 1,
                is_encrypted: false,
                has_export: false,
                quote_style: parser::QuoteStyle::None,
            };
            file.entries.push(new_entry);
            self.var_index = file.entries.len() - 1;
            self.dirty = true;
            self.start_edit();
        }
    }

    // Delete variable
    pub fn delete_variable(&mut self) {
        self.confirm_action = Some(ConfirmAction::DeleteVar);
        self.mode = AppMode::Confirm;
    }

    pub fn confirm_yes(&mut self) {
        match self.confirm_action.take() {
            Some(ConfirmAction::DeleteVar) => {
                let idx = self.var_index;
                if let Some(file) = self.env_files.get_mut(self.profile_index) {
                    if idx < file.entries.len() {
                        file.entries.remove(idx);
                        if self.var_index >= file.entries.len() && self.var_index > 0 {
                            self.var_index -= 1;
                        }
                        self.dirty = true;
                        self.set_status("Variable deleted (unsaved)");
                    }
                }
            }
            Some(ConfirmAction::SaveFile) => {
                self.save_current();
            }
            Some(ConfirmAction::QuitWithoutSave) => {
                self.running = false;
            }
            None => {}
        }
        self.mode = AppMode::Normal;
    }

    pub fn confirm_no(&mut self) {
        self.confirm_action = None;
        self.mode = AppMode::Normal;
    }

    // Tab between panels
    pub fn next_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Profiles => Panel::Variables,
            Panel::Variables => Panel::Details,
            Panel::Details => Panel::Profiles,
        };
    }

    pub fn prev_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Profiles => Panel::Details,
            Panel::Variables => Panel::Profiles,
            Panel::Details => Panel::Variables,
        };
    }

    pub fn quit(&mut self) {
        if self.dirty {
            self.confirm_action = Some(ConfirmAction::QuitWithoutSave);
            self.mode = AppMode::Confirm;
        } else {
            self.running = false;
        }
    }

    pub fn toggle_help(&mut self) {
        if self.mode == AppMode::Help {
            self.mode = AppMode::Normal;
        } else {
            self.mode = AppMode::Help;
        }
    }

    pub fn close_diff(&mut self) {
        self.mode = AppMode::Normal;
        self.diff_result = None;
    }

    pub fn close_scan(&mut self) {
        self.mode = AppMode::Normal;
    }

    pub fn scroll_down(&mut self, scroll: &str) {
        match scroll {
            "diff" => self.diff_scroll += 1,
            "scan" => self.scan_scroll += 1,
            _ => {}
        }
    }

    pub fn scroll_up(&mut self, scroll: &str) {
        match scroll {
            "diff" => self.diff_scroll = self.diff_scroll.saturating_sub(1),
            "scan" => self.scan_scroll = self.scan_scroll.saturating_sub(1),
            _ => {}
        }
    }

    pub fn go_to_first_var(&mut self) {
        self.var_index = 0;
    }

    pub fn go_to_last_var(&mut self) {
        let count = self.current_entry_count();
        if count > 0 {
            self.var_index = count - 1;
        }
    }

    pub fn should_clear_status(&self) -> bool {
        if let Some((_, instant)) = &self.status_message {
            instant.elapsed().as_secs() >= 3
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::parser::{EnvEntry, EnvFile, QuoteStyle};
    use std::path::PathBuf;

    fn make_app(entry_count: usize) -> App {
        let entries: Vec<EnvEntry> = (0..entry_count)
            .map(|i| EnvEntry {
                key: format!("KEY_{}", i),
                value: format!("value_{}", i),
                comment: None,
                line_number: i + 1,
                is_encrypted: false,
                has_export: false,
                quote_style: QuoteStyle::None,
            })
            .collect();
        let file = EnvFile {
            path: PathBuf::from("/tmp/.env"),
            entries,
            errors: vec![],
        };
        let mut app = App::new(PathBuf::from("/tmp"));
        app.env_files = vec![file];
        app.var_index = 0;
        app
    }

    fn make_app_multi(file_count: usize, entries_per_file: usize) -> App {
        let files: Vec<EnvFile> = (0..file_count)
            .map(|fi| {
                let entries: Vec<EnvEntry> = (0..entries_per_file)
                    .map(|i| EnvEntry {
                        key: format!("KEY_{}", i),
                        value: format!("value_{}_{}", fi, i),
                        comment: None,
                        line_number: i + 1,
                        is_encrypted: false,
                        has_export: false,
                        quote_style: QuoteStyle::None,
                    })
                    .collect();
                EnvFile {
                    path: PathBuf::from(format!("/tmp/.env.{}", fi)),
                    entries,
                    errors: vec![],
                }
            })
            .collect();
        let mut app = App::new(PathBuf::from("/tmp"));
        app.env_files = files;
        app
    }

    #[test]
    fn test_next_var_clamps_at_end() {
        let mut app = make_app(3);
        app.var_index = 2;
        app.next_var();
        assert_eq!(app.var_index, 2);
    }

    #[test]
    fn test_prev_var_stays_at_zero() {
        let mut app = make_app(3);
        app.var_index = 0;
        app.prev_var();
        assert_eq!(app.var_index, 0);
    }

    #[test]
    fn test_page_down_clamps() {
        let mut app = make_app(5);
        app.var_index = 0;
        app.page_down();
        assert_eq!(app.var_index, 4);
    }

    #[test]
    fn test_page_up_saturates() {
        let mut app = make_app(5);
        app.var_index = 3;
        app.page_up();
        assert_eq!(app.var_index, 0);
    }

    #[test]
    fn test_next_profile_wraps() {
        let mut app = make_app_multi(3, 1);
        app.profile_index = 2;
        app.next_profile();
        assert_eq!(app.profile_index, 0);
    }

    #[test]
    fn test_prev_profile_wraps() {
        let mut app = make_app_multi(3, 1);
        app.profile_index = 0;
        app.prev_profile();
        assert_eq!(app.profile_index, 2);
    }

    #[test]
    fn test_edit_insert_and_backspace_ascii() {
        let mut app = make_app(1);
        app.edit_buffer = "hello".to_string();
        app.edit_cursor = 5;
        app.edit_insert('!');
        assert_eq!(app.edit_buffer, "hello!");
        assert_eq!(app.edit_cursor, 6);
        app.edit_backspace();
        assert_eq!(app.edit_buffer, "hello");
        assert_eq!(app.edit_cursor, 5);
    }

    #[test]
    fn test_edit_insert_multibyte() {
        let mut app = make_app(1);
        app.edit_buffer = String::new();
        app.edit_cursor = 0;
        app.edit_insert('é');
        app.edit_insert('ñ');
        assert_eq!(app.edit_buffer, "éñ");
        assert_eq!(app.edit_cursor, 2);
        app.edit_backspace();
        assert_eq!(app.edit_buffer, "é");
        assert_eq!(app.edit_cursor, 1);
    }

    #[test]
    fn test_edit_backspace_at_zero() {
        let mut app = make_app(1);
        app.edit_buffer = "x".to_string();
        app.edit_cursor = 0;
        app.edit_backspace();
        assert_eq!(app.edit_buffer, "x");
        assert_eq!(app.edit_cursor, 0);
    }

    #[test]
    fn test_delete_last_entry_adjusts_index() {
        let mut app = make_app(2);
        app.var_index = 1;
        app.confirm_action = Some(ConfirmAction::DeleteVar);
        app.confirm_yes();
        assert_eq!(app.env_files[0].entries.len(), 1);
        assert_eq!(app.var_index, 0);
    }

    #[test]
    fn test_delete_only_entry() {
        let mut app = make_app(1);
        app.var_index = 0;
        app.confirm_action = Some(ConfirmAction::DeleteVar);
        app.confirm_yes();
        assert_eq!(app.env_files[0].entries.len(), 0);
        assert_eq!(app.var_index, 0);
    }

    #[test]
    fn test_search_case_insensitive() {
        let mut app = make_app(3);
        app.search_query = "key_1".to_string();
        app.update_search();
        assert_eq!(app.search_matches, vec![1]);
    }

    #[test]
    fn test_next_search_match_wraps() {
        let mut app = make_app(3);
        app.search_query = "key".to_string();
        app.update_search();
        assert_eq!(app.search_matches.len(), 3);
        app.search_match_index = 2;
        app.next_search_match();
        assert_eq!(app.search_match_index, 0);
    }

    #[test]
    fn test_toggle_diff_requires_two_files() {
        let mut app = make_app(3);
        app.toggle_diff();
        assert_eq!(app.mode, AppMode::Normal);
        assert!(app.status_message.is_some());
    }

    #[test]
    fn test_quit_with_dirty_shows_confirm() {
        let mut app = make_app(1);
        app.dirty = true;
        app.quit();
        assert_eq!(app.mode, AppMode::Confirm);
        assert_eq!(app.confirm_action, Some(ConfirmAction::QuitWithoutSave));
        assert!(app.running);
    }

    #[test]
    fn test_quit_clean_exits() {
        let mut app = make_app(1);
        app.dirty = false;
        app.quit();
        assert!(!app.running);
    }

    #[test]
    fn test_add_variable_unique_keys() {
        let mut app = make_app(0);
        app.mode = AppMode::Normal;
        app.add_variable();
        assert_eq!(app.env_files[0].entries[0].key, "NEW_VAR");
        app.mode = AppMode::Normal;
        app.add_variable();
        assert_eq!(app.env_files[0].entries[1].key, "NEW_VAR_1");
        app.mode = AppMode::Normal;
        app.add_variable();
        assert_eq!(app.env_files[0].entries[2].key, "NEW_VAR_2");
    }

    #[test]
    fn test_start_edit_on_empty_file() {
        let mut app = make_app(0);
        app.start_edit();
        assert_eq!(app.mode, AppMode::Normal);
    }
}
