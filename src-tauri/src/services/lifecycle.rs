//! App startup, window-close handling and shutdown.

use tauri::{AppHandle, Manager, Window, WindowEvent};

use crate::database::{backup, note_repository as repo, Database};
use crate::error::AppResult;
use crate::services::{notes, shortcut, transfer, tray, window};
use crate::state::AppState;

const DB_FILE: &str = "pinnote.db";

/// Passed by the OS login item (see `tauri_plugin_autostart::init` in lib.rs).
pub const AUTOSTART_ARG: &str = "--autostart";

/// 初始化 → 每日備份 → migrations → 載入 active notes → Main Window → Note Windows → Tray → Global Shortcut
pub fn startup(app: &AppHandle) -> AppResult<()> {
    let db_path = app.path().app_data_dir()?.join(DB_FILE);
    // A failed backup must not keep the user from their notes.
    if let Err(err) =
        backup::daily_backup(&db_path, &transfer::backup_dir(app)?, backup::KEEP_BACKUPS)
    {
        eprintln!("[pinnote] daily backup failed: {err}");
    }
    let db = Database::open(&db_path)?;
    let open_notes = db.with_conn(|conn| repo::list_open(conn))?;
    app.manage(db);

    // At login only the desktop notes come back; the manager stays in the tray.
    let launched_at_login = std::env::args().any(|arg| arg == AUTOSTART_ARG);
    if !launched_at_login {
        window::show_main_window(app)?;
    }

    for note in &open_notes {
        if let Err(err) = window::open_note_window(app, note, false) {
            eprintln!("[pinnote] failed to restore note {}: {err}", note.id);
        }
    }

    // Tray and shortcut failures must not prevent the app from running.
    if let Err(err) = tray::init(app) {
        eprintln!("[pinnote] failed to create tray icon: {err}");
    }
    shortcut::register_all(app);

    Ok(())
}

pub fn handle_window_event(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    let app = window.app_handle();
    if app.state::<AppState>().is_quitting() {
        return;
    }

    let label = window.label();
    if label == window::MAIN_LABEL {
        // The manager window only hides; the app keeps living in the tray.
        api.prevent_close();
        let _ = window.hide();
    } else if let Some(id) = window::note_id_from_label(label) {
        // Closing a note window is not deleting it — just remember it's closed.
        // Persist geometry first so a move right before closing isn't lost.
        if let Err(err) =
            persist_geometry(app, id).and_then(|_| notes::set_open(app, id, false).map(|_| ()))
        {
            eprintln!("[pinnote] failed to record close of note {id}: {err}");
        }
    }
}

pub fn quit(app: &AppHandle) {
    app.state::<AppState>().set_quitting();
    // Debounced saves in the note windows may not have fired yet.
    for label in app.webview_windows().keys() {
        if let Some(id) = window::note_id_from_label(label) {
            if let Err(err) = persist_geometry(app, id) {
                eprintln!("[pinnote] failed to save geometry of note {id}: {err}");
            }
        }
    }
    app.exit(0);
}

fn persist_geometry(app: &AppHandle, id: &str) -> AppResult<()> {
    let Some(webview) = app.get_webview_window(&window::note_label(id)) else {
        return Ok(());
    };
    let note = notes::get(app, id)?;
    if let Some(patch) = window::capture_geometry(&webview, note.collapsed)? {
        notes::update(app, id, &patch)?;
    }
    Ok(())
}
