#[cfg(test)]
mod tests;

use std::collections::HashSet;

use chrono::{DateTime, Utc};

use crate::models::game::{GameId, GameDetails};
use crate::models::steam::SteamAppDetails;

// "Multiplayer", "Co-op", "Online Co-op", "LAN Co-op" categories
const COOP_CAT_IDS: [u32; 4] = [1, 9, 38, 48];

// "Shared/Split Screen Co-op", "Shared/Split Screen" categories
const LOCAL_COOP_CAT_IDS: [u32; 2] = [39, 24];

pub(super) fn extract_game_details(
    id: &GameId,
    steam: &SteamAppDetails,
    now: &DateTime<Utc>
) -> GameDetails {
    let coop_ids: HashSet<&u32> = COOP_CAT_IDS.iter().collect();
    let local_coop_ids: HashSet<&u32> = LOCAL_COOP_CAT_IDS.iter().collect();

    let found_cat_ids: HashSet<&u32> = steam.categories.iter().map(|cat| &cat.id).collect();

    let coop = found_cat_ids.intersection(&coop_ids).next().is_some();
    let local_coop = found_cat_ids.intersection(&local_coop_ids).next().is_some();

    GameDetails {
        id: id.to_owned(),
        description: steam.short_description.clone(),
        controller_support: steam.controller_support.clone(),
        coop: coop || local_coop,
        local_coop: local_coop,
        metacritic_percent: steam.metacritic.clone().map(|m| m.score),
        is_released: steam.release_date.clone().map(|r| !r.coming_soon).unwrap_or(false),
        release_date: steam.release_date.clone().map(|r| r.date),
        recorded: now.clone(),
    }
}
