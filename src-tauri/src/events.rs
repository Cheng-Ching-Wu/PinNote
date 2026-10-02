//! Events broadcast to every webview so each window's store stays in sync.

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::models::{AppSettings, Note};

pub const NOTE_CHANGED: &str = "notes:changed";
pub const NOTE_REMOVED: &str = "notes:removed";
pub const SETTINGS_CHANGED: &str = "settings:changed";

fn emit<T: Serialize + Clone>(app: &AppHandle, event: &str, payload: T) {
    if let Err(err) = app.emit(event, payload) {
        eprintln!("[pinnote] failed to emit {event}: {err}");
    }
}

pub fn note_changed(app: &AppHandle, note: &Note) {
    emit(app, NOTE_CHANGED, note);
}

pub fn note_removed(app: &AppHandle, id: &str) {
    emit(app, NOTE_REMOVED, id);
}

pub fn settings_changed(app: &AppHandle, settings: &AppSettings) {
    emit(app, SETTINGS_CHANGED, settings);
}
