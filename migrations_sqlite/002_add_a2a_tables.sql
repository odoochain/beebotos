-- Migration: Add A2A protocol tables (SQLite version)

-- A2A Deals
CREATE TABLE a2a_deals (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    deal_id TEXT UNIQUE,
    buyer_agent_id TEXT NOT NULL REFERENCES agents(id),
    seller_agent_id TEXT NOT NULL REFERENCES agents(id),
    task_hash TEXT,
    amount TEXT NOT NULL, -- Large number stored as TEXT
    token_address TEXT,
    escrow_address TEXT,
    status TEXT DEFAULT 'pending',
    deadline TEXT,
    created_at TEXT DEFAULT (datetime('now')),
    funded_at TEXT,
    completed_at TEXT,
    dispute_id TEXT
);

CREATE INDEX idx_a2a_deals_buyer ON a2a_deals(buyer_agent_id);
CREATE INDEX idx_a2a_deals_seller ON a2a_deals(seller_agent_id);
CREATE INDEX idx_a2a_deals_status ON a2a_deals(status);

-- A2A Negotiations
CREATE TABLE a2a_negotiations (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    negotiation_id TEXT UNIQUE,
    initiator_agent_id TEXT NOT NULL REFERENCES agents(id),
    responder_agent_id TEXT NOT NULL REFERENCES agents(id),
    intent TEXT NOT NULL, -- JSON stored as TEXT
    current_terms TEXT, -- JSON stored as TEXT
    state TEXT DEFAULT 'offering',
    round_number INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now')),
    expires_at TEXT,
    deal_id TEXT REFERENCES a2a_deals(id)
);

CREATE INDEX idx_a2a_negotiations_initiator ON a2a_negotiations(initiator_agent_id);
CREATE INDEX idx_a2a_negotiations_responder ON a2a_negotiations(responder_agent_id);

-- A2A Capabilities registry
CREATE TABLE a2a_capabilities (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    capability_uri TEXT NOT NULL,
    name TEXT,
    version TEXT,
    description TEXT,
    input_schema TEXT, -- JSON stored as TEXT
    output_schema TEXT, -- JSON stored as TEXT
    pricing_model TEXT,
    price TEXT, -- Large number stored as TEXT
    active INTEGER DEFAULT 1, -- BOOLEAN as INTEGER in SQLite
    updated_at TEXT DEFAULT (datetime('now')),
    UNIQUE(agent_id, capability_uri)
);

CREATE INDEX idx_a2a_capabilities_agent ON a2a_capabilities(agent_id);
CREATE INDEX idx_a2a_capabilities_uri ON a2a_capabilities(capability_uri);

-- A2A Messages
CREATE TABLE a2a_messages (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    message_id TEXT UNIQUE,
    from_agent_id TEXT NOT NULL REFERENCES agents(id),
    to_agent_id TEXT NOT NULL REFERENCES agents(id),
    message_type TEXT NOT NULL,
    correlation_id TEXT,
    payload TEXT NOT NULL, -- JSON stored as TEXT
    signature TEXT,
    received_at TEXT DEFAULT (datetime('now')),
    processed_at TEXT
);

CREATE INDEX idx_a2a_messages_from ON a2a_messages(from_agent_id);
CREATE INDEX idx_a2a_messages_to ON a2a_messages(to_agent_id);
CREATE INDEX idx_a2a_messages_correlation ON a2a_messages(correlation_id);
CREATE INDEX idx_a2a_messages_received ON a2a_messages(received_at);

-- Agent reputation
CREATE TABLE agent_reputation (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    agent_id TEXT NOT NULL REFERENCES agents(id) UNIQUE,
    overall_score REAL DEFAULT 0,
    successful_deals INTEGER DEFAULT 0,
    failed_deals INTEGER DEFAULT 0,
    total_volume TEXT DEFAULT '0', -- Large number stored as TEXT
    average_rating REAL,
    review_count INTEGER DEFAULT 0,
    updated_at TEXT DEFAULT (datetime('now'))
);

CREATE INDEX idx_agent_reputation_score ON agent_reputation(overall_score);
