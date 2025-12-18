use tauri::ipc::Invoke;

pub mod app;
pub mod download;
pub mod network;
pub mod store;
pub mod system;
pub mod window;

pub fn get_handlers() -> impl Fn(Invoke) -> bool {
    tauri::generate_handler![
        app::get_app_version,
        app::check_app_update,
        app::download_app_update,
        app::quit_and_install,
        app::open_installer_directory,
        download::check_file_exists,
        download::start_download,
        download::get_media_download_task_list,
        download::add_media_download_task,
        download::pause_media_download_task,
        download::resume_media_download_task,
        download::retry_media_download_task,
        download::cancel_media_download_task,
        download::clear_media_download_task_list,
        network::http_request,
        network::get_cookie,
        network::http_get,
        network::http_post,
        network::get_proxy_port,
        network::wbi_sign_params, // Registered
        store::get_settings,
        store::set_settings,
        store::get_store,
        store::set_store,
        store::clear_store,
        store::clear_settings,
        system::select_directory,
        system::get_fonts,
        system::open_directory,
        system::show_file_in_folder,
        window::switch_to_mini,
        window::switch_to_main,
        window::minimize_window,
        window::toggle_maximize_window,
        window::close_window,
        window::is_maximized,
        window::is_full_screen,
        window::update_playback_state,
    ]
}