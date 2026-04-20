-- Skill ratings table
CREATE TABLE IF NOT EXISTS skill_ratings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    skill_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    rating INTEGER NOT NULL CHECK(rating >= 1 AND rating <= 5),
    review TEXT,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(skill_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_skill_ratings_skill_id ON skill_ratings(skill_id);
CREATE INDEX IF NOT EXISTS idx_skill_ratings_user_id ON skill_ratings(user_id);
