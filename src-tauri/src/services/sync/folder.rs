//! The sync folder inside a cloud drive (Google Drive, OneDrive…):
//!
//! ```text
//! PinNote Sync/
//!   notes/<id>.json      one file per note (SyncedNote)
//!   deleted/<id>.json    tombstones of notes deleted for good
//!   images/<file>        image files (names are unique and never change)
//!   settings.json        shared settings (tag colors)
//! ```
//!
//! Files are written to a temporary name and renamed, so the drive never uploads
//! half a file. Copies the drive makes on conflicts (`<id> (1).json`,
//! `<id>-PCNAME.json`) are read as extra versions of the note they contain.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::error::AppResult;
use crate::models::{SyncedNote, Tombstone};
use crate::tr;

pub const FOLDER_NAME: &str = "PinNote Sync";

/// Shared settings (`settings.json`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSettings {
    #[serde(default)]
    pub tag_colors: BTreeMap<String, String>,
}

/// Every version of one note found in `notes/`.
#[derive(Debug, Default)]
pub struct RemoteNote {
    /// `notes/<id>.json`.
    pub primary: Option<SyncedNote>,
    /// Conflict copies made by the drive, to merge and then remove.
    pub duplicates: Vec<(PathBuf, SyncedNote)>,
}

/// Parsed note files by path, reused while their size and mtime are unchanged —
/// a streamed drive (Google Drive's G:) is slow to read every file every time.
pub type ReadCache = HashMap<PathBuf, (SystemTime, u64, SyncedNote)>;

pub struct SyncFolder {
    root: PathBuf,
}

/// Ids become file names: only accept what our own ids (UUIDs) look like.
pub fn is_safe_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

impl SyncFolder {
    /// `chosen` is the folder picked by the user; the data lives in `PinNote Sync`
    /// inside it (or in it directly when that folder was picked). Nothing is created.
    pub fn locate(chosen: &Path) -> Self {
        let root = if chosen.file_name().is_some_and(|name| name == FOLDER_NAME) {
            chosen.to_path_buf()
        } else {
            chosen.join(FOLDER_NAME)
        };
        Self { root }
    }

    /// Locates and creates the folder structure.
    pub fn open(chosen: &Path) -> AppResult<Self> {
        let folder = Self::locate(chosen);
        folder.ensure()?;
        Ok(folder)
    }

    pub fn ensure(&self) -> AppResult<()> {
        for dir in [self.notes_dir(), self.deleted_dir(), self.images_dir()] {
            fs::create_dir_all(dir)?;
        }
        Ok(())
    }

    /// Holds synced data: a note file or a deletion record.
    pub fn has_data(&self) -> AppResult<bool> {
        Ok(!json_files(&self.notes_dir())?.is_empty()
            || !json_files(&self.deleted_dir())?.is_empty())
    }

    fn notes_dir(&self) -> PathBuf {
        self.root.join("notes")
    }

    fn deleted_dir(&self) -> PathBuf {
        self.root.join("deleted")
    }

    pub fn images_dir(&self) -> PathBuf {
        self.root.join("images")
    }

    fn settings_path(&self) -> PathBuf {
        self.root.join("settings.json")
    }

    pub fn read_notes(
        &self,
        cache: &mut ReadCache,
        warnings: &mut Vec<String>,
    ) -> AppResult<HashMap<String, RemoteNote>> {
        let mut notes: HashMap<String, RemoteNote> = HashMap::new();
        for path in json_files(&self.notes_dir())? {
            let note = match read_cached(&path, cache) {
                Ok(note) => note.normalized(),
                Err(err) => {
                    warnings.push(tr!(
                        "無法讀取 {}：{err}",
                        "Could not read {}: {err}",
                        display_name(&path)
                    ));
                    continue;
                }
            };
            if !is_safe_id(&note.id) {
                warnings.push(tr!(
                    "略過無效的便籤檔 {}",
                    "Skipped an invalid note file {}",
                    display_name(&path)
                ));
                continue;
            }
            let entry = notes.entry(note.id.clone()).or_default();
            if path.file_stem().is_some_and(|stem| *stem == *note.id) {
                entry.primary = Some(note);
            } else {
                entry.duplicates.push((path, note));
            }
        }
        cache.retain(|path, _| path.exists());
        Ok(notes)
    }

