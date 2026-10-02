use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use super::now;
use crate::error::{AppError, AppResult};
use crate::models::note::{DEFAULT_HEIGHT, DEFAULT_WIDTH};
use crate::models::{Note, NoteColor, NoteStatus, NoteUpdate};

const COLUMNS: &str = "id, title, content, color, x, y, width, height, always_on_top, collapsed, \
                       is_open, status, created_at, updated_at, deleted_at";

fn map_row(row: &Row) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get("id")?,
        title: row.get("title")?,
        content: row.get("content")?,
        color: NoteColor::parse(&row.get::<_, String>("color")?),
        x: row.get("x")?,
        y: row.get("y")?,
        width: row.get("width")?,
        height: row.get("height")?,
        always_on_top: row.get("always_on_top")?,
        collapsed: row.get("collapsed")?,
        is_open: row.get("is_open")?,
        status: NoteStatus::parse(&row.get::<_, String>("status")?),
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        deleted_at: row.get("deleted_at")?,
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
        "INSERT INTO notes (id, color, x, y, width, height, always_on_top, collapsed, is_open, status, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, 1, 'active', ?7, ?7)",
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
    save(conn, note)?;
    Ok(true)
}

fn save(conn: &Connection, note: &Note) -> AppResult<()> {
    conn.execute(
        "UPDATE notes SET
            title = ?2, content = ?3, color = ?4, x = ?5, y = ?6, width = ?7, height = ?8,
            always_on_top = ?9, collapsed = ?10, is_open = ?11, status = ?12,
            updated_at = ?13, deleted_at = ?14
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
        ],
    )?;
    Ok(())
}

pub fn update(conn: &Connection, id: &str, patch: &NoteUpdate) -> AppResult<Note> {
    let mut note = get(conn, id)?;
    if patch.apply_to(&mut note) {
        note.updated_at = now();
    }
    save(conn, &note)?;
    Ok(note)
}

pub fn set_status(conn: &Connection, id: &str, status: NoteStatus) -> AppResult<Note> {
    let mut note = get(conn, id)?;
    note.status = status;
    note.deleted_at = match status {
        NoteStatus::Deleted => Some(now()),
        _ => None,
    };
    save(conn, &note)?;
    Ok(note)
}

pub fn set_open(conn: &Connection, id: &str, is_open: bool) -> AppResult<Note> {
    let mut note = get(conn, id)?;
    note.is_open = is_open;
    save(conn, &note)?;
    Ok(note)
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
