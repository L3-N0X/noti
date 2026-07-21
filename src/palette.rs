use crate::app::AppState;
use gtk::gdk::Key;
use gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct PaletteOverlay {
    pub container: gtk::Box,
    pub entry: gtk::SearchEntry,
}

impl PaletteOverlay {
    pub fn new(app_state: Rc<RefCell<AppState>>) -> Self {
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Start)
            .width_request(450)
            .height_request(300)
            .margin_top(40)
            .visible(false)
            .css_classes(vec!["card".to_string(), "palette-overlay".to_string()])
            .build();

        let entry = gtk::SearchEntry::builder()
            .placeholder_text("Search commands...")
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

        let commands = vec![
            "New note",
            "Close current note",
            "Delete current note",
            "Open Markdown file…",
            "Recent notes",
            "Copy whole document",
            "Show revision history",
            "Reveal current file in file manager",
            "Open configuration file",
            "Reload configuration",
            "Toggle Markdown highlighting",
            "Toggle line wrapping",
            "Quit",
        ];

        for cmd in commands {
            let label = gtk::Label::builder()
                .label(cmd)
                .xalign(0.0)
                .margin_top(8)
                .margin_bottom(8)
                .margin_start(12)
                .margin_end(12)
                .build();

            let row = gtk::ListBoxRow::new();
            row.set_child(Some(&label));
            row.set_widget_name(cmd); // use name to identify command
            listbox.append(&row);
        }

        // 1. Text changed filter
        let listbox_clone = listbox.clone();
        entry.connect_search_changed(move |_| {
            listbox_clone.invalidate_filter();

            // Select the first visible item
            let mut i = 0;
            while let Some(row) = listbox_clone.row_at_index(i) {
                if row.get_visible() {
                    listbox_clone.select_row(Some(&row));
                    break;
                }
                i += 1;
            }
        });

        let entry_text = entry.clone();
        listbox.set_filter_func(move |row| {
            let text = entry_text.text().to_string().to_lowercase();
            if text.is_empty() {
                return true;
            }
            let cmd = row.widget_name();
            cmd.to_string().to_lowercase().contains(&text)
        });

        // 2. Row activated
        let app_state_clone = app_state.clone();
        listbox.connect_row_activated(move |_, row| {
            let cmd = row.widget_name();
            if let Ok(mut state) = app_state_clone.try_borrow_mut() {
                state.hide_overlays();
                let command_name = cmd.to_string();
                // Execute command in idle to avoid borrowing issue during callback
                let app_state_idle = app_state_clone.clone();
                glib::idle_add_local_once(move || {
                    AppState::execute_command_static(app_state_idle, &command_name);
                });
            }
        });

        // 3. Keys controller for entry
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

        // 4. Keys controller for listbox (to focus entry on Up or typing)
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

        if let Some(first_row) = listbox.row_at_index(0) {
            listbox.select_row(Some(&first_row));
        }

        Self { container, entry }
    }
}
