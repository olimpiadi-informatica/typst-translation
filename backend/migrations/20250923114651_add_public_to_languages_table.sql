-- Add migration script here
ALTER TABLE languages ADD COLUMN public BOOLEAN NOT NULL DEFAULT FALSE;