-- Migration: Add user settings table for per-user preferences
-- Supports theme, language, notifications, and other UI preferences

CREATE TABLE IF NOT EXISTS user_settings (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    theme TEXT DEFAULT 'dark',
    language TEXT DEFAULT 'en',
    notifications_enabled INTEGER DEFAULT 1,
    auto_update INTEGER DEFAULT 1,
    api_endpoint TEXT,
    wallet_address TEXT,
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_user_settings_user ON user_settings(user_id);

-- Trigger to auto-update updated_at
CREATE TRIGGER IF NOT EXISTS update_user_settings_updated_at AFTER UPDATE ON user_settings
BEGIN
    UPDATE user_settings SET updated_at = datetime('now') WHERE user_id = NEW.user_id;
END;
