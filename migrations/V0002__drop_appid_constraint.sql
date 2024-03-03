-- Drop the foreign key constraint on the steam_game table because the  "all apps" steam endpoint
-- doesn't seem to actually provide all apps, and even varies which subset it provides
-- dramatically over time. We'll have to just list unknown name for any game with no appid entry

ALTER TABLE owned_game DROP CONSTRAINT IF EXISTS owned_game_app_id_fkey;
ALTER TABLE played_game DROP CONSTRAINT IF EXISTS played_game_app_id_fkey;
ALTER TABLE game_tag DROP CONSTRAINT IF EXISTS game_tag_app_id_fkey;