    pub fn write_note(&self, note: &SyncedNote) -> AppResult<()> {
        write_json_atomic(&self.notes_dir().join(format!("{}.json", note.id)), note)
    }

    pub fn delete_note(&self, id: &str) -> AppResult<()> {
        remove_if_exists(&self.notes_dir().join(format!("{id}.json")))
    }

    pub fn remove_file(&self, path: &Path) -> AppResult<()> {
        remove_if_exists(path)
    }

    /// Note id → deletion time.
    pub fn read_tombstones(
        &self,
        warnings: &mut Vec<String>,
    ) -> AppResult<HashMap<String, String>> {
        let mut tombstones = HashMap::new();
        for path in json_files(&self.deleted_dir())? {
            match read_json::<Tombstone>(&path) {
                Ok(t) if is_safe_id(&t.id) => {
                    // Copies of the same tombstone: keep the latest deletion.
                    let entry = tombstones.entry(t.id).or_insert_with(String::new);
                    if t.deleted_at > *entry {
                        *entry = t.deleted_at;
                    }
                }
                Ok(_) => warnings.push(tr!(
                    "略過無效的刪除紀錄 {}",
                    "Skipped an invalid deletion record {}",
                    display_name(&path)
                )),
                Err(err) => warnings.push(tr!(
                    "無法讀取 {}：{err}",
                    "Could not read {}: {err}",
                    display_name(&path)
                )),
            }
        }
        Ok(tombstones)
    }

    pub fn write_tombstone(&self, id: &str, deleted_at: &str) -> AppResult<()> {
        let tombstone = Tombstone {
            id: id.to_string(),
            deleted_at: deleted_at.to_string(),
        };
        write_json_atomic(&self.deleted_dir().join(format!("{id}.json")), &tombstone)
    }

    /// Removes tombstones deleted before `cutoff`; every device has seen them by then.
    pub fn purge_tombstones(&self, cutoff: &str) -> AppResult<()> {
        for path in json_files(&self.deleted_dir())? {
            if read_json::<Tombstone>(&path).is_ok_and(|t| t.deleted_at.as_str() < cutoff) {
                remove_if_exists(&path)?;
            }
        }
        Ok(())
    }

    pub fn read_settings(&self) -> AppResult<Option<RemoteSettings>> {
        match read_json(&self.settings_path()) {
            Ok(settings) => Ok(Some(settings)),
            Err(err) if is_not_found(&err) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub fn write_settings(&self, settings: &RemoteSettings) -> AppResult<()> {
        write_json_atomic(&self.settings_path(), settings)
    }
}

/// Copies a file so the target never exists half-written.
pub fn copy_atomic(from: &Path, to: &Path) -> AppResult<()> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = temp_path(to);
    fs::copy(from, &tmp)?;
    fs::rename(&tmp, to).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })?;
    Ok(())
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> AppResult<()> {
    let tmp = temp_path(path);
    fs::write(&tmp, serde_json::to_vec_pretty(value)?)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })?;
    Ok(())
}

/// Hidden name next to the target, ignored when reading.
fn temp_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default();
    path.with_file_name(format!(".{name}.tmp"))
}

fn read_json<T: DeserializeOwned>(path: &Path) -> AppResult<T> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn read_cached(path: &Path, cache: &mut ReadCache) -> AppResult<SyncedNote> {
    let meta = fs::metadata(path)?;
    let modified = meta.modified()?;
    if let Some((time, len, note)) = cache.get(path) {
        if *time == modified && *len == meta.len() {
            return Ok(note.clone());
        }
    }
    let note: SyncedNote = read_json(path)?;
    cache.insert(path.to_path_buf(), (modified, meta.len(), note.clone()));
    Ok(note)
}

