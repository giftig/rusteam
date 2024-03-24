-- Add missing state column to noted_game
ALTER TABLE noted_game ADD COLUMN state VARCHAR(64);
CREATE INDEX IF NOT EXISTS noted_game_state ON noted_game USING BTREE(state);
