-- Milestone 52: 10-year longevity SQLite schema
CREATE TABLE IF NOT EXISTS life_events (
    id TEXT PRIMARY KEY,
    timestamp DATETIME NOT NULL,
    source_type TEXT NOT NULL,
    metadata TEXT, -- JSON structure
    content TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_life_events_timestamp ON life_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_life_events_source_type ON life_events(source_type);

-- Milestone 54: WAL mode is enabled via connection options in Rust
