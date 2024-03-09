-- Record last_played as at the time of recording a new delta
-- This'll give slightly more granular info about when exactly the game was
-- played between two events being recorded

ALTER TABLE played_game ADD last_played TIMESTAMP;
