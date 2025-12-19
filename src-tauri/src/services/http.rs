use crate::error::AppError;
use crate::state::models::{AppHttpClient, HttpInvokePayload};
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};
use reqwest::header::{HeaderMap, HeaderName, CONTENT_TYPE};
use reqwest::{Client, Method};
use serde_json;
use std::fs::File;
use std::io::BufReader;
use std::str::FromStr;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

/// Helper to construct a client with a specific cookie store.
/// This replaces the old `build_client` for use cases like the Proxy server
/// which already has access to the store.
pub fn construct_client(cookie_store: Arc<CookieStoreMutex>) -> Client {
    Client::builder()
        .cookie_provider(cookie_store)
        .user_agent(DEFAULT_USER_AGENT)
        .build()
        .unwrap_or_else(|_| Client::new())
}

/// Rewritten build_client to handle initialization from AppHandle.
/// 1. Locates config dir
/// 2. Loads cookies
/// 3. Injects store
pub fn build_client(app: &AppHandle) -> Result<(Client, Arc<CookieStoreMutex>), AppError> {
    // 1. Locate the Config Directory
    let app_data_dir = app.path().app_local_data_dir()
        .map_err(|e| AppError::TauriError(e))?;

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir).map_err(AppError::IoError)?;
    }

    // 2. Define the Cookie File Path
    let cookie_path = app_data_dir.join("cookies.json");

    // 3. Load Existing Cookies
    let cookie_store_inner = if cookie_path.exists() {
        let file = File::open(&cookie_path).map(BufReader::new).ok();
        file.and_then(|r| serde_json::from_reader(r).ok())
            .unwrap_or_default()
    } else {
        CookieStore::default()
    };

    let cookie_store = Arc::new(CookieStoreMutex::new(cookie_store_inner));

    // 4. Inject the Store
    let client = construct_client(cookie_store.clone());

    Ok((client, cookie_store))
}

/// Save cookies to disk
pub fn save_cookies(app: &AppHandle, store: &Arc<CookieStoreMutex>) -> Result<(), AppError> {
    let app_data_dir = app.path().app_local_data_dir()
        .map_err(|e| AppError::TauriError(e))?;
    let cookie_path = app_data_dir.join("cookies.json");

    let file = File::create(cookie_path).map_err(AppError::IoError)?;
    let store = store.lock().unwrap();
    serde_json::to_writer_pretty(file, &*store)
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    Ok(())
}

pub async fn make_request(
    client: &AppHttpClient,
    method: String,
    url: String,
    body: Option<serde_json::Value>,
    options: Option<HttpInvokePayload>,
) -> Result<serde_json::Value, AppError> {
    let req_method =
        Method::from_str(&method.to_uppercase()).map_err(|e| AppError::NetworkError(e.to_string()))?;
    let mut req = client.0.request(req_method, &url);
    let mut is_form = false;

    if let Some(payload) = options {
        if let Some(headers) = payload.headers {
            let mut hmap = HeaderMap::new();
            for (k, v) in headers {
                if let Ok(hname) = k.parse::<HeaderName>() {
                    if let Ok(hval) = v.parse::<tauri::http::HeaderValue>() {
                        if hname == CONTENT_TYPE
                            && hval
                                .to_str()
                                .unwrap_or("")
                                .contains("application/x-www-form-urlencoded")
                        {
                            is_form = true;
                        }
                        hmap.insert(hname, hval);
                    }
                }
            }
            req = req.headers(hmap);
        }
        if let Some(params) = payload.params {
            req = req.query(&params);
        }
        if let Some(timeout_ms) = payload.timeout {
            req = req.timeout(std::time::Duration::from_millis(timeout_ms));
        }
    }

    if let Some(b) = body {
        if is_form {
            req = req.form(&b);
        } else {
            req = req.json(&b);
        }
    }

    let res = req.send().await.map_err(|e| AppError::NetworkError(e.to_string()))?;
    let text_res = res.text().await.map_err(|e| AppError::NetworkError(e.to_string()))?;

    match serde_json::from_str::<serde_json::Value>(&text_res) {
        Ok(json) => Ok(json),
        Err(_) => Ok(serde_json::Value::String(text_res)),
    }
}