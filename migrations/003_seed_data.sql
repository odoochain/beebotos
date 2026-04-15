-- Seed data for BeeBotOS
-- Insert default system settings and sample data

-- System settings
INSERT INTO system_settings (key, value) VALUES
('version', '"1.0.0"'::jsonb),
('initialized_at', to_jsonb(now())),
('default_llm_provider', '"kimi"'::jsonb),
('max_agents_per_user', '10'::jsonb),
('enable_public_registration', 'true'::jsonb)
ON CONFLICT (key) DO UPDATE SET
    value = EXCLUDED.value,
    updated_at = NOW();

-- Insert sample skills (optional, for testing)
INSERT INTO skills (name, version, description, author, category, runtime_type, entrypoint, schema, config_schema, hash, verified)
VALUES 
(
    'text_processor',
    '1.0.0',
    'Basic text processing skill for message handling',
    'BeeBotOS Team',
    'utility',
    'wasm',
    'process_text',
    '{
        "input": {"type": "string"},
        "output": {"type": "string"}
    }'::jsonb,
    '{
        "type": "object",
        "properties": {
            "max_length": {"type": "integer", "default": 1000}
        }
    }'::jsonb,
    'a1b2c3d4e5f6',
    true
),
(
    'llm_chat',
    '1.0.0',
    'LLM chat integration for conversational AI',
    'BeeBotOS Team',
    'ai',
    'native',
    'chat',
    '{
        "input": {
            "type": "object",
            "properties": {
                "message": {"type": "string"},
                "context": {"type": "array"}
            }
        },
        "output": {
            "type": "object",
            "properties": {
                "response": {"type": "string"}
            }
        }
    }'::jsonb,
    '{
        "type": "object",
        "properties": {
            "model": {"type": "string", "default": "moonshot-v1-8k"},
            "temperature": {"type": "number", "default": 0.7}
        }
    }'::jsonb,
    'b2c3d4e5f6g7',
    true
)
ON CONFLICT (name) DO NOTHING;

-- Insert a default system agent (optional)
INSERT INTO agents (id, name, description, status, config, capabilities, metadata)
VALUES (
    '00000000-0000-0000-0000-000000000001'::uuid,
    'system_agent',
    'System default agent for handling platform messages',
    'active',
    '{
        "llm_provider": "kimi",
        "llm_model": "moonshot-v1-8k",
        "max_context_length": 4000
    }'::jsonb,
    ARRAY['text_processing', 'llm_chat']::jsonb[],
    '{
        "is_system": true,
        "platforms": ["lark", "dingtalk", "telegram"]
    }'::jsonb
)
ON CONFLICT (id) DO NOTHING;

-- Create default session for system agent
INSERT INTO sessions (agent_id, name, status, context)
VALUES (
    '00000000-0000-0000-0000-000000000001'::uuid,
    'default_session',
    'active',
    '{}'::jsonb
)
ON CONFLICT DO NOTHING;
