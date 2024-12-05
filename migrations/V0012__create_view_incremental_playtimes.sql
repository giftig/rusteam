BEGIN;

CREATE OR REPLACE VIEW played_game_named AS (
  SELECT
    pg.app_id,
    sg.name,
    pg.playtime,
    pg.recorded,
    pg.last_played
  FROM
    played_game pg LEFT JOIN steam_game sg ON pg.app_id = sg.app_id
);

CREATE OR REPLACE VIEW played_game_deltas AS (
  SELECT
    app_id,
    name,
    playtime,
    recorded,
    last_played,
    playtime - LAG(playtime) OVER (PARTITION BY app_id ORDER BY recorded) AS playtime_delta
  FROM played_game_named
);

COMMIT;
