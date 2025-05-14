-- A log of release updates, especially marking those which have been delayed:
-- a wall of shame for delayed releases
SELECT
  ru.app_id,
  sg.name,
  ru.prev_text,
  ru.new_text,
  ru.prev_estimate,
  ru.new_estimate,
  COALESCE(ru.new_estimate > ru.prev_estimate, ru.prev_estimate IS NOT NULL AND ru.new_estimate IS NULL, FALSE) delayed,
  ru.recorded
FROM
  release_update ru
  LEFT JOIN steam_game sg ON ru.app_id = sg.app_id
ORDER BY
  recorded DESC
