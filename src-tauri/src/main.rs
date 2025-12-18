// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = app_lib::run() {
        log::error!("App failed to run: {}", e);
        std::process::exit(1);
    }
}
