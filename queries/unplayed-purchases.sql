WITH played AS (
  SELECT
    pg.app_id,
    MAX(pg.last_played) last_played,
    SUM(pg.playtime) total_playtime
  FROM
    played_game pg
  GROUP BY
    pg.app_id
)
SELECT
  sg.name,
  og.app_id,
  DATE(og.first_recorded) first_seen_in_library
FROM
  owned_game og
  LEFT JOIN played ON og.app_id = played.app_id
  LEFT JOIN steam_game sg ON og.app_id = sg.app_id
WHERE
  played.last_played < '1971-01-01' OR
  played.last_played IS NULL OR
  played.total_playtime < '0:30:00'
ORDER BY
  DATE(og.first_recorded) DESC,
  played.total_playtime,
  sg.name;
