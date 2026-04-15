-- Migration: Add A2A protocol tables

-- A2A Deals
CREATE TABLE a2a_deals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    deal_id VARCHAR(66) UNIQUE,
    buyer_agent_id UUID NOT NULL REFERENCES agents(id),
    seller_agent_id UUID NOT NULL REFERENCES agents(id),
    task_hash VARCHAR(66),
    amount NUMERIC(78, 0) NOT NULL,
    token_address VARCHAR(42),
    escrow_address VARCHAR(42),
    status VARCHAR(50) DEFAULT 'pending',
    deadline TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    funded_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    dispute_id UUID
);

CREATE INDEX idx_a2a_deals_buyer ON a2a_deals(buyer_agent_id);
CREATE INDEX idx_a2a_deals_seller ON a2a_deals(seller_agent_id);
CREATE INDEX idx_a2a_deals_status ON a2a_deals(status);

-- A2A Negotiations
CREATE TABLE a2a_negotiations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    negotiation_id VARCHAR(66) UNIQUE,
    initiator_agent_id UUID NOT NULL REFERENCES agents(id),
    responder_agent_id UUID NOT NULL REFERENCES agents(id),
    intent JSONB NOT NULL,
    current_terms JSONB,
    state VARCHAR(50) DEFAULT 'offering',
    round_number INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    deal_id UUID REFERENCES a2a_deals(id)
);

CREATE INDEX idx_a2a_negotiations_initiator ON a2a_negotiations(initiator_agent_id);
CREATE INDEX idx_a2a_negotiations_responder ON a2a_negotiations(responder_agent_id);

-- A2A Capabilities registry
CREATE TABLE a2a_capabilities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    capability_uri VARCHAR(255) NOT NULL,
    name VARCHAR(255),
    version VARCHAR(50),
    description TEXT,
    input_schema JSONB,
    output_schema JSONB,
    pricing_model VARCHAR(50),
    price NUMERIC(78, 0),
    active BOOLEAN DEFAULT TRUE,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(agent_id, capability_uri)
);

CREATE INDEX idx_a2a_capabilities_agent ON a2a_capabilities(agent_id);
CREATE INDEX idx_a2a_capabilities_uri ON a2a_capabilities(capability_uri);

-- A2A Messages
CREATE TABLE a2a_messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id VARCHAR(66) UNIQUE,
    from_agent_id UUID NOT NULL REFERENCES agents(id),
    to_agent_id UUID NOT NULL REFERENCES agents(id),
    message_type VARCHAR(100) NOT NULL,
    correlation_id VARCHAR(66),
    payload JSONB NOT NULL,
    signature VARCHAR(132),
    received_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    processed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_a2a_messages_from ON a2a_messages(from_agent_id);
CREATE INDEX idx_a2a_messages_to ON a2a_messages(to_agent_id);
CREATE INDEX idx_a2a_messages_correlation ON a2a_messages(correlation_id);
CREATE INDEX idx_a2a_messages_received ON a2a_messages(received_at);

-- Agent reputation
CREATE TABLE agent_reputation (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agents(id) UNIQUE,
    overall_score FLOAT DEFAULT 0,
    successful_deals INTEGER DEFAULT 0,
    failed_deals INTEGER DEFAULT 0,
    total_volume NUMERIC(78, 0) DEFAULT 0,
    average_rating FLOAT,
    review_count INTEGER DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_agent_reputation_score ON agent_reputation(overall_score);
