-- Games which are marked as released but have a release date in the future
SELECT
  sg.app_id,
  sg.name,
  gd.release_date,
  gd.is_released
FROM game_details gd
  LEFT JOIN steam_game sg ON sg.app_id = gd.app_id
WHERE
  gd.release_estimate > NOW() AND
  gd.is_released IS TRUE
ORDER BY
  gd.is_released, sg.name;
