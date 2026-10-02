use tauri::State;

use crate::state::{AppState, ShortcutStatus};

#[tauri::command]
pub fn get_shortcut_status(state: State<'_, AppState>) -> Option<ShortcutStatus> {
    state.shortcut.lock().ok().map(|status| status.clone())
}
