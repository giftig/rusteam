-- Latest releases for watched games from any source
SELECT
  sg.app_id,
  sg.name,
  gd.release_date,
  gd.release_estimate
FROM game_details gd
  LEFT JOIN steam_game sg ON gd.app_id = sg.app_id
  LEFT JOIN ignored_game ig ON gd.app_id = ig.app_id
WHERE
  gd.is_released IS TRUE AND
  gd.release_estimate IS NOT NULL AND
  ig.app_id IS NULL
ORDER BY
  gd.release_estimate DESC;
