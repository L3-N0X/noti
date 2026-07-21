use crate::config::Config;
use crate::document::Document;
use crate::errors::{AppError, Result};
use crate::history_dialog::HistoryOverlay;
use crate::palette::PaletteOverlay;
use crate::recent_dialog::RecentOverlay;
use crate::state::{CursorState, PersistedState};
use adw::prelude::*;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

pub struct AppState {
    pub config: Config,
    pub document: Document,
    pub state: PersistedState,
    pub save_timer: Option<glib::SourceId>,
    pub config_warnings: Vec<String>,
    pub is_loading: bool,

    pub window: Option<adw::ApplicationWindow>,
    pub sourceview: Option<sourceview5::View>,
    pub buffer: Option<sourceview5::Buffer>,
    pub toast_overlay: Option<adw::ToastOverlay>,

    pub palette_overlay: Option<PaletteOverlay>,
    pub recent_overlay: Option<RecentOverlay>,
    pub history_overlay: Option<HistoryOverlay>,

    pub global_css_provider: Option<gtk::CssProvider>,
    pub editor_css_provider: Option<gtk::CssProvider>,

    pub cycle_list: Vec<std::path::PathBuf>,
    pub cycle_index: usize,
    pub last_cycle_time: Option<std::time::Instant>,
}

impl AppState {
    pub fn new() -> Self {
        let (mut config, config_warnings) = Config::load_or_create().unwrap_or_else(|e| {
            (
                Config::default(),
                vec![format!("Failed to load config: {}", e)],
            )
        });
        let state = PersistedState::load_or_default();
        if let Some(fs) = state.font_size {
            config.editor.font_size = Some(fs);
        }
        let document = Document::new(None);

        Self {
            config,
            document,
            state,
            save_timer: None,
            config_warnings,
            is_loading: false,
            window: None,
            sourceview: None,
            buffer: None,
            toast_overlay: None,
            palette_overlay: None,
            recent_overlay: None,
            history_overlay: None,
            global_css_provider: None,
            editor_css_provider: None,
            cycle_list: Vec::new(),
            cycle_index: 0,
            last_cycle_time: None,
        }
    }

    pub fn init(
        &mut self,
        window: adw::ApplicationWindow,
        sourceview: sourceview5::View,
        buffer: sourceview5::Buffer,
        toast_overlay: adw::ToastOverlay,
        palette_overlay: PaletteOverlay,
        recent_overlay: RecentOverlay,
        history_overlay: HistoryOverlay,
    ) {
        self.window = Some(window);
        self.sourceview = Some(sourceview);
        self.buffer = Some(buffer);
        self.toast_overlay = Some(toast_overlay);
        self.palette_overlay = Some(palette_overlay);
        self.recent_overlay = Some(recent_overlay);
        self.history_overlay = Some(history_overlay);

        // Apply styles on init
        if let (Some(win), Some(sv)) = (&self.window, &self.sourceview) {
            crate::style::apply_styles(
                win,
                sv,
                &self.config,
                &mut self.global_css_provider,
                &mut self.editor_css_provider,
            );

            let sv_clone = sv.clone();
            adw::StyleManager::default().connect_dark_notify(move |_| {
                if let Ok(source_buffer) = sv_clone.buffer().downcast::<sourceview5::Buffer>() {
                    crate::style::update_editor_scheme(&source_buffer);
                }
            });
        }

        // Clean up snapshots on start
        let _ = crate::snapshots::prune_old_snapshots(self.config.autosave.keep_snapshots_days);
    }

    pub fn load_initial_note(&mut self) -> Result<()> {
        let reopen = self.config.behavior.reopen_last_note;
        let last_opened = self.state.last_opened.clone();

        let path_to_open = if reopen { last_opened } else { None };

        if let Some(path) = path_to_open {
            if path.exists() && path.is_file() {
                if let Err(e) = self.open_note_path(&path) {
                    self.show_error(&format!("Failed to reopen last note: {}", e));
                    self.create_default_note_if_configured()?;
                } else {
                    // Restore cursor
                    if let (Some(cursor), Some(buffer)) = (&self.state.cursor, &self.buffer) {
                        if let Some(mut iter) = buffer.iter_at_line(cursor.line as i32) {
                            iter.set_line_offset(cursor.column as i32);
                            buffer.place_cursor(&iter);
                        }
                    }
                }
            } else {
                self.create_default_note_if_configured()?;
            }
        } else {
            self.create_default_note_if_configured()?;
        }

        Ok(())
    }

