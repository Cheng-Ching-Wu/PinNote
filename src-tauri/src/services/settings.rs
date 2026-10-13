use tauri::{AppHandle, Manager};

use crate::database::{settings_repository as repo, Database};
use crate::error::AppResult;
use crate::events;
use crate::models::{AppSettings, AppSettingsPatch};
use crate::services::notes;

pub fn get(app: &AppHandle) -> AppResult<AppSettings> {
    app.state::<Database>().with_conn(|conn| repo::get(conn))
}

/// Persists the patch and broadcasts the result so every note window reacts.
pub fn update(app: &AppHandle, patch: &AppSettingsPatch) -> AppResult<AppSettings> {
    let settings = app
        .state::<Database>()
        .with_conn(|conn| repo::update(conn, patch))?;
    events::settings_changed(app, &settings);
    if patch.trash_retention_days.is_some() {
        if let Err(err) = notes::purge_expired_trash(app) {
            eprintln!("[pinnote] failed to empty expired trash: {err}");
        }
    }
    Ok(settings)
}
