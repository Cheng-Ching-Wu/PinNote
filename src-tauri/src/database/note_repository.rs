use chrono::{Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use super::{now, timestamp};
use crate::error::{AppError, AppResult};
use crate::models::note::{normalize_tag, normalize_tags, tag_key, DEFAULT_HEIGHT, DEFAULT_WIDTH};
use std::collections::HashMap;

use crate::models::{Note, NoteColor, NoteRevision, NoteStatus, NoteUpdate, SyncedNote};

/// While a note is being edited, its previous text is saved at most this often.
pub const REVISION_INTERVAL_MINUTES: i64 = 10;
/// Older revisions beyond this count are dropped.
pub const MAX_REVISIONS_PER_NOTE: i64 = 50;

const COLUMNS: &str = "id, title, content, color, tags, x, y, width, height, always_on_top, \
                       collapsed, is_open, status, sort_order, created_at, updated_at, deleted_at, \
                       modified_at";

/// Unreadable JSON (e.g. edited by hand) is treated as "no tags" instead of failing the whole list.
fn parse_tags(json: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(json)
        .map(|tags| normalize_tags(&tags))
        .unwrap_or_default()
}

fn tags_json(tags: &[String]) -> String {
    serde_json::to_string(tags).unwrap_or_else(|_| "[]".into())
}

fn map_row(row: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get("id")?,
        title: row.get("title")?,
        content: row.get("content")?,
        color: NoteColor::parse(&row.get::<_, String>("color")?),
        tags: parse_tags(&row.get::<_, String>("tags")?),
        x: row.get("x")?,
        y: row.get("y")?,
        width: row.get("width")?,
        height: row.get("height")?,
        always_on_top: row.get("always_on_top")?,
        collapsed: row.get("collapsed")?,
        is_open: row.get("is_open")?,
        status: NoteStatus::parse(&row.get::<_, String>("status")?),
        sort_order: row.get("sort_order")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        deleted_at: row.get("deleted_at")?,
        modified_at: row.get("modified_at")?,
    })
}

pub fn list(conn: &Connection) -> AppResult<Vec<Note>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM notes ORDER BY updated_at DESC"
    ))?;
    let notes = stmt
        .query_map([], map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

/// Active notes whose window should be on the desktop.
pub fn list_open(conn: &Connection) -> AppResult<Vec<Note>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM notes WHERE status = 'active' AND is_open = 1 ORDER BY created_at ASC"
    ))?;
    let notes = stmt
        .query_map([], map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

pub fn find(conn: &Connection, id: &str) -> AppResult<Option<Note>> {
    let note = conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM notes WHERE id = ?1"),
            [id],
            map_row,
        )
        .optional()?;
    Ok(note)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Note> {
    find(conn, id)?.ok_or_else(|| AppError::NoteNotFound(id.to_string()))
}

pub fn insert(conn: &Connection, position: Option<(i64, i64)>) -> AppResult<Note> {
    let id = Uuid::new_v4().to_string();
    let timestamp = now();
    let (x, y) = position.unzip();
    conn.execute(
        "INSERT INTO notes (id, color, x, y, width, height, always_on_top, collapsed, is_open, status,
                            sort_order, created_at, updated_at, modified_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, 1, 'active',
                 (SELECT COALESCE(MIN(sort_order), 0) - 1 FROM notes), ?7, ?7, ?7)",
        params![
            id,
            NoteColor::Yellow.as_str(),
            x,
            y,
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT,
            timestamp
        ],
    )?;
    get(conn, &id)
}

/// Inserts a complete note (e.g. from an import), keeping its id and timestamps.
/// Returns false without changes when a note with that id already exists.
pub fn insert_if_absent(conn: &Connection, note: &Note) -> AppResult<bool> {
    let inserted = conn.execute(
        "INSERT OR IGNORE INTO notes (id, created_at, updated_at) VALUES (?1, ?2, ?3)",
        params![note.id, note.created_at, note.updated_at],
    )?;
    if inserted == 0 {
        return Ok(false);
    }
    save(conn, &mut note.clone())?;
    Ok(true)
}

