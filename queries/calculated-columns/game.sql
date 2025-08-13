-- SQL expression for a Superset calculated column producing a link to the steam page for a game
-- Assumes that app_id and name columns are included in the dataset
'<!-- ' || name || '--><a href="https://store.steampowered.com/app/' || app_id::TEXT || '">' || name || '</a>'