/// `*.json` files of a folder, skipping our temporary files.
fn json_files(dir: &Path) -> AppResult<Vec<PathBuf>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };
    let mut files = Vec::new();
    for entry in entries {
        let path = entry?.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if path.is_file() && name.ends_with(".json") && !name.starts_with('.') {
            files.push(path);
        }
    }
    Ok(files)
}

fn remove_if_exists(path: &Path) -> AppResult<()> {
    match fs::remove_file(path) {
        Err(err) if err.kind() != ErrorKind::NotFound => Err(err.into()),
        _ => Ok(()),
    }
}

fn is_not_found(err: &crate::error::AppError) -> bool {
    matches!(err, crate::error::AppError::Io(io) if io.kind() == ErrorKind::NotFound)
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NoteColor, NoteStatus};

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("pinnote-sync-{name}-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn note(id: &str, content: &str) -> SyncedNote {
        SyncedNote {
            id: id.into(),
            title: String::new(),
            content: content.into(),
            color: NoteColor::Yellow,
            tags: vec![],
            status: NoteStatus::Active,
            deleted_at: None,
            sort_order: 0,
            created_at: "2026-01-01T00:00:00.000Z".into(),
            updated_at: "2026-01-01T00:00:00.000Z".into(),
            modified_at: "2026-01-01T00:00:00.000Z".into(),
        }
    }

    #[test]
    fn notes_round_trip_and_conflict_copies_are_grouped() {
        let dir = temp_dir("notes");
        let folder = SyncFolder::open(&dir).unwrap();
        assert!(dir.join(FOLDER_NAME).join("notes").is_dir());

        folder.write_note(&note("abc", "main")).unwrap();
        // What Google Drive / OneDrive make on conflicts, plus junk to ignore.
        let notes_dir = folder.notes_dir();
        fs::copy(notes_dir.join("abc.json"), notes_dir.join("abc (1).json")).unwrap();
        fs::write(notes_dir.join(".abc.json.tmp"), "half").unwrap();
        fs::write(notes_dir.join("broken.json"), "{").unwrap();
        let mut evil = note("../x", "");
        evil.id = "../x".into();
        fs::write(
            notes_dir.join("evil.json"),
            serde_json::to_vec(&evil).unwrap(),
        )
        .unwrap();

        let mut cache = ReadCache::new();
        let mut warnings = Vec::new();
        let notes = folder.read_notes(&mut cache, &mut warnings).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes["abc"].primary.as_ref().unwrap().content, "main");
        assert_eq!(notes["abc"].duplicates.len(), 1);
        assert_eq!(warnings.len(), 2);

        folder.delete_note("abc").unwrap();
        folder.delete_note("abc").unwrap();
        assert!(!notes_dir.join("abc.json").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn tombstones_and_settings() {
        let dir = temp_dir("tomb");
        let folder = SyncFolder::open(&dir.join(FOLDER_NAME)).unwrap();
        assert!(!dir.join(FOLDER_NAME).join(FOLDER_NAME).exists());

        folder
            .write_tombstone("old", "2020-01-01T00:00:00.000Z")
            .unwrap();
        folder
            .write_tombstone("new", "2026-01-01T00:00:00.000Z")
            .unwrap();
        let mut warnings = Vec::new();
        assert_eq!(folder.read_tombstones(&mut warnings).unwrap().len(), 2);
        folder.purge_tombstones("2025-01-01T00:00:00.000Z").unwrap();
        let left = folder.read_tombstones(&mut warnings).unwrap();
        assert_eq!(left.keys().collect::<Vec<_>>(), vec!["new"]);

        assert_eq!(folder.read_settings().unwrap(), None);
        let settings = RemoteSettings {
            tag_colors: BTreeMap::from([("work".into(), "blue".into())]),
        };
        folder.write_settings(&settings).unwrap();
        assert_eq!(folder.read_settings().unwrap(), Some(settings));
        let _ = fs::remove_dir_all(dir);
    }
}
