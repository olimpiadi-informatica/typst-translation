-- Add migration script here
CREATE TABLE users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    login_epoch INTEGER NOT NULL,
    automatic_translation_budget INTEGER NOT NULL DEFAULT 0
);
