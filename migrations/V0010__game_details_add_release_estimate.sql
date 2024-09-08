BEGIN;

ALTER TABLE game_details ADD release_estimate TIMESTAMP DEFAULT NULL;
CREATE INDEX IF NOT EXISTS game_details_release_estimate ON game_details USING BTREE(release_estimate);

COMMIT;
