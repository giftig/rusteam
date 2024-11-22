use std::fs;

/// Convenience func to get a fixture from the standard path, as a string
pub fn fixture(s: &str) -> String {
    fs::read_to_string(format!("test/fixtures/{}", s)).unwrap()
}
