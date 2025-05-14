-- Released, unplayed games by my_rating on notion
-- Unplayed games with a rating are a measure of how eagerly I'm anticipating them
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
  sg.app_id,
  sg.name,
  ng.my_rating
FROM
  noted_game ng
  LEFT JOIN steam_game sg ON ng.app_id = sg.app_id
  LEFT JOIN game_details gd ON ng.app_id = gd.app_id
  LEFT JOIN owned_game og ON ng.app_id = og.app_id
  LEFT JOIN played p ON ng.app_id = p.app_id
  LEFT JOIN ignored_game ig ON ng.app_id = ig.app_id
WHERE
  gd.is_released AND
  p.playtime <= '0:30' AND
  ig.app_id IS NULL
ORDER BY
  ng.my_rating DESC NULLS LAST,
  name
