-- Tags of a note as a JSON array of strings, e.g. ["工作","待辦"].
-- Normalised in Rust (`models::note::normalize_tags`) before being written.
ALTER TABLE notes ADD COLUMN tags TEXT NOT NULL DEFAULT '[]';
