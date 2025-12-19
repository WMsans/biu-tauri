use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::async_runtime::JoinHandle;

// Store the dynamic port of our local proxy
pub struct ProxyPort(pub Arc<Mutex<u16>>);

// WBI Keys Cache
pub struct WbiKeysCache {
    pub img_key: String,
    pub sub_key: String,
    pub last_fetch: u64,
}

impl WbiKeysCache {
    pub fn new() -> Self {
        Self {
            img_key: String::new(),
            sub_key: String::new(),
            last_fetch: 0,
        }
    }
}

// Container for WBI Store
pub struct WbiStore(pub Arc<Mutex<WbiKeysCache>>);

// 1. Define the persistent state for tasks
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaDownloadTaskState {
    pub id: String,
    // Base fields from request
    pub output_file_type: String,
    pub title: String,
    pub cover: Option<String>,
    pub bvid: Option<String>,
    pub cid: Option<String>,
    pub sid: Option<String>,
    // Extended fields for status
    pub audio_codecs: Option<String>,
    pub audio_bandwidth: Option<u64>,
    pub video_resolution: Option<String>,
    pub video_frame_rate: Option<String>,
    pub save_path: Option<String>,
    pub total_bytes: Option<u64>,
    pub download_progress: Option<u64>,
    pub merge_progress: Option<u64>,
    pub convert_progress: Option<u64>,
    pub error: Option<String>,
    pub status: String, // "pending", "downloading", "merging", "converting", "completed", "failed"
}

// 2. Define the Store Container
pub struct TaskStore {
    pub tasks: Arc<Mutex<Vec<MediaDownloadTaskState>>>,
    pub handles: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(Vec::new())),
            handles: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub download_path: Option<String>,
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadOptions {
    pub id: String,
    pub filename: String,
    pub audio_url: String,
    pub is_lossless: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadProgress {
    pub id: String,
    pub status: String,
    pub progress: Option<u64>,
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpInvokePayload {
    pub url: String,
    pub params: Option<serde_json::Value>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub body: Option<serde_json::Value>,
    pub timeout: Option<u64>,
}

// Global State for HTTP Client
pub struct AppHttpClient(pub reqwest::Client);

// Input struct for creating a task (simpler than the State struct)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaDownloadRequest {
    pub output_file_type: String,
    pub title: String,
    pub cover: Option<String>,
    pub bvid: Option<String>,
    pub cid: Option<String>,
    pub sid: Option<String>,
}

// Bilibili API Response Helper Structs
#[derive(Debug, Deserialize)]
pub struct BiliPlayUrlResponse {
    pub code: i32,
    pub data: Option<BiliPlayUrlData>,
}

#[derive(Debug, Deserialize)]
pub struct BiliPlayUrlData {
    pub dash: Option<BiliDashData>,
    pub durl: Option<Vec<BiliDurlData>>,
}

#[derive(Debug, Deserialize)]
pub struct BiliDashData {
    pub audio: Option<Vec<BiliDashMedia>>,
}

#[derive(Debug, Deserialize)]
pub struct BiliDashMedia {
    pub base_url: String,
}

#[derive(Debug, Deserialize)]
pub struct BiliDurlData {
    pub url: String,
}