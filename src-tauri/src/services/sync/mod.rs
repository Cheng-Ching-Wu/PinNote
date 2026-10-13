//! Sync between devices through a folder in a cloud drive (Google Drive, OneDrive…).
//!
//! Each run: read local notes, the folder and the state after the last sync →
//! decide per note (`merge::plan_note`) → apply to the database → write the folder
//! → remember the agreed state. Window layout, pin, collapse and shortcuts stay
//! per device; tag colors are shared.
//!
//! Runs on a background thread: at startup, ~20 s after local changes, every two
//! minutes (file watching is unreliable on streamed drives) and on demand.

mod folder;
mod merge;

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration as StdDuration, Instant};

use chrono::{Duration, Utc};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::database::{note_repository as repo, now, settings_repository, timestamp, Database};
use crate::error::{AppError, AppResult};
use crate::events;
use crate::image_refs;
use crate::models::{AppSettingsPatch, NoteStatus, SyncedNote};
use crate::services::{images, notes, settings, window};

use folder::{copy_atomic, ReadCache, RemoteSettings, SyncFolder};
use merge::{
    plan_note, resolve_versions, BaseAction, LocalAction, NotePlan, NoteSides, RemoteAction,
};

const PERIOD: StdDuration = StdDuration::from_secs(120);
const AFTER_CHANGE: StdDuration = StdDuration::from_secs(20);
const TICK: StdDuration = StdDuration::from_secs(3);
/// Tombstones are kept this long, so devices that were off for a while still see them.
const TOMBSTONE_DAYS: i64 = 90;
/// Common ancestor of the shared settings (JSON), stored in the local settings table.
const SETTINGS_BASE_KEY: &str = "sync.base.settings";

/// Counts of the last run, shown in the sync dialog.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncReport {
    pub uploaded: usize,
    pub downloaded: usize,
    pub deleted: usize,
    /// Notes whose text was edited on two devices; the older text became a revision.
    pub conflicts: usize,
    pub images: usize,
    /// Problems with single files (unreadable, locked…) that didn't stop the run.
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    /// "" when sync is off.
    pub folder: String,
    pub running: bool,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
    pub last_report: Option<SyncReport>,
    /// The folder synced before is gone; sync is paused until the user decides.
    pub folder_missing: bool,
}

/// The decision for one note in a run.
struct PlannedNote {
    id: String,
    plan: NotePlan,
    /// Texts of the drive's conflict copies that lost, kept as revisions.
    copy_losers: Vec<(String, String)>,
    /// Conflict copy files to remove once the note file is written.
    duplicates: Vec<PathBuf>,
}

/// Process-wide sync state (managed by Tauri).
#[derive(Default)]
pub struct SyncState {
    /// One run at a time.
    run_lock: Mutex<()>,
    status: Mutex<SyncStatus>,
    next_due: Mutex<Option<Instant>>,
    /// Set while sync applies its own changes, so they don't schedule another run.
    applying: AtomicBool,
    cache: Mutex<ReadCache>,
}

fn state(app: &AppHandle) -> tauri::State<'_, SyncState> {
    app.state::<SyncState>()
}

/// Starts the background thread (called once at startup).
pub fn start(app: &AppHandle) {
    request_now(app);
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(TICK);
        let due = state(&app)
            .next_due
            .lock()
            .map(|next| next.is_some_and(|at| at <= Instant::now()))
            .unwrap_or(false);
        if due {
            run_and_report(&app);
        }
    });
}

/// A local change happened: sync soon (but let a burst of edits settle first).
pub fn mark_dirty(app: &AppHandle) {
    let Some(sync) = app.try_state::<SyncState>() else {
        return;
    };
    if sync.applying.load(Ordering::SeqCst) {
        return;
    }
    let soon = Instant::now() + AFTER_CHANGE;
    if let Ok(mut next) = sync.next_due.lock() {
        if next.is_none_or(|at| at > soon) {
            *next = Some(soon);
        }
    };
}

