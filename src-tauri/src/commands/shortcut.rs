use crate::commands::store;
use crate::error::AppError;
use crate::state::models::ShortcutConfig;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

#[tauri::command]
pub async fn register_shortcut(
    app: AppHandle,
    id: String,
    accelerator: String,
) -> Result<bool, AppError> {
    let _ = unregister_shortcut(app.clone(), id.clone()).await;

    let shortcut = accelerator
        .parse::<Shortcut>()
        .map_err(|e| AppError::ShortcutError(e.into()))?;

    let app_handle = app.clone();
    let id_clone = id.clone();

    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, _event| {
            let _ = app_handle.emit("shortcut:triggered", id_clone.clone());
        })
        .map_err(|e| AppError::ShortcutError(e))?;

    Ok(true)
}

#[tauri::command]
pub async fn unregister_shortcut(app: AppHandle, id: String) -> Result<(), AppError> {
    let settings = store::load_settings(&app)?;
    
    // Look up the accelerator string in settings to find what to unregister
    if let Some(shortcuts_val) = settings.extra.get("globalShortcuts") {
        if let Ok(shortcuts) = serde_json::from_value::<Vec<ShortcutConfig>>(shortcuts_val.clone()) {
            if let Some(config) = shortcuts.iter().find(|s| s.id == id) {
                if let Ok(shortcut) = config.shortcut.parse::<Shortcut>() {
                    let _ = app.global_shortcut().unregister(shortcut);
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn register_all_shortcuts(app: AppHandle) -> Result<(), AppError> {
    let settings = store::load_settings(&app)?;
    
    if let Some(shortcuts_val) = settings.extra.get("globalShortcuts") {
        if let Ok(shortcuts) = serde_json::from_value::<Vec<ShortcutConfig>>(shortcuts_val.clone()) {
            for config in shortcuts {
                // We attempt to register each; individual failures don't stop the loop
                let _ = register_shortcut(app.clone(), config.id, config.shortcut).await;
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn unregister_all_shortcuts(app: AppHandle) -> Result<(), AppError> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| AppError::ShortcutError(e))
}