use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Serialize;

/// Registration outcome of one global shortcut.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutStatus {
    /// Stable id, e.g. "new_note".
    pub action: String,
    pub label: String,
    /// "" when the user disabled this shortcut.
    pub accelerator: String,
    pub default_accelerator: String,
    pub registered: bool,
    pub error: Option<String>,
}

/// Process-wide runtime state (not persisted).
#[derive(Default)]
pub struct AppState {
    /// Set right before the app exits so window-close handlers can tell an
    /// explicit "close this note" apart from shutting the whole app down.
    quitting: AtomicBool,
    /// Note windows are temporarily hidden (Ctrl+Alt+H). They stay open, so
    /// this is never persisted — a restart always shows them again.
    notes_hidden: AtomicBool,
    pub shortcuts: Mutex<Vec<ShortcutStatus>>,
}

impl AppState {
    pub fn is_quitting(&self) -> bool {
        self.quitting.load(Ordering::SeqCst)
    }

    pub fn set_quitting(&self) {
        self.quitting.store(true, Ordering::SeqCst);
    }

    pub fn notes_hidden(&self) -> bool {
        self.notes_hidden.load(Ordering::SeqCst)
    }

    pub fn set_notes_hidden(&self, hidden: bool) {
        self.notes_hidden.store(hidden, Ordering::SeqCst);
    }
}
