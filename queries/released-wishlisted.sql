-- Games which have been released and are on the wishlist
-- Prioritise those which were wishlisted prior to release, in theory these
-- are ones which were being eagerly awaited once upon a time?
SELECT
  sg.name,
  w.wishlisted,
  gd.release_date,
  gd.release_estimate,
  (gd.release_estimate > w.wishlisted) wishlisted_before_release
FROM
  wishlist w
  LEFT JOIN steam_game sg ON w.app_id = sg.app_id
  LEFT JOIN game_details gd ON w.app_id = gd.app_id
  LEFT JOIN owned_game og ON w.app_id = og.app_id
WHERE
  gd.is_released = TRUE AND
  og.app_id IS NULL
ORDER BY
  wishlisted_before_release DESC,
  w.wishlisted DESC
