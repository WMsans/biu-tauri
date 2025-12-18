use crate::error::AppError;
use crate::services::wbi;
use crate::state::models::{AppHttpClient, HttpInvokePayload, ProxyPort, WbiStore};
use serde_json;
use std::collections::HashMap;
use tauri::State;
use reqwest::Url;

#[tauri::command]
pub async fn sync_cookies(
    client: State<'_, AppHttpClient>,
    cookie_str: String,
) -> Result<(), AppError> {
    let url = Url::parse("https://bilibili.com").unwrap();
    // Parse the cookie string (e.g. "key=value; key2=value2")
    for cookie in cookie_str.split(';') {
        let trimmed = cookie.trim();
        if !trimmed.is_empty() {
            client.cookie_store.add_cookie_str(trimmed, &url);
        }
    }
    Ok(())
}

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
    wbi::sign_params(&client.client, &wbi_store, params).await
}