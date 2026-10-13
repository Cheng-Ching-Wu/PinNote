//! Note use-cases shared by commands, the tray menu and the global shortcut.
//! Every mutation is persisted first, then broadcast to all windows.

use chrono::{Duration, Utc};
use tauri::{AppHandle, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;

use crate::database::{note_repository as repo, now, timestamp, Database};
use crate::error::AppResult;
use crate::events;
use crate::image_refs;
use crate::models::note::{normalize_tag, tag_key, DEFAULT_HEIGHT, DEFAULT_WIDTH};
use crate::models::settings::TagColors;
use crate::models::{AppSettingsPatch, Note, NoteRevision, NoteStatus, NoteUpdate};
use crate::services::{images, settings, sync, window};

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
/// notes (in any status, trash included) are kept. The deletion is recorded so
/// sync removes the note on other devices too.
pub fn delete_permanently(app: &AppHandle, id: &str) -> AppResult<()> {
    let note = get(app, id)?;
    db(app).with_conn(|conn| {
        repo::delete(conn, id)?;
        repo::add_tombstone(conn, id, &now())
    })?;
    events::note_removed(app, id);
    cleanup_images_of(app, id, &note.content);
    Ok(())
}

/// After a note is gone: removes the images only it was using.
pub fn cleanup_images_of(app: &AppHandle, id: &str, content: &str) {
    let own_images = image_refs::referenced_files(content);
    if own_images.is_empty() {
        return;
    }
    // The note itself is already gone, so this is "still used elsewhere".
    let result = images::referenced_by_notes(app).and_then(|still_used| {
        images::delete_if_unreferenced(&images::images_dir(app)?, &own_images, &still_used)
    });
    if let Err(err) = result {
        eprintln!("[pinnote] failed to remove images of note {id}: {err}");
    }
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

/// Renames a tag on every note (merging when the new name already exists)
/// and carries its color over.
pub fn rename_tag(app: &AppHandle, from: &str, to: &str) -> AppResult<()> {
    let changed = db(app).with_conn(|conn| repo::rename_tag(conn, from, to))?;
    for note in &changed {
        events::note_changed(app, note);
    }

    // `repo::rename_tag` already rejected names that normalise to nothing.
    let to_key = normalize_tag(to)
        .map(|name| tag_key(&name))
        .unwrap_or_default();
    let from_key = tag_key(from);
    if from_key != to_key {
        let mut colors = settings::get(app)?.tag_colors;
        if let Some(color) = colors.remove(&from_key) {
            colors.entry(to_key).or_insert(color);
            update_tag_colors(app, colors)?;
        }
    }
    Ok(())
}

/// Removes a tag from every note and forgets its color. Notes themselves are kept.
pub fn delete_tag(app: &AppHandle, name: &str) -> AppResult<()> {
    let changed = db(app).with_conn(|conn| repo::delete_tag(conn, name))?;
    for note in &changed {
        events::note_changed(app, note);
    }

    let mut colors = settings::get(app)?.tag_colors;
    if colors.remove(&tag_key(name)).is_some() {
        update_tag_colors(app, colors)?;
    }
    Ok(())
}

fn update_tag_colors(app: &AppHandle, colors: TagColors) -> AppResult<()> {
    let patch = AppSettingsPatch {
        tag_colors: Some(colors),
        ..Default::default()
    };
    settings::update(app, &patch).map(|_| ())
}

/// Saves the manual order of the manager list (ids in display order).
/// Not broadcast as note changes: only the manager shows this order and it
/// updates itself; other windows pick it up on their next load.
pub fn reorder(app: &AppHandle, ids: &[String]) -> AppResult<()> {
    db(app).with_conn(|conn| repo::reorder(conn, ids))?;
    sync::mark_dirty(app);
    Ok(())
}

/// Keeps a text as a revision, e.g. another device's edit that arrived while this
/// window still had unsaved typing.
pub fn add_revision(app: &AppHandle, id: &str, title: &str, content: &str) -> AppResult<()> {
    db(app).with_conn(|conn| repo::insert_revision(conn, id, title, content))
}

/// Permanently deletes trashed notes older than the retention setting.
/// Returns how many were removed (0 when the retention is "never").
pub fn purge_expired_trash(app: &AppHandle) -> AppResult<usize> {
    let days = settings::get(app)?.trash_retention_days;
    if days == 0 {
        return Ok(0);
    }
    let cutoff = timestamp(Utc::now() - Duration::days(days.into()));
    let expired = db(app).with_conn(|conn| repo::list_deleted_before(conn, &cutoff))?;
    for note in &expired {
        delete_permanently(app, &note.id)?;
    }
    Ok(expired.len())
}

/// "清空垃圾桶": permanently deletes every trashed note. Returns how many.
pub fn empty_trash(app: &AppHandle) -> AppResult<usize> {
    let trashed: Vec<Note> = list(app)?
        .into_iter()
        .filter(|note| note.status == NoteStatus::Deleted)
        .collect();
    for note in &trashed {
        delete_permanently(app, &note.id)?;
    }
    Ok(trashed.len())
}

pub fn list_revisions(app: &AppHandle, id: &str) -> AppResult<Vec<NoteRevision>> {
    db(app).with_conn(|conn| repo::list_revisions(conn, id))
}

pub fn restore_revision(app: &AppHandle, id: &str, revision_id: i64) -> AppResult<Note> {
    let note = db(app).with_conn(|conn| repo::restore_revision(conn, id, revision_id))?;
    window::sync_title(app, &note);
    events::note_changed(app, &note);
    Ok(note)
}

/// Desktop workspace: opens exactly the given active notes and closes every other
/// note window (closed notes can be brought back with "顯示全部便籤").
/// Returns how many notes are shown.
pub fn show_only(app: &AppHandle, ids: &[String]) -> AppResult<usize> {
    window::set_notes_hidden(app, false)?;
    let keep: std::collections::HashSet<&str> = ids.iter().map(String::as_str).collect();
    let mut shown = 0;
    for note in list(app)?
        .into_iter()
        .filter(|note| note.status == NoteStatus::Active)
    {
        if keep.contains(note.id.as_str()) {
            let note = if note.is_open {
                note
            } else {
                set_open(app, &note.id, true)?
            };
            window::open_note_window(app, &note, false)?;
            shown += 1;
        } else if note.is_open && !window::close_note_window(app, &note.id)? {
            // No window to close (e.g. never restored): record it directly.
            set_open(app, &note.id, false)?;
        }
    }
    Ok(shown)
}
