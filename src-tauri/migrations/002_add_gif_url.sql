-- Migration to make filepath optional and add gif_url field
-- This allows us to store Giphy GIFs by URL without downloading

-- Create new table with updated schema
CREATE TABLE IF NOT EXISTS favorites_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL,
    filepath TEXT, -- Made optional (removed NOT NULL)
    gif_url TEXT, -- Direct GIF URL for clipboard
    media_type TEXT NOT NULL,
    source TEXT,
    source_id TEXT,
    source_url TEXT, -- Giphy page URL
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

-- Copy existing data
INSERT INTO favorites_new SELECT
    id, filename, filepath, NULL as gif_url, media_type, source, source_id, source_url,
    tags, custom_tags, description, width, height, file_size, created_at, last_used, use_count
FROM favorites;

-- Drop old table and rename new one
DROP TABLE favorites;
ALTER TABLE favorites_new RENAME TO favorites;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_favorites_media_type ON favorites(media_type);
CREATE INDEX IF NOT EXISTS idx_favorites_source ON favorites(source);
CREATE INDEX IF NOT EXISTS idx_favorites_created_at ON favorites(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_favorites_use_count ON favorites(use_count DESC);

-- Add unique constraint on source_id for Giphy GIFs
CREATE UNIQUE INDEX IF NOT EXISTS idx_favorites_source_id ON favorites(source, source_id) WHERE source_id IS NOT NULL;
