use anyhow::Result;
use std::sync::{Arc, Mutex};

pub mod commands;
pub mod error;
pub mod services;
pub mod state;

use crate::services::{http, proxy::run_proxy_server};
use error::AppError;
use state::models::{AppHttpClient, ProxyPort, TaskStore, WbiStore, WbiKeysCache};

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
        .manage(AppHttpClient(http::build_client()))
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