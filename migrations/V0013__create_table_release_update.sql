BEGIN;

-- Track when release dates change so setbacks can be seen (and repeated setbacks for an anticipated
-- game can be frowned upon)
CREATE TABLE IF NOT EXISTS release_update (
  id SERIAL PRIMARY KEY,
  app_id BIGINT REFERENCES steam_game(app_id),
  prev_text VARCHAR(64),
  new_text VARCHAR(64),
  prev_estimate TIMESTAMP,
  new_estimate TIMESTAMP,
  recorded TIMESTAMP NOT NULL
);

COMMIT;
