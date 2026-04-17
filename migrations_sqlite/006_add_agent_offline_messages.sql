-- Migration: Add persistent offline message queue for agents

CREATE TABLE agent_offline_messages (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    agent_id TEXT NOT NULL,
    payload TEXT NOT NULL, -- JSON serialized UserMessageContext
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_offline_messages_agent ON agent_offline_messages(agent_id);
CREATE INDEX idx_offline_messages_created ON agent_offline_messages(created_at);
