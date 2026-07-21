use gtk::prelude::*;
use sourceview5::prelude::*;
use std::path::Path;

pub fn create_editor() -> (sourceview5::View, sourceview5::Buffer) {
    let buffer = sourceview5::Buffer::new(None);

    buffer.set_enable_undo(true);

    let view = sourceview5::View::with_buffer(&buffer);

    view.set_wrap_mode(gtk::WrapMode::Word);
    view.set_insert_spaces_instead_of_tabs(true);
    view.set_tab_width(4);
    view.set_indent_width(4);
    view.set_auto_indent(true);

    // Set some nice default text margin/padding
    view.set_left_margin(8);
    view.set_right_margin(8);
    view.set_top_margin(8);
    view.set_bottom_margin(8);

    (view, buffer)
}

pub fn configure_highlighting(buffer: &sourceview5::Buffer, path: Option<&Path>, enabled: bool) {
    if !enabled {
        buffer.set_language(None);
        return;
    }

    let lm = sourceview5::LanguageManager::default();

    let mut lang = None;
    if let Some(p) = path {
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if ext == "md" || ext == "markdown" {
                lang = lm.language("markdown");
            }
        }
    } else {
        lang = lm.language("markdown");
    }

    buffer.set_language(lang.as_ref());
}
