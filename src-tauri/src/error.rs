use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("window error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid file: {0}")]
    Json(#[from] serde_json::Error),

    #[error("note not found: {0}")]
    NoteNotFound(String),

    #[error("{0}")]
    Internal(String),
}

/// Commands return errors to the frontend as plain strings.
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
