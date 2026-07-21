use gtk::prelude::*;

pub fn copy_whole_document(buffer: &sourceview5::Buffer, toast_overlay: &adw::ToastOverlay) {
    let (start, end) = buffer.bounds();
    let text = buffer.text(&start, &end, false);

    if let Some(display) = gtk::gdk::Display::default() {
        let clipboard = display.clipboard();
        clipboard.set_text(&text);

        let toast = adw::Toast::new("Copied document");
        toast.set_timeout(2);
        toast_overlay.add_toast(toast);
    }
}
