BEGIN;

-- Alter noted_game so that the primary key is notion rather than steam, since the
-- table refers to notion entries more directly than steam ones.
-- This will also make it possible to insert rows which don't have an app_id
-- calculated yet, making it easier to track and update them in Notion.
DELETE FROM noted_game;
ALTER TABLE noted_game DROP CONSTRAINT noted_game_pkey;
ALTER TABLE noted_game ALTER COLUMN app_id DROP NOT NULL;
ALTER TABLE noted_game ADD note_id VARCHAR(64) NOT NULL;
ALTER TABLE noted_game ADD PRIMARY KEY (note_id);

COMMIT;
