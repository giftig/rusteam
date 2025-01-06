use super::*;

use chrono::{TimeZone, Utc};

use crate::models::game::{GameId, GameDetails};
use crate::models::steam::*;

// Some categories, including "Multi-player", indicating coop
fn generic_coop_categories() -> Vec<Category> {
    vec![
        Category {
            id: 28,
            description: "Full controller support".to_string()
        },
        Category {
            id: 1,
            description: "Multi-player".to_string()
        }
    ]
}

// Some categories, including "Shared/Split Screen Co-op", indicating local coop
fn local_coop_categories() -> Vec<Category> {
    vec![
        Category {
            id: 2,
            description: "Single-player".to_string()
        },
        Category {
            id: 39,
            description: "Shared/Split Screen Co-op".to_string()
        }
    ]
}

fn details_fixture(categories: Vec<Category>, released: bool) -> SteamAppDetails {
    SteamAppDetails {
        short_description: Some("Game buying simulator".to_string()),
        controller_support: Some("full".to_string()),
        categories: categories,
        metacritic: Some(MetacriticScore { score: 66 }),
        release_date: Some(
            ReleaseDate {
                coming_soon: !released,
                date: if released { "17 Jan 2020".to_string() } else { "2079".to_string() }
            }
        )
    }
}

#[test]
fn convert_steam_details_released() {
    let id = GameId { app_id: 666666 };
    let fix = details_fixture(vec![], true);
    let now = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();

    let expected = GameDetails {
        id: id.clone(),
        description: fix.short_description.clone(),
        controller_support: fix.controller_support.clone(),
        coop: false,
        local_coop: false,
        metacritic_percent: Some(66),
        is_released: true,
        release_date: Some("17 Jan 2020".to_string()),
        release_estimate: Some(Utc.with_ymd_and_hms(2020, 1, 17, 0, 0, 0).unwrap()),
        recorded: now.clone()
    };

    let actual = extract_game_details(&id, &fix, &now);

    assert_eq!(actual, expected);
}

#[test]
fn convert_steam_details_coming_soon() {
    let id = GameId { app_id: 666666 };
    let fix = details_fixture(vec![], false);
    let now = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();

    let expected = GameDetails {
        id: id.clone(),
        description: fix.short_description.clone(),
        controller_support: fix.controller_support.clone(),
        coop: false,
        local_coop: false,
        metacritic_percent: Some(66),
        is_released: false,
        release_date: Some("2079".to_string()),
        release_estimate: Some(Utc.with_ymd_and_hms(2080, 1, 1, 0, 0, 0).unwrap()),
        recorded: now.clone()
    };

    let actual = extract_game_details(&id, &fix, &now);

    assert_eq!(actual, expected);
}

#[test]
fn convert_steam_details_remote_coop() {
    let id = GameId { app_id: 666666 };
    let fix = details_fixture(generic_coop_categories(), false);
    let now = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();

    let expected = GameDetails {
        id: id.clone(),
        description: fix.short_description.clone(),
        controller_support: fix.controller_support.clone(),
        coop: true,
        local_coop: false,
        metacritic_percent: Some(66),
        is_released: false,
        release_date: Some("2079".to_string()),
        release_estimate: Some(Utc.with_ymd_and_hms(2080, 1, 1, 0, 0, 0).unwrap()),
        recorded: now.clone()
    };

    let actual = extract_game_details(&id, &fix, &now);

    assert_eq!(actual, expected);
}

#[test]
fn convert_steam_details_local_coop() {
    let id = GameId { app_id: 666666 };
    let fix = details_fixture(local_coop_categories(), false);
    let now = Utc.with_ymd_and_hms(2020, 1, 1, 0, 0, 0).unwrap();

    let expected = GameDetails {
        id: id.clone(),
        description: fix.short_description.clone(),
        controller_support: fix.controller_support.clone(),
        coop: true,
        local_coop: true,
        metacritic_percent: Some(66),
        is_released: false,
        release_date: Some("2079".to_string()),
        release_estimate: Some(Utc.with_ymd_and_hms(2080, 1, 1, 0, 0, 0).unwrap()),
        recorded: now.clone()
    };

    let actual = extract_game_details(&id, &fix, &now);

    assert_eq!(actual, expected);
}

#[test]
fn parse_release_date_exact_date() {
    let expected = Utc.with_ymd_and_hms(2025, 6, 5, 0, 0, 0).unwrap();
    let actual = parse_release_date("5 Jun, 2025");

    assert_eq!(actual, Some(expected));
}

#[test]
fn parse_release_date_year() {
    let expected = Utc.with_ymd_and_hms(2028, 1, 1, 0, 0, 0).unwrap();
    let actual = parse_release_date("2027");

    assert_eq!(actual, Some(expected));
}

#[test]
fn parse_release_date_month_year() {
    let expected = Utc.with_ymd_and_hms(2025, 5, 1, 0, 0, 0).unwrap();
    let actual = parse_release_date("Apr 2025");

    assert_eq!(actual, Some(expected));
}

#[test]
fn parse_release_date_quarter() {
    let expected = Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap();
    let actual = parse_release_date("Q2 2025");

    assert_eq!(actual, Some(expected));
}
