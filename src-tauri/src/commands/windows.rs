//! Window commands are `async` on purpose: creating a webview from a
//! synchronous command (which runs on the main thread) deadlocks on Windows.

use tauri::AppHandle;

use crate::error::{AppError, AppResult};
use crate::models::NoteStatus;
use crate::services::{notes, window};

#[tauri::command]
pub async fn open_note_window(app: AppHandle, id: String) -> AppResult<()> {
    let note = notes::get(&app, &id)?;
    if note.status != NoteStatus::Active {
        return Err(AppError::Internal(format!(
            "note {id} is {} and cannot be opened",
            note.status.as_str()
        )));
    }
    let note = if note.is_open {
        note
    } else {
        notes::set_open(&app, &id, true)?
    };
    window::open_note_window(&app, &note, true)?;
    Ok(())
}

/// Closes the note's window. The close handler records `is_open = false`;
/// if the window doesn't exist we record it directly.
#[tauri::command]
pub async fn close_note_window(app: AppHandle, id: String) -> AppResult<()> {
    if !window::close_note_window(&app, &id)? {
        notes::set_open(&app, &id, false)?;
    }
    Ok(())
}

/// Reopens the window of every active note. Returns the number of notes shown.
#[tauri::command]
pub async fn show_all_notes(app: AppHandle) -> AppResult<usize> {
    notes::show_all(&app)
}

/// Hides / shows every note window without closing it. Returns `true` when now hidden.
#[tauri::command]
pub async fn toggle_notes_hidden(app: AppHandle) -> AppResult<bool> {
    window::toggle_notes_hidden(&app)
}

#[tauri::command]
pub async fn show_main_window(app: AppHandle) -> AppResult<()> {
    window::show_main_window(&app)
}
