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
  p.app_id,
  s.name,
  TO_CHAR(p.playtime, 'HH24"h"MI"m"') total_playtime,
  TO_CHAR(p.last_played, 'Month') || ' ' || DATE_PART('year', p.last_played) last_played
FROM
  played p
  LEFT JOIN steam_game s ON p.app_id = s.app_id
ORDER BY
  p.playtime DESC
LIMIT 50;
