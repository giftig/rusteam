BEGIN;

CREATE TABLE IF NOT EXISTS game_details_blacklist (
  app_id BIGINT NOT NULL UNIQUE,
  failure_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS game_details_blacklist_app_id ON game_details_blacklist USING BTREE(app_id);

COMMIT;
