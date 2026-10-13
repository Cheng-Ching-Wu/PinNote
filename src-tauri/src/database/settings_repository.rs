use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use super::now;
use crate::error::AppResult;
use crate::models::{AppSettings, AppSettingsPatch};

pub fn get(conn: &Connection) -> AppResult<AppSettings> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let map = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<HashMap<String, String>, _>>()?;
    Ok(AppSettings::from_map(&map))
}

pub fn update(conn: &mut Connection, patch: &AppSettingsPatch) -> AppResult<AppSettings> {
    let tx = conn.transaction()?;
    let timestamp = now();
    for (key, value) in patch.entries() {
        tx.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, timestamp],
        )?;
    }
    tx.commit()?;
    get(conn)
}

/// Raw value of a single key (used for entries not modelled in `AppSettings`).
pub fn get_value(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let value = conn
        .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
        .optional()?;
    Ok(value)
}

pub fn set_value(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, now()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use crate::models::settings::TagColors;

    #[test]
    fn defaults_then_persists_updates() {
        let db = Database::open_in_memory().unwrap();
        db.with_conn(|conn| {
            assert_eq!(get(conn)?, AppSettings::default());
            assert!(get(conn)?.pinned_translucent);

            let patch = AppSettingsPatch {
                pinned_translucent: Some(false),
                ..Default::default()
            };
            assert!(!update(conn, &patch)?.pinned_translucent);
            assert!(!get(conn)?.pinned_translucent);

            let colors = TagColors::from([("work".to_string(), "blue".to_string())]);
            let patch = AppSettingsPatch {
                tag_colors: Some(colors.clone()),
                ..Default::default()
            };
            assert_eq!(update(conn, &patch)?.tag_colors, colors);

            // An empty patch changes nothing.
            let unchanged = update(conn, &AppSettingsPatch::default())?;
            assert!(!unchanged.pinned_translucent);
            assert_eq!(unchanged.tag_colors, colors);

            assert_eq!(get_value(conn, "shortcut.new_note")?, None);
            set_value(conn, "shortcut.new_note", "F15")?;
            set_value(conn, "shortcut.new_note", "")?;
            assert_eq!(get_value(conn, "shortcut.new_note")?.as_deref(), Some(""));
            Ok(())
        })
        .unwrap();
    }
}
