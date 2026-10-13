//! Events broadcast to every webview so each window's store stays in sync.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::models::{AppSettings, Note};
use crate::services::sync::{self, SyncStatus};

pub const NOTE_CHANGED: &str = "notes:changed";
pub const NOTE_REMOVED: &str = "notes:removed";
pub const SETTINGS_CHANGED: &str = "settings:changed";
/// Ids of notes whose text another device changed: open note windows reload it.
pub const NOTES_SYNCED: &str = "notes:synced";
pub const SYNC_STATUS: &str = "sync:status";

fn emit<T: Serialize + Clone>(app: &AppHandle, event: &str, payload: T) {
    if let Err(err) = app.emit(event, payload) {
        eprintln!("[pinnote] failed to emit {event}: {err}");
    }
}

// Every local change also schedules a sync (ignored while sync applies its own).

pub fn note_changed(app: &AppHandle, note: &Note) {
    emit(app, NOTE_CHANGED, note);
    sync::mark_dirty(app);
}

pub fn note_removed(app: &AppHandle, id: &str) {
    emit(app, NOTE_REMOVED, id);
    sync::mark_dirty(app);
}

pub fn settings_changed(app: &AppHandle, settings: &AppSettings) {
    emit(app, SETTINGS_CHANGED, settings);
    sync::mark_dirty(app);
}

pub fn notes_synced(app: &AppHandle, ids: &[String]) {
    emit(app, NOTES_SYNCED, ids);
}

pub fn sync_status(app: &AppHandle, status: &SyncStatus) {
    emit(app, SYNC_STATUS, status);
}
