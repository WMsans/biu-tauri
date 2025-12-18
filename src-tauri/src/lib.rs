use anyhow::Result;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager,
    Emitter
};

pub mod commands;
pub mod error;
pub mod services;
pub mod state;

use crate::services::{http, proxy::run_proxy_server};
use error::AppError;
use state::models::{ProxyPort, TaskStore, WbiKeysCache, WbiStore};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), AppError> {
    let proxy_port = Arc::new(Mutex::new(0u16));
    let proxy_port_clone = proxy_port.clone();

    // 1. Initialize Task Store
    let task_store = TaskStore::new();

    // 2. Initialize WBI Store
    let wbi_store = WbiStore(Arc::new(Mutex::new(WbiKeysCache::new())));

    // Start Proxy Server
    tauri::async_runtime::spawn(async move {
        if let Err(e) = run_proxy_server(proxy_port_clone).await {
            log::error!("Proxy server error: {}", e);
        }
    });

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .setup(|app| {
            // --- System Tray Implementation ---

            // 1. Create Menu Items
            let play_pause_i = MenuItem::with_id(app, "play_pause", "Play/Pause", true, None::<&str>)?;
            let prev_i = MenuItem::with_id(app, "prev", "Previous", true, None::<&str>)?;
            let next_i = MenuItem::with_id(app, "next", "Next", true, None::<&str>)?;
            let show_hide_i = MenuItem::with_id(app, "show_hide", "Show/Hide", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            // 2. Build the Menu
            let menu = Menu::with_items(
                app,
                &[
                    &play_pause_i,
                    &prev_i,
                    &next_i,
                    &show_hide_i, // Separators can be added here if needed using PredefinedMenuItem::separator(app)?
                    &quit_i,
                ],
            )?;

            // 3. Configure the Tray
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone()) // Uses the default app icon
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "play_pause" => {
                            let _ = app.emit("player:toggle", ()); // Emits to frontend
                        }
                        "prev" => {
                            let _ = app.emit("player:prev", ());
                        }
                        "next" => {
                            let _ = app.emit("player:next", ());
                        }
                        "show_hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                if window.is_visible().unwrap_or(false) {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .manage(http::build_client()) // Updated: build_client() now returns AppHttpClient
        .manage(ProxyPort(proxy_port))
        .manage(task_store)
        .manage(wbi_store) // Register WBI Store
        .invoke_handler(|invoke| {
            commands::get_handlers()(invoke);
            true
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}