-- Browser sandboxes table
CREATE TABLE IF NOT EXISTS browser_sandboxes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    profile_id TEXT,
    status TEXT DEFAULT 'active',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_browser_sandboxes_profile ON browser_sandboxes(profile_id);
