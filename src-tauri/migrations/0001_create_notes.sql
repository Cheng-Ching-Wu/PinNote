CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,

    title TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',

    color TEXT NOT NULL DEFAULT 'yellow',

    x INTEGER,
    y INTEGER,

    width INTEGER NOT NULL DEFAULT 320,
    height INTEGER NOT NULL DEFAULT 280,

    always_on_top INTEGER NOT NULL DEFAULT 0,
    collapsed INTEGER NOT NULL DEFAULT 0,

    status TEXT NOT NULL DEFAULT 'active',

    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    deleted_at TEXT
);
