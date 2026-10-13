use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::error::AppResult;
use crate::services::sync::{self, SyncStatus};

#[tauri::command]
pub fn get_sync_status(app: AppHandle) -> SyncStatus {
    sync::status(&app)
}

/// Opens a folder picker and turns sync on with the chosen folder.
/// Resolves to `None` when cancelled. Async: the dialog blocks.
#[tauri::command]
pub async fn choose_sync_folder(app: AppHandle) -> AppResult<Option<SyncStatus>> {
    let Some(path) = app
        .dialog()
        .file()
        .set_title("選擇同步資料夾（例如 Google 雲端硬碟或 OneDrive 裡的資料夾）")
        .blocking_pick_folder()
    else {
        return Ok(None);
    };
    let path = path
        .into_path()
        .map_err(|err| crate::error::AppError::Internal(err.to_string()))?;
    sync::set_folder(&app, &path.to_string_lossy()).map(Some)
}

/// Turns sync off. The folder and its files are left as they are.
#[tauri::command]
pub fn disable_sync(app: AppHandle) -> AppResult<SyncStatus> {
    sync::set_folder(&app, "")
}

/// Starts a run in the background; progress arrives as `sync:status` events.
#[tauri::command]
pub fn sync_now(app: AppHandle) -> SyncStatus {
    sync::request_now(&app);
    sync::status(&app)
}

/// The folder went missing: recreate it and upload this device's notes.
#[tauri::command]
pub fn recreate_sync_folder(app: AppHandle) -> AppResult<SyncStatus> {
    sync::recreate_folder(&app)
}