    fn create_default_note_if_configured(&mut self) -> Result<()> {
        if self.config.behavior.create_note_on_launch_if_none {
            self.new_note()?;
        }
        Ok(())
    }

    pub fn new_note(&mut self) -> Result<()> {
        self.last_cycle_time = None;
        self.cycle_list.clear();
        if self.document.dirty {
            let _ = self.save_current_document();
        }

        let buffer = self
            .buffer
            .as_ref()
            .ok_or_else(|| AppError::Generic("No buffer".to_string()))?;
        if let Some(ref current_path) = self.document.path {
            let (start, end) = buffer.bounds();
            let content = buffer.text(&start, &end, false).to_string();
            let _ = crate::snapshots::create_snapshot_if_needed(
                current_path,
                &content,
                self.config.autosave.snapshot_interval_minutes,
                true,
            );
        }

        self.is_loading = true;
        buffer.set_text("");
        self.is_loading = false;
        self.document = Document::new(None);

        let highlighting = self.config.editor.markdown_highlighting.unwrap_or(true);
        crate::editor::configure_highlighting(buffer, None, highlighting);

        self.show_message("New note created");
        if let Some(ref sv) = self.sourceview {
            sv.grab_focus();
        }
        Ok(())
    }

    pub fn open_note_path(&mut self, path: &Path) -> Result<()> {
        if self.document.dirty {
            let _ = self.save_current_document();
        }

        let buffer = self
            .buffer
            .as_ref()
            .ok_or_else(|| AppError::Generic("No buffer".to_string()))?;
        if let Some(ref current_path) = self.document.path {
            let (start, end) = buffer.bounds();
            let content = buffer.text(&start, &end, false).to_string();
            let _ = crate::snapshots::create_snapshot_if_needed(
                current_path,
                &content,
                self.config.autosave.snapshot_interval_minutes,
                true,
            );
        }

        let content = std::fs::read_to_string(path)?;
        self.is_loading = true;
        buffer.set_text(&content);
        self.is_loading = false;

        let mut doc = Document::new(Some(path.to_path_buf()));
        doc.last_saved_hash = Some(Document::compute_hash(&content));
        if let Ok(meta) = path.metadata() {
            doc.last_modified = meta.modified().ok();
        }
        self.document = doc;

        let highlighting = self.config.editor.markdown_highlighting.unwrap_or(true);
        crate::editor::configure_highlighting(buffer, Some(path), highlighting);

        self.state.last_opened = Some(path.to_path_buf());
        crate::recents::add_recent(
            &mut self.state,
            path.to_path_buf(),
            self.config.behavior.recent_limit,
        );
        let _ = self.state.save();

        let start_iter = buffer.start_iter();
        buffer.place_cursor(&start_iter);

        if let Some(ref sv) = self.sourceview {
            sv.grab_focus();
        }

        Ok(())
    }

    pub fn restore_snapshot(&mut self, snapshot_path: &Path) -> Result<()> {
        let buffer = self
            .buffer
            .as_ref()
            .ok_or_else(|| AppError::Generic("No buffer".to_string()))?;
        let note_path = self.document.path.as_ref().ok_or_else(|| {
            AppError::Generic("Cannot restore snapshot for untitled note".to_string())
        })?;

        let (start, end) = buffer.bounds();
        let current_content = buffer.text(&start, &end, false).to_string();
        let _ = crate::snapshots::create_snapshot_if_needed(
            note_path,
            &current_content,
            self.config.autosave.snapshot_interval_minutes,
            true,
        );

        let content = std::fs::read_to_string(snapshot_path)?;
        self.is_loading = true;
        buffer.set_text(&content);
        self.is_loading = false;

        self.save_current_document()?;
        self.show_message("Revision restored");

        Ok(())
    }

