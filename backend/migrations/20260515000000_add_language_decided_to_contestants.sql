ALTER TABLE contestants ADD COLUMN language_decided BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE contestants
SET language_decided = TRUE;
