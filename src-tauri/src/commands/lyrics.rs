use crate::error::AppError;
use crate::state::models::AppHttpClient;
use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT, ORIGIN};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{State, command};

const NETEASE_SEARCH_URL: &str = "https://interface.music.163.com/api/search/get";
const NETEASE_LYRIC_URL: &str = "https://interface.music.163.com/api/song/lyric";
const LRCLIB_SEARCH_URL: &str = "https://lrclib.net/api/search";
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

// --- Params Structs (Deserialize from Frontend) ---

#[derive(Debug, Deserialize)]
pub struct SearchSongByNeteaseParams {
    s: String,
    #[serde(rename = "type")]
    query_type: i32, 
    limit: i32,
    offset: i32,
}

#[derive(Debug, Deserialize)]
pub struct GetLyricsByNeteaseParams {
    id: i64,
}

#[derive(Debug, Deserialize)]
pub struct SearchSongByLrclibParams {
    q: String,
    track_name: Option<String>,
    artist_name: Option<String>,
    album_name: Option<String>,
}

// --- Query Param Structs (Serialize to URL) ---

#[derive(Serialize)]
struct NeteaseQueryParams {
    s: String,
    #[serde(rename = "type")]
    type_: i32,
    limit: i32,
    offset: i32,
}

#[derive(Serialize)]
struct NeteaseLyricQueryParams {
    id: i64,
    tv: i32,
    lv: i32,
    rv: i32,
    kv: i32,
    _nmclfl: i32,
}

#[derive(Serialize)]
struct LrclibQueryParams {
    q: String,
    track_name: Option<String>,
    artist_name: Option<String>,
    album_name: Option<String>,
}

// --- Commands ---

#[command]
pub async fn search_netease_songs(
    client: State<'_, AppHttpClient>,
    params: SearchSongByNeteaseParams,
) -> Result<Value, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(REFERER, HeaderValue::from_static("https://music.163.com/"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://music.163.com"));
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));

    let query = NeteaseQueryParams {
        s: params.s,
        type_: params.query_type,
        limit: params.limit,
        offset: params.offset,
    };

    let response = client.0.get(NETEASE_SEARCH_URL)
        .headers(headers)
        .query(&query)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    let json = response.json::<Value>().await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    Ok(json)
}

#[command]
pub async fn get_netease_lyrics(
    client: State<'_, AppHttpClient>,
    params: GetLyricsByNeteaseParams,
) -> Result<Value, AppError> {
    let mut headers = HeaderMap::new();
    headers.insert(REFERER, HeaderValue::from_static("https://music.163.com/"));
    headers.insert(ORIGIN, HeaderValue::from_static("https://music.163.com"));
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));

    let query = NeteaseLyricQueryParams {
        id: params.id,
        tv: -1,
        lv: -1,
        rv: -1,
        kv: -1,
        _nmclfl: 1,
    };

    let response = client.0.get(NETEASE_LYRIC_URL)
        .headers(headers)
        .query(&query)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    let json = response.json::<Value>().await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    Ok(json)
}

#[command]
pub async fn search_lrclib_lyrics(
    client: State<'_, AppHttpClient>,
    params: SearchSongByLrclibParams,
) -> Result<Value, AppError> {
    let query = LrclibQueryParams {
        q: params.q,
        track_name: params.track_name,
        artist_name: params.artist_name,
        album_name: params.album_name,
    };

    let response = client.0.get(LRCLIB_SEARCH_URL)
        .query(&query)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    let json = response.json::<Value>().await
        .map_err(|e| AppError::NetworkError(e.to_string()))?;

    Ok(json)
}
