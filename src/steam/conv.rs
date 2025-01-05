#[cfg(test)]
mod tests;

use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Months, TimeZone, Utc};
use serde_json;
use thiserror::Error;

use crate::models::game::{GameId, GameDetails};
use crate::models::steam::SteamAppDetails;

// "Multiplayer", "Co-op", "Online Co-op", "LAN Co-op" categories
const COOP_CAT_IDS: [u32; 4] = [1, 9, 38, 48];

// "Shared/Split Screen Co-op", "Shared/Split Screen" categories
const LOCAL_COOP_CAT_IDS: [u32; 2] = [39, 24];

#[derive(Error, Debug)]
#[error("{0}")]
pub struct ConvError(String);

type Result<T> = std::result::Result<T, ConvError>;

// Make an attempt to parse a release date into a DateTime estimate.
// - Attempt to parse exact dates from the human-readable format given
// - Treat month or years as the last day in that month / year
// - Treat "coming soon", "to be announced" etc. as unknown
pub fn parse_release_date(s: &str) -> Option<DateTime<Utc>> {
    if s == "To be announced" || s == "Coming soon" {
        return None;
    }

    let mut clean = s.replace(",", "").trim().to_owned();

    // Basic exact day format used by Steam is like "5 Jan, 2020"
    if let Ok(d) = NaiveDate::parse_from_str(&clean, "%d %b %Y") {
        let dt = NaiveDateTime::new(d, NaiveTime::from_hms_opt(0, 0, 0)?);
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
    }

    // Look for a year, like "2025", and return midnight, 1st Jan on the following year
    if let Ok(year) = clean.parse::<i32>() {
        return Utc.with_ymd_and_hms(year + 1, 1, 1, 0, 0, 0).single();
    }

    // Parse strings representing a quarter, e.g. "Q2 2025", by substituting the Qx part with the
    // last month in that quarter. It'll then be caught by the "monthly" case below.
    if clean.starts_with("Q") {
        let quarter = s.chars().nth(1)?;
        let month = match quarter {
            '1' => Some("Mar"),
            '2' => Some("Jun"),
            '3' => Some("Sep"),
            '4' => Some("Dec"),
            _ => None
        }?;
        let remainder = &clean[3..];

        let mut buf = "".to_string();
        buf.push_str(&month);
        buf.push_str(" ");
        buf.push_str(&remainder);

        clean = buf;
    }

    // Look for month and year like "Mar 2025". Unfortunately chrono doesn't help us much
    // here because it refuses to parse an imprecise date like this even into a NaiveDate, so we'll
    // try sticking a 1 before it and parsing it as above, then adding a month.
    if let Ok(d) = NaiveDate::parse_from_str(&format!("1 {}", &clean), "%d %b %Y") {
        let dt = NaiveDateTime::new(d + Months::new(1), NaiveTime::from_hms_opt(0, 0, 0)?);
        return Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc));
    }

    None
}

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
        release_estimate: steam.release_date.as_ref().and_then(|r| parse_release_date(&r.date)),
        recorded: now.clone(),
    }
}

/// Extract a wishlist from raw JSON userdata; this will contain an rgWishlist key listing appids
pub(crate) fn extract_wishlist(v: &serde_json::Value) -> Result<Vec<GameId>> {
    let m = v.as_object().ok_or(ConvError("Expected JSON object at root level".to_string()))?;

    m
        .get("rgWishlist")
        .ok_or(ConvError("missing rgWishlist field in userdata".to_string()))?
        .as_array()
        .ok_or(ConvError("Expected JSON array for rgWishlist".to_string()))?
        .into_iter()
        .map(|entry: &serde_json::Value| {
            let id: u32 = entry
                .as_u64()
                .ok_or(ConvError("appid in rgWishlist could not be converted to u64".to_string()))?
                .try_into()
                .map_err(|_| ConvError("appid in rgWishlist was too big to fit into u32!".to_string()))?;

            Ok(GameId::from(id))
        })
        .collect()
}
