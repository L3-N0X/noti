use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotesConfig {
    pub directory: String,
    pub extension: String,
}

impl Default for NotesConfig {
    fn default() -> Self {
        Self {
            directory: "~/Documents/Notes".to_string(),
            extension: "md".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosaveConfig {
    pub delay_ms: u64,
    pub snapshot_interval_minutes: u64,
    pub keep_snapshots_days: u64,
}

impl Default for AutosaveConfig {
    fn default() -> Self {
        Self {
            delay_ms: 500,
            snapshot_interval_minutes: 5,
            keep_snapshots_days: 90,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub reopen_last_note: bool,
    pub create_note_on_launch_if_none: bool,
    pub recent_limit: usize,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            reopen_last_note: true,
            create_note_on_launch_if_none: true,
            recent_limit: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EditorConfig {
    pub text_color: Option<String>,
    pub font: Option<String>,
    pub font_size: Option<f32>,
    pub padding: Option<i32>,
    pub line_numbers: Option<bool>,
    pub markdown_highlighting: Option<bool>,
    pub wrap_text: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowConfig {
    pub window_opacity: Option<f64>,
    pub color_scheme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppearanceConfig {
    pub css_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub notes: NotesConfig,
    pub autosave: AutosaveConfig,
    pub behavior: BehaviorConfig,
    pub editor: EditorConfig,
    pub window: WindowConfig,
    pub appearance: AppearanceConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            notes: NotesConfig::default(),
            autosave: AutosaveConfig::default(),
            behavior: BehaviorConfig::default(),
            editor: EditorConfig::default(),
            window: WindowConfig::default(),
            appearance: AppearanceConfig::default(),
        }
    }
}

pub fn expand_path(p: &str) -> PathBuf {
    if p.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(&p[2..]);
        }
    }
    PathBuf::from(p)
}

const DEFAULT_CONFIG_CONTENT: &str = r##"# noti configuration
#
# Changes take effect after restarting noti, or immediately after
# running "Reload configuration" from Ctrl+P.

[notes]
# Folder for newly created notes. Existing files can still be opened anywhere.
directory = "~/Documents/Notes"

# New notes are saved with this extension.
extension = "md"

[autosave]
# Wait this long after typing stops before saving.
delay_ms = 500

# Create a revision copy at most once per changed note during this interval.
snapshot_interval_minutes = 5

# Remove automatic revision snapshots older than this number of days.
keep_snapshots_days = 90

[behavior]
# Reopen the previously active note on launch.
reopen_last_note = true

# If no previous note exists, create a fresh empty note.
create_note_on_launch_if_none = true

# Maximum number of entries stored for Ctrl+R.
recent_limit = 30

[editor]
# Optional text colour. Leave commented out to use the GTK theme's normal
# foreground colour exactly.
# text_color = "#e6edf3"

# Optional font description, for example "Inter 16" or "JetBrains Mono 15".
# Leave commented out to use the font selected by the user's GTK theme.
# font = "Inter 16"

# Optional font size in points. If "font" description contains a size, this overrides it.
# font_size = 14

# Optional editor content padding in pixels.
# Leave commented out to keep the GTK/theme default spacing.
# padding = 24

# Optional line-number display. Default is false.
# line_numbers = false

# Optional Markdown highlighting. Default is true.
# markdown_highlighting = true

# Optional word wrapping. Default is true.
# wrap_text = true

[window]
# Optional opacity from 0.10 to 1.00.
# Leave commented out for a fully opaque normal GTK window.
# On Hyprland, values below 1.0 can allow compositor blur behind the window.
# window_opacity = 0.82

# Optional colour-scheme preference. Leave commented out to follow the OS.
# Valid values: "system", "light", "dark"
# color_scheme = "system"

[appearance]
# Optional application CSS file. Leave commented out to use no app CSS.
# This file is loaded after the GTK/Libadwaita theme and can override it.
# css_file = "~/.config/noti/noti.css"
"##;

impl Config {
    pub fn get_config_dir() -> Result<PathBuf> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("noti");
        Ok(xdg_dirs.get_config_home().unwrap())
    }

    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = Self::get_config_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    pub fn load_or_create() -> Result<(Self, Vec<String>)> {
        let config_path = Self::get_config_path()?;
        let mut warnings = Vec::new();

        if !config_path.exists() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&config_path, DEFAULT_CONFIG_CONTENT)?;
            return Ok((Config::default(), warnings));
        }

        let content = std::fs::read_to_string(&config_path)?;
        let parsed: toml::Value = toml::from_str(&content)?;

        // Manually parse or deserialize with warnings for invalid keys/values
        let mut config = Config::default();

        if let Some(notes) = parsed.get("notes") {
            if let Some(directory) = notes.get("directory").and_then(|v| v.as_str()) {
                config.notes.directory = directory.to_string();
            }
            if let Some(extension) = notes.get("extension").and_then(|v| v.as_str()) {
                config.notes.extension = extension.to_string();
            }
        }

        if let Some(autosave) = parsed.get("autosave") {
            if let Some(delay) = autosave.get("delay_ms").and_then(|v| v.as_integer()) {
                if delay > 0 {
                    config.autosave.delay_ms = delay as u64;
                } else {
                    warnings.push("autosave.delay_ms must be greater than 0".to_string());
                }
            }
            if let Some(int) = autosave
                .get("snapshot_interval_minutes")
                .and_then(|v| v.as_integer())
            {
                if int >= 0 {
                    config.autosave.snapshot_interval_minutes = int as u64;
                } else {
                    warnings.push("autosave.snapshot_interval_minutes must be >= 0".to_string());
                }
            }
            if let Some(keep) = autosave
                .get("keep_snapshots_days")
                .and_then(|v| v.as_integer())
            {
                if keep >= 0 {
                    config.autosave.keep_snapshots_days = keep as u64;
                } else {
                    warnings.push("autosave.keep_snapshots_days must be >= 0".to_string());
                }
            }
        }

        if let Some(behavior) = parsed.get("behavior") {
            if let Some(reopen) = behavior.get("reopen_last_note").and_then(|v| v.as_bool()) {
                config.behavior.reopen_last_note = reopen;
            }
            if let Some(create) = behavior
                .get("create_note_on_launch_if_none")
                .and_then(|v| v.as_bool())
            {
                config.behavior.create_note_on_launch_if_none = create;
            }
            if let Some(limit) = behavior.get("recent_limit").and_then(|v| v.as_integer()) {
                if limit >= 0 {
                    config.behavior.recent_limit = limit as usize;
                } else {
                    warnings.push("behavior.recent_limit must be >= 0".to_string());
                }
            }
        }

        if let Some(editor) = parsed.get("editor") {
            if let Some(color) = editor.get("text_color").and_then(|v| v.as_str()) {
                config.editor.text_color = Some(color.to_string());
            }
            if let Some(font) = editor.get("font").and_then(|v| v.as_str()) {
                config.editor.font = Some(font.to_string());
            }
            if let Some(fs) = editor.get("font_size") {
                if let Some(fs_float) = fs.as_float() {
                    config.editor.font_size = Some(fs_float as f32);
                } else if let Some(fs_int) = fs.as_integer() {
                    config.editor.font_size = Some(fs_int as f32);
                }
            }
            if let Some(pad) = editor.get("padding").and_then(|v| v.as_integer()) {
                config.editor.padding = Some(pad as i32);
            }
            if let Some(ln) = editor.get("line_numbers").and_then(|v| v.as_bool()) {
                config.editor.line_numbers = Some(ln);
            }
            if let Some(mh) = editor
                .get("markdown_highlighting")
                .and_then(|v| v.as_bool())
            {
                config.editor.markdown_highlighting = Some(mh);
            }
            if let Some(wt) = editor.get("wrap_text").and_then(|v| v.as_bool()) {
                config.editor.wrap_text = Some(wt);
            }
        }

        if let Some(window) = parsed.get("window") {
            if let Some(op) = window.get("window_opacity").and_then(|v| v.as_float()) {
                if (0.10..=1.00).contains(&op) {
                    config.window.window_opacity = Some(op);
                } else {
                    warnings
                        .push("window.window_opacity must be between 0.10 and 1.00".to_string());
                }
            } else if let Some(op) = window.get("window_opacity").and_then(|v| v.as_integer()) {
                let op_f = op as f64;
                if (0.10..=1.00).contains(&op_f) {
                    config.window.window_opacity = Some(op_f);
                } else {
                    warnings
                        .push("window.window_opacity must be between 0.10 and 1.00".to_string());
                }
            }
            if let Some(scheme) = window.get("color_scheme").and_then(|v| v.as_str()) {
                if ["system", "light", "dark"].contains(&scheme) {
                    config.window.color_scheme = Some(scheme.to_string());
                } else {
                    warnings.push(format!(
                        "window.color_scheme must be system, light, or dark. Got: {}",
                        scheme
                    ));
                }
            }
        }

        if let Some(appearance) = parsed.get("appearance") {
            if let Some(css) = appearance.get("css_file").and_then(|v| v.as_str()) {
                config.appearance.css_file = Some(css.to_string());
            }
        }

        Ok((config, warnings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_size() {
        let content = r#"
            [editor]
            font_size = 14.5
        "#;
        let parsed: toml::Value = toml::from_str(content).unwrap();
        let mut config = Config::default();
        if let Some(editor) = parsed.get("editor") {
            if let Some(fs) = editor.get("font_size") {
                if let Some(fs_float) = fs.as_float() {
                    config.editor.font_size = Some(fs_float as f32);
                } else if let Some(fs_int) = fs.as_integer() {
                    config.editor.font_size = Some(fs_int as f32);
                }
            }
        }
        assert_eq!(config.editor.font_size, Some(14.5));
    }
}
