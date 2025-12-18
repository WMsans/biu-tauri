use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Shell error: {0}")]
    ShellError(#[from] tauri_plugin_shell::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
