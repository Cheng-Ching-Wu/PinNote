use tauri::AppHandle;

use crate::error::AppResult;
use crate::models::{Note, NoteRevision, NoteStatus, NoteUpdate};
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

#[tauri::command]
pub fn rename_tag(app: AppHandle, from: String, to: String) -> AppResult<()> {
    notes::rename_tag(&app, &from, &to)
}

#[tauri::command]
pub fn delete_tag(app: AppHandle, name: String) -> AppResult<()> {
    notes::delete_tag(&app, &name)
}

/// Ids of one list in display order (manual sorting).
#[tauri::command]
pub fn reorder_notes(app: AppHandle, ids: Vec<String>) -> AppResult<()> {
    notes::reorder(&app, &ids)
}

/// Returns how many notes were permanently deleted.
#[tauri::command]
pub fn empty_trash(app: AppHandle) -> AppResult<usize> {
    notes::empty_trash(&app)
}

/// Earlier versions of a note, newest first.
#[tauri::command]
pub fn get_note_revisions(app: AppHandle, id: String) -> AppResult<Vec<NoteRevision>> {
    notes::list_revisions(&app, &id)
}

#[tauri::command]
pub fn restore_note_revision(app: AppHandle, id: String, revision_id: i64) -> AppResult<Note> {
    notes::restore_revision(&app, &id, revision_id)
}

#[tauri::command]
pub fn add_note_revision(
    app: AppHandle,
    id: String,
    title: String,
    content: String,
) -> AppResult<()> {
    notes::add_revision(&app, &id, &title, &content)
}
