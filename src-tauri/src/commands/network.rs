use crate::error::AppError;
use crate::state::models::{AppHttpClient, HttpInvokePayload, ProxyPort};
use serde_json;
use tauri::State;

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
