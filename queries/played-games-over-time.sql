SELECT *
FROM played_game_deltas
WHERE playtime IS NOT NULL AND playtime > '0:30:00'
ORDER BY recorded DESC;
