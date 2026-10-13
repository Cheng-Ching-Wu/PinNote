use serde::{Deserialize, Serialize};

use super::note::normalize_tags;
use super::{Note, NoteColor, NoteStatus};

/// The part of a note shared between devices — one JSON file per note in the
/// sync folder. Window geometry, pin, collapse and `is_open` stay per device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncedNote {
    pub id: String,
    pub title: String,
    pub content: String,
    pub color: NoteColor,
    #[serde(default)]
    pub tags: Vec<String>,
    pub status: NoteStatus,
    #[serde(default)]
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
    /// Last change of any field above (RFC 3339, comparable as text). Decides conflicts.
    pub modified_at: String,
}

impl SyncedNote {
    /// Same content, ignoring the timestamps that only describe it.
    pub fn same_data(&self, other: &Self) -> bool {
        self.id == other.id
            && self.title == other.title
            && self.content == other.content
            && self.color == other.color
            && self.tags == other.tags
            && self.status == other.status
            && self.deleted_at == other.deleted_at
            && self.sort_order == other.sort_order
    }

    /// Tags from another device are normalised the same way as local edits.
    pub fn normalized(mut self) -> Self {
        self.tags = normalize_tags(&self.tags);
        self
    }
}

impl From<&Note> for SyncedNote {
    fn from(note: &Note) -> Self {
        Self {
            id: note.id.clone(),
            title: note.title.clone(),
            content: note.content.clone(),
            color: note.color,
            tags: note.tags.clone(),
            status: note.status,
            deleted_at: note.deleted_at.clone(),
            sort_order: note.sort_order,
            created_at: note.created_at.clone(),
            updated_at: note.updated_at.clone(),
            modified_at: note.modified_at.clone(),
        }
    }
}

/// `deleted/<id>.json`: the note was deleted for good on some device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tombstone {
    pub id: String,
    pub deleted_at: String,
}
