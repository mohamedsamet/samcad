//! Utility for parsing simple `KEY=VALUE` config files.
//!
//! Format: one `key=value` per line, `#` starts a comment, blank lines ignored.
//! Values are raw strings; callers parse them into typed settings.

use std::collections::HashMap;

/// Parses the entire file into a key→raw-value map.
pub fn parse_env_file(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            map.insert(key.trim().to_owned(), value.trim().to_owned());
        }
    }

    map
}

/// Parses `"r, g, b"` or `"r, g, b, a"` into an egui colour.
pub fn parse_rgba(value: &str) -> Option<eframe::egui::Color32> {
    let parts: Vec<u8> = value
        .split(',')
        .map(|s| s.trim().parse().ok())
        .collect::<Option<Vec<_>>>()?;

    match parts.as_slice() {
        [r, g, b] => Some(eframe::egui::Color32::from_rgb(*r, *g, *b)),
        [r, g, b, a] => Some(eframe::egui::Color32::from_rgba_unmultiplied(*r, *g, *b, *a)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_env_file_handles_comments_and_whitespace() {
        let map = parse_env_file(
            "# comment\nKEY1=value1\n  KEY2 = value2  \nKEY3=\n",
        );
        assert_eq!(map.get("KEY1"), Some(&"value1".to_string()));
        assert_eq!(map.get("KEY2"), Some(&"value2".to_string()));
        assert_eq!(map.get("KEY3"), Some(&"".to_string()));
        assert_eq!(map.get("MISSING"), None);
    }

    #[test]
    fn parse_rgba_accepts_three_and_four_components() {
        assert_eq!(
            parse_rgba("255, 165, 0"),
            Some(eframe::egui::Color32::from_rgb(255, 165, 0))
        );
        assert_eq!(
            parse_rgba("10,20,30,40"),
            Some(eframe::egui::Color32::from_rgba_unmultiplied(10, 20, 30, 40))
        );
    }

    #[test]
    fn parse_rgba_rejects_malformed() {
        assert_eq!(parse_rgba(""), None);
        assert_eq!(parse_rgba("not, a, color"), None);
        assert_eq!(parse_rgba("1, 2"), None);
        assert_eq!(parse_rgba("256, 0, 0"), None);
    }
}