use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutStatus {
    pub accelerator: String,
    pub registered: bool,
    pub error: Option<String>,
}

/// Process-wide runtime state (not persisted).
pub struct AppState {
    /// Set right before the app exits so window-close handlers can tell an
    /// explicit "close this note" apart from shutting the whole app down.
    quitting: AtomicBool,
    pub shortcut: Mutex<ShortcutStatus>,
}

impl AppState {
    pub fn is_quitting(&self) -> bool {
        self.quitting.load(Ordering::SeqCst)
    }

    pub fn set_quitting(&self) {
        self.quitting.store(true, Ordering::SeqCst);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            quitting: AtomicBool::new(false),
            shortcut: Mutex::new(ShortcutStatus {
                accelerator: String::new(),
                registered: false,
                error: None,
            }),
        }
    }
}
