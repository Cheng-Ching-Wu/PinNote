-- Position in the manager list when sorting by hand; smaller comes first.
-- Existing notes start in their "last modified" order.
ALTER TABLE notes ADD COLUMN sort_order INTEGER NOT NULL DEFAULT 0;

UPDATE notes SET sort_order = (
    SELECT COUNT(*) FROM notes AS newer WHERE newer.updated_at > notes.updated_at
);
