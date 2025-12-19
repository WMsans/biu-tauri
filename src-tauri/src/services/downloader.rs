use crate::commands::store;
use crate::error::AppError;
use crate::services::http::DEFAULT_USER_AGENT;
use crate::services::wbi;
use crate::state::models::*;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, RANGE, REFERER, USER_AGENT};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;
use tokio::io::AsyncWriteExt;

pub async fn fetch_bili_url(
    client: &reqwest::Client,
    wbi_store: &WbiStore,
    bvid: &str,
    cid: &str,
) -> Result<String, AppError> {
    // 1. Prepare raw params
    let mut params = HashMap::new();
    params.insert("bvid".to_string(), bvid.to_string());
    params.insert("cid".to_string(), cid.to_string());
    params.insert("qn".to_string(), "80".to_string()); // 80: 1080P
    params.insert("fnval".to_string(), "16".to_string()); // 16: Dash
    params.insert("fnver".to_string(), "0".to_string());
    params.insert("fourk".to_string(), "1".to_string());

    // 2. Sign params using WBI service
    let signed_params = wbi::sign_params(client, wbi_store, params).await?;

    // 3. Construct URL
    let res = client
        .get("https://api.bilibili.com/x/player/wbi/playurl")
        .query(&signed_params)
        .header(REFERER, "https://www.bilibili.com")
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    let json: BiliPlayUrlResponse = res
        .json()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    if json.code != 0 {
        return Err(AppError::NetworkError(format!(
            "Bilibili API Error Code: {}",
            json.code
        )));
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
    .ok_or(AppError::NetworkError("No audio stream found".to_string()))
}

pub fn spawn_download_task(
    app: AppHandle,
    client: reqwest::Client,
    wbi_store: WbiStore,
    params: DownloadOptions,
) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = download_task_runner(app, client, params).await {
            log::error!("Download task failed: {}", e);
        }
    })
}

