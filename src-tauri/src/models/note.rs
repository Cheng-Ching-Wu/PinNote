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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub id: String,

    pub title: String,
    pub content: String,

    pub color: NoteColor,

    /// Physical (device pixel) outer position of the window.
    pub x: Option<i64>,
    pub y: Option<i64>,

    /// Logical size of the window when expanded. Never overwritten by the collapsed height.
    pub width: i64,
    pub height: i64,

    pub always_on_top: bool,
    pub collapsed: bool,
    pub is_open: bool,

    pub status: NoteStatus,

    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

/// Partial update sent from the frontend. `None` fields are left untouched.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteUpdate {
    pub title: Option<String>,
    pub content: Option<String>,
    pub color: Option<NoteColor>,

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
