use std::path::{Path, PathBuf};

use chrono::Local;
use rusqlite::Connection;

use crate::error::AppResult;

pub const KEEP_BACKUPS: usize = 7;

const FILE_PREFIX: &str = "pinnote-";
const FILE_EXT: &str = ".db";

/// Snapshots the database into `dir/pinnote-YYYYMMDD.db` at most once per day
/// and keeps only the newest `keep` snapshots.
///
/// Runs before migrations, so a failed upgrade can always be rolled back by hand.
pub fn daily_backup(db_path: &Path, dir: &Path, keep: usize) -> AppResult<Option<PathBuf>> {
    if !db_path.exists() {
        return Ok(None);
    }
    std::fs::create_dir_all(dir)?;

    let target = dir.join(format!(
        "{FILE_PREFIX}{}{FILE_EXT}",
        Local::now().format("%Y%m%d")
    ));
    let created = if target.exists() {
        false
    } else {
        // VACUUM INTO produces a consistent, compacted copy (WAL contents included).
        let conn = Connection::open(db_path)?;
        conn.execute("VACUUM INTO ?1", [target.to_string_lossy()])?;
        true
    };

    prune(dir, keep)?;
    Ok(created.then_some(target))
}

fn prune(dir: &Path, keep: usize) -> AppResult<()> {
    let mut backups: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(FILE_PREFIX) && name.ends_with(FILE_EXT))
        })
        .collect();

    // File names embed the date, so lexical order is chronological.
    backups.sort();
    let excess = backups.len().saturating_sub(keep);
    for old in &backups[..excess] {
        std::fs::remove_file(old)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("pinnote-test-{name}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn creates_one_backup_per_day() {
        let dir = temp_dir("backup");
        let db_path = dir.join("pinnote.db");
        let conn = Connection::open(&db_path).unwrap();
        conn.execute_batch("CREATE TABLE t (v TEXT); INSERT INTO t VALUES ('hello');")
            .unwrap();
        drop(conn);

        let backup_dir = dir.join("backups");
        let created = daily_backup(&db_path, &backup_dir, KEEP_BACKUPS)
            .unwrap()
            .expect("first run creates a backup");
        assert!(daily_backup(&db_path, &backup_dir, KEEP_BACKUPS)
            .unwrap()
            .is_none());

        let copy = Connection::open(&created).unwrap();
        let value: String = copy.query_row("SELECT v FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(value, "hello");

        drop(copy);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn skips_missing_database() {
        let dir = temp_dir("missing");
        assert!(daily_backup(&dir.join("nope.db"), &dir.join("backups"), 7)
            .unwrap()
            .is_none());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn keeps_only_newest_backups() {
        let dir = temp_dir("prune");
        for day in 1..=9 {
            std::fs::write(dir.join(format!("pinnote-202609{day:02}.db")), b"").unwrap();
        }
        std::fs::write(dir.join("unrelated.txt"), b"").unwrap();

        prune(&dir, 7).unwrap();

        let mut left: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        assert_eq!(left.len(), 8);
        assert_eq!(left[0], "pinnote-20260903.db");
        assert!(left.contains(&"unrelated.txt".to_string()));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
