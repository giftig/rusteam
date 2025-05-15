use super::*;

use std::fs;

use serde_json;

#[test]
fn read_steam_app_details_response() {
    let data = fs::read_to_string("resources/test/steam/app-details-response-1.json").unwrap();

    // Real API response for Stardew Valley, but some fields truncated for easier testing
    let expected_entry = SteamAppDetailsResponseEntry {
        data: SteamAppDetails {
            short_description: Some("SHORT DESC".to_string()),
            controller_support: Some("full".to_string()),
            categories: vec![
                Category { id: 2, description: "Single-player".to_string() },
                Category { id: 1, description: "Multi-player".to_string() },
                Category { id: 9, description: "Co-op".to_string() },
                Category { id: 38, description: "Online Co-op".to_string() },
            ],
            metacritic: Some(MetacriticScore { score: 89 }),
            release_date: Some(ReleaseDate { coming_soon: false, date: "26 Feb, 2016".to_string() }),
        }
    };
    let expected_entries: HashMap<String, SteamAppDetailsResponseEntry> = {
        HashMap::from([("413150".to_string(), expected_entry)])
    };

    let expected = SteamAppDetailsResponse { results: expected_entries };
    let actual: SteamAppDetailsResponse = serde_json::from_str(&data).unwrap();

    assert_eq!(actual, expected);
}
