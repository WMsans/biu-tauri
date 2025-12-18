use crate::state::models::AppSettings;
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
pub fn load_settings(app: &AppHandle) -> AppSettings {
    let path = get_settings_path(app);
    if path.exists() {
        let file = File::open(path).ok();
        if let Some(f) = file {
            let reader = std::io::BufReader::new(f);
            if let Ok(settings) = serde_json::from_reader(reader) {
                return settings;
            }
        }
    }
    // Defaults
    AppSettings {
        download_path: Some(
            app.path()
                .download_dir()
                .unwrap()
                .to_string_lossy()
                .to_string(),
        ),
        extra: std::collections::HashMap::new(),
    }
}
pub fn get_store_path(app: &AppHandle, key: &str) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap()
        .join(format!("{}.json", key))
}
#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    Ok(load_settings(&app))
}
#[tauri::command]
pub async fn set_settings(app: AppHandle, payload: AppSettings) -> Result<bool, String> {
    let path = get_settings_path(&app);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = File::create(path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file, &payload).map_err(|e| e.to_string())?;
    Ok(true)
}
#[tauri::command]
pub async fn get_store(app: AppHandle, key: String) -> Result<serde_json::Value, String> {
    let path = get_store_path(&app, &key);
    if path.exists() {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = std::io::BufReader::new(file);
        let data: serde_json::Value = serde_json::from_reader(reader).map_err(|e| e.to_string())?;
        Ok(data)
    } else {
        Ok(serde_json::Value::Null)
    }
}
#[tauri::command]
pub async fn set_store(app: AppHandle, key: String, data: serde_json::Value) -> Result<(), String> {
    let path = get_store_path(&app, &key);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = File::create(path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file, &data).map_err(|e| e.to_string())?;
    Ok(())
}
#[tauri::command]
pub async fn clear_store(app: AppHandle, key: String) -> Result<(), String> {
    let path = get_store_path(&app, &key);
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
#[tauri::command]
pub async fn clear_settings(app: AppHandle) -> Result<bool, String> {
    let path = get_settings_path(&app);
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(true)
}
