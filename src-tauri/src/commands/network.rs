use crate::state::models::{AppHttpClient, HttpInvokePayload, ProxyPort};
use reqwest::header::{HeaderMap, HeaderName, CONTENT_TYPE};
use reqwest::Method;
use std::str::FromStr;
use tauri::State;
use serde_json;

#[tauri::command]
pub async fn http_request(
    client: State<'_, AppHttpClient>,
    method: String,
    url: String,
    body: Option<serde_json::Value>,
    options: Option<HttpInvokePayload>,
) -> Result<serde_json::Value, String> {
    let req_method = Method::from_str(&method.to_uppercase()).map_err(|e| e.to_string())?;
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

    let res = req.send().await.map_err(|e| e.to_string())?;
    let text_res = res.text().await.map_err(|e| e.to_string())?;

    match serde_json::from_str::<serde_json::Value>(&text_res) {
        Ok(json) => Ok(json),
        Err(_) => Ok(serde_json::Value::String(text_res)),
    }
}

#[tauri::command]
pub async fn get_cookie(
    _client: State<'_, AppHttpClient>,
    _key: String,
) -> Result<Option<String>, String> {
    Ok(None)
}

#[tauri::command]
pub async fn http_get(
    client: State<'_, AppHttpClient>,
    url: String,
    options: Option<HttpInvokePayload>,
) -> Result<serde_json::Value, String> {
    let mut req = client.0.get(&url);
    if let Some(payload) = options {
        if let Some(headers) = payload.headers {
            let mut hmap = HeaderMap::new();
            for (k, v) in headers {
                if let Ok(hname) = k.parse::<HeaderName>() {
                    if let Ok(hval) = v.parse() {
                        hmap.insert(hname, hval);
                    }
                }
            }
            req = req.headers(hmap);
        }
        if let Some(params) = payload.params {
            req = req.query(&params);
        }
    }

    let res = req.send().await.map_err(|e| e.to_string())?;
    let text_res = res.text().await.map_err(|e| e.to_string())?;
    match serde_json::from_str::<serde_json::Value>(&text_res) {
        Ok(json) => Ok(json),
        Err(_) => Ok(serde_json::Value::String(text_res)),
    }
}

#[tauri::command]
pub async fn http_post(
    client: State<'_, AppHttpClient>,
    url: String,
    body: Option<serde_json::Value>,
    options: Option<HttpInvokePayload>,
) -> Result<serde_json::Value, String> {
    let mut req = client.0.post(&url);
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
    }

    if let Some(b) = body {
        if is_form {
            req = req.form(&b);
        } else {
            req = req.json(&b);
        }
    }

    let res = req.send().await.map_err(|e| e.to_string())?;
    let text_res = res.text().await.map_err(|e| e.to_string())?;
    match serde_json::from_str::<serde_json::Value>(&text_res) {
        Ok(json) => Ok(json),
        Err(_) => Ok(serde_json::Value::String(text_res)),
    }
}

#[tauri::command]
pub async fn get_proxy_port(state: State<'_, ProxyPort>) -> Result<u16, String> {
    let port = *state.0.lock().unwrap();
    if port == 0 {
        return Err("Proxy not ready".into());
    }
    Ok(port)
}
