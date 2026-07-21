use crate::errors::{AppError, Result};
use crate::state::PersistedState;
use chrono::TimeZone;
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_snapshots_for_note(note_path: &Path) -> Result<Vec<PathBuf>> {
    let filename = match note_path.file_name() {
        Some(f) => f.to_string_lossy(),
        None => return Ok(Vec::new()),
    };

    let history_dir = PersistedState::get_state_dir()?.join("history");
    if !history_dir.exists() {
        return Ok(Vec::new());
    }

    let mut snapshots = Vec::new();

    fn visit_dirs(dir: &Path, filename: &str, snapshots: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, filename, snapshots)?;
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.len() > 20 && &name[20..] == filename {
                        snapshots.push(path);
                    }
                }
            }
        }
        Ok(())
    }

    let _ = visit_dirs(&history_dir, &filename, &mut snapshots);
    snapshots.sort();
    Ok(snapshots)
}

pub fn prune_old_snapshots(keep_days: u64) -> Result<()> {
    let history_dir = PersistedState::get_state_dir()?.join("history");
    if !history_dir.exists() {
        return Ok(());
    }

    let cutoff = chrono::Local::now() - chrono::Duration::days(keep_days as i64);
    let cutoff_date = cutoff.date_naive();

    fn prune_dir(dir: &Path, cutoff_date: chrono::NaiveDate) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    prune_dir(&path, cutoff_date)?;
                    if std::fs::read_dir(&path)?.next().is_none() {
                        let _ = std::fs::remove_dir(&path);
                    }
                } else if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.len() >= 10 {
                        if let Ok(date) =
                            chrono::NaiveDate::parse_from_str(&name[0..10], "%Y-%m-%d")
                        {
                            if date < cutoff_date {
                                let _ = std::fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    let _ = prune_dir(&history_dir, cutoff_date);
    Ok(())
}

pub fn create_snapshot_if_needed(
    note_path: &Path,
    content: &str,
    interval_mins: u64,
    force: bool,
) -> Result<Option<PathBuf>> {
    let snapshots = get_snapshots_for_note(note_path)?;

    if let Some(latest_path) = snapshots.last() {
        if let Ok(latest_content) = std::fs::read_to_string(latest_path) {
            if latest_content == content {
                return Ok(None);
            }
        }
    }

    if !force {
        if let Some(latest_path) = snapshots.last() {
            if let Some(name) = latest_path.file_name().and_then(|n| n.to_str()) {
                if name.len() >= 19 {
                    if let Ok(last_time) =
                        chrono::NaiveDateTime::parse_from_str(&name[0..19], "%Y-%m-%d_%H-%M-%S")
                    {
                        if let Some(last_dt) =
                            chrono::Local.from_local_datetime(&last_time).single()
                        {
                            let now = chrono::Local::now();
                            let duration = now.signed_duration_since(last_dt);
                            if duration.num_minutes() < interval_mins as i64 {
                                return Ok(None);
                            }
                        }
                    }
                }
            }
        }
    }

    let now = chrono::Local::now();
    let year_str = now.format("%Y").to_string();
    let date_str = now.format("%Y-%m-%d").to_string();
    let time_str = now.format("%Y-%m-%d_%H-%M-%S").to_string();

    let history_dir = PersistedState::get_state_dir()?
        .join("history")
        .join(&year_str)
        .join(&date_str);

    fs::create_dir_all(&history_dir)?;

    let original_filename = note_path
        .file_name()
        .ok_or_else(|| AppError::Generic("No filename".to_string()))?
        .to_string_lossy();

    let snapshot_filename = format!("{}-{}", time_str, original_filename);
    let snapshot_path = history_dir.join(&snapshot_filename);

    fs::write(&snapshot_path, content)?;
    Ok(Some(snapshot_path))
}
