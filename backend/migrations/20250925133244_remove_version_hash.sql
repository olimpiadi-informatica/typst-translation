-- Recreate the table without the version_hash column
CREATE TABLE statement_versions_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    task_id INTEGER NOT NULL,
    content_manifest TEXT NOT NULL,
    is_live BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Copy data from the old table to the new table
INSERT INTO statement_versions_new (id, task_id, content_manifest, is_live, created_at)
SELECT id, task_id, content_manifest, is_live, created_at
FROM statement_versions;

-- Drop the old table
DROP TABLE statement_versions;

-- Rename the new table to the original table name
ALTER TABLE statement_versions_new RENAME TO statement_versions;