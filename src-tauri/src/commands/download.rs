use std::fs;
use std::path::PathBuf;

use futures_util::StreamExt;
use reqwest::header::{HeaderMap, REFERER, USER_AGENT, RANGE};
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_shell::ShellExt;
use tokio::io::AsyncWriteExt;

use crate::state::models::*;
use crate::DEFAULT_USER_AGENT;
use crate::commands::store;


async fn fetch_bili_url(client: &reqwest::Client, bvid: &str, cid: &str) -> Result<String, String> {
    let api_url = format!(
        "https://api.bilibili.com/x/player/playurl?bvid={}&cid={}&qn=80&fnval=16",
        bvid, cid
    );
    let res = client
        .get(&api_url)
        .header(REFERER, "https://www.bilibili.com")
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let json: BiliPlayUrlResponse = res.json().await.map_err(|e| format!("JSON error: {}", e))?;

    if json.code != 0 {
        return Err(format!("Bilibili API Error Code: {}", json.code));
    }

    if let Some(data) = &json.data {
        if let Some(dash) = &data.dash {
            dash.audio
                .as_ref()
                .and_then(|audios| audios.first().map(|a| a.base_url.clone()))
        } else if let Some(durls) = &data.durl {
            durls.first().map(|d| d.url.clone())
        } else {
            None
        }
    } else {
        None
    }
    .ok_or("No audio stream found".to_string())
}

fn spawn_download_task(
    app: AppHandle,
    client: reqwest::Client,
    params: DownloadOptions,
) -> JoinHandle<()> {
    let settings = store::load_settings(&app);
    let download_dir = PathBuf::from(settings.download_path.unwrap_or_default());
    let _ = fs::create_dir_all(&download_dir);
    let output_path = download_dir.join(&params.filename);
    let temp_dir = app.path().temp_dir().unwrap().join("biu-downloads");
    let _ = fs::create_dir_all(&temp_dir);
    let temp_audio_path = temp_dir.join(format!("{}.audio.tmp", params.id));

    let options_clone = params.id.clone();
    let is_lossless = params.is_lossless;
    let audio_url = params.audio_url.clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        // Closure to update both UI events and Backend State
        let update_status = |status: &str, progress: Option<u64>, downloaded: Option<u64>, total: Option<u64>, error: Option<String>| {
            // UI Event
            let _ = app_handle.emit("download:progress", DownloadProgress {
                id: options_clone.clone(), status: status.to_string(), progress, downloaded_bytes: downloaded, total_bytes: total, error: error.clone(),
            });
            // Store Update
            if let Some(store) = app_handle.try_state::<TaskStore>() {
                let mut tasks = store.tasks.lock().unwrap(); // NOTE: Access .tasks specifically
                if let Some(t) = tasks.iter_mut().find(|t| t.id == options_clone) {
                    t.status = status.to_string();
                    if let Some(p) = progress { t.download_progress = Some(p); }
                    if let Some(tot) = total { t.total_bytes = Some(tot); }
                    if let Some(err) = &error { t.error = Some(err.clone()); }
                    if status == "merging" { t.merge_progress = Some(50); }
                    if status == "converting" { t.convert_progress = Some(10); }
                    if status == "completed" {
                        t.download_progress = Some(100);
                        t.merge_progress = Some(100);
                        t.convert_progress = Some(100);
                        // Clean up handle from store on completion (optional but good practice)
                         if let Some(store) = app_handle.try_state::<TaskStore>() {
                            let mut handles = store.handles.lock().unwrap();
                            handles.remove(&options_clone);
                        }
                    }
                    // Emit Sync
                    let updated = t.clone();
                    drop(tasks);
                    let _ = app_handle.emit("download:list-sync", serde_json::json!({ "type": "update", "data": [updated] }));
                }
            }
        };

        let mut start_byte = 0;
        if temp_audio_path.exists() {
             if let Ok(metadata) = fs::metadata(&temp_audio_path) { start_byte = metadata.len(); }
        }
        
        let mut headers = HeaderMap::new();
        headers.insert(REFERER, "https://www.bilibili.com".parse().unwrap());
        headers.insert(USER_AGENT, DEFAULT_USER_AGENT.parse().unwrap());
        if start_byte > 0 {
            headers.insert(RANGE, format!("bytes={}-", start_byte).parse().unwrap());
        }

        match client.get(&audio_url).headers(headers).send().await {
            Ok(res) => {
                if !res.status().is_success() {
                    update_status("failed", None, None, None, Some(format!("HTTP {}", res.status())));
                    return;
                }
                let total_size = res.content_length().map(|l| l + start_byte);
                let mut stream = res.bytes_stream();
                let mut file = tokio::fs::OpenOptions::new().create(true).append(true).open(&temp_audio_path).await.expect("Failed to open temp");
                let mut downloaded = start_byte;
                
                update_status("downloading", Some(0), Some(downloaded), total_size, None);
                
                while let Some(item) = stream.next().await {
                    if let Ok(chunk) = item {
                        if file.write_all(&chunk).await.is_err() {
                            update_status("failed", None, None, None, Some("Write error".into()));
                            return;
                        }
                        downloaded += chunk.len() as u64;
                        let pct = if let Some(total) = total_size { (downloaded as f64 / total as f64 * 100.0) as u64 } else { 0 };
                        update_status("downloading", Some(pct), Some(downloaded), total_size, None);
                    } else { break; }
                }
                
                // Merging/Converting Logic (Simplified copy from original)
                update_status("merging", None, None, None, None);
                if is_lossless {
                     let _ = fs::rename(&temp_audio_path, &output_path);
                } else {
                    update_status("converting", None, None, None, None);
                     let shell = app_handle.shell();
                     let status = shell.command("ffmpeg").args(&["-y", "-i", temp_audio_path.to_str().unwrap(), "-vn", "-codec:a", "libmp3lame", "-q:a", "2", output_path.to_str().unwrap()]).output().await;
                     if let Ok(o) = status { if o.status.success() { let _ = fs::remove_file(&temp_audio_path); } else { update_status("failed", None, None, None, Some("FFmpeg failed".into())); return; } }
                }
                update_status("completed", Some(100), None, None, None);
            }
            Err(e) => update_status("failed", None, None, None, Some(e.to_string())),
        }
    })
}

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
