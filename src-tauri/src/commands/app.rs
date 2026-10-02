use tauri::{AppHandle, State};
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

use crate::error::{AppError, AppResult};
use crate::services::transfer::{self, TransferResult};
use crate::state::{AppState, ShortcutStatus};

#[tauri::command]
pub fn get_shortcut_status(state: State<'_, AppState>) -> Option<ShortcutStatus> {
    state.shortcut.lock().ok().map(|status| status.clone())
}

#[tauri::command]
pub fn get_autostart(app: AppHandle) -> AppResult<bool> {
    app.autolaunch()
        .is_enabled()
        .map_err(|err| AppError::Internal(err.to_string()))
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> AppResult<bool> {
    let autolaunch = app.autolaunch();
    let result = if enabled {
        autolaunch.enable()
    } else {
        autolaunch.disable()
    };
    result.map_err(|err| AppError::Internal(err.to_string()))?;
    autolaunch
        .is_enabled()
        .map_err(|err| AppError::Internal(err.to_string()))
}

/// Async: the file dialog blocks until the user picks a file.
#[tauri::command]
pub async fn export_notes(app: AppHandle) -> AppResult<Option<TransferResult>> {
    transfer::export(&app)
}

#[tauri::command]
pub async fn import_notes(app: AppHandle) -> AppResult<Option<TransferResult>> {
    transfer::import(&app)
}

/// Opens a link from a note in the default browser. Only web / mail links are
/// allowed so note content can't launch arbitrary programs or files.
#[tauri::command]
pub fn open_external(app: AppHandle, url: String) -> AppResult<()> {
    const ALLOWED: [&str; 3] = ["http://", "https://", "mailto:"];
    if !ALLOWED.iter().any(|scheme| url.starts_with(scheme)) {
        return Err(AppError::Internal(format!("不支援的連結: {url}")));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|err| AppError::Internal(err.to_string()))
}

#[tauri::command]
pub fn open_backup_folder(app: AppHandle) -> AppResult<()> {
    transfer::open_backup_dir(&app)
}
