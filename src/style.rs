use crate::config::Config;
use gtk::prelude::*;
use sourceview5::prelude::*;

pub fn update_editor_scheme(buffer: &sourceview5::Buffer) {
    let style_manager = adw::StyleManager::default();
    let is_dark = style_manager.is_dark();
    let scheme_manager = sourceview5::StyleSchemeManager::default();

    let scheme_id = if is_dark {
        if scheme_manager.scheme("Adwaita-dark").is_some() {
            "Adwaita-dark"
        } else if scheme_manager.scheme("adwaita-dark").is_some() {
            "adwaita-dark"
        } else {
            "oblivion"
        }
    } else {
        if scheme_manager.scheme("Adwaita").is_some() {
            "Adwaita"
        } else if scheme_manager.scheme("adwaita").is_some() {
            "adwaita"
        } else {
            "classic"
        }
    };

    if let Some(scheme) = scheme_manager.scheme(scheme_id) {
        buffer.set_style_scheme(Some(&scheme));
    } else {
        buffer.set_style_scheme(None);
    }
}

pub fn apply_styles(
    window: &adw::ApplicationWindow,
    sourceview: &sourceview5::View,
    config: &Config,
    old_global_provider: &mut Option<gtk::CssProvider>,
    old_editor_provider: &mut Option<gtk::CssProvider>,
) {
    // 1. Theme/Color Scheme
    let style_manager = adw::StyleManager::default();
    if let Some(ref scheme) = config.window.color_scheme {
        match scheme.as_str() {
            "light" => style_manager.set_color_scheme(adw::ColorScheme::ForceLight),
            "dark" => style_manager.set_color_scheme(adw::ColorScheme::ForceDark),
            _ => style_manager.set_color_scheme(adw::ColorScheme::Default),
        }
    } else {
        style_manager.set_color_scheme(adw::ColorScheme::Default);
    }

    // 2. Window Opacity
    if let Some(opacity) = config.window.window_opacity {
        window.set_opacity(opacity);
    } else {
        window.set_opacity(1.0);
    }

    // 3. Editor Padding
    if let Some(padding) = config.editor.padding {
        sourceview.set_left_margin(padding);
        sourceview.set_right_margin(padding);
        sourceview.set_top_margin(padding);
        sourceview.set_bottom_margin(padding);
    } else {
        sourceview.set_left_margin(8);
        sourceview.set_right_margin(8);
        sourceview.set_top_margin(8);
        sourceview.set_bottom_margin(8);
    }

    // 4. Editor line numbers & wrapping
    let show_ln = config.editor.line_numbers.unwrap_or(false);
    sourceview.set_show_line_numbers(show_ln);

    let wrap = config.editor.wrap_text.unwrap_or(true);
    if wrap {
        sourceview.set_wrap_mode(gtk::WrapMode::Word);
    } else {
        sourceview.set_wrap_mode(gtk::WrapMode::None);
    }

    // 5. Editor background transparency, text_color & font (via CSS provider)
    if let Some(provider) = old_editor_provider.take() {
        sourceview.style_context().remove_provider(&provider);
    }

    let mut editor_css =
        String::from("textview, textview text {\n    background-color: transparent;\n}\n");
    if let Some(ref color) = config.editor.text_color {
        editor_css.push_str(&format!("textview text {{ color: {}; }}\n", color));
    }
    if let Some(ref font) = config.editor.font {
        let mut parts: Vec<&str> = font.split_whitespace().collect();
        if let Some(last) = parts.last() {
            if let Ok(parsed_size) = last.parse::<f32>() {
                parts.pop();
                let family = parts.join(" ");
                editor_css.push_str(&format!(
                    "textview {{ font-family: \"{}\"; font-size: {}pt; }}\n",
                    family, parsed_size
                ));
            } else {
                editor_css.push_str(&format!("textview {{ font-family: \"{}\"; }}\n", font));
            }
        }
    }
    if let Some(size) = config.editor.font_size {
        editor_css.push_str(&format!("textview {{ font-size: {}pt; }}\n", size));
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_data(&editor_css);
    sourceview
        .style_context()
        .add_provider(&provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    *old_editor_provider = Some(provider);

    // 6. Global custom CSS
    if let Some(provider) = old_global_provider.take() {
        if let Some(display) = gtk::gdk::Display::default() {
            gtk::style_context_remove_provider_for_display(&display, &provider);
        }
    }

    if let Some(ref css_file) = config.appearance.css_file {
        let expanded = crate::config::expand_path(css_file);
        if expanded.exists() {
            if let Ok(css_data) = std::fs::read_to_string(&expanded) {
                let provider = gtk::CssProvider::new();
                provider.load_from_data(&css_data);
                if let Some(display) = gtk::gdk::Display::default() {
                    gtk::style_context_add_provider_for_display(
                        &display,
                        &provider,
                        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                    *old_global_provider = Some(provider);
                }
            }
        }
    }

    // 7. Update GtkSourceView style scheme to match theme dark/light state
    if let Ok(source_buffer) = sourceview.buffer().downcast::<sourceview5::Buffer>() {
        update_editor_scheme(&source_buffer);
    }
}
