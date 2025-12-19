use crate::commands::store;
use crate::error::AppError;
use crate::services::downloader::{fetch_bili_url, spawn_download_task};
use crate::state::models::*;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager, State};

#[tauri::command]
pub async fn check_file_exists(app: AppHandle, filename: String) -> Result<bool, AppError> {
    let settings = store::load_settings(&app)?;
    let download_dir = PathBuf::from(settings.download_path.unwrap_or_default());
    Ok(download_dir.join(filename).exists())
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    client: State<'_, AppHttpClient>,
    wbi_store: State<'_, WbiStore>, 
    params: DownloadOptions,
) -> Result<serde_json::Value, AppError> {
    spawn_download_task(app, client.0.clone(), WbiStore(wbi_store.0.clone()), params);
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
pub async fn get_media_download_task_list(
    state: State<'_, TaskStore>,
) -> Result<Vec<MediaDownloadTaskState>, AppError> {
    let tasks = state.tasks.lock().unwrap();
    Ok(tasks.clone())
}

#[tauri::command]
pub async fn add_media_download_task(
    app: AppHandle,
    client: State<'_, AppHttpClient>,
    store: State<'_, TaskStore>,
    wbi_store: State<'_, WbiStore>, 
    task: MediaDownloadRequest,
) -> Result<serde_json::Value, AppError> {
    let bvid = task.bvid.clone().ok_or(AppError::DatabaseError("Missing bvid".to_string()))?;
    let cid = task.cid.clone().ok_or(AppError::DatabaseError("Missing cid".to_string()))?;

    // Updated: fetch both audio and video urls
    let (audio_url, video_url) = fetch_bili_url(&client.0, &wbi_store, &bvid, &cid).await?;

    let is_video_mode = task.output_file_type == "video";

    let ext = if is_video_mode { 
        "mp4" 
    } else if task.output_file_type == "mp3" { 
        "mp3" 
    } else { 
        "m4a" 
    };

    let safe_title: String = task
        .title
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect();
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
        audio_codecs: None,
        audio_bandwidth: None,
        video_resolution: None,
        video_frame_rate: None,
        save_path: Some(filename.clone()),
        total_bytes: None,
        download_progress: Some(0),
        merge_progress: None,
        convert_progress: None,
        error: None,
        status: "pending".to_string(),
    };

    {
        let mut tasks = store.tasks.lock().unwrap();
        tasks.push(new_task_state.clone());
        // Save to disk
        let _ = store::save_tasks(&app, &tasks);

        app.emit(
            "download:list-sync",
            serde_json::json!({ "type": "full", "data": *tasks }),
        )?;
    }

    let options = DownloadOptions {
        id: task_id.clone(),
        filename,
        audio_url,
        // Only provide video_url if we are in video mode
        video_url: if is_video_mode { video_url } else { None },
        // "video" mode is also considered lossless in terms of audio transcoding (we just copy/merge)
        is_lossless: task.output_file_type != "mp3",
    };

    let handle = spawn_download_task(app, client.0.clone(), WbiStore(wbi_store.0.clone()), options);
    store.handles.lock().unwrap().insert(task_id, handle);

    Ok(serde_json::json!({ "success": true, "message": "Download started" }))
}

#[tauri::command]
pub async fn pause_media_download_task(
    app: AppHandle,
    store: State<'_, TaskStore>,
    id: String,
) -> Result<(), AppError> {
    {
        let mut handles = store.handles.lock().unwrap();
        if let Some(handle) = handles.remove(&id) {
            handle.abort();
        }
    }

    let mut updated_task: Option<MediaDownloadTaskState> = None;
    
    {
        let mut tasks = store.tasks.lock().unwrap();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == id) {
            if task.status != "completed" {
                task.status = "paused".to_string();
                updated_task = Some(task.clone());
            }
        }
        
        // Save while lock is still held, but `task` borrow is ended because we exited the if-let scope
        if updated_task.is_some() {
            let _ = store::save_tasks(&app, &tasks);
        }
    } // drop tasks lock

    if let Some(updated) = updated_task {
        let _ = app.emit(
            "download:list-sync",
            serde_json::json!({ "type": "update", "data": [updated] }),
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn resume_media_download_task(
    app: AppHandle,
    client: State<'_, AppHttpClient>,
    store: State<'_, TaskStore>,
    wbi_store: State<'_, WbiStore>, 
    id: String,
) -> Result<(), AppError> {
    let task_opt = {
        let tasks = store.tasks.lock().unwrap();
        tasks.iter().find(|t| t.id == id).cloned()
    };

    if let Some(task) = task_opt {
        let bvid = task.bvid.ok_or(AppError::DatabaseError("No BVID".to_string()))?;
        let cid = task.cid.ok_or(AppError::DatabaseError("No CID".to_string()))?;

        // Updated fetch call
        let (audio_url, video_url) = fetch_bili_url(&client.0, &wbi_store, &bvid, &cid).await?;

        let filename = task.save_path.ok_or(AppError::DatabaseError("No save path".to_string()))?;
        let is_video_mode = task.output_file_type == "video";

        let options = DownloadOptions {
            id: id.clone(),
            filename,
            audio_url,
            video_url: if is_video_mode { video_url } else { None },
            is_lossless: task.output_file_type != "mp3",
        };

        let handle = spawn_download_task(app, client.0.clone(), WbiStore(wbi_store.0.clone()), options);
        store.handles.lock().unwrap().insert(id, handle);
    }
    Ok(())
}

#[tauri::command]
pub async fn retry_media_download_task(
    app: AppHandle,
    client: State<'_, AppHttpClient>,
    store: State<'_, TaskStore>,
    wbi_store: State<'_, WbiStore>,
    id: String,
) -> Result<(), AppError> {
    resume_media_download_task(app, client, store, wbi_store, id).await
}

#[tauri::command]
pub async fn cancel_media_download_task(
    app: AppHandle,
    store: State<'_, TaskStore>,
    id: String,
) -> Result<(), AppError> {
    {
        let mut handles = store.handles.lock().unwrap();
        if let Some(handle) = handles.remove(&id) {
            handle.abort();
        }
    }

    let mut tasks = store.tasks.lock().unwrap();
    if let Some(index) = tasks.iter().position(|t| t.id == id) {
        tasks.remove(index);
        // Save to disk
        let _ = store::save_tasks(&app, &tasks);

        let temp_dir = app.path().temp_dir().unwrap().join("biu-downloads");
        let temp_audio = temp_dir.join(format!("{}.audio.tmp", id));
        let temp_video = temp_dir.join(format!("{}.video.tmp", id));
        let _ = fs::remove_file(temp_audio);
        let _ = fs::remove_file(temp_video);

        let _ = app.emit(
            "download:list-sync",
            serde_json::json!({
                "type": "full",
                "data": *tasks
            }),
        );
    }
    Ok(())
}

#[tauri::command]
pub async fn clear_media_download_task_list(
    app: AppHandle,
    store: State<'_, TaskStore>,
) -> Result<(), AppError> {
    let mut tasks = store.tasks.lock().unwrap();
    tasks.clear();
    // Save to disk
    let _ = store::save_tasks(&app, &tasks);

    app.emit(
        "download:list-sync",
        serde_json::json!({
            "type": "full",
            "data": Vec::<MediaDownloadTaskState>::new()
        }),
    )?;

    Ok(())
}