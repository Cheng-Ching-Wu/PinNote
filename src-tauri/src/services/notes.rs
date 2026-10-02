//! Note use-cases shared by commands, the tray menu and the global shortcut.
//! Every mutation is persisted first, then broadcast to all windows.

use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::database::{note_repository as repo, Database};
use crate::error::AppResult;
use crate::events;
use crate::image_refs;
use crate::models::note::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
use crate::models::{Note, NoteStatus, NoteUpdate};
use crate::services::{images, window};

fn db(app: &AppHandle) -> tauri::State<'_, Database> {
    app.state::<Database>()
}

pub fn list(app: &AppHandle) -> AppResult<Vec<Note>> {
    db(app).with_conn(|conn| repo::list(conn))
}

pub fn get(app: &AppHandle, id: &str) -> AppResult<Note> {
    db(app).with_conn(|conn| repo::get(conn, id))
}

pub fn create(app: &AppHandle) -> AppResult<Note> {
    let open_count = db(app).with_conn(|conn| repo::count_open(conn))?;
    let position = cascade_position(app, open_count);
    let note = db(app).with_conn(|conn| repo::insert(conn, position))?;
    events::note_changed(app, &note);
    Ok(note)
}

/// Used by the tray menu and the global shortcut.
pub fn create_and_open(app: &AppHandle) -> AppResult<Note> {
    let note = create(app)?;
    window::open_note_window(app, &note, true)?;
    Ok(note)
}

/// Ctrl+Alt+V: a new note holding the clipboard text. A clipboard without
/// text (e.g. an image) just gives an empty note.
pub fn create_from_clipboard(app: &AppHandle) -> AppResult<Note> {
    let text = app
        .clipboard()
        .read_text()
        .unwrap_or_default()
        .replace("\r\n", "\n");
    let text = text.trim_end();

    let mut note = create(app)?;
    if !text.is_empty() {
        let patch = NoteUpdate {
            content: Some(text.to_string()),
            ..Default::default()
        };
        note = update(app, &note.id, &patch)?;
    }
    window::open_note_window(app, &note, true)?;
    Ok(note)
}

pub fn update(app: &AppHandle, id: &str, patch: &NoteUpdate) -> AppResult<Note> {
    let note = db(app).with_conn(|conn| repo::update(conn, id, patch))?;
    if patch.title.is_some() {
        window::sync_title(app, &note);
    }
    events::note_changed(app, &note);
    Ok(note)
}

pub fn set_status(app: &AppHandle, id: &str, status: NoteStatus) -> AppResult<Note> {
    let note = db(app).with_conn(|conn| repo::set_status(conn, id, status))?;
    events::note_changed(app, &note);
    Ok(note)
}

pub fn set_open(app: &AppHandle, id: &str, is_open: bool) -> AppResult<Note> {
    let note = db(app).with_conn(|conn| repo::set_open(conn, id, is_open))?;
    events::note_changed(app, &note);
    Ok(note)
}

/// Puts every active note back on the desktop (also undoing Ctrl+Alt+H).
/// Returns how many notes are shown.
pub fn show_all(app: &AppHandle) -> AppResult<usize> {
    window::set_notes_hidden(app, false)?;
    let active: Vec<Note> = list(app)?
        .into_iter()
        .filter(|note| note.status == NoteStatus::Active)
        .collect();
    for note in &active {
        let note = if note.is_open {
            note.clone()
        } else {
            set_open(app, &note.id, true)?
        };
        window::open_note_window(app, &note, false)?;
    }
    Ok(active.len())
}

/// Deletes the note and the images only it was using. Images shared with other
/// notes (in any status, trash included) are kept.
pub fn delete_permanently(app: &AppHandle, id: &str) -> AppResult<()> {
    let note = get(app, id)?;
    db(app).with_conn(|conn| repo::delete(conn, id))?;
    events::note_removed(app, id);

    let own_images = image_refs::referenced_files(&note.content);
    if !own_images.is_empty() {
        // The note itself is already gone, so this is "still used elsewhere".
        let result = images::referenced_by_notes(app).and_then(|still_used| {
            images::delete_if_unreferenced(&images::images_dir(app)?, &own_images, &still_used)
        });
        if let Err(err) = result {
            eprintln!("[pinnote] failed to remove images of note {id}: {err}");
        }
    }
    Ok(())
}

/// Places new notes around the centre of the primary monitor, stepping
/// diagonally so consecutive notes don't stack exactly on top of each other.
fn cascade_position(app: &AppHandle, open_count: i64) -> Option<(i64, i64)> {
    let monitor = app.primary_monitor().ok().flatten()?;
    let scale = monitor.scale_factor();
    let origin = monitor.position();
    let size = monitor.size();

    let width = (DEFAULT_WIDTH as f64 * scale) as i64;
    let height = (DEFAULT_HEIGHT as f64 * scale) as i64;
    let step = (28.0 * scale) as i64;
    let offset = step * (open_count % 8 - 3);

    let x = origin.x as i64 + (size.width as i64 - width) / 2 + offset;
    let y = origin.y as i64 + (size.height as i64 - height) / 2 + offset;
    Some((x, y))
}
