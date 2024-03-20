BEGIN;

CREATE TABLE IF NOT EXISTS game_details (
  app_id BIGINT PRIMARY KEY NOT NULL,
  description TEXT,
  controller_support VARCHAR(64),
  coop BOOLEAN,
  local_coop BOOLEAN,
  metacritic_percent INTEGER,
  is_released BOOLEAN,
  release_date VARCHAR(64),
  recorded TIMESTAMP
);

CREATE TABLE IF NOT EXISTS game_price (
  id SERIAL PRIMARY KEY,
  app_id BIGINT NOT NULL,
  price INTEGER NOT NULL,
  discount_percent INTEGER NOT NULL DEFAULT 0,
  recorded TIMESTAMP
);
CREATE INDEX IF NOT EXISTS game_price_app_id ON game_price USING BTREE(app_id);
CREATE INDEX IF NOT EXISTS game_price_recorded ON game_price USING BTREE(recorded);
CREATE INDEX IF NOT EXISTS game_price_discount_percent ON game_price USING BTREE(discount_percent);

COMMIT;
