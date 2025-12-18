use crate::commands::system;
use serde_json;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    app.package_info().version.to_string()
}

#[tauri::command]
pub async fn check_app_update(_app: AppHandle) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "isUpdateAvailable": false,
        "latestVersion": "",
        "releaseNotes": ""
    }))
}

#[tauri::command]
pub async fn download_app_update() -> Result<(), String> {
    Err("Auto-update not configured".to_string())
}

#[tauri::command]
pub async fn quit_and_install(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub async fn open_installer_directory(app: AppHandle) -> Result<bool, String> {
    let path = app.path().download_dir().unwrap_or_default();
    system::open_directory(app, Some(path.to_string_lossy().to_string())).await
}
