use std::path::PathBuf;
use gtk::prelude::*;

pub fn select_file_to_open<F>(
    parent_window: &adw::ApplicationWindow,
    start_dir: Option<PathBuf>,
    on_selected: F,
) where
    F: Fn(PathBuf) + 'static,
{
    let dialog = gtk::FileChooserNative::new(
        Some("Open Note"),
        Some(parent_window),
        gtk::FileChooserAction::Open,
        Some("Open"),
        Some("Cancel"),
    );

    if let Some(dir) = start_dir {
        if dir.exists() {
            let file = gio::File::for_path(dir);
            let _ = dialog.set_current_folder(Some(&file));
        }
    }

    let filter_md = gtk::FileFilter::new();
    filter_md.set_name(Some("Markdown (*.md, *.markdown)"));
    filter_md.add_pattern("*.md");
    filter_md.add_pattern("*.markdown");
    dialog.add_filter(&filter_md);

    let filter_txt = gtk::FileFilter::new();
    filter_txt.set_name(Some("Text files (*.txt)"));
    filter_txt.add_pattern("*.txt");
    dialog.add_filter(&filter_txt);

    let filter_all = gtk::FileFilter::new();
    filter_all.set_name(Some("All files"));
    filter_all.add_pattern("*");
    dialog.add_filter(&filter_all);

    dialog.connect_response(move |dialog, response| {
        if response == gtk::ResponseType::Accept {
            if let Some(file) = dialog.file() {
                if let Some(path) = file.path() {
                    on_selected(path);
                }
            }
        }
        dialog.destroy();
    });

    dialog.show();
}
