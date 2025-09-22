-- Alter existing users table
ALTER TABLE users ADD COLUMN tokens_used INTEGER NOT NULL DEFAULT 0;
ALTER TABLE users ADD COLUMN name TEXT NOT NULL DEFAULT 'Default User'; -- Provide a default for existing rows

-- Create languages table
CREATE TABLE languages (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    code TEXT NOT NULL UNIQUE,
    user_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Create contests table
CREATE TABLE contests (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL
);

-- Create tasks table
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    contest_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    FOREIGN KEY (contest_id) REFERENCES contests(id)
);

-- Create statement_versions table
CREATE TABLE statement_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    task_id INTEGER NOT NULL,
    version_hash TEXT NOT NULL UNIQUE,
    content_manifest TEXT NOT NULL, -- Storing JSON as TEXT
    is_live BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES tasks(id)
);

-- Create contestants table
CREATE TABLE contestants (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    online_bit BOOLEAN NOT NULL,
    user_id INTEGER NOT NULL,
    language_id INTEGER, -- NULL if no specific translation needed
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (language_id) REFERENCES languages(id)
);

-- Create translations table
CREATE TABLE translations (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    task_id INTEGER NOT NULL,
    language_id INTEGER NOT NULL,
    content_hash TEXT NOT NULL,
    last_updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (task_id, language_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id),
    FOREIGN KEY (language_id) REFERENCES languages(id)
);

-- Create user_contest_status table
CREATE TABLE user_contest_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_id INTEGER NOT NULL,
    contest_id INTEGER NOT NULL,
    finalized_translations BOOLEAN NOT NULL DEFAULT FALSE,
    skip_envelope_verification BOOLEAN NOT NULL DEFAULT FALSE,
    envelope_received_at DATETIME, -- NULL if not received
    UNIQUE (user_id, contest_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (contest_id) REFERENCES contests(id)
);

-- Create printed_documents table
CREATE TABLE printed_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    contestant_id INTEGER NOT NULL,
    statement_version_id INTEGER NOT NULL,
    language_id INTEGER, -- NULL for original English statement
    printed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (contestant_id, statement_version_id, language_id),
    FOREIGN KEY (contestant_id) REFERENCES contestants(id),
    FOREIGN KEY (statement_version_id) REFERENCES statement_versions(id),
    FOREIGN KEY (language_id) REFERENCES languages(id)
);

-- Create draft_print_queue table
CREATE TABLE draft_print_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    statement_version_id INTEGER NOT NULL,
    language_id INTEGER, -- NULL for original statement
    added_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (statement_version_id) REFERENCES statement_versions(id),
    FOREIGN KEY (language_id) REFERENCES languages(id)
);

-- Create rendered_pdf_cache table
CREATE TABLE rendered_pdf_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    statement_version_id INTEGER NOT NULL,
    language_id INTEGER, -- NULL for original statement
    pdf_hash TEXT NOT NULL,
    UNIQUE (statement_version_id, language_id),
    FOREIGN KEY (statement_version_id) REFERENCES statement_versions(id),
    FOREIGN KEY (language_id) REFERENCES languages(id)
);
