BEGIN;

CREATE TABLE IF NOT EXISTS ignored_game (
  -- N.B. deliberately not a foreign key constraint because some app IDs may end
  -- up being deleted from steam and never be found in the steam_game table; owned_game
  -- lacks this relation for the same reason
  app_id BIGINT PRIMARY KEY,
  ignored_at TIMESTAMP DEFAULT NOW()
);

COMMIT;
