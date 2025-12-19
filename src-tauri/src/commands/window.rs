use crate::error::AppError;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Window};

#[tauri::command]
pub async fn switch_to_mini(app: AppHandle, _window: Window) -> Result<(), AppError> {
    if let Some(main_win) = app.get_webview_window("main") {
        main_win.hide()?;
    }

    if app.get_webview_window("mini").is_none() {
        let _mini = WebviewWindowBuilder::new(
            &app,
            "mini",
            WebviewUrl::App("index.html#/mini-player".into()),
        )
        .title("Mini Player")
        .inner_size(300.0, 300.0)
        .always_on_top(true)
        .decorations(false)
        .transparent(true)
        .build()?;
    } else {
        if let Some(mini) = app.get_webview_window("mini") {
            mini.show()?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn switch_to_main(app: AppHandle) -> Result<(), AppError> {
    if let Some(mini_win) = app.get_webview_window("mini") {
        mini_win.close()?;
    }
    if let Some(main_win) = app.get_webview_window("main") {
        main_win.show()?;
        main_win.set_focus()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn toggle_mini_player(app: AppHandle) -> Result<(), AppError> {
    if let Some(mini_win) = app.get_webview_window("mini") {
        mini_win.close()?;
        if let Some(main_win) = app.get_webview_window("main") {
            main_win.show()?;
            main_win.set_focus()?;
        }
    } else {
        if let Some(main_win) = app.get_webview_window("main") {
            main_win.hide()?;
        }
        WebviewWindowBuilder::new(
            &app,
            "mini",
            WebviewUrl::App("index.html#/mini-player".into()),
        )
        .title("Mini Player")
        .inner_size(300.0, 300.0)
        .always_on_top(true)
        .decorations(false)
        .transparent(true)
        .build()?;
    }
    Ok(())
}

#[tauri::command]
pub fn minimize_window(window: Window) {
    let _ = window.minimize();
}

#[tauri::command]
pub fn toggle_maximize_window(window: Window) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
        let _ = window.emit("window:unmaximize", ());
    } else {
        let _ = window.maximize();
        let _ = window.emit("window:maximize", ());
    }
}

#[tauri::command]
pub fn close_window(window: Window) {
    let _ = window.close();
}

#[tauri::command]
pub fn is_maximized(window: Window) -> bool {
    window.is_maximized().unwrap_or(false)
}

#[tauri::command]
pub fn is_full_screen(window: Window) -> bool {
    window.is_fullscreen().unwrap_or(false)
}

#[tauri::command]
pub async fn update_playback_state(app: AppHandle, is_playing: bool) -> Result<(), AppError> {
    app.emit("playback-state-update", is_playing)?;
    Ok(())
}
