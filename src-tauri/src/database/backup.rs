use std::collections::HashSet;
use std::path::{Path, PathBuf};

use chrono::Local;
use rusqlite::{Connection, OpenFlags};

use crate::error::AppResult;
use crate::image_refs;

pub const KEEP_BACKUPS: usize = 7;
/// Backed-up image files, shared by all snapshots (`backups/images/`).
pub const IMAGES_SUBDIR: &str = "images";

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

/// Keeps `dir/images` holding exactly the images referenced by the retained
/// snapshots: missing ones are copied from `images_dir` (images are immutable,
/// so existing copies are never rewritten) and no-longer-needed ones removed.
/// Returns how many files were copied.
pub fn sync_images(images_dir: &Path, dir: &Path) -> AppResult<usize> {
    let target = dir.join(IMAGES_SUBDIR);
    let mut needed = HashSet::new();
    for snapshot in snapshots(dir)? {
        match snapshot_image_refs(&snapshot) {
            Ok(refs) => needed.extend(refs),
            // Unreadable snapshot: keep every backed-up image rather than risk losing one.
            Err(err) => {
                eprintln!("[pinnote] cannot read backup {}: {err}", snapshot.display());
                return Ok(0);
            }
        }
    }

    let mut copied = 0;
    for name in &needed {
        let source = images_dir.join(name);
        let destination = target.join(name);
        if !destination.exists() && source.is_file() {
            std::fs::create_dir_all(&target)?;
            std::fs::copy(&source, &destination)?;
            copied += 1;
        }
    }

    if let Ok(entries) = std::fs::read_dir(&target) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if image_refs::is_safe_file_name(&name) && !needed.contains(&name) {
                std::fs::remove_file(entry.path())?;
            }
        }
    }
    Ok(copied)
}

fn snapshot_image_refs(snapshot: &Path) -> AppResult<HashSet<String>> {
    let conn = Connection::open_with_flags(snapshot, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut stmt = conn.prepare("SELECT content FROM notes")?;
    let contents = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(image_refs::referenced_by_all(
        contents.iter().map(String::as_str),
    ))
}

fn snapshots(dir: &Path) -> AppResult<Vec<PathBuf>> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(Vec::new());
    };
    let mut snapshots: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with(FILE_PREFIX) && name.ends_with(FILE_EXT))
        })
        .collect();
    // File names embed the date, so lexical order is chronological.
    snapshots.sort();
    Ok(snapshots)
}

fn prune(dir: &Path, keep: usize) -> AppResult<()> {
    let backups = snapshots(dir)?;
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
    fn sync_images_backs_up_exactly_what_snapshots_reference() {
        let dir = temp_dir("images");
        let images = dir.join("images");
        let backups = dir.join("backups");
        std::fs::create_dir_all(&images).unwrap();
        std::fs::create_dir_all(&backups).unwrap();
        for name in ["kept.png", "fresh.png", "stale.png"] {
            std::fs::write(images.join(name), name).unwrap();
        }

        let snapshot = Connection::open(backups.join("pinnote-20261001.db")).unwrap();
        snapshot
            .execute_batch(
                "CREATE TABLE notes (content TEXT);
                 INSERT INTO notes VALUES ('![](images/kept.png)');",
            )
            .unwrap();
        drop(snapshot);
        // Left over from an older snapshot that has since been pruned.
        std::fs::create_dir_all(backups.join("images")).unwrap();
        std::fs::write(backups.join("images/stale.png"), "old").unwrap();

        assert_eq!(sync_images(&images, &backups).unwrap(), 1);
        assert!(backups.join("images/kept.png").exists());
        assert!(
            !backups.join("images/fresh.png").exists(),
            "not in any snapshot"
        );
        assert!(
            !backups.join("images/stale.png").exists(),
            "no longer referenced"
        );

        // Second run copies nothing new.
        assert_eq!(sync_images(&images, &backups).unwrap(), 0);
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
