BEGIN;

-- Stetch i32 app_id into i64 so it can safely represent any u32. Postgres doesn't have u32.
ALTER TABLE steam_game ALTER COLUMN app_id TYPE BIGINT;
ALTER TABLE owned_game ALTER COLUMN app_id TYPE BIGINT;
ALTER TABLE noted_game ALTER COLUMN app_id TYPE BIGINT;
ALTER TABLE game_tag ALTER COLUMN app_id TYPE BIGINT;

-- Better represent was this field actually means
ALTER TABLE owned_game RENAME COLUMN purchased TO first_recorded;

COMMIT;
