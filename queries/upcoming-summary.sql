-- See the states of unreleased games in notes and steam
SELECT
  n.app_id,
  s.name,
  n.state,
  d.is_released,
  d.release_date
FROM noted_game n
  LEFT JOIN steam_game s ON n.app_id = s.app_id
  LEFT JOIN game_details d ON n.app_id = d.app_id
WHERE
  n.state IN ('No release', 'Upcoming')
ORDER BY
  s.name;
