use serde::{Deserialize, Serialize};

pub const DEFAULT_WIDTH: i64 = 320;
pub const DEFAULT_HEIGHT: i64 = 280;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteStatus {
    Active,
    Archived,
    Deleted,
}

impl NoteStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
            Self::Deleted => "deleted",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "archived" => Self::Archived,
            "deleted" => Self::Deleted,
            _ => Self::Active,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoteColor {
    Yellow,
    Blue,
    Green,
    Pink,
    Gray,
}

impl NoteColor {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Yellow => "yellow",
            Self::Blue => "blue",
            Self::Green => "green",
            Self::Pink => "pink",
            Self::Gray => "gray",
        }
    }

    /// Unknown values (e.g. written by a future version) fall back to yellow.
    pub fn parse(value: &str) -> Self {
        match value {
            "blue" => Self::Blue,
            "green" => Self::Green,
            "pink" => Self::Pink,
            "gray" => Self::Gray,
            _ => Self::Yellow,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,

    pub title: String,
    pub content: String,

    pub color: NoteColor,

    /// Normalised with `normalize_tags`. Missing in exports made before tags existed.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Physical (device pixel) outer position of the window.
    pub x: Option<i64>,
    pub y: Option<i64>,

    /// Logical size of the window when expanded. Never overwritten by the collapsed height.
    pub width: i64,
    pub height: i64,

    pub always_on_top: bool,
    pub collapsed: bool,
    #[serde(default)]
    pub is_open: bool,

    pub status: NoteStatus,

    /// Manual position in the manager list (smaller first). Changed only through `reorder`.
    #[serde(default)]
    pub sort_order: i64,

    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    /// Last change of any synced field (see `SyncedNote`); maintained by the repository.
    #[serde(default)]
    pub modified_at: String,
}

/// Partial update sent from the frontend. `None` fields are left untouched.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteUpdate {
    pub title: Option<String>,
    pub content: Option<String>,
    pub color: Option<NoteColor>,
    pub tags: Option<Vec<String>>,

    pub x: Option<i64>,
    pub y: Option<i64>,
    pub width: Option<i64>,
    pub height: Option<i64>,

    pub always_on_top: Option<bool>,
    pub collapsed: Option<bool>,
}

impl NoteUpdate {
    /// Applies the patch. Returns true when user-visible content changed,
    /// which is what `updated_at` (and the list ordering) should reflect.
    /// Window geometry / pin / collapse are layout state and do not bump it.
    pub fn apply_to(&self, note: &mut Note) -> bool {
        let mut content_changed = false;

        if let Some(title) = &self.title {
            content_changed |= *title != note.title;
            note.title = title.clone();
        }
        if let Some(content) = &self.content {
            content_changed |= *content != note.content;
            note.content = content.clone();
        }
        if let Some(color) = self.color {
            content_changed |= color != note.color;
            note.color = color;
        }
        if let Some(tags) = &self.tags {
            let tags = normalize_tags(tags);
            content_changed |= tags != note.tags;
            note.tags = tags;
        }

        if let Some(x) = self.x {
            note.x = Some(x);
        }
        if let Some(y) = self.y {
            note.y = Some(y);
        }
        if let Some(width) = self.width {
            note.width = width.max(1);
        }
        if let Some(height) = self.height {
            note.height = height.max(1);
        }
        if let Some(always_on_top) = self.always_on_top {
            note.always_on_top = always_on_top;
        }
        if let Some(collapsed) = self.collapsed {
            note.collapsed = collapsed;
        }

        content_changed
    }
}

/// Longest tag name, in characters. Longer names are cut.
pub const MAX_TAG_LENGTH: usize = 30;

/// Cleans a single tag name: no leading `#`, single spaces, at most
/// `MAX_TAG_LENGTH` characters. Returns `None` when nothing is left.
/// Mirrored by `normalizeTag` in `src/utils/tags.ts`.
pub fn normalize_tag(raw: &str) -> Option<String> {
    let name = raw.trim().trim_start_matches('#');
    let name: String = name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(MAX_TAG_LENGTH)
        .collect();
    let name = name.trim_end();
    (!name.is_empty()).then(|| name.to_string())
}

/// Tags are compared without case: "Work" and "work" are the same tag.
pub fn tag_key(name: &str) -> String {
    name.to_lowercase()
}

/// Normalises every tag and drops duplicates, keeping the first spelling and the order.
pub fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    tags.iter()
        .filter_map(|tag| normalize_tag(tag))
        .filter(|tag| seen.insert(tag_key(tag)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn normalize_tag_cleans_names() {
        assert_eq!(normalize_tag("  #工作  "), Some("工作".into()));
        assert_eq!(normalize_tag("a   b\tc"), Some("a b c".into()));
        assert_eq!(normalize_tag("##"), None);
        assert_eq!(normalize_tag("   "), None);
        let long = "字".repeat(40);
        assert_eq!(
            normalize_tag(&long).unwrap().chars().count(),
            MAX_TAG_LENGTH
        );
    }

    #[test]
    fn normalize_tags_dedupes_without_case() {
        assert_eq!(
            normalize_tags(&strings(&["Work", "", "待辦", "work", "#待辦", "Home"])),
            strings(&["Work", "待辦", "Home"])
        );
    }
}

/// An earlier version of a note's title and content.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteRevision {
    pub id: i64,
    pub note_id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
}
