WITH played AS (
  SELECT
    pg.app_id,
    MAX(pg.last_played) last_played,
    MAX(pg.playtime) playtime
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
  LEFT JOIN ignored_game ig ON ig.app_id = og.app_id
WHERE
  ig.app_id IS NULL AND (
    played.last_played < '1971-01-01' OR
    played.last_played IS NULL OR
    played.playtime < '0:30:00'
  )
ORDER BY
  DATE(og.first_recorded) DESC,
  played.playtime,
  sg.name;
