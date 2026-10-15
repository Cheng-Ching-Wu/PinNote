//! In-app updates from GitHub Releases (`tauri-plugin-updater`).
//!
//! The release workflow attaches a signed `latest.json`; the endpoint in
//! `tauri.conf.json` points at the latest *published* release, so drafts are
//! never offered. Checks run in Rust because the app usually lives in the tray
//! with no window open.

use std::sync::Mutex;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};
use tokio::time::sleep;

use crate::error::{AppError, AppResult};
use crate::services::{lifecycle, settings, tray};

/// After launch (also at login), so the check doesn't compete with startup.
const FIRST_CHECK_DELAY: Duration = Duration::from_secs(30);
/// The app stays in the tray for days; look again once a day.
const CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
/// Note windows save pending typing on `NOTES_FLUSH`; give them a moment before the installer closes the app.
const FLUSH_WAIT: Duration = Duration::from_millis(800);

pub const UPDATE_AVAILABLE: &str = "update:available";
pub const UPDATE_PROGRESS: &str = "update:progress";
pub const NOTES_FLUSH: &str = "notes:flush";
/// Tray → "update to vX": the manager window opens the update dialog.
pub const UPDATE_OPEN: &str = "update:open";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    /// Release notes (the release body).
    pub notes: String,
    /// RFC 3339, when the release says.
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub downloaded: u64,
    /// Unknown when the server doesn't send a length.
    pub total: Option<u64>,
}

/// The newest update found so far; kept so "install" doesn't need another check.
#[derive(Default)]
pub struct UpdateState {
    available: Mutex<Option<Update>>,
}

fn info(update: &Update) -> UpdateInfo {
    UpdateInfo {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        notes: update.body.clone().unwrap_or_default(),
        date: update.date.map(|date| date.to_string()),
    }
}

pub fn available(app: &AppHandle) -> Option<UpdateInfo> {
    let state = app.state::<UpdateState>();
    let update = state.available.lock().ok()?;
    update.as_ref().map(info)
}

/// Asks the release server; `None` means this is the latest version.
pub async fn check(app: &AppHandle) -> AppResult<Option<UpdateInfo>> {
    let found = app
        .updater()
        .map_err(|err| AppError::Internal(err.to_string()))?
        .check()
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;
    let result = found.as_ref().map(info);
    if let Ok(mut available) = app.state::<UpdateState>().available.lock() {
        *available = found;
    }
    if let Some(info) = &result {
        if let Err(err) = app.emit(UPDATE_AVAILABLE, info) {
            eprintln!("[pinnote] failed to emit {UPDATE_AVAILABLE}: {err}");
        }
    }
    // The tray shows an "update to vX" item while one is available.
    if let Err(err) = tray::refresh(app) {
        eprintln!("[pinnote] failed to update tray menu: {err}");
    }
    Ok(result)
}

/// Background checks while "automatically check for updates" is on.
pub fn start(app: &AppHandle) {
    // Debug builds have no published release to compare against.
    if cfg!(debug_assertions) {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        sleep(FIRST_CHECK_DELAY).await;
        loop {
            let enabled = settings::get(&app).map_or(true, |s| s.auto_check_updates);
            if enabled {
                // Offline or no release yet: try again next time, quietly.
                if let Err(err) = check(&app).await {
                    eprintln!("[pinnote] update check failed: {err}");
                }
            }
            sleep(CHECK_INTERVAL).await;
        }
    });
}

/// Downloads (reporting `update:progress`), saves every open note, then runs
/// the installer. On Windows the installer closes PinNote and starts the new version.
pub async fn install(app: &AppHandle) -> AppResult<()> {
    let update = app
        .state::<UpdateState>()
        .available
        .lock()
        .ok()
        .and_then(|update| update.clone())
        .ok_or_else(|| AppError::Internal("no update available".into()))?;

    let mut downloaded = 0u64;
    let bytes = update
        .download(
            |chunk, total| {
                downloaded += chunk as u64;
                let _ = app.emit(UPDATE_PROGRESS, UpdateProgress { downloaded, total });
            },
            || {},
        )
        .await
        .map_err(|err| AppError::Internal(err.to_string()))?;

    // Last chance to save: typing still waiting for the debounce, and window geometry.
    let _ = app.emit(NOTES_FLUSH, ());
    sleep(FLUSH_WAIT).await;
    lifecycle::prepare_exit(app);

    update
        .install(bytes)
        .map_err(|err| AppError::Internal(err.to_string()))?;
    // Only reached on macOS / Linux, where the new version needs a relaunch.
    app.restart();
}
