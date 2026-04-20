-- WebChat side questions table
CREATE TABLE IF NOT EXISTS webchat_side_questions (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    question TEXT NOT NULL,
    answer TEXT,
    status TEXT DEFAULT 'pending',
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_webchat_side_questions_session ON webchat_side_questions(session_id);

-- WebChat usage tracking (daily aggregates)
CREATE TABLE IF NOT EXISTS webchat_usage_daily (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    date TEXT NOT NULL,
    session_count INTEGER DEFAULT 0,
    message_count INTEGER DEFAULT 0,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    UNIQUE(user_id, date)
);

CREATE INDEX IF NOT EXISTS idx_webchat_usage_user_date ON webchat_usage_daily(user_id, date);
