use chrono::{DateTime, Utc};

use crate::models::game::{GameId, GameDetails};
use crate::models::steam::SteamAppDetails;

pub(super) fn extract_game_details(
    id: &GameId,
    steam: &SteamAppDetails,
    now: &DateTime<Utc>
) -> GameDetails {
    // FIXME: Derive these from categories etc.
    let coop = false;
    let local_coop = false;

    GameDetails {
        id: id.to_owned(),
        description: steam.short_description.clone(),
        controller_support: steam.controller_support.clone(),
        coop: coop,
        local_coop: local_coop,
        metacritic_percent: steam.metacritic.clone().map(|m| m.score),
        is_released: steam.release_date.clone().map(|r| !r.coming_soon).unwrap_or(false),
        release_date: steam.release_date.clone().map(|r| r.date),
        recorded: now.clone(),
    }
}
