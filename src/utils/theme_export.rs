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

/// Build a complete theme color palette from base colors
/// Generates derived colors for borders, hover states, etc.
#[allow(dead_code)]
pub fn build_theme_palette(
    background: Rgb,
    foreground: Rgb,
    accent: Rgb,
) -> HashMap<String, String> {
    let mut colors = HashMap::new();

    // Core colors
    colors.insert("background".to_string(), background.to_hex());
    colors.insert("foreground".to_string(), foreground.to_hex());
    colors.insert("accent".to_string(), accent.to_hex());

    // Calculate if theme is dark or light based on background luminosity
    let is_dark = {
        let lum = (0.299 * background.r as f32
            + 0.587 * background.g as f32
            + 0.114 * background.b as f32)
            / 255.0;
        lum < 0.5
    };

    // Derive border color (slightly lighter/darker than background)
    let border = if is_dark {
        lighten(background, 0.1)
    } else {
        darken(background, 0.08)
    };
    colors.insert("border".to_string(), border.to_hex());

    // Derive muted background
    let muted_bg = if is_dark {
        lighten(background, 0.05)
    } else {
        darken(background, 0.03)
    };
    colors.insert("muted.background".to_string(), muted_bg.to_hex());

    // Derive muted foreground (less contrast)
    let muted_fg = if is_dark {
        darken(foreground, 0.3)
    } else {
        lighten(foreground, 0.3)
    };
    colors.insert("muted.foreground".to_string(), muted_fg.to_hex());

    // Primary colors (using accent)
    colors.insert("primary.background".to_string(), accent.to_hex());
    colors.insert(
        "primary.foreground".to_string(),
        if is_dark {
            Rgb::new(250, 250, 250).to_hex()
        } else {
            Rgb::new(16, 15, 15).to_hex()
        },
    );

    // Secondary colors
    colors.insert("secondary.background".to_string(), muted_bg.to_hex());
    colors.insert("secondary.foreground".to_string(), foreground.to_hex());

    // Hover states
    let hover_bg = if is_dark {
        lighten(background, 0.08)
    } else {
        darken(background, 0.05)
    };
    colors.insert("secondary.hover.background".to_string(), hover_bg.to_hex());

    colors
}

/// Lighten an RGB color by a factor (0.0 to 1.0)
fn lighten(color: Rgb, factor: f32) -> Rgb {
    let r = (color.r as f32 + (255.0 - color.r as f32) * factor).min(255.0) as u8;
    let g = (color.g as f32 + (255.0 - color.g as f32) * factor).min(255.0) as u8;
    let b = (color.b as f32 + (255.0 - color.b as f32) * factor).min(255.0) as u8;
    Rgb::new(r, g, b)
}

/// Darken an RGB color by a factor (0.0 to 1.0)
fn darken(color: Rgb, factor: f32) -> Rgb {
    let r = (color.r as f32 * (1.0 - factor)).max(0.0) as u8;
    let g = (color.g as f32 * (1.0 - factor)).max(0.0) as u8;
    let b = (color.b as f32 * (1.0 - factor)).max(0.0) as u8;
    Rgb::new(r, g, b)
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
