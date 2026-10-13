use rusqlite::{params, Connection};

use super::now;
use crate::error::AppResult;

struct Migration {
    version: i64,
    name: &'static str,
    sql: &'static str,
}

/// Ordered list of schema migrations. Append only — never edit a released migration.
const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "create_notes",
        sql: include_str!("../../migrations/0001_create_notes.sql"),
    },
    Migration {
        version: 2,
        name: "add_note_is_open",
        sql: include_str!("../../migrations/0002_add_note_is_open.sql"),
    },
    Migration {
        version: 3,
        name: "create_settings",
        sql: include_str!("../../migrations/0003_create_settings.sql"),
    },
    Migration {
        version: 4,
        name: "add_note_tags",
        sql: include_str!("../../migrations/0004_add_note_tags.sql"),
    },
    Migration {
        version: 5,
        name: "add_note_sort_order",
        sql: include_str!("../../migrations/0005_add_note_sort_order.sql"),
    },
    Migration {
        version: 6,
        name: "create_note_revisions",
        sql: include_str!("../../migrations/0006_create_note_revisions.sql"),
    },
    Migration {
        version: 7,
        name: "sync",
        sql: include_str!("../../migrations/0007_sync.sql"),
    },
];

/// Applies every migration newer than the recorded schema version,
/// each inside its own transaction.
pub fn run(conn: &mut Connection) -> AppResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );",
    )?;

    let current: i64 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get(0),
    )?;

    for migration in MIGRATIONS.iter().filter(|m| m.version > current) {
        let tx = conn.transaction()?;
        tx.execute_batch(migration.sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, name, applied_at) VALUES (?1, ?2, ?3)",
            params![migration.version, migration.name, now()],
        )?;
        tx.commit()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_idempotent() {
        let mut conn = Connection::open_in_memory().unwrap();
        run(&mut conn).unwrap();
        run(&mut conn).unwrap();

        let applied: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(applied, MIGRATIONS.len() as i64);
    }
}
