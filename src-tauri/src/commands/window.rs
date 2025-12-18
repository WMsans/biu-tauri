use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, Window};

#[tauri::command]
pub async fn switch_to_mini(app: AppHandle, _window: Window) -> Result<(), String> {
    if let Some(main_win) = app.get_webview_window("main") {
        main_win.hide().unwrap();
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
        .build()
        .map_err(|e| e.to_string())?;
    } else {
        if let Some(mini) = app.get_webview_window("mini") {
            mini.show().unwrap();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn switch_to_main(app: AppHandle) -> Result<(), String> {
    if let Some(mini_win) = app.get_webview_window("mini") {
        mini_win.close().unwrap();
    }
    if let Some(main_win) = app.get_webview_window("main") {
        main_win.show().unwrap();
        main_win.set_focus().unwrap();
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
pub async fn update_playback_state(app: AppHandle, is_playing: bool) -> Result<(), String> {
    app.emit("playback-state-update", is_playing)
        .map_err(|e| e.to_string())?;
    Ok(())
}
