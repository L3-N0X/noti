use crate::app::AppState;
use crate::state::RecentEntry;
use gtk::gdk::Key;
use gtk::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub struct RecentOverlay {
    pub container: gtk::Box,
    pub entry: gtk::SearchEntry,
    pub listbox: gtk::ListBox,
}

impl RecentOverlay {
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .width_request(450)
            .height_request(300)
            .margin_top(40)
            .visible(false)
            .css_classes(vec!["card".to_string(), "recent-overlay".to_string()])
            .build();

        let entry = gtk::SearchEntry::builder()
            .placeholder_text("Search recent notes...")
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
            let path_str = row.widget_name();
            path_str.to_string().to_lowercase().contains(&text)
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

        // 3. Row activated
        let app_state_clone = app_state.clone();
        listbox.connect_row_activated(move |_, row| {
            let path_str = row.widget_name();
            let path = PathBuf::from(path_str.to_string());
            if let Ok(mut state) = app_state_clone.try_borrow_mut() {
                state.hide_overlays();
                let app_state_idle = app_state_clone.clone();
                glib::idle_add_local_once(move || {
                    if let Ok(mut state) = app_state_idle.try_borrow_mut() {
                        if let Err(e) = state.open_note_path(&path) {
                            state.show_error(&format!("Failed to open note: {}", e));
                        }
                    }
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

    pub fn populate(&self, recents: &[RecentEntry]) {
        // Clear current listbox
        while let Some(row) = self.listbox.row_at_index(0) {
            self.listbox.remove(&row);
        }

        for entry in recents {
            let path = &entry.path;
            if !path.exists() {
                continue;
            }

            let filename = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled Note".to_string());

            let parent_dir = path
                .parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();

            let box_row = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .margin_top(6)
                .margin_bottom(6)
                .margin_start(12)
                .margin_end(12)
                .build();

            let title_lbl = gtk::Label::builder().label(&filename).xalign(0.0).build();

            // Set small size/dim look
            let subtitle_lbl = gtk::Label::builder()
                .label(&parent_dir)
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

        // Select first item by default
        if let Some(first_row) = self.listbox.row_at_index(0) {
            self.listbox.select_row(Some(&first_row));
        }
    }
}
