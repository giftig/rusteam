CREATE TABLE IF NOT EXISTS steam_game (
  app_id INTEGER PRIMARY KEY,
  name VARCHAR(256)
);

CREATE INDEX IF NOT EXISTS steam_game_name ON steam_game USING BTREE(name);

CREATE TABLE IF NOT EXISTS owned_game (
  app_id INTEGER PRIMARY KEY REFERENCES steam_game(app_id),
  purchased TIMESTAMP NOT NULL
);
CREATE INDEX IF NOT EXISTS owned_game_purchased ON owned_game USING BTREE(purchased);

-- A granular event recording playtime at a given time, entered whenever we see playtime change
CREATE TABLE IF NOT EXISTS played_game (
  id SERIAL PRIMARY KEY,
  app_id INTEGER REFERENCES steam_game(app_id),
  playtime INTERVAL NOT NULL DEFAULT INTERVAL '0',
  recorded TIMESTAMP NOT NULL
);
CREATE INDEX IF NOT EXISTS played_game_recorded ON played_game USING BTREE(recorded);

CREATE TABLE IF NOT EXISTS noted_game (
  app_id INTEGER PRIMARY KEY,
  first_noted TIMESTAMP NOT NULL,
  my_rating SMALLINT,
  notes TEXT
);

CREATE TABLE IF NOT EXISTS tag (
  id SERIAL PRIMARY KEY,
  name VARCHAR(64)
);
CREATE INDEX IF NOT EXISTS tag_name ON tag USING BTREE(name);

CREATE TABLE IF NOT EXISTS game_tag (
  id SERIAL PRIMARY KEY,
  app_id INTEGER REFERENCES steam_game(app_id),
  tag_id INTEGER REFERENCES tag(id)
);
