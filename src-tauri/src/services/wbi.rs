use crate::error::AppError;
use crate::services::http::DEFAULT_USER_AGENT;
use crate::state::models::WbiStore;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const MIXIN_KEY_ENC_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29, 28, 14, 39, 12, 38, 41,
    13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34,
    44, 52,
];

fn get_mixin_key(orig: &str) -> String {
    let mut s = String::with_capacity(32);
    for &i in MIXIN_KEY_ENC_TAB.iter().take(32) {
        if let Some(c) = orig.chars().nth(i) {
            s.push(c);
        }
    }
    s
}

async fn get_wbi_keys(client: &Client, store: &WbiStore) -> Result<(String, String), AppError> {
    // 1. Check Cache
    {
        let cache = store.0.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // Use cached keys if they exist and are less than 24 hours old
        if !cache.img_key.is_empty() && !cache.sub_key.is_empty() && (now - cache.last_fetch < 86400) {
            return Ok((cache.img_key.clone(), cache.sub_key.clone()));
        }
    }

    // 2. Fetch Keys
    let res = client
        .get("https://api.bilibili.com/x/web-interface/nav")
        .header(reqwest::header::USER_AGENT, DEFAULT_USER_AGENT)
        .header(reqwest::header::REFERER, "https://www.bilibili.com/")
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    let json: Value = res
        .json()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    // 3. Extract Keys
    if let Some(data) = json.get("data") {
        if let Some(wbi_img) = data.get("wbi_img") {
            let img_url = wbi_img.get("img_url").and_then(|v| v.as_str()).unwrap_or("");
            let sub_url = wbi_img.get("sub_url").and_then(|v| v.as_str()).unwrap_or("");

            let img_key = img_url
                .split('/')
                .last()
                .unwrap_or("")
                .split('.')
                .next()
                .unwrap_or("")
                .to_string();
            let sub_key = sub_url
                .split('/')
                .last()
                .unwrap_or("")
                .split('.')
                .next()
                .unwrap_or("")
                .to_string();

            if !img_key.is_empty() && !sub_key.is_empty() {
                // Update Cache
                let mut cache = store.0.lock().unwrap();
                cache.img_key = img_key.clone();
                cache.sub_key = sub_key.clone();
                cache.last_fetch = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                return Ok((img_key, sub_key));
            }
        }
    }

    Err(AppError::NetworkError("Failed to fetch WBI keys".to_string()))
}

pub async fn sign_params(
    client: &Client,
    store: &WbiStore,
    mut params: HashMap<String, String>,
) -> Result<HashMap<String, String>, AppError> {
    let (img_key, sub_key) = get_wbi_keys(client, store).await?;
    let mixin_key = get_mixin_key(&format!("{}{}", img_key, sub_key));
    let curr_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Add wts
    params.insert("wts".to_string(), curr_time.to_string());

    // Sort params by key
    let mut keys: Vec<&String> = params.keys().collect();
    keys.sort();

    // Construct query string for signing
    let mut query_parts = Vec::new();
    for key in keys {
        let val = params.get(key).unwrap();
        // Filter restricted characters from value
        let val_filtered: String = val.chars().filter(|c| !"!'()*".contains(*c)).collect();
        // URL Encode
        let encoded_key = urlencoding::encode(key);
        let encoded_val = urlencoding::encode(&val_filtered);
        query_parts.push(format!("{}={}", encoded_key, encoded_val));
    }
    let query_str = query_parts.join("&");

    // Calculate Hash
    let hash_input = format!("{}{}", query_str, mixin_key);
    let w_rid = format!("{:x}", md5::compute(hash_input));

    params.insert("w_rid".to_string(), w_rid);

    Ok(params)
}