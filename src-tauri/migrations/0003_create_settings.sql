-- App preferences as key/value pairs. Unknown keys are ignored, missing keys use defaults.
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
