mod errors;
mod config;
mod state;
mod recents;
mod document;
mod storage;
mod snapshots;
mod style;
mod clipboard;
mod open_dialog;
mod palette;
mod recent_dialog;
mod history_dialog;
mod editor;
mod autosave;
mod app;

use std::rc::Rc;
use std::cell::RefCell;
use gtk::prelude::*;
use adw::prelude::*;
use gtk::gdk::Key;
use crate::app::AppState;
use crate::palette::PaletteOverlay;
use crate::recent_dialog::RecentOverlay;
use crate::history_dialog::HistoryOverlay;

fn main() -> glib::ExitCode {
    let application = adw::Application::builder()
        .application_id("io.github.L3-N0X.noti")
        .build();

    application.connect_activate(|app| {
        // Initialize style provider for the overlays
        let overlay_css = gtk::CssProvider::new();
        overlay_css.load_from_data(r#"
            .palette-overlay, .recent-overlay, .history-overlay {
                background-color: @theme_base_color;
                box-shadow: 0 4px 16px rgba(0,0,0,0.25);
                border-radius: 8px;
            }
            .dim-label {
                opacity: 0.55;
                font-size: 0.85em;
            }
            toast, .toast {
                background: @theme_base_color !important;
                background-color: @theme_base_color !important;
                background-image: none !important;
                color: @theme_text_color !important;
                border: 1px solid mix(@theme_fg_color, @theme_base_color, 0.12) !important;
                border-radius: 12px;
                padding: 10px 20px;
                margin-bottom: 24px;
                box-shadow: 0 6px 20px rgba(0, 0, 0, 0.2);
            }
            toast label, .toast label, toast label.heading, .toast label.heading, toast * {
                color: @theme_text_color !important;
                font-weight: 500;
            }
        "#);

        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &overlay_css,
                gtk::STYLE_PROVIDER_PRIORITY_USER,
            );
        }

        let app_state = Rc::new(RefCell::new(AppState::new()));

        let (width, height, maximized) = {
            let state = app_state.borrow();
            (state.state.window_width, state.state.window_height, state.state.maximized)
        };

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("noti")
            .default_width(width)
            .default_height(height)
            .build();

        if maximized {
            window.maximize();
        }

        let toast_overlay = adw::ToastOverlay::new();
        let overlay = gtk::Overlay::builder()
            .vexpand(true)
            .hexpand(true)
            .build();

        let (view, buffer) = crate::editor::create_editor();

        let scroll_controller = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        let app_state_scroll = app_state.clone();
        scroll_controller.connect_scroll(move |controller, _dx, dy| {
            let state = controller.current_event_state();
            if state.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
                if let Ok(mut app_state) = app_state_scroll.try_borrow_mut() {
                    let current_size = app_state.current_font_size();
                    let delta = if dy < 0.0 { 1.0 } else { -1.0 };
                    app_state.update_font_size((current_size + delta).max(1.0));
                }
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        view.add_controller(scroll_controller);

        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .vexpand(true)
            .hexpand(true)
            .child(&view)
            .build();

        overlay.set_child(Some(&scrolled));

        // Create overlays
        let palette = PaletteOverlay::new(app_state.clone());
        let recent = RecentOverlay::new(app_state.clone());
        let history = HistoryOverlay::new(app_state.clone());

        overlay.add_overlay(&palette.container);
        overlay.add_overlay(&recent.container);
        overlay.add_overlay(&history.container);

        toast_overlay.set_child(Some(&overlay));
        window.set_content(Some(&toast_overlay));

        // Connect changed signal for autosave
        let app_state_changed = app_state.clone();
        buffer.connect_changed(move |_| {
            let mut is_user_change = false;
            if let Ok(state) = app_state_changed.try_borrow() {
                if !state.is_loading {
                    is_user_change = true;
                }
            }

            if is_user_change {
                if let Ok(mut state) = app_state_changed.try_borrow_mut() {
                    state.document.dirty = true;
                    state.last_cycle_time = None;
                    state.cycle_list.clear();
                    drop(state);
                    crate::autosave::schedule_autosave(app_state_changed.clone());
                }
            }
        });

        // Event handler for window shortcuts
        let app_state_key = app_state.clone();
        let key_controller = gtk::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        key_controller.connect_key_pressed(move |_, keyval, _, state| {
            let has_ctrl = state.contains(gtk::gdk::ModifierType::CONTROL_MASK);
            let has_shift = state.contains(gtk::gdk::ModifierType::SHIFT_MASK);
            let has_alt = state.contains(gtk::gdk::ModifierType::ALT_MASK);
            let keyval_lower = keyval.to_lower();

            let any_overlay_visible = {
                if let Ok(state) = app_state_key.try_borrow() {
                    let p_vis = state.palette_overlay.as_ref().map(|o| o.container.get_visible()).unwrap_or(false);
                    let r_vis = state.recent_overlay.as_ref().map(|o| o.container.get_visible()).unwrap_or(false);
                    let h_vis = state.history_overlay.as_ref().map(|o| o.container.get_visible()).unwrap_or(false);
                    p_vis || r_vis || h_vis
                } else {
                    false
                }
            };

            if any_overlay_visible && keyval == Key::Escape {
                if let Ok(mut state) = app_state_key.try_borrow_mut() {
                    state.hide_overlays();
                }
                return glib::Propagation::Stop;
            }

            if has_alt && !has_shift && !has_ctrl {
                if keyval == Key::Left {
                    if !any_overlay_visible {
                        if let Ok(mut state) = app_state_key.try_borrow_mut() {
                            if let Err(e) = state.cycle_recent_notes(false) {
                                state.show_error(&format!("Failed to cycle notes: {}", e));
                            }
                        }
                        return glib::Propagation::Stop;
                    }
                } else if keyval == Key::Right {
                    if !any_overlay_visible {
                        if let Ok(mut state) = app_state_key.try_borrow_mut() {
                            if let Err(e) = state.cycle_recent_notes(true) {
                                state.show_error(&format!("Failed to cycle notes: {}", e));
                            }
                        }
                        return glib::Propagation::Stop;
                    }
                }
            }

            if has_ctrl {
                if keyval_lower == Key::p && !has_shift {
                    if let Ok(mut state) = app_state_key.try_borrow_mut() {
                        state.show_palette_overlay();
                    }
                    glib::Propagation::Stop
                } else if keyval_lower == Key::r && !has_shift {
                    if let Ok(mut state) = app_state_key.try_borrow_mut() {
                        state.show_recent_overlay();
                    }
                    glib::Propagation::Stop
                } else if keyval_lower == Key::o && !has_shift {
                    AppState::execute_command_static(app_state_key.clone(), "Open Markdown file…");
                    glib::Propagation::Stop
                } else if keyval_lower == Key::n && !has_shift {
                    AppState::execute_command_static(app_state_key.clone(), "New note");
                    glib::Propagation::Stop
                } else if keyval_lower == Key::c && has_shift {
                    AppState::execute_command_static(app_state_key.clone(), "Copy whole document");
                    glib::Propagation::Stop
                } else if keyval_lower == Key::q && !has_shift {
                    AppState::execute_command_static(app_state_key.clone(), "Quit");
                    glib::Propagation::Stop
                } else if keyval_lower == Key::w && !has_shift {
                    AppState::execute_command_static(app_state_key.clone(), "Close current note");
                    glib::Propagation::Stop
                } else if keyval == Key::plus || keyval == Key::equal || keyval == Key::KP_Add {
                    if let Ok(mut state) = app_state_key.try_borrow_mut() {
                        let current_size = state.current_font_size();
                        state.update_font_size((current_size + 1.0).max(1.0));
                    }
                    glib::Propagation::Stop
                } else if keyval == Key::minus || keyval == Key::KP_Subtract {
                    if let Ok(mut state) = app_state_key.try_borrow_mut() {
                        let current_size = state.current_font_size();
                        state.update_font_size((current_size - 1.0).max(1.0));
                    }
                    glib::Propagation::Stop
                } else {
                    glib::Propagation::Proceed
                }
            } else {
                glib::Propagation::Proceed
            }
        });
        window.add_controller(key_controller);

        // Window close handler
        let app_state_close = app_state.clone();
        window.connect_close_request(move |_| {
            if let Ok(mut state) = app_state_close.try_borrow_mut() {
                state.on_window_close();
            }
            glib::Propagation::Proceed
        });

        // Initialize state, views, etc.
        if let Ok(mut state) = app_state.try_borrow_mut() {
            state.init(
                window.clone(),
                view,
                buffer,
                toast_overlay,
                palette,
                recent,
                history,
            );
            if let Err(e) = state.load_initial_note() {
                state.show_error(&format!("Failed to load note: {}", e));
            }
        }

        window.present();

        // Focus the editor on startup
        if let Ok(state) = app_state.try_borrow() {
            if let Some(ref sv) = state.sourceview {
                sv.grab_focus();
            }
        }
    });

    application.run()
}
