use crate::errors::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorState {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEntry {
    pub path: PathBuf,
    pub opened_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedState {
    pub last_opened: Option<PathBuf>,
    pub cursor: Option<CursorState>,
    pub recent: Vec<RecentEntry>,
    #[serde(default = "default_window_width")]
    pub window_width: i32,
    #[serde(default = "default_window_height")]
    pub window_height: i32,
    #[serde(default)]
    pub maximized: bool,
    pub font_size: Option<f32>,
}

fn default_window_width() -> i32 {
    800
}
fn default_window_height() -> i32 {
    600
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            last_opened: None,
            cursor: None,
            recent: Vec::new(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            maximized: false,
            font_size: None,
        }
    }
}

impl PersistedState {
    pub fn get_state_dir() -> Result<PathBuf> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("noti");
        Ok(xdg_dirs.get_data_home().unwrap())
    }

    pub fn get_state_path() -> Result<PathBuf> {
        let state_dir = Self::get_state_dir()?;
        Ok(state_dir.join("state.toml"))
    }

    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(state) => state,
            Err(_) => {
                // If it's corrupted or missing, return default.
                PersistedState::default()
            }
        }
    }

    fn load() -> Result<Self> {
        let state_path = Self::get_state_path()?;
        if !state_path.exists() {
            return Ok(PersistedState::default());
        }
        let content = std::fs::read_to_string(&state_path)?;
        let state: PersistedState = toml::from_str(&content)?;
        Ok(state)
    }

    pub fn save(&self) -> Result<()> {
        let state_path = Self::get_state_path()?;
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&state_path, content)?;
        Ok(())
    }
}
