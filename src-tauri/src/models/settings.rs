use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const PINNED_TRANSLUCENT: &str = "pinned_translucent";

/// User preferences stored in the `settings` table.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Pinned (always-on-top) notes turn translucent while not focused / hovered.
    pub pinned_translucent: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            pinned_translucent: true,
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
        }
    }
}

/// Partial update from the frontend. `None` fields are left untouched.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsPatch {
    pub pinned_translucent: Option<bool>,
}

impl AppSettingsPatch {
    pub fn entries(&self) -> Vec<(&'static str, String)> {
        let mut entries = Vec::new();
        if let Some(value) = self.pinned_translucent {
            entries.push((PINNED_TRANSLUCENT, value.to_string()));
        }
        entries
    }
}
