use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub font_sans: String,
    pub font_serif: String,
    #[allow(dead_code)]
    pub font_mono: String,
    /// Preferred theme name to apply (e.g., "Flexoki Light" / "Flexoki Dark")
    #[serde(default = "default_theme_name")]
    pub theme_name: String,
    /// Path to a specific theme file or directory to load (e.g., "./themes" or "./themes/flexoki.json")
    /// Defaults to "./themes" so ThemeRegistry can watch that directory.
    #[serde(default = "default_theme_file")]
    pub theme_file: String,
    /// WebView zoom level as percentage (e.g., 120 for 120%)
    #[serde(default = "default_webview_zoom")]
    pub webview_zoom: u32,
    /// How the app should inject theme colors into the WebView content.
    /// Options (config accepts lowercase strings):
    /// - "none": don't inject
    /// - "light": inject only when app theme is light
    /// - "dark": inject only when app theme is dark
    /// - "both": inject for both themes
    #[serde(default = "default_webview_theme_injection")]
    pub webview_theme_injection: String,
    /// Maximum run length before inserting soft-wrap break characters.
    /// Set to 0 to disable the soft-wrap insertion behavior.
    #[serde(default = "default_soft_wrap_max_run")]
    pub soft_wrap_max_run: usize,
    /// Window width in pixels
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    /// Window height in pixels
    #[serde(default = "default_window_height")]
    pub window_height: f32,
}

fn default_webview_theme_injection() -> String {
    // Default to not injecting theme into WebView content.
    // Unknown/absent config -> treat as "none" (do not inject).
    "none".to_string()
}

fn default_webview_zoom() -> u32 {
    120
}

/// Default maximum run length before inserting soft-wrap characters.
/// A value of 0 disables the soft-wrap behavior.
fn default_soft_wrap_max_run() -> usize {
    20
}

fn default_window_width() -> f32 {
    980.0
}

fn default_window_height() -> f32 {
    720.0
}

fn default_theme_name() -> String {
    "Flexoki Light".to_string()
}

fn default_theme_file() -> String {
    "./themes".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            font_sans: "IBM Plex Sans".to_string(),
            font_serif: "IBM Plex Serif".to_string(),
            font_mono: "IBM Plex Mono".to_string(),
            theme_name: default_theme_name(),
            theme_file: default_theme_file(),
            webview_zoom: 120,
            webview_theme_injection: default_webview_theme_injection(),
            soft_wrap_max_run: default_soft_wrap_max_run(),
            window_width: 980.0,
            window_height: 720.0,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        // Look for config.ron in current directory or next to executable
        let mut candidates = Vec::new();

        // 1. Current working directory
        candidates.push(PathBuf::from("config.ron"));

        // 2. Next to executable
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            candidates.push(dir.join("config.ron"));
        }

        for path in candidates {
            if path.exists()
                && let Ok(content) = fs::read_to_string(&path)
            {
                match ron::from_str::<AppConfig>(&content) {
                    Ok(config) => {
                        tracing::info!("Loaded config from {}", path.display());
                        return config;
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse config at {}: {}", path.display(), e);
                    }
                }
            }
        }

        tracing::info!("No config file found, using defaults");
        Self::default()
    }
}