pub fn request_now(app: &AppHandle) {
    if let Ok(mut next) = state(app).next_due.lock() {
        *next = Some(Instant::now());
    }
}

pub fn status(app: &AppHandle) -> SyncStatus {
    let mut status = state(app)
        .status
        .lock()
        .map(|s| s.clone())
        .unwrap_or_default();
    status.folder = settings::get(app)
        .map(|s| s.sync_folder)
        .unwrap_or_default();
    status
}

/// Turns sync on with this folder (or off with ""), then syncs right away.
/// A different folder starts like a first sync: its contents and this device's
/// notes are merged, nothing is deleted.
pub fn set_folder(app: &AppHandle, folder: &str) -> AppResult<SyncStatus> {
    if !folder.is_empty() {
        SyncFolder::open(Path::new(folder))?;
        let current = settings::get(app)?.sync_folder;
        if !same_path(&current, folder) {
            app.state::<Database>()
                .with_conn(|conn| repo::clear_all_sync_bases(conn))?;
        }
    }
    settings::update(
        app,
        &AppSettingsPatch {
            sync_folder: Some(folder.to_string()),
            ..Default::default()
        },
    )?;
    if let Ok(mut status) = state(app).status.lock() {
        *status = SyncStatus::default();
    }
    if let Ok(mut cache) = state(app).cache.lock() {
        cache.clear();
    }
    if !folder.is_empty() {
        request_now(app);
    }
    let status = status(app);
    events::sync_status(app, &status);
    Ok(status)
}

/// The folder went missing and the user chose to start it over: recreate it and
/// upload this device's notes (other devices then merge theirs back in).
pub fn recreate_folder(app: &AppHandle) -> AppResult<SyncStatus> {
    let folder = settings::get(app)?.sync_folder;
    if folder.is_empty() {
        return Err(AppError::Internal("尚未設定同步資料夾".into()));
    }
    SyncFolder::open(Path::new(&folder))?;
    app.state::<Database>()
        .with_conn(|conn| repo::clear_all_sync_bases(conn))?;
    if let Ok(mut cache) = state(app).cache.lock() {
        cache.clear();
    }
    update_status(app, |status| {
        status.folder_missing = false;
        status.last_error = None;
    });
    request_now(app);
    Ok(status(app))
}

fn same_path(a: &str, b: &str) -> bool {
    let normalize = |p: &str| p.trim_end_matches(['\\', '/']).to_lowercase();
    normalize(a) == normalize(b)
}

/// A folder that held this device's notes before but is now missing or empty
/// was most likely deleted (or the drive moved). Recreating it silently would hide
/// that, so the run stops until the user decides. It resumes by itself if the
/// folder comes back (e.g. the drive app was restarting).
fn check_folder(db: &Database, folder: &SyncFolder) -> AppResult<()> {
    let synced_before = db.with_conn(|conn| repo::has_sync_bases(conn))?;
    if synced_before && !folder.has_data()? {
        return Err(AppError::SyncFolderMissing);
    }
    folder.ensure()
}

fn run_and_report(app: &AppHandle) {
    let sync = state(app);
    if let Ok(mut next) = sync.next_due.lock() {
        *next = Some(Instant::now() + PERIOD);
    }
    if settings::get(app)
        .map(|s| s.sync_folder.is_empty())
        .unwrap_or(true)
    {
        return;
    }
    let Ok(_running) = sync.run_lock.try_lock() else {
        return;
    };

    update_status(app, |status| status.running = true);
    let result = run(app);
    update_status(app, |status| {
        status.running = false;
        match result {
            Ok(report) => {
                status.last_sync_at = Some(now());
                status.last_error = None;
                status.last_report = Some(report);
                status.folder_missing = false;
            }
            Err(err) => {
                status.folder_missing = matches!(err, AppError::SyncFolderMissing);
                status.last_error = Some(err.to_string());
            }
        }
    });
}

fn update_status(app: &AppHandle, change: impl FnOnce(&mut SyncStatus)) {
    if let Ok(mut status) = state(app).status.lock() {
        change(&mut status);
    }
    events::sync_status(app, &status(app));
}