    pub fn save_current_document(&mut self) -> Result<()> {
        let buffer = self
            .buffer
            .as_ref()
            .ok_or_else(|| AppError::Generic("No buffer".to_string()))?;
        let (start, end) = buffer.bounds();
        let content = buffer.text(&start, &end, false).to_string();
        let hash = Document::compute_hash(&content);

        if self.document.path.is_some() && self.document.last_saved_hash == Some(hash) {
            self.document.dirty = false;
            return Ok(());
        }

        let path = match self.document.path.clone() {
            Some(p) => p,
            None => {
                let notes_dir = crate::config::expand_path(&self.config.notes.directory);
                let generated = crate::storage::generate_unique_path(
                    &notes_dir,
                    &content,
                    &self.config.notes.extension,
                );
                self.document.path = Some(generated.clone());
                generated
            }
        };

        if path.exists() {
            if let Ok(metadata) = path.metadata() {
                if let (Some(last_mod), Ok(current_mod)) =
                    (self.document.last_modified, metadata.modified())
                {
                    if current_mod > last_mod {
                        if let Ok(disk_content) = std::fs::read_to_string(&path) {
                            let _ = crate::snapshots::create_snapshot_if_needed(
                                &path,
                                &disk_content,
                                self.config.autosave.snapshot_interval_minutes,
                                true,
                            );
                        }
                    }
                }
            }
        }

        crate::storage::write_atomic(&path, &content)?;

        self.document.dirty = false;
        self.document.last_saved_hash = Some(hash);
        if let Ok(metadata) = path.metadata() {
            self.document.last_modified = metadata.modified().ok();
        }

        self.state.last_opened = Some(path.clone());
        crate::recents::add_recent(
            &mut self.state,
            path.clone(),
            self.config.behavior.recent_limit,
        );
        let _ = self.state.save();

        let _ = crate::snapshots::create_snapshot_if_needed(
            &path,
            &content,
            self.config.autosave.snapshot_interval_minutes,
            false,
        );

        let _ = crate::snapshots::prune_old_snapshots(self.config.autosave.keep_snapshots_days);

        Ok(())
    }

    pub fn reload_config(&mut self) -> Result<()> {
        let (config, warnings) = Config::load_or_create()?;
        self.config = config;
        self.config_warnings = warnings;
        self.state.font_size = self.config.editor.font_size;

        if let (Some(win), Some(sv)) = (&self.window, &self.sourceview) {
            crate::style::apply_styles(
                win,
                sv,
                &self.config,
                &mut self.global_css_provider,
                &mut self.editor_css_provider,
            );
        }

        if let Some(ref buffer) = self.buffer {
            let highlighting = self.config.editor.markdown_highlighting.unwrap_or(true);
            crate::editor::configure_highlighting(
                buffer,
                self.document.path.as_deref(),
                highlighting,
            );
        }

        Ok(())
    }

    pub fn trigger_open_dialog(app_state: Rc<RefCell<AppState>>) {
        let (window, notes_dir) = {
            let state = app_state.borrow();
            let window = state.window.clone();
            let notes_dir = crate::config::expand_path(&state.config.notes.directory);
            (window, notes_dir)
        };

        if let Some(win) = window {
            let app_state_cb = app_state.clone();
            crate::open_dialog::select_file_to_open(&win, Some(notes_dir), move |path| {
                if let Ok(mut state) = app_state_cb.try_borrow_mut() {
                    if let Err(e) = state.open_note_path(&path) {
                        state.show_error(&format!("Failed to open file: {}", e));
                    }
                }
            });
        }
    }

    pub fn on_window_close(&mut self) {
        if self.document.dirty {
            let _ = self.save_current_document();
        }

        if let Some(ref win) = self.window {
            self.state.maximized = win.is_maximized();
            self.state.window_width = win.width();
            self.state.window_height = win.height();
        }

        if let (Some(buffer), Some(_path)) = (&self.buffer, &self.document.path) {
            if let Some(mark) = buffer.mark("insert") {
                let iter = buffer.iter_at_mark(&mark);
                self.state.cursor = Some(CursorState {
                    line: iter.line() as u32,
                    column: iter.line_offset() as u32,
                });
            }
        }

        self.state.font_size = self.config.editor.font_size;

        let _ = self.state.save();
    }

