use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::errors::{AppError, Result};

pub fn generate_slug(content: &str) -> String {
    let first_line = content.lines()
        .map(|line| line.trim())
        .find(|line| !line.is_empty());
    
    let line = match first_line {
        Some(l) => l,
        None => return "untitled".to_string(),
    };

    let cleaned = line
        .replace("#", "")
        .replace("*", "")
        .replace("_", "")
        .replace("`", "")
        .replace("[", "")
        .replace("]", "")
        .replace("(", "")
        .replace(")", "")
        .replace("-", " ")
        .replace("+", " ");

    let words: Vec<String> = cleaned
        .split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|w| !w.is_empty())
        .collect();

    if words.is_empty() {
        return "untitled".to_string();
    }

    let three_words = words.iter().take(3).cloned().collect::<Vec<String>>();
    three_words.join("-")
}

pub fn generate_unique_path(directory: &Path, content: &str, extension: &str) -> PathBuf {
    let slug = generate_slug(content);
    let now = chrono::Local::now();
    let timestamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    
    let extension = extension.trim_start_matches('.');
    
    let filename = format!("{}-{}.{}", timestamp, slug, extension);
    let mut path = directory.join(&filename);
    let mut counter = 2;
    
    while path.exists() {
        let name_without_ext = format!("{}-{}-{}", timestamp, slug, counter);
        path = directory.join(format!("{}.{}", name_without_ext, extension));
        counter += 1;
    }
    
    path
}

pub fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        AppError::Generic("Path has no parent directory".to_string())
    })?;
    fs::create_dir_all(parent)?;

    let file_name = path.file_name().ok_or_else(|| {
        AppError::Generic("Path has no file name".to_string())
    })?;

    let mut temp_name = std::ffi::OsString::new();
    temp_name.push(".");
    temp_name.push(file_name);
    temp_name.push(".tmp");
    let temp_path = parent.join(temp_name);

    // Write and sync to temp file
    {
        let mut file = File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
    }

    // Rename temp file over target file atomically
    fs::rename(&temp_path, path)?;
    Ok(())
}
