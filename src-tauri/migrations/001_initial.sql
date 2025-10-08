-- Create favorites table
CREATE TABLE IF NOT EXISTS favorites (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL,
    filepath TEXT NOT NULL UNIQUE,
    media_type TEXT NOT NULL,
    source TEXT,
    source_id TEXT,
    source_url TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    custom_tags TEXT NOT NULL DEFAULT '[]',
    description TEXT,
    width INTEGER,
    height INTEGER,
    file_size INTEGER,
    created_at TEXT NOT NULL,
    last_used TEXT,
    use_count INTEGER NOT NULL DEFAULT 0
);

-- Create settings table
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Create search history table
CREATE TABLE IF NOT EXISTS search_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query TEXT NOT NULL,
    timestamp TEXT NOT NULL
);

-- Create indexes for better search performance
CREATE INDEX IF NOT EXISTS idx_favorites_media_type ON favorites(media_type);
CREATE INDEX IF NOT EXISTS idx_favorites_source ON favorites(source);
CREATE INDEX IF NOT EXISTS idx_favorites_created_at ON favorites(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_favorites_use_count ON favorites(use_count DESC);
CREATE INDEX IF NOT EXISTS idx_search_history_timestamp ON search_history(timestamp DESC);
