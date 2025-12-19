use anyhow::Result;
use std::sync::{Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Manager,
    Emitter,
    RunEvent
};

pub mod commands;
pub mod error;
pub mod services;
pub mod state;

use crate::services::{http, proxy::run_proxy_server};
use error::AppError;
use state::models::{AppHttpClient, AppCookieStore, ProxyPort, TaskStore, WbiKeysCache, WbiStore};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), AppError> {
    let proxy_port = Arc::new(Mutex::new(0u16));
    let proxy_port_clone = proxy_port.clone();

    // 1. Initialize Task Store
    let task_store = TaskStore::new();

    // 2. Initialize WBI Store
    let wbi_store = WbiStore(Arc::new(Mutex::new(WbiKeysCache::new())));

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .manage(ProxyPort(proxy_port))
        .manage(task_store)
        .manage(wbi_store)
        .setup(move |app| {
            // --- Persistence Setup ---
            
            // 1. Initialize HTTP Client & Load Cookies
            let (client, cookie_store) = http::build_client(app.handle())?;

            // 2. Load Tasks
            if let Ok(saved_tasks) = commands::store::load_tasks(app.handle()) {
                let store = app.state::<TaskStore>();
                let mut tasks = store.tasks.lock().unwrap();
                *tasks = saved_tasks;
            }
            
            // --- Start Background Tasks ---
            
            // A. Auto-save Cookies Task (Lazy Timer)
            let cookie_store_for_save = cookie_store.clone();
            let app_handle_for_save = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    if let Err(e) = http::save_cookies(&app_handle_for_save, &cookie_store_for_save) {
                        log::error!("Background cookie save failed: {}", e);
                    }
                }
            });

            // B. Start Proxy Server
            let proxy_port_for_server = proxy_port_clone.clone(); 
            let cookie_store_for_server = cookie_store.clone();
            
            tauri::async_runtime::spawn(async move {
                if let Err(e) = run_proxy_server(proxy_port_for_server, cookie_store_for_server).await {
                    log::error!("Proxy server error: {}", e);
                }
            });

            // 3. Manage States
            app.manage(AppCookieStore(cookie_store));
            app.manage(AppHttpClient(client));

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
                    &show_hide_i, 
                    &quit_i,
                ],
            )?;

            // 3. Configure the Tray
            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| {
                    match event.id().as_ref() {
                        "play_pause" => {
                            let _ = app.emit("shortcut:triggered", "togglePlay"); 
                        }
                        "prev" => {
                            let _ = app.emit("shortcut:triggered", "prev");
                        }
                        "next" => {
                            let _ = app.emit("shortcut:triggered", "next");
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
        .invoke_handler(|invoke| {
            commands::get_handlers()(invoke);
            true
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let RunEvent::ExitRequested { .. } = event {
            let cookie_store_state = app_handle.state::<AppCookieStore>();
            if let Err(e) = http::save_cookies(app_handle, &cookie_store_state.0) {
                log::error!("Failed to save cookies on exit: {}", e);
            }
        }
    });

    Ok(())
}