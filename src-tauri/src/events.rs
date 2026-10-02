//! Events broadcast to every webview so each window's store stays in sync.

use tauri::{AppHandle, Emitter};

use crate::models::Note;

pub const NOTE_CHANGED: &str = "notes:changed";
pub const NOTE_REMOVED: &str = "notes:removed";

pub fn note_changed(app: &AppHandle, note: &Note) {
    if let Err(err) = app.emit(NOTE_CHANGED, note) {
        eprintln!("[pinnote] failed to emit {NOTE_CHANGED}: {err}");
    }
}

pub fn note_removed(app: &AppHandle, id: &str) {
    if let Err(err) = app.emit(NOTE_REMOVED, id) {
        eprintln!("[pinnote] failed to emit {NOTE_REMOVED}: {err}");
    }
}
