use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use tauri_plugin_opener::OpenerExt;

use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, AppSettingsPatch};
use crate::services::transfer::{self, TransferResult};
use crate::services::{images, settings, shortcut};
use crate::state::ShortcutStatus;
use crate::tr;

#[tauri::command]
pub fn get_shortcut_status(app: AppHandle) -> Vec<ShortcutStatus> {
    shortcut::statuses(&app)
}

/// `accelerator` "" disables the action. Returns all statuses after the change.
#[tauri::command]
pub fn set_shortcut(
    app: AppHandle,
    action: String,
    accelerator: String,
) -> AppResult<Vec<ShortcutStatus>> {
    shortcut::set(&app, &action, &accelerator)
}

/// Paused while the user records a new shortcut; resuming re-registers all.
#[tauri::command]
pub fn set_shortcuts_paused(app: AppHandle, paused: bool) -> AppResult<Vec<ShortcutStatus>> {
    if paused {
        shortcut::pause(&app)?;
    } else {
        shortcut::register_all(&app);
    }
    Ok(shortcut::statuses(&app))
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> AppResult<AppSettings> {
    settings::get(&app)
}

#[tauri::command]
pub fn update_settings(app: AppHandle, data: AppSettingsPatch) -> AppResult<AppSettings> {
    settings::update(&app, &data)
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
        return Err(AppError::Internal(tr!(
            "不支援的連結: {url}",
            "Unsupported link: {url}"
        )));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|err| AppError::Internal(err.to_string()))
}

/// Absolute path of the image folder; the frontend turns `images/<file>`
/// into an asset-protocol URL with it.
#[tauri::command]
pub fn get_images_dir(app: AppHandle) -> AppResult<String> {
    Ok(images::images_dir(&app)?.to_string_lossy().into_owned())
}

/// Raw binary body (no JSON array encoding); the extension comes in the
/// `x-image-ext` header. Returns the Markdown path, e.g. `images/<uuid>.png`.
#[tauri::command]
pub fn save_image(app: AppHandle, request: tauri::ipc::Request<'_>) -> AppResult<String> {
    let tauri::ipc::InvokeBody::Raw(bytes) = request.body() else {
        return Err(AppError::Internal("expected raw image bytes".into()));
    };
    let extension = request
        .headers()
        .get("x-image-ext")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("png");
    images::save(&app, bytes, extension)
}

/// Settings → 清理未使用的圖片. Same grace period as the automatic cleanup,
/// so images just pasted (not saved into a note yet) are never removed.
#[tauri::command]
pub fn cleanup_images(app: AppHandle) -> AppResult<images::CleanupResult> {
    images::cleanup(&app, images::ORPHAN_GRACE)
}

#[tauri::command]
pub fn open_backup_folder(app: AppHandle) -> AppResult<()> {
    transfer::open_backup_dir(&app)
}
