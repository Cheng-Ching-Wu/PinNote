pub mod backup;
pub mod migrations;
pub mod note_repository;
pub mod settings_repository;

use std::path::Path;
use std::sync::Mutex;

use chrono::{SecondsFormat, Utc};
use rusqlite::Connection;

use crate::error::{AppError, AppResult};

/// SQLite connection shared by all commands.
///
/// Never hold the lock while creating/closing windows: window events handled
/// on the main thread also need the database.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> AppResult<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        Self::init(Connection::open(path)?)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> AppResult<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> AppResult<Self> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )?;
        migrations::run(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn with_conn<T>(&self, f: impl FnOnce(&mut Connection) -> AppResult<T>) -> AppResult<T> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| AppError::Internal("database lock poisoned".into()))?;
        f(&mut conn)
    }
}

pub fn now() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}
