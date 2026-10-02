//! JSON export / import and the backup folder.

use std::path::PathBuf;

use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tauri_plugin_opener::OpenerExt;

use crate::database::{note_repository as repo, now, Database};
use crate::error::{AppError, AppResult};
use crate::events;
use crate::models::Note;
use crate::services::{notes, window};

const EXPORT_FORMAT_VERSION: u32 = 1;
const BACKUP_DIR: &str = "backups";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportFile {
    app: String,
    format_version: u32,
    exported_at: String,
    notes: Vec<Note>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    pub path: String,
    /// Notes written (export) or added (import).
    pub count: usize,
    /// Import only: notes skipped because their id already exists.
    pub skipped: usize,
}

pub fn backup_dir(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(app.path().app_data_dir()?.join(BACKUP_DIR))
}

pub fn open_backup_dir(app: &AppHandle) -> AppResult<()> {
    let dir = backup_dir(app)?;
    std::fs::create_dir_all(&dir)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(|err| AppError::Internal(err.to_string()))
}

/// Exports every note (all statuses). Returns `None` when the dialog is cancelled.
pub fn export(app: &AppHandle) -> AppResult<Option<TransferResult>> {
    let file_name = format!("pinnote-export-{}.json", Local::now().format("%Y%m%d"));
    let Some(path) = file_dialog(app, "匯出便籤")
        .set_file_name(file_name)
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path = path
        .into_path()
        .map_err(|err| AppError::Internal(err.to_string()))?;

    let file = ExportFile {
        app: "PinNote".into(),
        format_version: EXPORT_FORMAT_VERSION,
        exported_at: now(),
        notes: notes::list(app)?,
    };
    std::fs::write(&path, serde_json::to_vec_pretty(&file)?)?;

    Ok(Some(TransferResult {
        path: path.to_string_lossy().into_owned(),
        count: file.notes.len(),
        skipped: 0,
    }))
}

/// Imports notes from an export file. Notes whose id already exists are left
/// untouched, so importing the same file twice is harmless. Imported notes are
/// not opened on the desktop automatically.
pub fn import(app: &AppHandle) -> AppResult<Option<TransferResult>> {
    let Some(path) = file_dialog(app, "匯入便籤").blocking_pick_file() else {
        return Ok(None);
    };
    let path = path
        .into_path()
        .map_err(|err| AppError::Internal(err.to_string()))?;

    let file: ExportFile = serde_json::from_slice(&std::fs::read(&path)?)?;
    if file.format_version > EXPORT_FORMAT_VERSION {
        return Err(AppError::Internal(format!(
            "此檔案由較新版本匯出（格式 v{}），請先更新 PinNote",
            file.format_version
        )));
    }

    let imported = app.state::<Database>().with_conn(|conn| {
        let tx = conn.transaction()?;
        let mut imported = Vec::new();
        for mut note in file.notes.iter().cloned() {
            note.is_open = false;
            if repo::insert_if_absent(&tx, &note)? {
                imported.push(note);
            }
        }
        tx.commit()?;
        Ok(imported)
    })?;

    for note in &imported {
        events::note_changed(app, note);
    }

    Ok(Some(TransferResult {
        path: path.to_string_lossy().into_owned(),
        count: imported.len(),
        skipped: file.notes.len() - imported.len(),
    }))
}

fn file_dialog(app: &AppHandle, title: &str) -> FileDialogBuilder<tauri::Wry> {
    let mut dialog = app
        .dialog()
        .file()
        .set_title(title)
        .add_filter("PinNote JSON", &["json"]);
    if let Some(main) = app.get_webview_window(window::MAIN_LABEL) {
        dialog = dialog.set_parent(&main);
    }
    dialog
}
