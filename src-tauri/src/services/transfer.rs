//! JSON export / import and the backup folder.

use std::path::PathBuf;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};
use tauri_plugin_opener::OpenerExt;

use crate::database::{note_repository as repo, now, Database};
use crate::error::{AppError, AppResult};
use crate::events;
use crate::image_refs;
use crate::models::Note;
use crate::services::{images, notes, window};

/// v1: notes only. v2: notes + embedded images. v3: notes carry `tags`.
const EXPORT_FORMAT_VERSION: u32 = 3;
const BACKUP_DIR: &str = "backups";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExportFile {
    app: String,
    format_version: u32,
    exported_at: String,
    notes: Vec<Note>,
    /// Images referenced by the notes, so one file restores everything.
    #[serde(default)]
    images: Vec<ExportImage>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportImage {
    /// File name inside the images folder (notes reference `images/<name>`).
    name: String,
    /// Base64-encoded file content.
    data: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferResult {
    pub path: String,
    /// Notes written (export) or added (import).
    pub count: usize,
    /// Import only: notes skipped because their id already exists.
    pub skipped: usize,
    /// Images written (export) or added (import).
    pub images: usize,
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

/// Exports every note (all statuses) and the images they use.
/// Returns `None` when the dialog is cancelled.
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

    let notes = notes::list(app)?;
    let images_dir = images::images_dir(app)?;
    let mut names: Vec<String> =
        image_refs::referenced_by_all(notes.iter().map(|note| note.content.as_str()))
            .into_iter()
            .collect();
    names.sort();
    let images = names
        .into_iter()
        // A referenced file may already be gone (e.g. removed by hand); skip it.
        .filter_map(|name| {
            let bytes = std::fs::read(images_dir.join(&name)).ok()?;
            Some(ExportImage {
                name,
                data: BASE64.encode(bytes),
            })
        })
        .collect::<Vec<_>>();

    let file = ExportFile {
        app: "PinNote".into(),
        format_version: EXPORT_FORMAT_VERSION,
        exported_at: now(),
        notes,
        images,
    };
    std::fs::write(&path, serde_json::to_vec_pretty(&file)?)?;

    Ok(Some(TransferResult {
        path: path.to_string_lossy().into_owned(),
        count: file.notes.len(),
        skipped: 0,
        images: file.images.len(),
    }))
}

/// Imports notes (and their images) from an export file. Notes whose id
/// already exists are left untouched, so importing the same file twice is
/// harmless. Imported notes are not opened on the desktop automatically.
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

    // Images first, so imported notes never point at a missing file.
    let images_dir = images::images_dir(app)?;
    let mut images_added = 0;
    for image in &file.images {
        let written = BASE64
            .decode(&image.data)
            .map_err(|err| AppError::Internal(err.to_string()))
            .and_then(|bytes| images::write_named(&images_dir, &image.name, &bytes));
        match written {
            Ok(true) => images_added += 1,
            Ok(false) => {}
            Err(err) => eprintln!("[pinnote] skipped image {}: {err}", image.name),
        }
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
        images: images_added,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_files_without_images_still_parse() {
        let v1 = r#"{"app":"PinNote","formatVersion":1,"exportedAt":"2026-10-01T00:00:00.000Z","notes":[]}"#;
        let file: ExportFile = serde_json::from_str(v1).unwrap();
        assert!(file.images.is_empty());
    }

    #[test]
    fn images_round_trip_through_base64() {
        let png = [0x89, b'P', b'N', b'G', 1, 2, 3];
        let image = ExportImage {
            name: "abc.png".into(),
            data: BASE64.encode(png),
        };
        let json = serde_json::to_string(&image).unwrap();
        let back: ExportImage = serde_json::from_str(&json).unwrap();
        assert_eq!(BASE64.decode(back.data).unwrap(), png);
    }
}
