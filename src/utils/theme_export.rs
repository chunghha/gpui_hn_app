/// Utility functions for theme operations: export, import, color conversion
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert RGB to hex string (e.g., "#FF0000")
    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Convert hex string to RGB (e.g., "#FF0000" -> Rgb { r: 255, g: 0, b: 0 })
    #[allow(dead_code)]
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 {
            return Err(format!("Invalid hex color: {}", hex));
        }

        let r = u8::from_str_radix(&hex[0..2], 16)
            .map_err(|_| format!("Invalid red component: {}", &hex[0..2]))?;
        let g = u8::from_str_radix(&hex[2..4], 16)
            .map_err(|_| format!("Invalid green component: {}", &hex[2..4]))?;
        let b = u8::from_str_radix(&hex[4..6], 16)
            .map_err(|_| format!("Invalid blue component: {}", &hex[4..6]))?;

        Ok(Self { r, g, b })
    }
}

/// Theme color palette for export
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    #[serde(flatten)]
    pub colors: HashMap<String, String>,
}

/// Generate complementary theme name (toggle Light ↔ Dark)
#[allow(dead_code)]
pub fn generate_complementary_name(name: &str) -> String {
    if name.ends_with(" Light") {
        name.replace(" Light", " Dark")
    } else if name.ends_with(" Dark") {
        name.replace(" Dark", " Light")
    } else {
        // Default to adding " Dark" if no suffix
        format!("{} Dark", name)
    }
}

/// Discover all theme JSON files in a directory
#[allow(dead_code)]
pub fn discover_themes(theme_dir: &PathBuf) -> Vec<String> {
    let mut themes = Vec::new();

    if let Ok(entries) = fs::read_dir(theme_dir) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension()
                && extension == "json"
                && let Some(file_name) = entry.path().file_stem()
            {
                themes.push(file_name.to_string_lossy().to_string());
            }
        }
    }

    themes.sort();
    themes
}

/// Export theme colors to JSON format
#[allow(dead_code)]
pub fn export_theme_to_json(
    theme_name: &str,
    mode: &str,
    colors: &HashMap<String, String>,
    output_path: &PathBuf,
) -> Result<(), String> {
    let theme_data = serde_json::json!({
        "name": theme_name,
        "themes": [{
            "name": format!("{} {}", theme_name, mode.to_uppercase()),
            "mode": mode,
            "colors": colors,
        }]
    });

    let json_string = serde_json::to_string_pretty(&theme_data)
        .map_err(|e| format!("Failed to serialize theme: {}", e))?;

    fs::write(output_path, json_string)
        .map_err(|e| format!("Failed to write theme file: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_hex() {
        assert_eq!(Rgb::new(255, 0, 0).to_hex(), "#FF0000");
        assert_eq!(Rgb::new(0, 255, 0).to_hex(), "#00FF00");
        assert_eq!(Rgb::new(0, 0, 255).to_hex(), "#0000FF");
        assert_eq!(Rgb::new(128, 128, 128).to_hex(), "#808080");
    }

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(Rgb::from_hex("#FF0000").unwrap(), Rgb::new(255, 0, 0));
        assert_eq!(Rgb::from_hex("#00FF00").unwrap(), Rgb::new(0, 255, 0));
        assert_eq!(Rgb::from_hex("#0000FF").unwrap(), Rgb::new(0, 0, 255));
        assert_eq!(Rgb::from_hex("#808080").unwrap(), Rgb::new(128, 128, 128));
    }

    #[test]
    fn test_hex_to_rgb_without_hash() {
        assert_eq!(Rgb::from_hex("FF0000").unwrap(), Rgb::new(255, 0, 0));
    }

    #[test]
    fn test_hex_to_rgb_invalid() {
        assert!(Rgb::from_hex("#FFF").is_err());
        assert!(Rgb::from_hex("#GGGGGG").is_err());
        assert!(Rgb::from_hex("invalid").is_err());
    }

    #[test]
    fn test_generate_complementary_name() {
        assert_eq!(generate_complementary_name("Flexoki Light"), "Flexoki Dark");
        assert_eq!(generate_complementary_name("Flexoki Dark"), "Flexoki Light");
        assert_eq!(generate_complementary_name("MyTheme"), "MyTheme Dark");
    }
}
