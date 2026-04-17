-- Migration: Add User-Agent-Channel binding tables (SQLite version)
-- Supports multi-instance channels per user and multi-channel per agent

-- User channel bindings (supports multiple instances per platform per user)
CREATE TABLE user_channels (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform TEXT NOT NULL,
    instance_name TEXT NOT NULL DEFAULT 'default',
    platform_user_id TEXT,
    config_encrypted TEXT NOT NULL,
    status TEXT DEFAULT 'active',
    webhook_path TEXT UNIQUE,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    UNIQUE(user_id, platform, instance_name)
);

CREATE INDEX idx_user_channels_user ON user_channels(user_id);
CREATE INDEX idx_user_channels_platform ON user_channels(platform);
CREATE INDEX idx_user_channels_webhook ON user_channels(webhook_path);
CREATE INDEX idx_user_channels_platform_user ON user_channels(platform, platform_user_id);

-- Agent to channel bindings (N channels -> 1 agent)
CREATE TABLE agent_channel_bindings (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    user_channel_id TEXT NOT NULL REFERENCES user_channels(id) ON DELETE CASCADE,
    binding_name TEXT,
    is_default INTEGER DEFAULT 0,
    priority INTEGER DEFAULT 0,
    routing_rules TEXT DEFAULT '{}',
    created_at TEXT DEFAULT (datetime('now')),
    UNIQUE(agent_id, user_channel_id)
);

CREATE INDEX idx_acb_agent ON agent_channel_bindings(agent_id);
CREATE INDEX idx_acb_channel ON agent_channel_bindings(user_channel_id);
CREATE INDEX idx_acb_default ON agent_channel_bindings(user_channel_id, is_default);

-- Trigger: update user_channels timestamp on modification
CREATE TRIGGER update_user_channels_updated_at AFTER UPDATE ON user_channels
BEGIN
    UPDATE user_channels SET updated_at = datetime('now') WHERE id = NEW.id;
END;
