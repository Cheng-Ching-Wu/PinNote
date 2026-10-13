-- Folder sync (see `services/sync`).

-- When any synced field (title, content, color, tags, status, sort order) last
-- changed. Unlike `updated_at` it also moves on archive / delete / reorder.
ALTER TABLE notes ADD COLUMN modified_at TEXT NOT NULL DEFAULT '';
UPDATE notes SET modified_at = updated_at;

-- The synced fields of each note as they were after the last successful sync
-- (JSON of `SyncedNote`): the common ancestor for three-way merges.
-- No foreign key: it must outlive a local deletion until that is synced.
CREATE TABLE IF NOT EXISTS sync_base (
    note_id TEXT PRIMARY KEY,
    data TEXT NOT NULL
);

-- Notes deleted for good on this device, waiting to be written to the sync folder.
CREATE TABLE IF NOT EXISTS sync_tombstones (
    note_id TEXT PRIMARY KEY,
    deleted_at TEXT NOT NULL
);
