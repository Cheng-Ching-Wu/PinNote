use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

const PINNED_TRANSLUCENT: &str = "pinned_translucent";
const TAG_COLORS: &str = "tag_colors";
const TRASH_RETENTION_DAYS: &str = "trash_retention_days";
const SYNC_FOLDER: &str = "sync_folder";
const LANGUAGE: &str = "language";
const AUTO_CHECK_UPDATES: &str = "auto_check_updates";
const DISMISSED_UPDATE: &str = "dismissed_update";
const SNAP_WINDOWS: &str = "snap_windows";

/// Follow the OS language (see `crate::i18n`).
pub const DEFAULT_LANGUAGE: &str = "system";

/// Notes stay in the trash this many days before being deleted for good.
pub const DEFAULT_TRASH_RETENTION_DAYS: u32 = 30;

/// Tag key (`note::tag_key`) → color name. The palette lives in the frontend
/// (`src/constants/tagColors.ts`); tags without an entry get an automatic color.
pub type TagColors = BTreeMap<String, String>;

/// User preferences stored in the `settings` table.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Pinned (always-on-top) notes turn translucent while not focused / hovered.
    pub pinned_translucent: bool,
    pub tag_colors: TagColors,
    /// 0 keeps trashed notes until they are deleted by hand.
    pub trash_retention_days: u32,
    /// Folder shared through a cloud drive; "" = sync off. Per device, never synced itself.
    pub sync_folder: String,
    /// "system" | "zh-TW" | "en". Per device, never synced.
    pub language: String,
    /// Look for a new version at startup and once a day.
    pub auto_check_updates: bool,
    /// Version whose update notice was dismissed ("later"); "" = none.
    pub dismissed_update: String,
    /// Dragged / resized notes snap to other notes and the screen edges. Per device, never synced.
    pub snap_windows: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            pinned_translucent: true,
            tag_colors: TagColors::new(),
            trash_retention_days: DEFAULT_TRASH_RETENTION_DAYS,
            sync_folder: String::new(),
            language: DEFAULT_LANGUAGE.into(),
            auto_check_updates: true,
            dismissed_update: String::new(),
            snap_windows: true,
        }
    }
}

impl AppSettings {
    pub fn from_map(map: &HashMap<String, String>) -> Self {
        let defaults = Self::default();
        Self {
            pinned_translucent: map
                .get(PINNED_TRANSLUCENT)
                .map_or(defaults.pinned_translucent, |v| v == "true"),
            tag_colors: map
                .get(TAG_COLORS)
                .and_then(|v| serde_json::from_str(v).ok())
                .unwrap_or(defaults.tag_colors),
            trash_retention_days: map
                .get(TRASH_RETENTION_DAYS)
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.trash_retention_days),
            sync_folder: map.get(SYNC_FOLDER).cloned().unwrap_or_default(),
            language: map.get(LANGUAGE).cloned().unwrap_or(defaults.language),
            auto_check_updates: map
                .get(AUTO_CHECK_UPDATES)
                .map_or(defaults.auto_check_updates, |v| v == "true"),
            dismissed_update: map.get(DISMISSED_UPDATE).cloned().unwrap_or_default(),
            snap_windows: map
                .get(SNAP_WINDOWS)
                .map_or(defaults.snap_windows, |v| v == "true"),
        }
    }
}

/// Partial update from the frontend. `None` fields are left untouched.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsPatch {
    pub pinned_translucent: Option<bool>,
    /// Replaces the whole map.
    pub tag_colors: Option<TagColors>,
    pub trash_retention_days: Option<u32>,
    pub sync_folder: Option<String>,
    pub language: Option<String>,
    pub auto_check_updates: Option<bool>,
    pub dismissed_update: Option<String>,
    pub snap_windows: Option<bool>,
}

impl AppSettingsPatch {
    pub fn entries(&self) -> Vec<(&'static str, String)> {
        let mut entries = Vec::new();
        if let Some(value) = self.pinned_translucent {
            entries.push((PINNED_TRANSLUCENT, value.to_string()));
        }
        if let Some(colors) = &self.tag_colors {
            let json = serde_json::to_string(colors).unwrap_or_else(|_| "{}".into());
            entries.push((TAG_COLORS, json));
        }
        if let Some(days) = self.trash_retention_days {
            entries.push((TRASH_RETENTION_DAYS, days.to_string()));
        }
        if let Some(folder) = &self.sync_folder {
            entries.push((SYNC_FOLDER, folder.clone()));
        }
        if let Some(language) = &self.language {
            entries.push((LANGUAGE, language.clone()));
        }
        if let Some(enabled) = self.auto_check_updates {
            entries.push((AUTO_CHECK_UPDATES, enabled.to_string()));
        }
        if let Some(version) = &self.dismissed_update {
            entries.push((DISMISSED_UPDATE, version.clone()));
        }
        if let Some(enabled) = self.snap_windows {
            entries.push((SNAP_WINDOWS, enabled.to_string()));
        }
        entries
    }
}
