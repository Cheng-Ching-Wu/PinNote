//! User-configurable global shortcuts. Each action has one accelerator,
//! stored in the `settings` table as `shortcut.<action id>` ("" = disabled).

use std::str::FromStr;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::database::{settings_repository as repo, Database};
use crate::error::{AppError, AppResult};
use crate::services::{notes, window};
use crate::state::{AppState, ShortcutStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShortcutAction {
    NewNote,
    NewFromClipboard,
    ToggleHidden,
}

impl ShortcutAction {
    pub const ALL: [Self; 3] = [Self::NewNote, Self::NewFromClipboard, Self::ToggleHidden];

    pub fn id(self) -> &'static str {
        match self {
            Self::NewNote => "new_note",
            Self::NewFromClipboard => "new_from_clipboard",
            Self::ToggleHidden => "toggle_hidden",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }

    fn label(self) -> &'static str {
        match self {
            Self::NewNote => "新增便籤",
            Self::NewFromClipboard => "從剪貼簿新增",
            Self::ToggleHidden => "隱藏 / 顯示便籤",
        }
    }

    fn default_accelerator(self) -> &'static str {
        match self {
            Self::NewNote => "Ctrl+Alt+N",
            Self::NewFromClipboard => "Ctrl+Alt+V",
            Self::ToggleHidden => "Ctrl+Alt+H",
        }
    }

    fn settings_key(self) -> String {
        format!("shortcut.{}", self.id())
    }

    fn run(self, app: &AppHandle) -> AppResult<()> {
        match self {
            Self::NewNote => notes::create_and_open(app).map(|_| ()),
            Self::NewFromClipboard => notes::create_from_clipboard(app).map(|_| ()),
            Self::ToggleHidden => window::toggle_notes_hidden(app).map(|_| ()),
        }
    }
}

/// Parses an accelerator such as "Ctrl+Alt+N" or "F15". Empty means "disabled".
///
/// A key without modifiers is only accepted for F1–F24 (e.g. keyboard macro
/// keys); a bare letter would swallow normal typing system-wide.
pub fn parse_accelerator(accelerator: &str) -> AppResult<Option<Shortcut>> {
    let accelerator = accelerator.trim();
    if accelerator.is_empty() {
        return Ok(None);
    }
    let shortcut = Shortcut::from_str(accelerator)
        .map_err(|err| AppError::Internal(format!("無法辨識的按鍵「{accelerator}」：{err}")))?;
    if shortcut.mods.is_empty() && !is_function_key(accelerator) {
        return Err(AppError::Internal(
            "單一按鍵只支援 F1–F24，其他按鍵請搭配 Ctrl / Alt / Shift / Win".into(),
        ));
    }
    Ok(Some(shortcut))
}

fn is_function_key(accelerator: &str) -> bool {
    accelerator
        .to_uppercase()
        .strip_prefix('F')
        .and_then(|n| n.parse::<u8>().ok())
        .is_some_and(|n| (1..=24).contains(&n))
}

/// The accelerator saved for an action, or its default when never configured.
fn configured(app: &AppHandle, action: ShortcutAction) -> String {
    app.state::<Database>()
        .with_conn(|conn| repo::get_value(conn, &action.settings_key()))
        .ok()
        .flatten()
        .unwrap_or_else(|| action.default_accelerator().to_string())
}

fn try_register(app: &AppHandle, action: ShortcutAction, shortcut: Shortcut) -> Result<(), String> {
    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _shortcut, event| {
            if event.state() != ShortcutState::Pressed {
                return;
            }
            if let Err(err) = action.run(app) {
                eprintln!("[pinnote] shortcut {} failed: {err}", action.id());
            }
        })
        .map_err(|err| err.to_string())
}

fn register_one(app: &AppHandle, action: ShortcutAction) -> ShortcutStatus {
    let accelerator = configured(app, action);
    let result = match parse_accelerator(&accelerator) {
        Ok(None) => Ok(false),
        Ok(Some(shortcut)) => try_register(app, action, shortcut).map(|_| true),
        Err(err) => Err(err.to_string()),
    };
    if let Err(err) = &result {
        eprintln!("[pinnote] failed to register {accelerator}: {err}");
    }
    ShortcutStatus {
        action: action.id().into(),
        label: action.label().into(),
        accelerator,
        default_accelerator: action.default_accelerator().into(),
        registered: matches!(result, Ok(true)),
        error: result.err(),
    }
}

