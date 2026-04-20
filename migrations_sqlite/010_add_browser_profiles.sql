-- Migration: Add browser profiles and instances tables for automation

CREATE TABLE IF NOT EXISTS browser_profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT DEFAULT '#3b82f6',
    cdp_port INTEGER DEFAULT 9222,
    headless INTEGER DEFAULT 1,
    proxy TEXT,
    user_agent TEXT,
    viewport_width INTEGER DEFAULT 1920,
    viewport_height INTEGER DEFAULT 1080,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS browser_instances (
    id TEXT PRIMARY KEY,
    profile_id TEXT REFERENCES browser_profiles(id) ON DELETE SET NULL,
    status TEXT DEFAULT 'disconnected',
    current_url TEXT DEFAULT 'about:blank',
    connected_at TEXT,
    disconnected_at TEXT
);
