use tauri::AppHandle;

use crate::error::AppResult;
use crate::models::{Note, NoteStatus, NoteUpdate};
use crate::services::notes;

#[tauri::command]
pub fn get_notes(app: AppHandle) -> AppResult<Vec<Note>> {
    notes::list(&app)
}

#[tauri::command]
pub fn get_note(app: AppHandle, id: String) -> AppResult<Note> {
    notes::get(&app, &id)
}

#[tauri::command]
pub fn create_note(app: AppHandle) -> AppResult<Note> {
    notes::create(&app)
}

#[tauri::command]
pub fn update_note(app: AppHandle, id: String, data: NoteUpdate) -> AppResult<Note> {
    notes::update(&app, &id, &data)
}

#[tauri::command]
pub fn archive_note(app: AppHandle, id: String) -> AppResult<Note> {
    notes::set_status(&app, &id, NoteStatus::Archived)
}

#[tauri::command]
pub fn restore_note(app: AppHandle, id: String) -> AppResult<Note> {
    notes::set_status(&app, &id, NoteStatus::Active)
}

#[tauri::command]
pub fn soft_delete_note(app: AppHandle, id: String) -> AppResult<Note> {
    notes::set_status(&app, &id, NoteStatus::Deleted)
}

#[tauri::command]
pub fn restore_deleted_note(app: AppHandle, id: String) -> AppResult<Note> {
    notes::set_status(&app, &id, NoteStatus::Active)
}

#[tauri::command]
pub fn delete_note_permanently(app: AppHandle, id: String) -> AppResult<()> {
    notes::delete_permanently(&app, &id)
}
