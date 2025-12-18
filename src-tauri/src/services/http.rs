use reqwest::cookie::Jar;
use reqwest::Client;
use std::sync::Arc;
use crate::state::models::HttpInvokePayload;
use reqwest::header::{HeaderMap, HeaderName, CONTENT_TYPE};
use reqwest::Method;
use std::str::FromStr;
use serde_json;
use crate::state::models::AppHttpClient;

pub const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub fn build_client() -> Client {
    let jar = Arc::new(Jar::default());
    Client::builder()
        .cookie_provider(jar)
        .user_agent(DEFAULT_USER_AGENT)
        .build()
        .unwrap()
}

pub async fn make_request(
    client: &AppHttpClient,
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