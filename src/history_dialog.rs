use crate::app::AppState;
use adw::prelude::*;
use gtk::gdk::Key;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub struct HistoryOverlay {
    pub container: gtk::Box,
    pub entry: gtk::SearchEntry,
    pub listbox: gtk::ListBox,
}

impl HistoryOverlay {
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .width_request(450)
            .height_request(300)
            .margin_top(40)
            .visible(false)
            .css_classes(vec!["card".to_string(), "history-overlay".to_string()])
            .build();

        let entry = gtk::SearchEntry::builder()
            .placeholder_text("Search revision history...")
            .margin_top(8)
            .margin_start(8)
            .margin_end(8)
            .margin_bottom(8)
            .build();

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .build();

        let listbox = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .build();

        scrolled.set_child(Some(&listbox));
        container.append(&entry);
        container.append(&scrolled);

        // 1. Filter function
        let entry_text = entry.clone();
        listbox.set_filter_func(move |row| {
            let text = entry_text.text().to_string().to_lowercase();
            if text.is_empty() {
                return true;
            }
            let name = row.widget_name();
            name.to_string().to_lowercase().contains(&text)
        });

        // 2. Search changed -> invalidate filter
        let listbox_clone = listbox.clone();
        entry.connect_search_changed(move |_| {
            listbox_clone.invalidate_filter();

            // Select first visible
            let mut i = 0;
            while let Some(row) = listbox_clone.row_at_index(i) {
                if row.get_visible() {
                    listbox_clone.select_row(Some(&row));
                    break;
                }
                i += 1;
            }
        });

        // 3. Row activated (ask confirmation and restore)
        let app_state_clone = app_state.clone();
        listbox.connect_row_activated(move |_, row| {
            let path_str = row.widget_name();
            let snapshot_path = PathBuf::from(path_str.to_string());
            
            if let Ok(mut state) = app_state_clone.try_borrow_mut() {
                state.hide_overlays();
                
                let window = state.window.clone();
                let app_state_confirm = app_state_clone.clone();
                
                glib::idle_add_local_once(move || {
                    let dialog = adw::AlertDialog::builder()
                        .heading("Restore Revision?")
                        .body("This will replace the current editor contents with the selected revision. A backup snapshot of your current contents will be created first.")
                        .build();
                    
                    dialog.add_response("cancel", "Cancel");
                    dialog.add_response("restore", "Restore");
                    dialog.set_default_response(Some("cancel"));
                    dialog.set_close_response("cancel");
                    
                    let app_state_cb = app_state_confirm.clone();
                    dialog.choose(window.as_ref(), gio::Cancellable::NONE, move |response| {
                        if response == "restore" {
                            if let Ok(mut state) = app_state_cb.try_borrow_mut() {
                                if let Err(e) = state.restore_snapshot(&snapshot_path) {
                                    state.show_error(&format!("Restore failed: {}", e));
                                }
                            }
                        }
                    });
                });
            }
        });

        // 4. Key events
        let listbox_key = listbox.clone();
        let app_state_key = app_state.clone();
        let key_controller = gtk::EventControllerKey::new();
        key_controller.connect_key_pressed(move |_, keyval, _, _| match keyval {
            Key::Escape => {
                if let Ok(mut state) = app_state_key.try_borrow_mut() {
                    state.hide_overlays();
                }
                glib::Propagation::Stop
            }
            Key::Down => {
                if let Some(row) = listbox_key.selected_row() {
                    row.grab_focus();
                } else if let Some(row) = listbox_key.row_at_index(0) {
                    row.grab_focus();
                }
                glib::Propagation::Stop
            }
            Key::Return | Key::KP_Enter => {
                if let Some(row) = listbox_key.selected_row() {
                    row.activate();
                }
                glib::Propagation::Stop
            }
            _ => glib::Propagation::Proceed,
        });
        entry.add_controller(key_controller);

        // 5. Keys controller for listbox (to focus entry on Up or typing)
        let entry_focus = entry.clone();
        let app_state_listbox = app_state.clone();
        let listbox_clone = listbox.clone();
        let listbox_key_ctrl = gtk::EventControllerKey::new();
        listbox_key_ctrl.connect_key_pressed(move |_, keyval, _, _| match keyval {
            Key::Escape => {
                if let Ok(mut state) = app_state_listbox.try_borrow_mut() {
                    state.hide_overlays();
                }
                glib::Propagation::Stop
            }
            Key::Up => {
                if let Some(row) = listbox_clone.selected_row() {
                    if row.index() == 0 {
                        entry_focus.grab_focus();
                        return glib::Propagation::Stop;
                    }
                }
                glib::Propagation::Proceed
            }
            Key::Return | Key::KP_Enter => glib::Propagation::Proceed,
            _ => {
                if let Some(c) = keyval.to_unicode() {
                    if !c.is_control() {
                        entry_focus.grab_focus();
                    }
                }
                glib::Propagation::Proceed
            }
        });
        listbox.add_controller(listbox_key_ctrl);

        Self {
            container,
            entry,
            listbox,
        }
    }

    pub fn populate(&self, snapshots: &[PathBuf]) {
        while let Some(row) = self.listbox.row_at_index(0) {
            self.listbox.remove(&row);
        }

        for path in snapshots.iter().rev() {
            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if filename.len() < 19 {
                continue;
            }

            // Convert 2026-07-21_20-18-34-project-ideas.md to a human-readable date/time
            let date_part = &filename[0..10];
            let time_part = filename[11..19].replace("-", ":");
            let timestamp = format!("{} {}", date_part, time_part);

            let excerpt = std::fs::read_to_string(path)
                .ok()
                .and_then(|content| {
                    content
                        .lines()
                        .map(|l| l.trim())
                        .find(|l| !l.is_empty())
                        .map(|l| {
                            let stripped = l.replace("#", "").trim().to_string();
                            if stripped.len() > 50 {
                                format!("{}...", &stripped[0..50])
                            } else {
                                stripped
                            }
                        })
                })
                .unwrap_or_else(|| "Empty document".to_string());

            let box_row = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(12)
                .margin_end(12)
                .build();

            let title_lbl = gtk::Label::builder().label(&timestamp).xalign(0.0).build();

            let subtitle_lbl = gtk::Label::builder()
                .label(&excerpt)
                .xalign(0.0)
                .css_classes(vec!["dim-label".to_string()])
                .build();

            box_row.append(&title_lbl);
            box_row.append(&subtitle_lbl);

            let row = gtk::ListBoxRow::new();
            row.set_child(Some(&box_row));
            row.set_widget_name(&path.to_string_lossy());
            self.listbox.append(&row);
        }

        if let Some(first_row) = self.listbox.row_at_index(0) {
            self.listbox.select_row(Some(&first_row));
        }
    }
}