    pub fn current_font_size(&self) -> f32 {
        if let Some(size) = self.config.editor.font_size {
            return size;
        }
        if let Some(size) = self
            .config
            .editor
            .font
            .as_ref()
            .and_then(|f| f.split_whitespace().last())
            .and_then(|s| s.parse::<f32>().ok())
        {
            return size;
        }
        11.0
    }

    pub fn update_font_size(&mut self, new_size: f32) {
        self.config.editor.font_size = Some(new_size);
        self.state.font_size = Some(new_size);
        if let (Some(win), Some(sv)) = (&self.window, &self.sourceview) {
            crate::style::apply_styles(
                win,
                sv,
                &self.config,
                &mut self.global_css_provider,
                &mut self.editor_css_provider,
            );
        }
    }

    pub fn hide_overlays(&mut self) {
        self.last_cycle_time = None;
        self.cycle_list.clear();
        if let Some(ref palette) = self.palette_overlay {
            palette.container.set_visible(false);
        }
        if let Some(ref recent) = self.recent_overlay {
            recent.container.set_visible(false);
        }
        if let Some(ref history) = self.history_overlay {
            history.container.set_visible(false);
        }
        if let Some(ref view) = self.sourceview {
            view.grab_focus();
        }
    }

    pub fn show_palette_overlay(&mut self) {
        self.hide_overlays();
        if !self.config_warnings.is_empty() {
            let warnings = self.config_warnings.clone();
            self.config_warnings.clear();
            for warning in warnings {
                self.show_message(&format!("Warning: {}", warning));
            }
        }
        if let Some(ref palette) = self.palette_overlay {
            palette.container.set_visible(true);
            palette.entry.set_text("");
            palette.entry.grab_focus();
        }
    }

    pub fn show_recent_overlay(&mut self) {
        self.hide_overlays();
        crate::recents::clean_recents(&mut self.state);
        let recents = self.state.recent.clone();
        if let Some(ref recent) = self.recent_overlay {
            recent.populate(&recents);
            recent.container.set_visible(true);
            recent.entry.set_text("");
            recent.entry.grab_focus();
        }
    }

    pub fn show_history_overlay(&mut self) {
        self.hide_overlays();
        if let Some(ref path) = self.document.path {
            if let Ok(snapshots) = crate::snapshots::get_snapshots_for_note(path) {
                if snapshots.is_empty() {
                    self.show_message("No history snapshots exist for this note");
                    return;
                }
                if let Some(ref history) = self.history_overlay {
                    history.populate(&snapshots);
                    history.container.set_visible(true);
                    history.entry.set_text("");
                    history.entry.grab_focus();
                    return;
                }
            }
        }
        self.show_message("No history available for unsaved note");
    }

    pub fn close_current_note(&mut self) -> Result<()> {
        self.last_cycle_time = None;
        self.cycle_list.clear();
        // 1. Save the current document
        self.save_current_document()?;

        // Get the path of the closed note (it has a path now if save succeeded)
        let current_path = self.document.path.clone();

        // 2. Find the most recent note other than the current one
        crate::recents::clean_recents(&mut self.state);

        let target_path = self
            .state
            .recent
            .iter()
            .map(|entry| entry.path.clone())
            .find(|path| {
                if let Some(ref curr) = current_path {
                    let p_canon = path.canonicalize().unwrap_or_else(|_| path.clone());
                    let curr_canon = curr.canonicalize().unwrap_or_else(|_| curr.clone());
                    p_canon != curr_canon
                } else {
                    true
                }
            });

        // 3. Open that note, or if none exists, create a new empty/untitled note
        if let Some(path) = target_path {
            self.open_note_path(&path)?;
            self.show_message("Closed note and switched to most recent note");
        } else {
            self.new_note()?;
            self.state.last_opened = None;
            let _ = self.state.save();
            self.show_message("Closed note; no other notes to switch to");
        }

        Ok(())
    }

