-- Initial database migration for BeeBotOS
-- Creates core tables for agents, sessions, and system state

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Agents table
CREATE TABLE agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    owner_address VARCHAR(42),
    status VARCHAR(50) DEFAULT 'inactive',
    config JSONB,
    capabilities JSONB[],
    model_provider VARCHAR(100),
    model_name VARCHAR(100),
    owner_id VARCHAR(255),
    last_heartbeat TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_active_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB
);

CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_agents_owner ON agents(owner_address);
CREATE INDEX idx_agents_owner_id ON agents(owner_id);

-- Agent status history table
CREATE TABLE agent_status_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL,
    reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_agent_status_history_agent ON agent_status_history(agent_id);

-- Sessions table
CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    name VARCHAR(255),
    status VARCHAR(50) DEFAULT 'active',
    context JSONB,
    encryption_key_hash VARCHAR(128),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    archived_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_sessions_agent ON sessions(agent_id);
CREATE INDEX idx_sessions_status ON sessions(status);

-- Transactions table
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tx_hash VARCHAR(66) UNIQUE,
    from_address VARCHAR(42),
    to_address VARCHAR(42),
    amount NUMERIC(78, 0),
    token_address VARCHAR(42),
    status VARCHAR(50) DEFAULT 'pending',
    gas_used BIGINT,
    gas_price NUMERIC(78, 0),
    block_number BIGINT,
    data JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    confirmed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_transactions_from ON transactions(from_address);
CREATE INDEX idx_transactions_to ON transactions(to_address);
CREATE INDEX idx_transactions_status ON transactions(status);
CREATE INDEX idx_transactions_block ON transactions(block_number);

-- Events table
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(100) NOT NULL,
    source VARCHAR(255),
    agent_id UUID REFERENCES agents(id),
    payload JSONB,
    severity VARCHAR(20) DEFAULT 'info',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_agent ON events(agent_id);
CREATE INDEX idx_events_created ON events(created_at);

-- Tasks table
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agents(id),
    session_id UUID REFERENCES sessions(id),
    task_type VARCHAR(100) NOT NULL,
    input JSONB,
    output JSONB,
    status VARCHAR(50) DEFAULT 'pending',
    priority INTEGER DEFAULT 0,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    error_message TEXT,
    retries INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_tasks_agent ON tasks(agent_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_created ON tasks(created_at);

-- Memories table
CREATE TABLE memories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agents(id),
    memory_type VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    embedding VECTOR(1536),
    importance FLOAT DEFAULT 0.5,
    metadata JSONB,
    retrieval_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_accessed_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_memories_agent ON memories(agent_id);
CREATE INDEX idx_memories_type ON memories(memory_type);
CREATE INDEX idx_memories_embedding ON memories USING ivfflat (embedding vector_cosine_ops);

-- Skills table
CREATE TABLE skills (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    author VARCHAR(255),
    category VARCHAR(100),
    runtime_type VARCHAR(50),
    entrypoint VARCHAR(255),
    schema JSONB,
    config_schema JSONB,
    hash VARCHAR(64),
    verified BOOLEAN DEFAULT FALSE,
    downloads INTEGER DEFAULT 0,
    rating FLOAT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_skills_category ON skills(category);
CREATE INDEX idx_skills_runtime ON skills(runtime_type);

-- Agent-Skill installations
CREATE TABLE agent_skills (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    skill_id UUID NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
    config JSONB,
    installed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    active BOOLEAN DEFAULT TRUE,
    UNIQUE(agent_id, skill_id)
);

CREATE INDEX idx_agent_skills_agent ON agent_skills(agent_id);

-- DAO Proposals
CREATE TABLE dao_proposals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    proposal_id BIGINT UNIQUE,
    proposer VARCHAR(42) NOT NULL,
    title VARCHAR(500) NOT NULL,
    description TEXT,
    targets JSONB,
    values JSONB,
    calldatas JSONB,
    status VARCHAR(50) DEFAULT 'pending',
    for_votes NUMERIC(78, 0) DEFAULT 0,
    against_votes NUMERIC(78, 0) DEFAULT 0,
    abstain_votes NUMERIC(78, 0) DEFAULT 0,
    start_block BIGINT,
    end_block BIGINT,
    executed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_dao_proposals_status ON dao_proposals(status);
CREATE INDEX idx_dao_proposals_proposer ON dao_proposals(proposer);

-- DAO Votes
CREATE TABLE dao_votes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    proposal_id BIGINT NOT NULL REFERENCES dao_proposals(proposal_id),
    voter VARCHAR(42) NOT NULL,
    support SMALLINT NOT NULL,
    weight NUMERIC(78, 0),
    reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(proposal_id, voter)
);

CREATE INDEX idx_dao_votes_proposal ON dao_votes(proposal_id);
CREATE INDEX idx_dao_votes_voter ON dao_votes(voter);

-- System settings
CREATE TABLE system_settings (
    key VARCHAR(255) PRIMARY KEY,
    value JSONB NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Update timestamps trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_agents_updated_at BEFORE UPDATE ON agents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sessions_updated_at BEFORE UPDATE ON sessions
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
