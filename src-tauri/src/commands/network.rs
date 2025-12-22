use crate::error::AppError;
use crate::services::wbi;
use crate::state::models::{AppCookieStore, AppHttpClient, HttpInvokePayload, ProxyPort, WbiStore};
use serde_json;
use std::collections::HashMap;
use tauri::{AppHandle, State};
use url::Url;

#[tauri::command]
pub async fn http_request(
    client: State<'_, AppHttpClient>,
    method: String,
    url: String,
    body: Option<serde_json::Value>,
    options: Option<HttpInvokePayload>,
) -> Result<serde_json::Value, AppError> {
    crate::services::http::make_request(&client, method, url, body, options).await
}

#[tauri::command]
pub async fn get_cookie(
    _client: State<'_, AppHttpClient>,
    _key: String,
) -> Result<Option<String>, AppError> {
    Ok(None)
}

// --- NEW COMMAND ---
#[tauri::command]
pub async fn set_cookie(
    app: AppHandle,
    cookie_store: State<'_, AppCookieStore>,
    name: String,
    value: String,
    expiration_date: Option<i64>,
) -> Result<(), AppError> {
    let url_str = "https://bilibili.com/";
    let url = Url::parse(url_str).map_err(|e| AppError::NetworkError(e.to_string()))?;

    // Construct Set-Cookie header string matching the reference implementation:
    // domain: ".bilibili.com", path: "/", secure: true, sameSite: "no_restriction" (None)
    let mut cookie_str = format!("{}={}; Domain=.bilibili.com; Path=/; Secure; SameSite=None", name, value);

    // Handle expiration (flushStore reference implies persistence, expiration handles validity)
    if let Some(exp) = expiration_date {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        let max_age = exp - now;
        if max_age > 0 {
            cookie_str.push_str(&format!("; Max-Age={}", max_age));
        }
    }

    // 1. Update In-Memory Store
    {
        let mut store = cookie_store.0.lock().unwrap();
        // We use parse() to handle all the attributes (Domain, Path, etc.) correctly
        store.parse(&cookie_str, &url)
            .map_err(|e| AppError::NetworkError(format!("Failed to parse cookie: {}", e)))?;
    }

    // 2. Flush to Disk (Equivalent to session.defaultSession.cookies.flushStore())
    crate::services::http::save_cookies(&app, &cookie_store.0)?;

    Ok(())
}
// -------------------

#[tauri::command]
pub async fn http_get(
    client: State<'_, AppHttpClient>,
    url: String,
    options: Option<HttpInvokePayload>,
) -> Result<serde_json::Value, AppError> {
    crate::services::http::make_request(&client, "GET".to_string(), url, None, options).await
}

#[tauri::command]
pub async fn http_post(
    client: State<'_, AppHttpClient>,
    url: String,
    body: Option<serde_json::Value>,
    options: Option<HttpInvokePayload>,
) -> Result<serde_json::Value, AppError> {
    crate::services::http::make_request(&client, "POST".to_string(), url, body, options).await
}

#[tauri::command]
pub async fn get_proxy_port(state: State<'_, ProxyPort>) -> Result<u16, AppError> {
    let port = *state.0.lock().unwrap();
    if port == 0 {
        return Err(AppError::NetworkError("Proxy not ready".to_string()));
    }
    Ok(port)
}

#[tauri::command]
pub async fn wbi_sign_params(
    client: State<'_, AppHttpClient>,
    wbi_store: State<'_, WbiStore>,
    params: HashMap<String, String>,
) -> Result<HashMap<String, String>, AppError> {
    wbi::sign_params(&client.0, &wbi_store, params).await
}