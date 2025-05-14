-- Full current wishlist, showing most recently added games first
SELECT
  sg.name,
  w.wishlisted,
  gd.release_date,
  gd.release_estimate
FROM
  wishlist w
  LEFT JOIN steam_game sg ON w.app_id = sg.app_id
  LEFT JOIN game_details gd ON w.app_id = gd.app_id
WHERE
  w.deleted IS NULL
ORDER BY
  w.wishlisted DESC