async fn download_task_runner(
    app: AppHandle,
    client: reqwest::Client,
    params: DownloadOptions,
) -> Result<(), AppError> {
    let settings = store::load_settings(&app)?;
    let download_dir = PathBuf::from(settings.download_path.unwrap_or_default());
    fs::create_dir_all(&download_dir)?;
    let output_path = download_dir.join(&params.filename);
    let temp_dir = app.path().temp_dir().unwrap().join("biu-downloads");
    fs::create_dir_all(&temp_dir)?;
    let temp_audio_path = temp_dir.join(format!("{}.audio.tmp", params.id));

    let options_clone = params.id.clone();
    let is_lossless = params.is_lossless;
    let audio_url = params.audio_url.clone();
    let app_handle = app.clone();

    // Closure to update both UI events and Backend State
    let update_status =
        |status: &str,
         progress: Option<u64>,
         downloaded: Option<u64>,
         total: Option<u64>,
         error: Option<String>| {
            // UI Event
            let _ = app_handle.emit(
                "download:progress",
                DownloadProgress {
                    id: options_clone.clone(),
                    status: status.to_string(),
                    progress,
                    downloaded_bytes: downloaded,
                    total_bytes: total,
                    error: error.clone(),
                },
            );
            // Store Update
            if let Some(store) = app_handle.try_state::<TaskStore>() {
                let mut tasks = store.tasks.lock().unwrap();
                if let Some(t) = tasks.iter_mut().find(|t| t.id == options_clone) {
                    t.status = status.to_string();
                    if let Some(p) = progress {
                        // If converting, update convert_progress specific field
                        if status == "converting" {
                            t.convert_progress = Some(p);
                        } else {
                            t.download_progress = Some(p);
                        }
                    }
                    if let Some(tot) = total {
                        t.total_bytes = Some(tot);
                    }
                    if let Some(err) = &error {
                        t.error = Some(err.clone());
                    }
                    if status == "merging" {
                        t.merge_progress = Some(50);
                    }
                    // For converting, we rely on the granular progress now
                    // if status == "converting" { ... }

                    if status == "completed" {
                        t.download_progress = Some(100);
                        t.merge_progress = Some(100);
                        t.convert_progress = Some(100);
                        // Clean up handle from store on completion
                        if let Some(store) = app_handle.try_state::<TaskStore>() {
                            let mut handles = store.handles.lock().unwrap();
                            handles.remove(&options_clone);
                        }
                    }
                    // Emit Sync
                    let updated = t.clone();
                    drop(tasks);
                    let _ = app_handle.emit(
                        "download:list-sync",
                        serde_json::json!({ "type": "update", "data": [updated] }),
                    );
                }
            }
        };

    let mut start_byte = 0;
    if temp_audio_path.exists() {
        if let Ok(metadata) = fs::metadata(&temp_audio_path) {
            start_byte = metadata.len();
        }
    }

    let mut headers = HeaderMap::new();
    headers.insert(REFERER, "https://www.bilibili.com".parse().unwrap());
    headers.insert(USER_AGENT, DEFAULT_USER_AGENT.parse().unwrap());
    if start_byte > 0 {
        headers.insert(RANGE, format!("bytes={}-", start_byte).parse().unwrap());
    }

    let res = client.get(&audio_url).headers(headers).send().await?;
    if !res.status().is_success() {
        update_status(
            "failed",
            None,
            None,
            None,
            Some(format!("HTTP {}", res.status())),
        );
        return Ok(());
    }
    let total_size = res.content_length().map(|l| l + start_byte);
    let mut stream = res.bytes_stream();
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&temp_audio_path)
        .await?;
    let mut downloaded = start_byte;

    update_status("downloading", Some(0), Some(downloaded), total_size, None);

    while let Some(item) = stream.next().await {
        if let Ok(chunk) = item {
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            let pct = if let Some(total) = total_size {
                (downloaded as f64 / total as f64 * 100.0) as u64
            } else {
                0
            };
            update_status("downloading", Some(pct), Some(downloaded), total_size, None);
        } else {
            break;
        }
    }

    // Merging/Converting Logic
    update_status("merging", None, None, None, None);
    if is_lossless {
        if let Err(_) = fs::rename(&temp_audio_path, &output_path) {
            fs::copy(&temp_audio_path, &output_path)?;
            fs::remove_file(&temp_audio_path)?;
        }
    } else {
        update_status("converting", Some(0), None, None, None);
        let shell = app_handle.shell();
        let cmd = shell.command("ffmpeg").args(&[
            "-y",
            "-i",
            temp_audio_path.to_str().unwrap(),
            "-vn",
            "-codec:a",
            "libmp3lame",
            "-q:a",
            "2",
            output_path.to_str().unwrap(),
        ]);

        let (mut rx, mut _child) = cmd.spawn().map_err(|e| AppError::ShellError(e))?;

        let mut total_duration_secs: Option<f64> = None;

        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    
                    // 1. Parse Duration (e.g., "Duration: 00:03:59.04,")
                    if total_duration_secs.is_none() {
                        if let Some(idx) = line.find("Duration: ") {
                            let remainder = &line[idx + 10..];
                            if let Some(comma_idx) = remainder.find(',') {
                                let dur_str = &remainder[..comma_idx];
                                total_duration_secs = parse_ffmpeg_time(dur_str);
                            }
                        }
                    }

                    // 2. Parse Progress (e.g., "time=00:00:55.82")
                    if let Some(total) = total_duration_secs {
                        if let Some(idx) = line.find("time=") {
                            let remainder = &line[idx + 5..];
                            let end_idx = remainder.find(' ').unwrap_or(remainder.len());
                            let time_str = &remainder[..end_idx];
                            
                            if let Some(current) = parse_ffmpeg_time(time_str) {
                                if total > 0.0 {
                                    let pct = ((current / total) * 100.0) as u64;
                                    // Limit to 99 until finished
                                    let safe_pct = if pct >= 100 { 99 } else { pct };
                                    update_status("converting", Some(safe_pct), None, None, None);
                                }
                            }
                        }
                    }
                }
                CommandEvent::Terminated(payload) => {
                    if let Some(code) = payload.code {
                        if code != 0 {
                            update_status(
                                "failed",
                                None,
                                None,
                                None,
                                Some(format!("FFmpeg exited with code {}", code)),
                            );
                            return Ok(());
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Remove temp file after conversion
        if temp_audio_path.exists() {
            let _ = fs::remove_file(&temp_audio_path);
        }
    }

    update_status("completed", Some(100), None, None, None);
    Ok(())
}

fn parse_ffmpeg_time(time_str: &str) -> Option<f64> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let h: f64 = parts[0].parse().ok()?;
        let m: f64 = parts[1].parse().ok()?;
        let s: f64 = parts[2].parse().ok()?;
        Some(h * 3600.0 + m * 60.0 + s)
    } else {
        None
    }
}