-- Initial database migration for BeeBotOS (SQLite version)
-- Creates core tables for agents, sessions, and system state

-- Agents table
CREATE TABLE agents (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    description TEXT,
    owner_address TEXT,
    status TEXT DEFAULT 'inactive',
    config TEXT, -- JSON stored as TEXT
    capabilities TEXT, -- JSON array stored as TEXT
    model_provider TEXT,
    model_name TEXT,
    owner_id TEXT,
    last_heartbeat TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    last_active_at TEXT,
    metadata TEXT -- JSON stored as TEXT
);

CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_agents_owner ON agents(owner_address);
CREATE INDEX idx_agents_owner_id ON agents(owner_id);

-- Agent status history table
CREATE TABLE agent_status_history (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    reason TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_agent_status_history_agent ON agent_status_history(agent_id);

-- Sessions table
CREATE TABLE sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    name TEXT,
    status TEXT DEFAULT 'active',
    context TEXT, -- JSON stored as TEXT
    encryption_key_hash TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    expires_at TEXT,
    archived_at TEXT
);

CREATE INDEX idx_sessions_agent ON sessions(agent_id);
CREATE INDEX idx_sessions_status ON sessions(status);

-- Transactions table
CREATE TABLE transactions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    tx_hash TEXT UNIQUE,
    from_address TEXT,
    to_address TEXT,
    amount TEXT, -- Large number stored as TEXT
    token_address TEXT,
    status TEXT DEFAULT 'pending',
    gas_used INTEGER,
    gas_price TEXT, -- Large number stored as TEXT
    block_number INTEGER,
    data TEXT, -- JSON stored as TEXT
    created_at TEXT DEFAULT (datetime('now')),
    confirmed_at TEXT
);

CREATE INDEX idx_transactions_from ON transactions(from_address);
CREATE INDEX idx_transactions_to ON transactions(to_address);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_block ON transactions(block_number);

-- Events table
CREATE TABLE events (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    event_type TEXT NOT NULL,
    source TEXT,
    agent_id TEXT REFERENCES agents(id),
    payload TEXT, -- JSON stored as TEXT
    severity TEXT DEFAULT 'info',
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_agent ON events(agent_id);
CREATE INDEX idx_events_created ON events(created_at);

-- Tasks table
CREATE TABLE tasks (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    agent_id TEXT NOT NULL REFERENCES agents(id),
    session_id TEXT REFERENCES sessions(id),
    task_type TEXT NOT NULL,
    input TEXT, -- JSON stored as TEXT
    output TEXT, -- JSON stored as TEXT
    status TEXT DEFAULT 'pending',
    priority INTEGER DEFAULT 0,
    started_at TEXT,
    completed_at TEXT,
    error_message TEXT,
    retries INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_tasks_agent ON tasks(agent_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_created ON tasks(created_at);

-- Memories table (without vector embedding for SQLite)
CREATE TABLE memories (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    agent_id TEXT NOT NULL REFERENCES agents(id),
    memory_type TEXT NOT NULL,
    content TEXT NOT NULL,
    embedding TEXT, -- Vector stored as TEXT for SQLite (no native support)
    importance REAL DEFAULT 0.5,
    metadata TEXT, -- JSON stored as TEXT
    retrieval_count INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    last_accessed_at TEXT,
    expires_at TEXT
);

CREATE INDEX idx_memories_agent ON memories(agent_id);
CREATE INDEX idx_memories_type ON memories(memory_type);

-- Skills table
CREATE TABLE skills (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    description TEXT,
    author TEXT,
    category TEXT,
    runtime_type TEXT,
    entrypoint TEXT,
    schema TEXT, -- JSON stored as TEXT
    config_schema TEXT, -- JSON stored as TEXT
    hash TEXT,
    verified INTEGER DEFAULT 0, -- BOOLEAN as INTEGER in SQLite
    downloads INTEGER DEFAULT 0,
    rating REAL,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_skills_category ON skills(category);
CREATE INDEX idx_skills_runtime ON skills(runtime_type);

-- Agent-Skill installations
CREATE TABLE agent_skills (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    config TEXT, -- JSON stored as TEXT
    installed_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now')),
    active INTEGER DEFAULT 1, -- BOOLEAN as INTEGER in SQLite
    UNIQUE(agent_id, skill_id)
);

CREATE INDEX idx_agent_skills_agent ON agent_skills(agent_id);

-- DAO Proposals
CREATE TABLE dao_proposals (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    proposal_id INTEGER UNIQUE,
    proposer TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    targets TEXT, -- JSON stored as TEXT
    `values` TEXT, -- JSON stored as TEXT (escaped reserved keyword)
    calldatas TEXT, -- JSON stored as TEXT
    status TEXT DEFAULT 'pending',
    for_votes TEXT DEFAULT '0', -- Large number stored as TEXT
    against_votes TEXT DEFAULT '0',
    abstain_votes TEXT DEFAULT '0',
    start_block INTEGER,
    end_block INTEGER,
    executed_at TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_dao_proposals_status ON dao_proposals(status);
CREATE INDEX idx_dao_proposals_proposer ON dao_proposals(proposer);

-- DAO Votes
CREATE TABLE dao_votes (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    proposal_id INTEGER NOT NULL REFERENCES dao_proposals(proposal_id),
    voter TEXT NOT NULL,
    support INTEGER NOT NULL, -- SMALLINT as INTEGER
    weight TEXT, -- Large number stored as TEXT
    reason TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    UNIQUE(proposal_id, voter)
);

CREATE INDEX idx_dao_votes_proposal ON dao_votes(proposal_id);
CREATE INDEX idx_dao_votes_voter ON dao_votes(voter);

-- System settings
CREATE TABLE system_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL, -- JSON stored as TEXT
    updated_at TEXT DEFAULT (datetime('now'))
);

-- Update timestamps trigger for agents
CREATE TRIGGER update_agents_updated_at AFTER UPDATE ON agents
BEGIN
    UPDATE agents SET updated_at = datetime('now') WHERE id = NEW.id;
END;

-- Update timestamps trigger for sessions
CREATE TRIGGER update_sessions_updated_at AFTER UPDATE ON sessions
BEGIN
    UPDATE sessions SET updated_at = datetime('now') WHERE id = NEW.id;
END;
