use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State};
use crate::state::models::*;
use crate::commands::store;
use crate::services::downloader::{fetch_bili_url, spawn_download_task};

#[tauri::command]
pub async fn check_file_exists(app: AppHandle, filename: String) -> Result<bool, String> {
    let settings = store::load_settings(&app);
    let download_dir = PathBuf::from(settings.download_path.unwrap_or_default());
    Ok(download_dir.join(filename).exists())
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    client: State<'_, AppHttpClient>,
    params: DownloadOptions,
) -> Result<serde_json::Value, String> {
    // Legacy simple download without state tracking
    spawn_download_task(app, client.0.clone(), params);
    Ok(serde_json::json!({ "success": true }))
}

// --- NEW: Command to get the list ---
#[tauri::command]
pub async fn get_media_download_task_list(
    state: State<'_, TaskStore>,
) -> Result<Vec<MediaDownloadTaskState>, String> {
    let tasks = state.tasks.lock().unwrap();
    Ok(tasks.clone())
}

#[tauri::command]
pub async fn add_media_download_task(
    app: AppHandle,
    client: State<'_, AppHttpClient>,
    store: State<'_, TaskStore>,
    task: MediaDownloadRequest,
) -> Result<serde_json::Value, String> {
    let bvid = task.bvid.clone().ok_or("Missing bvid")?;
    let cid = task.cid.clone().ok_or("Missing cid")?;

    // Use Helper
    let audio_url = fetch_bili_url(&client.0, &bvid, &cid).await?;

    let ext = if task.output_file_type == "mp3" { "mp3" } else { "m4a" };
    let safe_title: String = task.title.chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_').collect();
    let filename = format!("{}.{}", safe_title, ext);
    let task_id = uuid::Uuid::new_v4().to_string();

    let new_task_state = MediaDownloadTaskState {
        id: task_id.clone(),
        output_file_type: task.output_file_type.clone(),
        title: task.title.clone(),
        cover: task.cover.clone(),
        bvid: task.bvid.clone(),
        cid: task.cid.clone(),
        sid: task.sid.clone(),
        audio_codecs: None, audio_bandwidth: None, video_resolution: None, video_frame_rate: None,
        save_path: Some(filename.clone()),
        total_bytes: None, download_progress: Some(0), merge_progress: None, convert_progress: None,
        error: None,
        status: "pending".to_string(),
    };

    {
        let mut tasks = store.tasks.lock().unwrap();
        tasks.push(new_task_state.clone());
        app.emit("download:list-sync", serde_json::json!({ "type": "full", "data": *tasks })).map_err(|e| e.to_string())?;
    }

    let options = DownloadOptions {
        id: task_id.clone(),
        filename,
        audio_url,
        is_lossless: task.output_file_type != "mp3",
    };

    // Store handle
    let handle = spawn_download_task(app, client.0.clone(), options);
    store.handles.lock().unwrap().insert(task_id, handle);

    Ok(serde_json::json!({ "success": true, "message": "Download started" }))
}

#[tauri::command]
pub async fn pause_media_download_task(
    app: AppHandle,
    store: State<'_, TaskStore>,
    id: String,
) -> Result<(), String> {
    // 1. Abort execution
    {
        let mut handles = store.handles.lock().unwrap();
        if let Some(handle) = handles.remove(&id) {
            handle.abort();
        }
    }

    // 2. Update status manually (as abort stops the thread before it can update)
    let mut tasks = store.tasks.lock().unwrap();
    if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
        if task.status != "completed" {
            task.status = "paused".to_string();
            // Emit sync
            let updated = task.clone();
            drop(tasks); // release lock
            let _ = app.emit("download:list-sync", serde_json::json!({ "type": "update", "data": [updated] }));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn resume_media_download_task(
    app: AppHandle,
    client: State<'_, AppHttpClient>,
    store: State<'_, TaskStore>,
    id: String,
) -> Result<(), String> {
    let task_opt = {
        let tasks = store.tasks.lock().unwrap();
        tasks.iter().find(|t| t.id == id).cloned()
    };

    if let Some(task) = task_opt {
        // Re-fetch URL because it might have expired
        let bvid = task.bvid.ok_or("No BVID")?;
        let cid = task.cid.ok_or("No CID")?;
        
        let audio_url = fetch_bili_url(&client.0, &bvid, &cid).await?;
        
        let filename = task.save_path.ok_or("No save path")?;
        
        let options = DownloadOptions {
            id: id.clone(),
            filename,
            audio_url,
            is_lossless: task.output_file_type != "mp3",
        };

        let handle = spawn_download_task(app, client.0.clone(), options);
        store.handles.lock().unwrap().insert(id, handle);
    }
    Ok(())
}

#[tauri::command]
pub async fn retry_media_download_task(
    app: AppHandle,
    client: State<'_, AppHttpClient>,
    store: State<'_, TaskStore>,
    id: String,
) -> Result<(), String> {
    // Retry is effectively the same as resume in this architecture 
    // (range headers will handle partials, or start from 0 if missing)
    resume_media_download_task(app, client, store, id).await
}

#[tauri::command]
pub async fn cancel_media_download_task(
    app: AppHandle,
    store: State<'_, TaskStore>,
    id: String,
) -> Result<(), String> {
    // 1. Abort
    {
        let mut handles = store.handles.lock().unwrap();
        if let Some(handle) = handles.remove(&id) {
            handle.abort();
        }
    }

    // 2. Remove from List and Delete Files
    let mut tasks = store.tasks.lock().unwrap();
    if let Some(index) = tasks.iter().position(|t| t.id == id) {
        tasks.remove(index);
        
        // Delete Temp Files
        let temp_dir = app.path().temp_dir().unwrap().join("biu-downloads");
        let temp_audio = temp_dir.join(format!("{}.audio.tmp", id));
        let _ = fs::remove_file(temp_audio);

        // Emit Full Sync
        let _ = app.emit("download:list-sync", serde_json::json!({
            "type": "full",
            "data": *tasks
        }));
    }
    Ok(())
}

#[tauri::command]
pub async fn clear_media_download_task_list(
    app: AppHandle,
    store: State<'_, TaskStore>,
) -> Result<(), String> {
    let mut tasks = store.tasks.lock().unwrap();
    tasks.clear();

    // Emit full sync event with empty list
    app.emit(
        "download:list-sync",
        serde_json::json!({
            "type": "full",
            "data": Vec::<MediaDownloadTaskState>::new()
        }),
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