/// (Re-)registers every action from the saved settings. Failures (e.g. the
/// combination is owned by another app) are recorded, never fatal.
pub fn register_all(app: &AppHandle) {
    let _ = app.global_shortcut().unregister_all();
    let statuses = ShortcutAction::ALL
        .into_iter()
        .map(|action| register_one(app, action))
        .collect();
    if let Ok(mut current) = app.state::<AppState>().shortcuts.lock() {
        *current = statuses;
    }
}

/// Temporarily releases every shortcut, e.g. while the user records a new one
/// (otherwise pressing the old combination would trigger it).
pub fn pause(app: &AppHandle) -> AppResult<()> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|err| AppError::Internal(err.to_string()))
}

pub fn statuses(app: &AppHandle) -> Vec<ShortcutStatus> {
    app.state::<AppState>()
        .shortcuts
        .lock()
        .map(|statuses| statuses.clone())
        .unwrap_or_default()
}

/// Changes one action's accelerator ("" disables it). On any failure the
/// previous accelerator stays active and saved.
pub fn set(app: &AppHandle, action_id: &str, accelerator: &str) -> AppResult<Vec<ShortcutStatus>> {
    let action = ShortcutAction::parse(action_id)
        .ok_or_else(|| AppError::Internal(format!("unknown shortcut action: {action_id}")))?;
    let accelerator = accelerator.trim();
    let next = parse_accelerator(accelerator)?;

    if let Some(next) = next {
        for other in ShortcutAction::ALL.into_iter().filter(|a| *a != action) {
            if parse_accelerator(&configured(app, other)).ok().flatten() == Some(next) {
                return Err(AppError::Internal(format!(
                    "「{accelerator}」已用於「{}」",
                    other.label()
                )));
            }
        }
    }

    let previous = parse_accelerator(&configured(app, action)).ok().flatten();
    if let Some(previous) = previous {
        let _ = app.global_shortcut().unregister(previous);
    }
    if let Some(next) = next {
        if let Err(err) = try_register(app, action, next) {
            if let Some(previous) = previous {
                let _ = try_register(app, action, previous);
            }
            return Err(AppError::Internal(format!(
                "「{accelerator}」無法註冊，可能已被其他程式占用（{err}）"
            )));
        }
    }

    app.state::<Database>()
        .with_conn(|conn| repo::set_value(conn, &action.settings_key(), accelerator))?;

    if let Ok(mut current) = app.state::<AppState>().shortcuts.lock() {
        if let Some(status) = current.iter_mut().find(|s| s.action == action.id()) {
            status.accelerator = accelerator.to_string();
            status.registered = next.is_some();
            status.error = None;
        }
    }
    Ok(statuses(app))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_combinations_and_function_keys() {
        assert!(parse_accelerator("Ctrl+Alt+N").unwrap().is_some());
        assert!(parse_accelerator("Ctrl+Shift+ArrowUp").unwrap().is_some());
        assert!(parse_accelerator("Super+Digit1").unwrap().is_some());
        assert!(parse_accelerator("F15").unwrap().is_some());
        assert!(parse_accelerator("Alt+F24").unwrap().is_some());
        assert!(parse_accelerator("").unwrap().is_none());
        assert_eq!(
            parse_accelerator("Ctrl+Alt+N").unwrap(),
            parse_accelerator("ctrl+alt+KeyN").unwrap()
        );
    }

    #[test]
    fn rejects_bare_typing_keys_and_garbage() {
        assert!(parse_accelerator("N").is_err());
        assert!(parse_accelerator("Space").is_err());
        assert!(parse_accelerator("F25").is_err());
        assert!(parse_accelerator("Ctrl+Nope").is_err());
    }

    #[test]
    fn action_ids_round_trip() {
        for action in ShortcutAction::ALL {
            assert_eq!(ShortcutAction::parse(action.id()), Some(action));
        }
        assert_eq!(ShortcutAction::parse("nope"), None);
    }
}
