-- Recreate the translations table to make content_hash nullable
CREATE TABLE translations_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    task_id INTEGER NOT NULL,
    language_id INTEGER NOT NULL,
    content_hash TEXT,
    last_updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    session_token TEXT,
    UNIQUE (task_id, language_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id),
    FOREIGN KEY (language_id) REFERENCES languages(id)
);

-- Copy data from the old table to the new table
INSERT INTO translations_new (id, task_id, language_id, content_hash, last_updated_at)
SELECT id, task_id, language_id, content_hash, last_updated_at
FROM translations;

-- Drop the old table
DROP TABLE translations;

-- Rename the new table to the original table name
ALTER TABLE translations_new RENAME TO translations;
