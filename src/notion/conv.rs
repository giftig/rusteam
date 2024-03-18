use std::str::FromStr;

use ::notion::ids::PropertyId;
use ::notion::models::properties::PropertyValue;
use ::notion::models::text::{RichText, RichTextCommon, Text};

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
