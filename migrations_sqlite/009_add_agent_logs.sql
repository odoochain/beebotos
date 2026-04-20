-- Migration: Add agent logs table and chain transactions table

CREATE TABLE IF NOT EXISTS agent_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    level TEXT DEFAULT 'info',
    message TEXT NOT NULL,
    source TEXT,
    timestamp TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_agent_logs_agent ON agent_logs(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_logs_timestamp ON agent_logs(timestamp);

-- Chain transactions tracking (used by treasury)
CREATE TABLE IF NOT EXISTS chain_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tx_hash TEXT NOT NULL UNIQUE,
    tx_type TEXT DEFAULT 'transfer',
    to_address TEXT,
    amount TEXT,
    status TEXT DEFAULT 'pending',
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_chain_tx_hash ON chain_transactions(tx_hash);
