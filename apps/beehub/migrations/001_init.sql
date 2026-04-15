-- Initial schema for ClawHub

CREATE TABLE IF NOT EXISTS skills (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    description TEXT,
    author TEXT NOT NULL,
    license TEXT NOT NULL,
    repository TEXT,
    hash TEXT NOT NULL,
    downloads INTEGER DEFAULT 0,
    rating REAL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, version)
);

CREATE INDEX idx_skills_name ON skills(name);
CREATE INDEX idx_skills_author ON skills(author);
CREATE INDEX idx_skills_downloads ON skills(downloads);
