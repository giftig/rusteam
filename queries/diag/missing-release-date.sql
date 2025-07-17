SELECT
  sg.app_id,
  sg.name,
  gd.release_date,
  gd.is_released
FROM game_details gd
  LEFT JOIN steam_game sg ON sg.app_id = gd.app_id
WHERE
  gd.release_date IS NULL OR gd.release_date = ''
ORDER BY
  gd.is_released, sg.name;