    pub fn perform_delete(&mut self) -> Result<()> {
        self.last_cycle_time = None;
        self.cycle_list.clear();
        self.document.dirty = false; // Prevent saving the deleted file or auto-saving on transition

        // 1. Get current path
        let current_path = self.document.path.clone();

        // 2. Delete the file if it exists
        if let Some(ref path) = current_path {
            if path.exists() {
                std::fs::remove_file(path)?;
            }

            // Clean from last_opened if matches
            if let Some(ref last) = self.state.last_opened {
                let last_canon = last.canonicalize().unwrap_or_else(|_| last.clone());
                let curr_canon = path.canonicalize().unwrap_or_else(|_| path.clone());
                if last_canon == curr_canon {
                    self.state.last_opened = None;
                }
            }
        }

        // 3. Clean recents
        crate::recents::clean_recents(&mut self.state);

        // 4. Find the most recent note other than the deleted one
        let target_path = self
            .state
            .recent
            .iter()
            .map(|entry| entry.path.clone())
            .find(|path| {
                if let Some(ref curr) = current_path {
                    let p_canon = path.canonicalize().unwrap_or_else(|_| path.clone());
                    let curr_canon = curr.canonicalize().unwrap_or_else(|_| curr.clone());
                    p_canon != curr_canon
                } else {
                    true
                }
            });

        // 5. Open that note, or if none exists, create a new empty/untitled note
        if let Some(path) = target_path {
            self.open_note_path(&path)?;
            self.show_message("Note deleted");
        } else {
            self.new_note()?;
            self.state.last_opened = None;
            let _ = self.state.save();
            self.show_message("Note deleted");
        }

        Ok(())
    }

    pub fn delete_current_note(app_state: Rc<RefCell<AppState>>) {
        let (window, has_path, filename) = {
            let state = app_state.borrow();
            let has_path = state.document.path.is_some();
            let filename = state
                .document
                .path
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|f| f.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Untitled Note".to_string());
            (state.window.clone(), has_path, filename)
        };

        let heading = format!("Delete \"{}\"?", filename);
        let body = if has_path {
            "Are you sure you want to delete this note? The file will be permanently removed from disk."
        } else {
            "Are you sure you want to discard this unsaved note? All contents will be lost."
        };

        let dialog = adw::AlertDialog::builder()
            .heading(&heading)
            .body(body)
            .build();

        dialog.add_response("cancel", "Cancel");
        dialog.add_response("delete", "Delete");
        dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        let app_state_cb = app_state.clone();
        dialog.choose(window.as_ref(), gio::Cancellable::NONE, move |response| {
            if response == "delete" {
                if let Ok(mut state) = app_state_cb.try_borrow_mut() {
                    if let Err(e) = state.perform_delete() {
                        state.show_error(&format!("Failed to delete note: {}", e));
                    }
                }
            }
        });
    }

    pub fn cycle_recent_notes(&mut self, forward: bool) -> Result<()> {
        let now = std::time::Instant::now();
        let is_session_active = if let Some(last_time) = self.last_cycle_time {
            now.duration_since(last_time) < std::time::Duration::from_millis(1500)
                && !self.cycle_list.is_empty()
        } else {
            false
        };

        if !is_session_active {
            // Start a new session
            crate::recents::clean_recents(&mut self.state);
            let mut list: Vec<std::path::PathBuf> =
                self.state.recent.iter().map(|e| e.path.clone()).collect();

            // If the current note is dirty, save it first, which might add it to recents
            if self.document.dirty {
                let _ = self.save_current_document();
                // Refresh list if it changed
                list = self.state.recent.iter().map(|e| e.path.clone()).collect();
            }

            if list.is_empty() {
                self.show_message("No recent notes to cycle through");
                return Ok(());
            }

            // Find current note index
            let current_path = self.document.path.as_ref();
            let found_idx = current_path.and_then(|curr| {
                let curr_canon = curr.canonicalize().ok().unwrap_or_else(|| curr.clone());
                list.iter().position(|p| {
                    let p_canon = p.canonicalize().ok().unwrap_or_else(|| p.clone());
                    p_canon == curr_canon
                })
            });

            let start_idx = match found_idx {
                Some(idx) => idx,
                None => {
                    if forward {
                        list.len() - 1
                    } else {
                        0
                    }
                }
            };

            self.cycle_list = list;
            self.cycle_index = start_idx;
        }

        let len = self.cycle_list.len();
        if len <= 1 {
            self.show_message("No other recent notes to cycle through");
            self.last_cycle_time = Some(now); // keep session alive if they spam it
            return Ok(());
        }

        if forward {
            self.cycle_index = (self.cycle_index + 1) % len;
        } else {
            self.cycle_index = (self.cycle_index + len - 1) % len;
        }

        let target_path = self.cycle_list[self.cycle_index].clone();
        self.open_note_path(&target_path)?;

        self.last_cycle_time = Some(now);
        Ok(())
    }

