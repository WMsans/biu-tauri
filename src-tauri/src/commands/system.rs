use crate::commands::store;
use font_kit::source::SystemSource;
use serde_json;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

#[tauri::command]
pub async fn show_file_in_folder(app: AppHandle, path: String) -> Result<(), String> {
    let settings = store::load_settings(&app);
    let download_dir = PathBuf::from(settings.download_path.unwrap_or_default());
    let full_path = download_dir.join(&path);

    let path_str = full_path.to_string_lossy().to_string();

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", &path_str]) // Comma is important
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-R", &path_str])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux doesn't have a standard "highlight file" command. Open parent dir.
        if let Some(parent) = full_path.parent() {
            std::process::Command::new("xdg-open")
                .arg(parent)
                .spawn()
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn select_directory(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file_path = app.dialog().file().blocking_pick_folder();
    match file_path {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_fonts() -> Result<Vec<serde_json::Value>, String> {
    let source = SystemSource::new();
    let fonts = source.all_fonts().map_err(|e| e.to_string())?;
    let font_infos: Vec<serde_json::Value> = fonts
        .into_iter()
        .filter_map(|handle| {
            handle.load().ok().map(|f| {
                serde_json::json!({
                    "name": f.full_name(),
                    "familyName": f.family_name()
                })
            })
        })
        .collect::<Vec<_>>();
    Ok(font_infos)
}

#[tauri::command]
pub async fn open_directory(app: AppHandle, path: Option<String>) -> Result<bool, String> {
    let target_dir = if let Some(d) = path {
        d
    } else {
        store::load_settings(&app).download_path.unwrap_or_default()
    };
    println!("Path: {}", &target_dir);
    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("explorer")
        .arg(&target_dir)
        .spawn();
    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open").arg(&target_dir).spawn();
    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(&target_dir)
        .spawn();

    match result {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