/// One full sync. Errors on single files become warnings; only a missing or
/// unusable folder fails the run.
fn run(app: &AppHandle) -> AppResult<SyncReport> {
    let folder_path = settings::get(app)?.sync_folder;
    if folder_path.is_empty() {
        return Err(AppError::Internal("尚未設定同步資料夾".into()));
    }
    let folder = SyncFolder::locate(Path::new(&folder_path));
    let mut report = SyncReport::default();
    let db = app.state::<Database>();
    check_folder(&db, &folder).map_err(|err| match err {
        AppError::SyncFolderMissing => err,
        err => AppError::Internal(format!("無法使用同步資料夾：{err}")),
    })?;

    // 1–3. Read both sides, decide per note, apply to the database.
    let sync = state(app);
    let changes = {
        let mut cache = sync
            .cache
            .lock()
            .map_err(|_| AppError::Internal("sync cache poisoned".into()))?;
        plan_and_apply(&db, &folder, &mut cache, &mut report)?
    };

    // 4. Images both ways, before the windows show the new notes.
    report.images = sync_images(app, &folder, &mut report.warnings)?;

    // 5. Tell the windows (without scheduling another run for our own changes).
    sync.applying.store(true, Ordering::SeqCst);
    for id in &changes.upserted {
        if let Ok(note) = notes::get(app, id) {
            events::note_changed(app, &note);
            if note.status != NoteStatus::Active {
                let _ = window::close_note_window(app, id);
            } else {
                window::sync_title(app, &note);
            }
        }
    }
    for (id, content) in &changes.removed {
        let _ = window::close_note_window(app, id);
        events::note_removed(app, id);
        notes::cleanup_images_of(app, id, content);
    }
    if !changes.text_changed.is_empty() {
        events::notes_synced(app, &changes.text_changed);
    }
    let settings_result = sync_settings(app, &folder);
    sync.applying.store(false, Ordering::SeqCst);
    if let Err(err) = settings_result {
        report.warnings.push(format!("無法同步設定：{err}"));
    }

    // 6. Write the folder, then remember what was agreed for each note that made it.
    write_folder(&db, &folder, changes.plans, &changes.skipped, &mut report)?;

    // 7. Old tombstones: every device has had months to see them.
    let cutoff = timestamp(Utc::now() - Duration::days(TOMBSTONE_DAYS));
    if let Err(err) = folder.purge_tombstones(&cutoff) {
        report.warnings.push(format!("無法清理刪除紀錄：{err}"));
    }
    db.with_conn(|conn| repo::purge_tombstones(conn, &cutoff))?;

    Ok(report)
}

/// What one run changed on this device, for the windows and the folder step.
struct LocalChanges {
    plans: Vec<PlannedNote>,
    /// Edited locally while the run was going: left for the next run.
    skipped: HashSet<String>,
    upserted: Vec<String>,
    /// Upserted notes whose title / content changed (open windows reload them).
    text_changed: Vec<String>,
    /// Deleted notes with their content (for image cleanup).
    removed: Vec<(String, String)>,
}

