WITH latest_sessions AS (
  SELECT
    app_id, MAX(last_played) last_played
  FROM
    played_game p
  GROUP BY
    app_id
)
SELECT
  p.app_id,
  s.name,
  TO_CHAR(p.playtime, 'HH24"h"MI"m"') total_playtime,
  p.last_played,
  TO_CHAR(p.last_played, 'Month') || ' ' || DATE_PART('year', p.last_played) simple_last_played
FROM
  latest_sessions l
  LEFT JOIN steam_game s ON l.app_id = s.app_id
  LEFT JOIN played_game p ON l.app_id = p.app_id AND l.last_played = p.last_played
ORDER BY
  p.last_played DESC
LIMIT 20;