    pub fn execute_command_static(app_state: Rc<RefCell<AppState>>, cmd: &str) {
        match cmd {
            "New note" => {
                if let Ok(mut state) = app_state.try_borrow_mut() {
                    if let Err(e) = state.new_note() {
                        state.show_error(&format!("Failed to create new note: {}", e));
                    }
                }
            }
            "Close current note" => {
                if let Ok(mut state) = app_state.try_borrow_mut() {
                    if let Err(e) = state.close_current_note() {
                        state.show_error(&format!("Failed to close note: {}", e));
                    }
                }
            }
            "Delete current note" => {
                Self::delete_current_note(app_state);
            }
            "Open Markdown file…" => {
                Self::trigger_open_dialog(app_state);
            }
            "Recent notes" => {
                if let Ok(mut state) = app_state.try_borrow_mut() {
                    state.show_recent_overlay();
                }
            }
            "Copy whole document" => {
                let (buf, toast) = {
                    let state = app_state.borrow();
                    (state.buffer.clone(), state.toast_overlay.clone())
                };
                if let (Some(buffer), Some(overlay)) = (buf, toast) {
                    crate::clipboard::copy_whole_document(&buffer, &overlay);
                }
            }
            "Show revision history" => {
                if let Ok(mut state) = app_state.try_borrow_mut() {
                    state.show_history_overlay();
                }
            }
            "Reveal current file in file manager" => {
                let path = {
                    let state = app_state.borrow();
                    state.document.path.clone()
                };
                if let Some(p) = path {
                    if let Some(parent) = p.parent() {
                        let _ = std::process::Command::new("xdg-open").arg(parent).spawn();
                    }
                } else {
                    if let Ok(state) = app_state.try_borrow() {
                        state.show_message("Untitled note is not saved on disk yet");
                    }
                }
            }
            "Open configuration file" => {
                if let Ok(config_path) = Config::get_config_path() {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(config_path)
                        .spawn();
                }
            }
            "Reload configuration" => {
                if let Ok(mut state) = app_state.try_borrow_mut() {
                    if let Err(e) = state.reload_config() {
                        state.show_error(&format!("Reload config failed: {}", e));
                    } else {
                        state.show_message("Configuration reloaded");
                    }
                }
            }
            "Toggle Markdown highlighting" => {
                if let Ok(mut state) = app_state.try_borrow_mut() {
                    let enabled = !state.config.editor.markdown_highlighting.unwrap_or(true);
                    state.config.editor.markdown_highlighting = Some(enabled);
                    if let Some(ref buffer) = state.buffer {
                        crate::editor::configure_highlighting(
                            buffer,
                            state.document.path.as_deref(),
                            enabled,
                        );
                    }
                    state.show_message(&format!(
                        "Markdown highlighting: {}",
                        if enabled { "On" } else { "Off" }
                    ));
                }
            }
            "Toggle line wrapping" => {
                if let Ok(mut state) = app_state.try_borrow_mut() {
                    let enabled = !state.config.editor.wrap_text.unwrap_or(true);
                    state.config.editor.wrap_text = Some(enabled);
                    if let Some(ref view) = state.sourceview {
                        view.set_wrap_mode(if enabled {
                            gtk::WrapMode::Word
                        } else {
                            gtk::WrapMode::None
                        });
                    }
                    state.show_message(&format!(
                        "Line wrapping: {}",
                        if enabled { "On" } else { "Off" }
                    ));
                }
            }
            "Quit" => {
                let win = {
                    let state = app_state.borrow();
                    state.window.clone()
                };
                if let Some(w) = win {
                    w.close();
                }
            }
            _ => {}
        }
    }

    pub fn show_message(&self, msg: &str) {
        if let Some(ref overlay) = self.toast_overlay {
            let toast = adw::Toast::new(msg);
            toast.set_timeout(2);
            overlay.add_toast(toast);
        }
    }

    pub fn show_error(&self, msg: &str) {
        self.show_message(msg);
    }
}
