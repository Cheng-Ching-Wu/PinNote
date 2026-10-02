use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

use crate::services::notes;
use crate::state::{AppState, ShortcutStatus};

const NEW_NOTE_ACCELERATOR: &str = "Ctrl+Alt+N";

/// Registers Ctrl+Alt+N. A failure (e.g. the combination is owned by another
/// app) is recorded and surfaced in the UI instead of aborting startup.
pub fn register(app: &AppHandle) {
    let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyN);

    let result = app
        .global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            if let Err(err) = notes::create_and_open(app) {
                eprintln!("[pinnote] shortcut: failed to create note: {err}");
            }
        });

    let status = match result {
        Ok(()) => ShortcutStatus {
            accelerator: NEW_NOTE_ACCELERATOR.into(),
            registered: true,
            error: None,
        },
        Err(err) => {
            eprintln!("[pinnote] failed to register {NEW_NOTE_ACCELERATOR}: {err}");
            ShortcutStatus {
                accelerator: NEW_NOTE_ACCELERATOR.into(),
                registered: false,
                error: Some(err.to_string()),
            }
        }
    };

    if let Ok(mut current) = app.state::<AppState>().shortcut.lock() {
        *current = status;
    }
}
