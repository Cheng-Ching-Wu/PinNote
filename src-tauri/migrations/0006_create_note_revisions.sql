-- Earlier versions of a note's title / content, written while it is being edited
-- (see `note_repository::snapshot_if_due`). Removed with the note.
CREATE TABLE IF NOT EXISTS note_revisions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    note_id TEXT NOT NULL REFERENCES notes (id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_note_revisions_note_created ON note_revisions (note_id, created_at DESC);
