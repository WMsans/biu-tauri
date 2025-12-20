use crate::error::AppError;
use crate::state::models::{AppSettings, MediaDownloadTaskState};
use serde_json;
use std::fs::{self, File};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn get_settings_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap()
        .join("app-settings.json")
}

pub fn get_tasks_path(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap()
        .join("tasks.json")
}

// Helper to get the system default download directory safely
fn get_default_download_dir(app: &AppHandle) -> String {
    app.path()
        .download_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string())
}

pub fn load_settings(app: &AppHandle) -> Result<AppSettings, AppError> {
    let path = get_settings_path(app);
    if path.exists() {
        let file = File::open(path)?;
        let reader = std::io::BufReader::new(file);

        // Read as raw Value first to handle structural mismatch (frontend wraps in "appSettings")
        let val: serde_json::Value = serde_json::from_reader(reader)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Extract the actual settings object if wrapped, otherwise use root
        let settings_val = val.get("appSettings").unwrap_or(&val);

        let mut settings: AppSettings = serde_json::from_value(settings_val.clone())
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // FIX: If download_path is None or empty, fallback to system default
        if settings.download_path.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
            settings.download_path = Some(get_default_download_dir(app));
        }

        return Ok(settings);
    }

    // Defaults if file doesn't exist
    let mut settings = AppSettings::default();
    settings.download_path = Some(get_default_download_dir(app));
    Ok(settings)
}

pub fn load_tasks(app: &AppHandle) -> Result<Vec<MediaDownloadTaskState>, AppError> {
    let path = get_tasks_path(app);
    if path.exists() {
        let file = File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let mut tasks: Vec<MediaDownloadTaskState> = serde_json::from_reader(reader)
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Reset active statuses to paused on load (since the app was restarted)
        for task in &mut tasks {
            if task.status == "downloading" || task.status == "merging" || task.status == "converting" {
                task.status = "paused".to_string();
            }
        }
        Ok(tasks)
    } else {
        Ok(Vec::new())
    }
}

pub fn save_tasks(app: &AppHandle, tasks: &[MediaDownloadTaskState]) -> Result<(), AppError> {
    let path = get_tasks_path(app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, tasks)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub fn get_store_path(app: &AppHandle, key: &str) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap()
        .join(format!("{}.json", key))
}
#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<AppSettings, AppError> {
    load_settings(&app)
}
#[tauri::command]
pub async fn set_settings(app: AppHandle, payload: AppSettings) -> Result<bool, AppError> {
    let path = get_settings_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &payload)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(true)
}
#[tauri::command]
pub async fn get_store(app: AppHandle, key: String) -> Result<serde_json::Value, AppError> {
    let path = get_store_path(&app, &key);
    if path.exists() {
        let file = File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let data: serde_json::Value =
            serde_json::from_reader(reader).map_err(|e| AppError::DatabaseError(e.to_string()))?;
        Ok(data)
    } else {
        Ok(serde_json::Value::Null)
    }
}
#[tauri::command]
pub async fn set_store(
    app: AppHandle,
    key: String,
    data: serde_json::Value,
) -> Result<(), AppError> {
    let path = get_store_path(&app, &key);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &data)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}
#[tauri::command]
pub async fn clear_store(app: AppHandle, key: String) -> Result<(), AppError> {
    let path = get_store_path(&app, &key);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}
#[tauri::command]
pub async fn clear_settings(app: AppHandle) -> Result<bool, AppError> {
    let path = get_settings_path(&app);
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(true)
}