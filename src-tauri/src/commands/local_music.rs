use crate::error::AppError;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

const AUDIO_EXTENSIONS: [&str; 8] = ["mp3", "flac", "wav", "m4a", "aac", "ogg", "wma", "aiff"];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocalMusicItem {
    pub id: String,
    pub path: String,
    pub dir: String,
    pub title: String,
    pub size: u64,
    pub format: String,
    pub duration: Option<f64>,
    pub created_time: Option<f64>,
}

fn get_file_id(path: &str) -> String {
    let path_lower = path.to_lowercase();
    format!("{:x}", md5::compute(path_lower.as_bytes()))
}

fn to_safe_title(file_path: &str, meta_title: Option<&str>) -> String {
    if let Some(title) = meta_title {
        let trimmed = title.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    Path::new(file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string()
}

fn walk_directory(dir: &str, files: &mut Vec<String>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()) {
                        if let Some(path_str) = path.to_str() {
                            files.push(path_str.to_string());
                        }
                    }
                }
            }
        }
    }
}

#[tauri::command]
pub async fn scan_local_music(dirs: Vec<String>) -> Result<Vec<LocalMusicItem>, AppError> {
    let mut result: Vec<LocalMusicItem> = Vec::new();

    if dirs.is_empty() {
        return Ok(result);
    }

    for dir in dirs {
        let mut files: Vec<String> = Vec::new();
        walk_directory(&dir, &mut files);

        for file in files {
            let path = Path::new(&file);

            let metadata = match fs::metadata(&file) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size = metadata.len();
            let created_time = metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as f64)
                .or_else(|| {
                    metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_millis() as f64)
                });

            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let (title, duration) = match Probe::open(&file).ok().and_then(|p| p.read().ok()) {
                Some(tagged_file) => {
                    let audio_title = tagged_file
                        .primary_tag()
                        .and_then(|t| t.title())
                        .map(|s| s.to_string());
                    let dur = tagged_file.properties().duration().as_secs_f64();
                    (to_safe_title(&file, audio_title.as_deref()), Some(dur))
                }
                None => (to_safe_title(&file, None), None),
            };

            let id = get_file_id(&file);

            result.push(LocalMusicItem {
                id,
                path: file.clone(),
                dir: dir.clone(),
                title,
                size,
                format,
                duration,
                created_time,
            });
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn delete_local_music_file(path: String) -> Result<bool, AppError> {
    if path.is_empty() {
        return Ok(false);
    }

    let file_path = Path::new(&path);

    if !file_path.exists() || !file_path.is_file() {
        return Ok(false);
    }

    match fs::remove_file(file_path) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
