-- Whether the note's desktop window was open when last seen.
-- Closing a note window (×) sets this to 0 so it is not reopened on next launch;
-- quitting the app keeps it at 1 so the desktop layout is restored as-is.
ALTER TABLE notes ADD COLUMN is_open INTEGER NOT NULL DEFAULT 1;

CREATE INDEX IF NOT EXISTS idx_notes_status_updated_at ON notes (status, updated_at DESC);
