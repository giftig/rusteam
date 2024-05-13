use std::str::FromStr;

use ::notion::ids::PropertyId;
use ::notion::models::properties::{Color, PropertyValue, SelectedValue};
use ::notion::models::text::{RichText, RichTextCommon, Text};

use crate::models::game::GameState;

// Convert a plain string with no formatting into a richtext object representing that plain string
pub fn to_rich_text(s: &str) -> RichText {
    RichText::Text {
        rich_text: RichTextCommon {
            plain_text: s.to_string(),
            href: None,
            annotations: None
        },
        text: Text {
            content: s.to_string(),
            link: None
        }
    }
}

// Create a Text property from plain string
pub fn to_text(s: &str) -> PropertyValue {
    PropertyValue::Text {
        id: PropertyId::from_str("0").unwrap(),
        rich_text: vec![to_rich_text(s)]
    }
}

// Create a Title property from plain string
pub fn to_title(s: &str) -> PropertyValue {
    PropertyValue::Title {
        id: PropertyId::from_str("0").unwrap(),
        title: vec![to_rich_text(s)]
    }
}

// Create a Select property. Unfortunately Color is compulsory on the type,
// and has to match the existing color when provided to the API.
pub fn to_select(s: &str, col: &Color) -> PropertyValue {
    PropertyValue::Select {
        id: PropertyId::from_str("0").unwrap(),
        select: Some(
            SelectedValue {
                id: None,
                name: Some(s.to_string()),
                color: col.to_owned()
            }
        )
    }
}

pub fn to_state(state: &GameState) -> PropertyValue {
    // TODO: Colour must be provided to the rust API, and the notion API enforces
    // the colour is correct if provided. Annoyingly that means we have to choose
    // the right colour for the game state here at the moment :(
    let col = match state {
        GameState::Completed => Color::Purple,
        GameState::InProgress => Color::Yellow,
        GameState::NoRelease => Color::Red,
        GameState::PlayAgain => Color::Purple,
        GameState::PlaySoon => Color::Green,
        GameState::Released => Color::Blue,
        GameState::Tried => Color::Gray,
        GameState::Upcoming => Color::Orange,
        _ => Color::Default,
    };

    let s: String = state.to_owned().into();
    to_select(&s, &col)
}