/// Steps 1–3 of a run: read both sides, decide per note, apply to the database.
fn plan_and_apply(
    db: &Database,
    folder: &SyncFolder,
    cache: &mut ReadCache,
    report: &mut SyncReport,
) -> AppResult<LocalChanges> {
    // 1. Both sides.
    let (local_notes, bases, local_tombstones) = db.with_conn(|conn| {
        Ok((
            repo::list(conn)?,
            repo::sync_bases(conn)?,
            repo::tombstones(conn)?,
        ))
    })?;
    let local: HashMap<String, SyncedNote> = local_notes
        .iter()
        .map(|note| (note.id.clone(), SyncedNote::from(note)))
        .collect();
    let remote_notes = folder.read_notes(cache, &mut report.warnings)?;
    let remote_tombstones = folder.read_tombstones(&mut report.warnings)?;

    // 2. Decide per note.
    let ids: HashSet<&String> = local
        .keys()
        .chain(remote_notes.keys())
        .chain(bases.keys())
        .chain(local_tombstones.keys())
        .collect();
    let mut plans: Vec<PlannedNote> = Vec::new();
    for id in ids {
        // Ids become file names; an imported note could carry anything.
        if !folder::is_safe_id(id) {
            report
                .warnings
                .push(format!("略過 id 無法作為檔名的便籤：{id}"));
            continue;
        }
        let remote_entry = remote_notes.get(id);
        let versions: Vec<SyncedNote> = remote_entry
            .map(|r| {
                r.primary
                    .iter()
                    .cloned()
                    .chain(r.duplicates.iter().map(|(_, n)| n.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let (remote, copy_losers) = match resolve_versions(versions) {
            Some((winner, losers)) => (Some(winner), losers),
            None => (None, Vec::new()),
        };
        let duplicates: Vec<_> = remote_entry
            .map(|r| r.duplicates.iter().map(|(path, _)| path.clone()).collect())
            .unwrap_or_default();

        let mut plan = plan_note(NoteSides {
            local: local.get(id),
            local_tombstone: local_tombstones.get(id).map(String::as_str),
            remote: remote.as_ref(),
            remote_tombstone: remote_tombstones.get(id).map(String::as_str),
            base: bases.get(id),
        });
        // A conflict copy won, or the main file is missing: rewrite the main file.
        let primary = remote_entry.and_then(|r| r.primary.as_ref());
        if let (RemoteAction::None, Some(merged)) = (&plan.remote, &plan.merged) {
            if Some(merged) != primary {
                plan.remote = RemoteAction::Write(merged.clone());
            }
        }
        plans.push(PlannedNote {
            id: id.clone(),
            plan,
            copy_losers,
            duplicates,
        });
    }

    // 3. Apply to this device in one transaction. A note edited since step 1 is
    //    skipped this round (it is merged next time) so no typing is overwritten.
    let mut skipped: HashSet<String> = HashSet::new();
    let mut upserted: Vec<String> = Vec::new();
    let mut text_changed: Vec<String> = Vec::new();
    let mut removed: Vec<(String, String)> = Vec::new();
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        for PlannedNote {
            id,
            plan,
            copy_losers,
            ..
        } in &plans
        {
            let current = repo::find(&tx, id)?;
            if current.as_ref().map(|n| &n.modified_at) != local.get(id).map(|n| &n.modified_at) {
                skipped.insert(id.clone());
                continue;
            }
            match &plan.local {
                LocalAction::None => {}
                LocalAction::Upsert(note) => {
                    match &current {
                        Some(current) => {
                            if (&current.title, &current.content) != (&note.title, &note.content) {
                                text_changed.push(id.clone());
                            }
                            repo::apply_synced(&tx, note)?;
                        }
                        None => repo::insert_synced(&tx, note)?,
                    }
                    if current
                        .as_ref()
                        .is_none_or(|c| !SyncedNote::from(c).same_data(note))
                    {
                        report.downloaded += 1;
                    }
                    upserted.push(id.clone());
                }
                LocalAction::Delete => {
                    if let Some(current) = &current {
                        repo::delete(&tx, id)?;
                        removed.push((id.clone(), current.content.clone()));
                        report.deleted += 1;
                    }
                }
            }
            if plan.clear_local_tombstone {
                repo::remove_tombstone(&tx, id)?;
            }
            if plan.local != LocalAction::Delete && plan.merged.is_some() {
                for (title, content) in plan.revision.iter().chain(copy_losers) {
                    repo::insert_revision(&tx, id, title, content)?;
                }
                report.conflicts += usize::from(plan.revision.is_some());
            }
        }
        tx.commit()?;
        Ok(())
    })?;

    Ok(LocalChanges {
        plans,
        skipped,
        upserted,
        text_changed,
        removed,
    })
}

/// Step 6: write the folder, then remember what was agreed for each note that made it.
fn write_folder(
    db: &Database,
    folder: &SyncFolder,
    plans: Vec<PlannedNote>,
    skipped: &HashSet<String>,
    report: &mut SyncReport,
) -> AppResult<()> {
    let mut base_updates: Vec<(String, BaseAction)> = Vec::new();
    for PlannedNote {
        id,
        plan,
        duplicates,
        ..
    } in plans
    {
        if skipped.contains(&id) {
            continue;
        }
        let written = match &plan.remote {
            RemoteAction::None => Ok(()),
            RemoteAction::Write(note) => folder.write_note(note).map(|_| report.uploaded += 1),
            RemoteAction::Delete { tombstone } => folder.delete_note(&id).and_then(|_| {
                tombstone
                    .as_ref()
                    .map_or(Ok(()), |t| folder.write_tombstone(&id, t))
            }),
            RemoteAction::Tombstone(deleted_at) => folder.write_tombstone(&id, deleted_at),
        };
        match written {
            Ok(()) => {
                for path in duplicates {
                    if let Err(err) = folder.remove_file(&path) {
                        report.warnings.push(format!("無法刪除衝突副本：{err}"));
                    }
                }
                base_updates.push((id, plan.base));
            }
            Err(err) => report.warnings.push(format!("無法寫入便籤 {id}：{err}")),
        }
    }
    db.with_conn(|conn| {
        let tx = conn.transaction()?;
        for (id, base) in &base_updates {
            match base {
                BaseAction::Keep => {}
                BaseAction::Set(note) => repo::set_sync_base(&tx, note)?,
                BaseAction::Clear => repo::clear_sync_base(&tx, id)?,
            }
        }
        tx.commit()?;
        Ok(())
    })?;
    Ok(())
}

/// Copies images used by any note (or revision) to whichever side lacks them.
/// Images are never deleted from the folder: other devices may still need them.
fn sync_images(
    app: &AppHandle,
    folder: &SyncFolder,
    warnings: &mut Vec<String>,
) -> AppResult<usize> {
    let local_dir = images::images_dir(app)?;
    let remote_dir = folder.images_dir();
    let mut copied = 0;
    for name in images::referenced_by_notes(app)? {
        if !image_refs::is_safe_file_name(&name) {
            continue;
        }
        let (local, remote) = (local_dir.join(&name), remote_dir.join(&name));
        let result = match (local.is_file(), remote.is_file()) {
            (true, false) => copy_atomic(&local, &remote).map(|_| 1),
            (false, true) => copy_atomic(&remote, &local).map(|_| 1),
            _ => Ok(0),
        };
        match result {
            Ok(n) => copied += n,
            Err(err) => warnings.push(format!("無法同步圖片 {name}：{err}")),
        }
    }
    Ok(copied)
}

/// Tag colors: three-way merged per tag, like the notes.
fn sync_settings(app: &AppHandle, folder: &SyncFolder) -> AppResult<()> {
    let db = app.state::<Database>();
    let local = RemoteSettings {
        tag_colors: settings::get(app)?.tag_colors,
    };
    let base: RemoteSettings = db
        .with_conn(|conn| settings_repository::get_value(conn, SETTINGS_BASE_KEY))?
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default();
    let remote = folder.read_settings()?;

    let merged = match &remote {
        Some(remote) => RemoteSettings {
            tag_colors: merge::merge_map(&base.tag_colors, &local.tag_colors, &remote.tag_colors),
        },
        None => local.clone(),
    };
    if merged.tag_colors != local.tag_colors {
        let patch = AppSettingsPatch {
            tag_colors: Some(merged.tag_colors.clone()),
            ..Default::default()
        };
        settings::update(app, &patch)?;
    }
    if remote.as_ref() != Some(&merged) {
        folder.write_settings(&merged)?;
    }
    if base != merged {
        let json = serde_json::to_string(&merged)?;
        db.with_conn(|conn| settings_repository::set_value(conn, SETTINGS_BASE_KEY, &json))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Note, NoteColor, NoteUpdate};

    /// One computer: its own database and read cache, sharing the folder with the others.
    struct Device {
        db: Database,
        cache: ReadCache,
    }

    impl Device {
        fn new() -> Self {
            Self {
                db: Database::open_in_memory().unwrap(),
                cache: ReadCache::new(),
            }
        }

        /// Everything `run` does except the window / image / settings side effects.
        fn sync(&mut self, folder: &SyncFolder) -> SyncReport {
            let mut report = SyncReport::default();
            let changes = plan_and_apply(&self.db, folder, &mut self.cache, &mut report).unwrap();
            write_folder(
                &self.db,
                folder,
                changes.plans,
                &changes.skipped,
                &mut report,
            )
            .unwrap();
            report
        }

        fn create(&self, content: &str) -> Note {
            self.db
                .with_conn(|conn| {
                    let note = repo::insert(conn, Some((5, 5)))?;
                    repo::update(conn, &note.id, &text(content))
                })
                .unwrap()
        }

        fn update(&self, id: &str, patch: NoteUpdate) {
            // Distinct millisecond timestamps, as edits by a person would have.
            std::thread::sleep(StdDuration::from_millis(3));
            self.db
                .with_conn(|conn| repo::update(conn, id, &patch))
                .unwrap();
        }

        fn find(&self, id: &str) -> Option<Note> {
            self.db.with_conn(|conn| repo::find(conn, id)).unwrap()
        }

        fn revisions(&self, id: &str) -> Vec<String> {
            self.db
                .with_conn(|conn| repo::list_revisions(conn, id))
                .unwrap()
                .into_iter()
                .map(|r| r.content)
                .collect()
        }
    }

    fn text(content: &str) -> NoteUpdate {
        NoteUpdate {
            content: Some(content.into()),
            ..Default::default()
        }
    }

    fn shared_folder() -> (PathBuf, SyncFolder) {
        let dir = std::env::temp_dir().join(format!("pinnote-sync-run-{}", uuid::Uuid::new_v4()));
        let folder = SyncFolder::open(&dir).unwrap();
        (dir, folder)
    }

    #[test]
    fn two_devices_converge() {
        let (dir, folder) = shared_folder();
        let (mut a, mut b) = (Device::new(), Device::new());

        // New note travels A → B, without B's desktop layout.
        let note = a.create("hello");
        assert_eq!(a.sync(&folder).uploaded, 1);
        assert_eq!(b.sync(&folder).downloaded, 1);
        let on_b = b.find(&note.id).unwrap();
        assert_eq!(on_b.content, "hello");
        assert!(!on_b.is_open);
        assert_eq!(on_b.x, None);

        // Different fields on each side merge.
        b.update(&note.id, text("hello from B"));
        a.update(
            &note.id,
            NoteUpdate {
                color: Some(NoteColor::Blue),
                ..Default::default()
            },
        );
        b.sync(&folder);
        a.sync(&folder);
        b.sync(&folder);
        for device in [&a, &b] {
            let n = device.find(&note.id).unwrap();
            assert_eq!(
                (n.content.as_str(), n.color),
                ("hello from B", NoteColor::Blue)
            );
        }
        assert_eq!(
            a.find(&note.id).unwrap().x,
            Some(5),
            "layout stays per device"
        );

        // Same text edited on both: the newer wins, the older becomes a revision.
        b.update(&note.id, text("B's version"));
        a.update(&note.id, text("A's newer version"));
        b.sync(&folder);
        let report = a.sync(&folder);
        assert_eq!(report.conflicts, 1);
        assert!(a.revisions(&note.id).contains(&"B's version".to_string()));
        b.sync(&folder);
        assert_eq!(b.find(&note.id).unwrap().content, "A's newer version");

        // Nothing left to do once both agree.
        let quiet = a.sync(&folder);
        assert_eq!((quiet.uploaded, quiet.downloaded), (0, 0));

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn archive_and_permanent_delete_reach_other_devices() {
        let (dir, folder) = shared_folder();
        let (mut a, mut b) = (Device::new(), Device::new());
        let kept = a.create("archive me");
        let gone = a.create("delete me");
        a.sync(&folder);
        b.sync(&folder);

        std::thread::sleep(StdDuration::from_millis(3));
        a.db.with_conn(|conn| {
            repo::set_status(conn, &kept.id, NoteStatus::Archived)?;
            repo::delete(conn, &gone.id)?;
            repo::add_tombstone(conn, &gone.id, &now())
        })
        .unwrap();
        a.sync(&folder);
        let report = b.sync(&folder);
        assert_eq!(report.deleted, 1);
        assert_eq!(b.find(&kept.id).unwrap().status, NoteStatus::Archived);
        assert!(b.find(&gone.id).is_none());

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn edits_after_a_deletion_bring_the_note_back() {
        let (dir, folder) = shared_folder();
        let (mut a, mut b) = (Device::new(), Device::new());
        let note = a.create("v1");
        a.sync(&folder);
        b.sync(&folder);

        a.db.with_conn(|conn| {
            repo::delete(conn, &note.id)?;
            repo::add_tombstone(conn, &note.id, &now())
        })
        .unwrap();
        b.update(&note.id, text("edited later on B"));
        a.sync(&folder);
        b.sync(&folder);
        a.sync(&folder);
        assert_eq!(a.find(&note.id).unwrap().content, "edited later on B");
        assert!(b.find(&note.id).is_some());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_deleted_folder_pauses_sync_until_recreated() {
        let (dir, folder) = shared_folder();
        let mut a = Device::new();

        // Never synced: an empty folder is simply set up.
        assert!(check_folder(&a.db, &folder).is_ok());
        let note = a.create("keep me");
        a.sync(&folder);

        std::fs::remove_dir_all(&dir).unwrap();
        assert!(matches!(
            check_folder(&a.db, &folder),
            Err(AppError::SyncFolderMissing)
        ));
        assert!(!dir.exists(), "nothing is recreated behind the user's back");
        assert!(a.find(&note.id).is_some(), "local notes are untouched");

        // "重新建立並上傳": forget the old state, then a normal run uploads everything.
        a.db.with_conn(|conn| repo::clear_all_sync_bases(conn))
            .unwrap();
        assert!(check_folder(&a.db, &folder).is_ok());
        assert_eq!(a.sync(&folder).uploaded, 1);
        assert!(check_folder(&a.db, &folder).is_ok());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn same_path_ignores_case_and_trailing_separators() {
        assert!(same_path(r"G:\My Drive\Notes", r"g:\my drive\notes\"));
        assert!(!same_path(r"G:\My Drive\Notes", r"G:\My Drive\Other"));
    }

    #[test]
    fn drive_conflict_copies_are_merged_and_removed() {
        let (dir, folder) = shared_folder();
        let mut a = Device::new();
        let note = a.create("original");
        a.sync(&folder);

        // Another computer's newer version ended up as a copy, as Google Drive does.
        let notes_dir = dir.join(folder::FOLDER_NAME).join("notes");
        let mut copy = SyncedNote::from(&a.find(&note.id).unwrap());
        copy.content = "from the copy".into();
        copy.modified_at = "2099-01-01T00:00:00.000Z".into();
        let copy_path = notes_dir.join(format!("{} (1).json", note.id));
        std::fs::write(&copy_path, serde_json::to_vec(&copy).unwrap()).unwrap();

        a.sync(&folder);
        assert_eq!(a.find(&note.id).unwrap().content, "from the copy");
        assert!(a.revisions(&note.id).contains(&"original".to_string()));
        assert!(!copy_path.exists());
        let main: SyncedNote = serde_json::from_slice(
            &std::fs::read(notes_dir.join(format!("{}.json", note.id))).unwrap(),
        )
        .unwrap();
        assert_eq!(main.content, "from the copy");
        let _ = std::fs::remove_dir_all(dir);
    }
}
