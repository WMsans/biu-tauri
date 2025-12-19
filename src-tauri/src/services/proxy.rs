use anyhow::Result;
use futures_util::StreamExt;
use reqwest::header::{CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE, REFERER};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use crate::services::http::build_client;
use reqwest_cookie_store::CookieStoreMutex;

// --- Helper Functions ---

// Helper to extract param from "key=value" string in a request
fn extract_query_param(request: &str, key: &str) -> Option<String> {
    if let Some(start) = request.find(key) {
        let rest = &request[start + key.len()..];
        let end = rest.find(|c| c == '&' || c == ' ').unwrap_or(rest.len());
        return Some(rest[..end].to_string());
    }
    None
}

// --- Proxy Server Logic ---
pub async fn run_proxy_server(port_state: Arc<Mutex<u16>>, cookie_store: Arc<CookieStoreMutex>) -> Result<()> {
    // Bind to a random available port on localhost
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    // Save the port so frontend can ask for it
    {
        let mut p = port_state.lock().unwrap();
        *p = port;
    }

    // Pass the cookie_store to build_client
    let client = build_client(cookie_store);

    loop {
        if let Ok((mut socket, _)) = listener.accept().await {
            let client_clone = client.clone();
            tokio::spawn(async move {
                let mut buf = [0; 2048];
                // Read the HTTP request (just enough to get headers)
                if let Ok(n) = socket.read(&mut buf).await {
                    if n == 0 {
                        return;
                    }
                    let request_str = String::from_utf8_lossy(&buf[..n]);

                    // 1. Parse URL parameters manually to avoid dependencies
                    // Expecting: GET /?url=...&referer=... HTTP/1.1
                    let first_line = request_str.lines().next().unwrap_or("");
                    if !first_line.contains("GET") {
                        return;
                    }

                    // Extract query params
                    let target_url = extract_query_param(&request_str, "url=");
                    let referer_url = extract_query_param(&request_str, "referer=");

                    if let Some(url) = target_url {
                        let decoded_url = 
                            urlencoding::decode(&url).unwrap_or(std::borrow::Cow::Borrowed(&url));
                        let decoded_referer = if let Some(r) = referer_url {
                            urlencoding::decode(&r)
                                .unwrap_or(std::borrow::Cow::Borrowed("https://www.bilibili.com/"))
                                .into_owned()
                        } else {
                            "https://www.bilibili.com/".to_string()
                        };

                        // 2. Extract Range Header
                        let range_header = request_str
                            .lines()
                            .find(|l| l.to_lowercase().starts_with("range:"))
                            .map(|l| l.split(':').nth(1).unwrap_or("").trim());

                        // 3. Prepare Request to Bilibili
                        let mut req_builder = client_clone
                            .get(decoded_url.as_ref())
                            .header(REFERER, decoded_referer);

                        if let Some(range) = range_header {
                            req_builder = req_builder.header(RANGE, range);
                        }

                        // 4. Stream Response back to Socket
                        match req_builder.send().await {
                            Ok(res) => {
                                let status_line = format!("HTTP/1.1 {} OK\r\n", res.status());
                                let _ = socket.write_all(status_line.as_bytes()).await;

                                // Forward Headers
                                let mut headers_str = String::new();
                                headers_str.push_str("Access-Control-Allow-Origin: *\r\n");
                                headers_str.push_str("Connection: close\r\n"); // Keep it simple

                                if let Some(ct) = res.headers().get(CONTENT_TYPE) {
                                    headers_str.push_str(&format!(
                                        "Content-Type: {}\r\n",
                                        ct.to_str().unwrap_or("application/octet-stream")
                                    ));
                                }
                                if let Some(cl) = res.headers().get(CONTENT_LENGTH) {
                                    headers_str.push_str(&format!(
                                        "Content-Length: {}\r\n",
                                        cl.to_str().unwrap_or("0")
                                    ));
                                }
                                if let Some(cr) = res.headers().get(CONTENT_RANGE) {
                                    headers_str.push_str(&format!(
                                        "Content-Range: {}\r\n",
                                        cr.to_str().unwrap_or("")
                                    ));
                                }
                                headers_str.push_str("Accept-Ranges: bytes\r\n\r\n");

                                let _ = socket.write_all(headers_str.as_bytes()).await;

                                // Pipe Body
                                let mut stream = res.bytes_stream();
                                while let Some(chunk_result) = stream.next().await {
                                    if let Ok(chunk) = chunk_result {
                                        if let Err(_) = socket.write_all(&chunk).await {
                                            break; // Client closed connection
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                let _ = socket
                                    .write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n")
                                    .await;
                            }
                        }
                    }
                }
            });
        }
    }
}