/// Writes the note. `modified_at` moves to now whenever a synced field changed,
/// so every kind of edit (text, color, tags, archive, trash…) reaches other devices.
fn save(conn: &Connection, note: &mut Note) -> AppResult<()> {
    let changed = find(conn, &note.id)?
        .is_none_or(|stored| !SyncedNote::from(&stored).same_data(&SyncedNote::from(&*note)));
    if changed {
        note.modified_at = now();
    }
    write_row(conn, note)
}

fn write_row(conn: &Connection, note: &Note) -> AppResult<()> {
    conn.execute(
        "UPDATE notes SET
            title = ?2, content = ?3, color = ?4, x = ?5, y = ?6, width = ?7, height = ?8,
            always_on_top = ?9, collapsed = ?10, is_open = ?11, status = ?12,
            updated_at = ?13, deleted_at = ?14, tags = ?15, sort_order = ?16, modified_at = ?17
         WHERE id = ?1",
        params![
            note.id,
            note.title,
            note.content,
            note.color.as_str(),
            note.x,
            note.y,
            note.width,
            note.height,
            note.always_on_top,
            note.collapsed,
            note.is_open,
            note.status.as_str(),
            note.updated_at,
            note.deleted_at,
            tags_json(&note.tags),
            note.sort_order,
            note.modified_at,
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, id: &str, patch: &NoteUpdate) -> AppResult<Note> {
    let mut note = get(conn, id)?;
    let before = (note.title.clone(), note.content.clone());
    if patch.apply_to(&mut note) {
        note.updated_at = now();
    }
    if (note.title.as_str(), note.content.as_str()) != (before.0.as_str(), before.1.as_str()) {
        snapshot_if_due(conn, id, &before.0, &before.1)?;
    }
    save(conn, &mut note)?;
    Ok(note)
}

/// Keeps the text from before an edit as a revision — unless one was saved in the last
/// `REVISION_INTERVAL_MINUTES`, so continuous typing yields one revision per interval.
fn snapshot_if_due(conn: &Connection, id: &str, title: &str, content: &str) -> AppResult<()> {
    if title.trim().is_empty() && content.trim().is_empty() {
        return Ok(());
    }
    let latest: Option<String> = conn
        .query_row(
            "SELECT MAX(created_at) FROM note_revisions WHERE note_id = ?1",
            [id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    let cutoff = timestamp(Utc::now() - Duration::minutes(REVISION_INTERVAL_MINUTES));
    if latest.is_some_and(|latest| latest > cutoff) {
        return Ok(());
    }
    insert_revision(conn, id, title, content)
}

/// Also used by sync to keep the losing side of a conflict.
pub fn insert_revision(conn: &Connection, id: &str, title: &str, content: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO note_revisions (note_id, title, content, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, title, content, now()],
    )?;
    conn.execute(
        "DELETE FROM note_revisions WHERE note_id = ?1 AND id NOT IN (
            SELECT id FROM note_revisions WHERE note_id = ?1
            ORDER BY created_at DESC, id DESC LIMIT ?2
         )",
        params![id, MAX_REVISIONS_PER_NOTE],
    )?;
    Ok(())
}

/// Newest first.
pub fn list_revisions(conn: &Connection, note_id: &str) -> AppResult<Vec<NoteRevision>> {
    let mut stmt = conn.prepare(
        "SELECT id, note_id, title, content, created_at FROM note_revisions
         WHERE note_id = ?1 ORDER BY created_at DESC, id DESC",
    )?;
    let revisions = stmt
        .query_map([note_id], |row| {
            Ok(NoteRevision {
                id: row.get(0)?,
                note_id: row.get(1)?,
                title: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(revisions)
}

/// Puts a revision's title and content back. The current text is kept as a
/// revision first, so restoring can itself be undone.
pub fn restore_revision(conn: &mut Connection, note_id: &str, revision_id: i64) -> AppResult<Note> {
    let tx = conn.transaction()?;
    let (title, content): (String, String) = tx
        .query_row(
            "SELECT title, content FROM note_revisions WHERE id = ?1 AND note_id = ?2",
            params![revision_id, note_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?
        .ok_or_else(|| AppError::Internal("找不到這個版本".into()))?;

    let mut note = get(&tx, note_id)?;
    if (note.title.as_str(), note.content.as_str()) != (title.as_str(), content.as_str()) {
        if !(note.title.trim().is_empty() && note.content.trim().is_empty()) {
            insert_revision(&tx, note_id, &note.title, &note.content)?;
        }
        note.title = title;
        note.content = content;
        note.updated_at = now();
        save(&tx, &mut note)?;
    }
    tx.commit()?;
    Ok(note)
}

/// Content of every revision, so images used only by an old version aren't cleaned up.
pub fn revision_contents(conn: &Connection) -> AppResult<Vec<String>> {
    let mut stmt = conn.prepare("SELECT content FROM note_revisions")?;
    let contents = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(contents)
}

pub fn set_status(conn: &Connection, id: &str, status: NoteStatus) -> AppResult<Note> {
    let mut note = get(conn, id)?;
    note.status = status;
    note.deleted_at = match status {
        NoteStatus::Deleted => Some(now()),
        _ => None,
    };
    save(conn, &mut note)?;
    Ok(note)
}

pub fn set_open(conn: &Connection, id: &str, is_open: bool) -> AppResult<Note> {
    let mut note = get(conn, id)?;
    note.is_open = is_open;
    save(conn, &mut note)?;
    Ok(note)
}

/// Renames a tag on every note (merging into an existing tag of the new name).
/// Returns the notes that changed. `updated_at` is kept: this is housekeeping, not an edit.
pub fn rename_tag(conn: &mut Connection, from: &str, to: &str) -> AppResult<Vec<Note>> {
    let to = normalize_tag(to).ok_or_else(|| AppError::Internal("標籤名稱不可為空".into()))?;
    let key = tag_key(from);
    replace_tags(conn, |tags| {
        let renamed: Vec<String> = tags
            .iter()
            .map(|tag| {
                if tag_key(tag) == key {
                    to.clone()
                } else {
                    tag.clone()
                }
            })
            .collect();
        normalize_tags(&renamed)
    })
}

/// Removes a tag from every note. Returns the notes that changed.
pub fn delete_tag(conn: &mut Connection, name: &str) -> AppResult<Vec<Note>> {
    let key = tag_key(name);
    replace_tags(conn, |tags| {
        tags.iter()
            .filter(|tag| tag_key(tag) != key)
            .cloned()
            .collect()
    })
}

/// Rewrites every note's tags with `edit` in one transaction, saving only the notes that changed.
fn replace_tags(
    conn: &mut Connection,
    edit: impl Fn(&[String]) -> Vec<String>,
) -> AppResult<Vec<Note>> {
    let tx = conn.transaction()?;
    let mut changed = Vec::new();
    for mut note in list(&tx)? {
        let tags = edit(&note.tags);
        if tags != note.tags {
            note.tags = tags;
            save(&tx, &mut note)?;
            changed.push(note);
        }
    }
    tx.commit()?;
    Ok(changed)
}

/// Gives the listed notes the positions 0, 1, 2… in that order.
/// Unknown ids are ignored; notes not listed keep their position.
pub fn reorder(conn: &mut Connection, ids: &[String]) -> AppResult<()> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "UPDATE notes SET sort_order = ?2, modified_at = ?3 WHERE id = ?1 AND sort_order != ?2",
        )?;
        let timestamp = now();
        for (index, id) in ids.iter().enumerate() {
            stmt.execute(params![id, index as i64, timestamp])?;
        }
    }
    tx.commit()?;
    Ok(())
}

/// Trashed notes deleted before `cutoff` (an RFC 3339 timestamp like `now()`).
pub fn list_deleted_before(conn: &Connection, cutoff: &str) -> AppResult<Vec<Note>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM notes WHERE status = 'deleted' AND deleted_at < ?1"
    ))?;
    let notes = stmt
        .query_map([cutoff], map_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

pub fn count_open(conn: &Connection) -> AppResult<i64> {
    let count = conn.query_row(
        "SELECT COUNT(*) FROM notes WHERE status = 'active' AND is_open = 1",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    let affected = conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
    if affected == 0 {
        return Err(AppError::NoteNotFound(id.to_string()));
    }
    Ok(())
}

// ---- Sync ----

/// Writes the synced fields from another device as they are (incl. `modified_at`);
/// window geometry and the other per-device fields are kept.
pub fn apply_synced(conn: &Connection, note: &SyncedNote) -> AppResult<()> {
    conn.execute(
        "UPDATE notes SET title = ?2, content = ?3, color = ?4, tags = ?5, status = ?6,
            deleted_at = ?7, sort_order = ?8, created_at = ?9, updated_at = ?10, modified_at = ?11
         WHERE id = ?1",
        params![
            note.id,
            note.title,
            note.content,
            note.color.as_str(),
            tags_json(&note.tags),
            note.status.as_str(),
            note.deleted_at,
            note.sort_order,
            note.created_at,
            note.updated_at,
            note.modified_at,
        ],
    )?;
    Ok(())
}

/// A note first seen in the sync folder. It is not put on this desktop.
pub fn insert_synced(conn: &Connection, note: &SyncedNote) -> AppResult<()> {
    conn.execute(
        "INSERT INTO notes (id, width, height, always_on_top, collapsed, is_open, created_at, updated_at)
         VALUES (?1, ?2, ?3, 0, 0, 0, ?4, ?5)",
        params![
            note.id,
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT,
            note.created_at,
            note.updated_at
        ],
    )?;
    apply_synced(conn, note)
}

pub fn sync_bases(conn: &Connection) -> AppResult<HashMap<String, SyncedNote>> {
    let mut stmt = conn.prepare("SELECT note_id, data FROM sync_base")?;
    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    // An unreadable base just means "no common ancestor" for that note.
    Ok(rows
        .into_iter()
        .filter_map(|(id, data)| Some((id, serde_json::from_str(&data).ok()?)))
        .collect())
}

pub fn set_sync_base(conn: &Connection, note: &SyncedNote) -> AppResult<()> {
    conn.execute(
        "INSERT INTO sync_base (note_id, data) VALUES (?1, ?2)
         ON CONFLICT(note_id) DO UPDATE SET data = excluded.data",
        params![note.id, serde_json::to_string(note)?],
    )?;
    Ok(())
}

/// Forgets every common ancestor, e.g. when switching to another folder: the next
/// run then merges like a first sync (nothing is deleted, the newer edit wins).
pub fn clear_all_sync_bases(conn: &Connection) -> AppResult<()> {
    conn.execute("DELETE FROM sync_base", [])?;
    Ok(())
}

pub fn has_sync_bases(conn: &Connection) -> AppResult<bool> {
    Ok(
        conn.query_row("SELECT EXISTS (SELECT 1 FROM sync_base)", [], |row| {
            row.get(0)
        })?,
    )
}

pub fn clear_sync_base(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM sync_base WHERE note_id = ?1", [id])?;
    Ok(())
}

/// Recorded on every permanent deletion, so the deletion reaches other devices.
pub fn add_tombstone(conn: &Connection, id: &str, deleted_at: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO sync_tombstones (note_id, deleted_at) VALUES (?1, ?2)
         ON CONFLICT(note_id) DO UPDATE SET deleted_at = excluded.deleted_at",
        params![id, deleted_at],
    )?;
    Ok(())
}

pub fn tombstones(conn: &Connection) -> AppResult<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT note_id, deleted_at FROM sync_tombstones")?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<HashMap<_, _>, _>>()?;
    Ok(rows)
}

pub fn remove_tombstone(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM sync_tombstones WHERE note_id = ?1", [id])?;
    Ok(())
}

pub fn purge_tombstones(conn: &Connection, cutoff: &str) -> AppResult<()> {
    conn.execute(
        "DELETE FROM sync_tombstones WHERE deleted_at < ?1",
        [cutoff],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;

    fn with_db(f: impl FnOnce(&mut Connection)) {
        let db = Database::open_in_memory().unwrap();
        db.with_conn(|conn| {
            f(conn);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn create_uses_defaults() {
        with_db(|conn| {
            let note = insert(conn, None).unwrap();
            assert_eq!(note.width, 320);
            assert_eq!(note.height, 280);
            assert_eq!(note.color, NoteColor::Yellow);
            assert!(!note.always_on_top);
            assert!(!note.collapsed);
            assert!(note.is_open);
            assert_eq!(note.status, NoteStatus::Active);
            assert_eq!(note.x, None);
        });
    }

    #[test]
    fn layout_changes_do_not_bump_updated_at() {
        with_db(|conn| {
            let note = insert(conn, Some((10, 20))).unwrap();
            let moved = update(
                conn,
                &note.id,
                &NoteUpdate {
                    x: Some(100),
                    width: Some(400),
                    collapsed: Some(true),
                    always_on_top: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(moved.updated_at, note.updated_at);
            assert_eq!((moved.x, moved.y, moved.width), (Some(100), Some(20), 400));
            assert!(moved.collapsed && moved.always_on_top);
            assert_eq!(moved.height, 280);

            std::thread::sleep(std::time::Duration::from_millis(5));
            let edited = update(
                conn,
                &note.id,
                &NoteUpdate {
                    content: Some("hello".into()),
                    ..Default::default()
                },
            )
            .unwrap();
            assert!(edited.updated_at > note.updated_at);
            assert_eq!(get(conn, &note.id).unwrap().content, "hello");
        });
    }

    #[test]
    fn status_lifecycle() {
        with_db(|conn| {
            let note = insert(conn, None).unwrap();

            let archived = set_status(conn, &note.id, NoteStatus::Archived).unwrap();
            assert_eq!(archived.status, NoteStatus::Archived);
            assert!(list_open(conn).unwrap().is_empty());

            let deleted = set_status(conn, &note.id, NoteStatus::Deleted).unwrap();
            assert!(deleted.deleted_at.is_some());

            let restored = set_status(conn, &note.id, NoteStatus::Active).unwrap();
            assert_eq!(restored.deleted_at, None);
            assert_eq!(list_open(conn).unwrap().len(), 1);

            set_open(conn, &note.id, false).unwrap();
            assert!(list_open(conn).unwrap().is_empty());
            assert_eq!(count_open(conn).unwrap(), 0);

            delete(conn, &note.id).unwrap();
            assert!(find(conn, &note.id).unwrap().is_none());
            assert!(delete(conn, &note.id).is_err());
        });
    }

    #[test]
    fn tags_are_normalised_and_persisted() {
        with_db(|conn| {
            let note = insert(conn, None).unwrap();
            assert!(note.tags.is_empty());

            let tagged = update(
                conn,
                &note.id,
                &NoteUpdate {
                    tags: Some(vec!["#Work".into(), " 待辦 ".into(), "work".into()]),
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(tagged.tags, vec!["Work", "待辦"]);
            assert!(tagged.updated_at >= note.updated_at);
            assert_eq!(get(conn, &note.id).unwrap().tags, vec!["Work", "待辦"]);
        });
    }

    #[test]
    fn rename_and_delete_tags_across_notes() {
        with_db(|conn| {
            let tag = |conn: &mut Connection, tags: &[&str]| {
                let note = insert(conn, None).unwrap();
                let tags = tags.iter().map(|t| t.to_string()).collect();
                update(
                    conn,
                    &note.id,
                    &NoteUpdate {
                        tags: Some(tags),
                        ..Default::default()
                    },
                )
                .unwrap()
            };
            let a = tag(conn, &["work", "home"]);
            let b = tag(conn, &["Work"]);
            let c = tag(conn, &["other"]);

            // Renaming into an existing tag merges them.
            let changed = rename_tag(conn, "WORK", "home").unwrap();
            assert_eq!(changed.len(), 2);
            assert_eq!(get(conn, &a.id).unwrap().tags, vec!["home"]);
            assert_eq!(get(conn, &b.id).unwrap().tags, vec!["home"]);
            assert_eq!(get(conn, &b.id).unwrap().updated_at, b.updated_at);
            assert!(rename_tag(conn, "home", "  ").is_err());

            let changed = delete_tag(conn, "Home").unwrap();
            assert_eq!(changed.len(), 2);
            assert!(get(conn, &a.id).unwrap().tags.is_empty());
            assert_eq!(get(conn, &c.id).unwrap().tags, vec!["other"]);
        });
    }

    #[test]
    fn new_notes_go_first_and_reorder_sets_positions() {
        with_db(|conn| {
            let a = insert(conn, None).unwrap();
            let b = insert(conn, None).unwrap();
            assert!(b.sort_order < a.sort_order);

            reorder(conn, &[a.id.clone(), "missing".into(), b.id.clone()]).unwrap();
            assert_eq!(get(conn, &a.id).unwrap().sort_order, 0);
            assert_eq!(get(conn, &b.id).unwrap().sort_order, 2);
            // Reordering is layout, not an edit.
            assert_eq!(get(conn, &a.id).unwrap().updated_at, a.updated_at);
        });
    }

    #[test]
    fn lists_trash_older_than_cutoff() {
        with_db(|conn| {
            let old = insert(conn, None).unwrap();
            let recent = insert(conn, None).unwrap();
            let kept = insert(conn, None).unwrap();
            set_status(conn, &old.id, NoteStatus::Deleted).unwrap();
            set_status(conn, &recent.id, NoteStatus::Deleted).unwrap();
            conn.execute(
                "UPDATE notes SET deleted_at = '2020-01-01T00:00:00.000Z' WHERE id = ?1",
                [&old.id],
            )
            .unwrap();

            let expired = list_deleted_before(conn, "2021-01-01T00:00:00.000Z").unwrap();
            let ids: Vec<_> = expired.iter().map(|n| n.id.as_str()).collect();
            assert_eq!(ids, vec![old.id.as_str()]);
            assert_eq!(get(conn, &kept.id).unwrap().status, NoteStatus::Active);
        });
    }

    #[test]
    fn edits_keep_earlier_versions_at_intervals() {
        with_db(|conn| {
            let note = insert(conn, None).unwrap();
            let edit = |conn: &mut Connection, text: &str| {
                let patch = NoteUpdate {
                    content: Some(text.into()),
                    ..Default::default()
                };
                update(conn, &note.id, &patch).unwrap()
            };

            // An empty note has nothing worth keeping.
            edit(conn, "v1");
            assert!(list_revisions(conn, &note.id).unwrap().is_empty());

            // The first edit of non-empty text keeps it; edits right after don't.
            edit(conn, "v2");
            edit(conn, "v3");
            let revisions = list_revisions(conn, &note.id).unwrap();
            assert_eq!(revisions.len(), 1);
            assert_eq!(revisions[0].content, "v1");

            // Once the interval has passed, the next edit keeps a new version.
            conn.execute(
                "UPDATE note_revisions SET created_at = '2020-01-01T00:00:00.000Z'",
                [],
            )
            .unwrap();
            edit(conn, "v4");
            let revisions = list_revisions(conn, &note.id).unwrap();
            assert_eq!(revisions.len(), 2);
            assert_eq!(revisions[0].content, "v3");

            // Layout changes never create revisions.
            let moved = NoteUpdate {
                x: Some(5),
                ..Default::default()
            };
            update(conn, &note.id, &moved).unwrap();
            assert_eq!(list_revisions(conn, &note.id).unwrap().len(), 2);
        });
    }

    #[test]
    fn restoring_a_revision_keeps_the_current_text() {
        with_db(|conn| {
            let note = insert(conn, None).unwrap();
            for text in ["first", "second"] {
                let patch = NoteUpdate {
                    content: Some(text.into()),
                    ..Default::default()
                };
                update(conn, &note.id, &patch).unwrap();
            }
            let first = list_revisions(conn, &note.id).unwrap()[0].clone();
            assert_eq!(first.content, "first");

            let restored = restore_revision(conn, &note.id, first.id).unwrap();
            assert_eq!(restored.content, "first");
            let contents: Vec<_> = list_revisions(conn, &note.id)
                .unwrap()
                .into_iter()
                .map(|r| r.content)
                .collect();
            assert!(contents.contains(&"second".to_string()));
            assert!(restore_revision(conn, "other", first.id).is_err());

            // Revisions go away with the note.
            delete(conn, &note.id).unwrap();
            assert!(list_revisions(conn, &note.id).unwrap().is_empty());
        });
    }

    #[test]
    fn old_revisions_are_capped() {
        with_db(|conn| {
            let note = insert(conn, None).unwrap();
            for i in 0..(MAX_REVISIONS_PER_NOTE + 5) {
                insert_revision(conn, &note.id, "", &format!("v{i}")).unwrap();
            }
            let revisions = list_revisions(conn, &note.id).unwrap();
            assert_eq!(revisions.len() as i64, MAX_REVISIONS_PER_NOTE);
            assert_eq!(
                revisions[0].content,
                format!("v{}", MAX_REVISIONS_PER_NOTE + 4)
            );
        });
    }

    #[test]
    fn modified_at_follows_synced_fields_only() {
        with_db(|conn| {
            let note = insert(conn, None).unwrap();
            assert_eq!(note.modified_at, note.created_at);
            let later = |conn: &mut Connection| {
                std::thread::sleep(std::time::Duration::from_millis(5));
                get(conn, &note.id).unwrap().modified_at
            };

            let moved = NoteUpdate {
                x: Some(1),
                collapsed: Some(true),
                ..Default::default()
            };
            update(conn, &note.id, &moved).unwrap();
            set_open(conn, &note.id, false).unwrap();
            assert_eq!(later(conn), note.modified_at);

            set_status(conn, &note.id, NoteStatus::Archived).unwrap();
            let archived = later(conn);
            assert!(archived > note.modified_at);

            reorder(conn, std::slice::from_ref(&note.id)).unwrap();
            assert_eq!(get(conn, &note.id).unwrap().sort_order, 0);
            let reordered = later(conn);
            assert!(reordered > archived);
            // Same position again: nothing changed.
            reorder(conn, std::slice::from_ref(&note.id)).unwrap();
            assert_eq!(later(conn), reordered);
        });
    }

    #[test]
    fn synced_notes_keep_local_layout() {
        with_db(|conn| {
            let note = insert(conn, Some((10, 20))).unwrap();
            let mut remote = SyncedNote::from(&note);
            remote.content = "from another device".into();
            remote.modified_at = "2030-01-01T00:00:00.000Z".into();
            apply_synced(conn, &remote).unwrap();
            let stored = get(conn, &note.id).unwrap();
            assert_eq!(stored.content, "from another device");
            assert_eq!(stored.modified_at, "2030-01-01T00:00:00.000Z");
            assert_eq!(
                (stored.x, stored.y, stored.is_open),
                (Some(10), Some(20), true)
            );

            let mut fresh = remote.clone();
            fresh.id = "from-elsewhere".into();
            insert_synced(conn, &fresh).unwrap();
            let stored = get(conn, "from-elsewhere").unwrap();
            assert!(!stored.is_open);
            assert_eq!(SyncedNote::from(&stored), fresh);

            set_sync_base(conn, &fresh).unwrap();
            assert_eq!(sync_bases(conn).unwrap()["from-elsewhere"], fresh);
            clear_sync_base(conn, "from-elsewhere").unwrap();
            assert!(sync_bases(conn).unwrap().is_empty());

            add_tombstone(conn, "gone", "2026-01-01T00:00:00.000Z").unwrap();
            assert_eq!(
                tombstones(conn).unwrap()["gone"],
                "2026-01-01T00:00:00.000Z"
            );
            purge_tombstones(conn, "2027-01-01T00:00:00.000Z").unwrap();
            assert!(tombstones(conn).unwrap().is_empty());
        });
    }

    #[test]
    fn insert_if_absent_keeps_existing_notes() {
        with_db(|conn| {
            let original = insert(conn, None).unwrap();
            update(
                conn,
                &original.id,
                &NoteUpdate {
                    content: Some("local".into()),
                    ..Default::default()
                },
            )
            .unwrap();

            let mut imported = original.clone();
            imported.content = "from file".into();
            assert!(!insert_if_absent(conn, &imported).unwrap());
            assert_eq!(get(conn, &original.id).unwrap().content, "local");

            imported.id = "imported-1".into();
            imported.color = NoteColor::Pink;
            assert!(insert_if_absent(conn, &imported).unwrap());
            let stored = get(conn, "imported-1").unwrap();
            assert_eq!(stored.content, "from file");
            assert_eq!(stored.color, NoteColor::Pink);
            assert_eq!(stored.created_at, original.created_at);
        });
    }
}
