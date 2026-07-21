use std::path::PathBuf;
use crate::state::{PersistedState, RecentEntry};

pub fn add_recent(state: &mut PersistedState, path: PathBuf, limit: usize) {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.clone());
    
    // Remove duplicates. We match on canonicalized paths if possible.
    state.recent.retain(|entry| {
        let entry_canonical = entry.path.canonicalize().unwrap_or_else(|_| entry.path.clone());
        entry_canonical != canonical
    });

    let opened_at = chrono::DateTime::from(chrono::Local::now());

    state.recent.insert(0, RecentEntry {
        path,
        opened_at,
    });

    if state.recent.len() > limit {
        state.recent.truncate(limit);
    }
}

pub fn clean_recents(state: &mut PersistedState) {
    state.recent.retain(|entry| {
        entry.path.exists() && entry.path.is_file()
    });
}
