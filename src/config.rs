use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub enum Action {
    Quit,
    Back,
    FocusSearch,
    CycleSearchMode,
    CycleSortOption,
    ToggleSortOrder,
    ScrollDown,
    ScrollUp,
    ScrollToTop,
    ToggleBookmark,
    ShowBookmarks,
    ShowHistory,
    ClearHistory,
    OpenThemeEditor,
    ShowLogViewer,
    None,
}

pub type KeyMap = std::collections::HashMap<String, Action>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UiConfig {
    #[serde(default = "default_padding")]
    pub padding: f32,
    #[serde(default = "default_status_bar_format")]
    pub status_bar_format: String,
    #[serde(default = "default_list_view_items")]
    pub list_view_items: Vec<String>,
}

fn default_padding() -> f32 {
    16.0
}

fn default_status_bar_format() -> String {
    "{mode} | {category} | {count} items".to_string()
}

fn default_list_view_items() -> Vec<String> {
    vec![
        "score".to_string(),
        "comments".to_string(),
        "domain".to_string(),
        "age".to_string(),
        "author".to_string(),
    ]
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            padding: default_padding(),
            status_bar_format: default_status_bar_format(),
            list_view_items: default_list_view_items(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_initial_retry_delay_ms")]
    pub initial_retry_delay_ms: u64,
    #[serde(default = "default_max_retry_delay_ms")]
    pub max_retry_delay_ms: u64,
}

fn default_max_retries() -> u32 {
    3
}

fn default_initial_retry_delay_ms() -> u64 {
    1000
}

fn default_max_retry_delay_ms() -> u64 {
    30000
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_retry_delay_ms: default_initial_retry_delay_ms(),
            max_retry_delay_ms: default_max_retry_delay_ms(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    #[serde(default)]
    pub module_filters: std::collections::HashMap<String, String>,
    #[serde(default = "default_enable_performance_metrics")]
    pub enable_performance_metrics: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_dir() -> String {
    "./logs".to_string()
}

fn default_enable_performance_metrics() -> bool {
    false
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            log_dir: default_log_dir(),
            module_filters: std::collections::HashMap::new(),
            enable_performance_metrics: default_enable_performance_metrics(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub struct AccessibilityConfig {
    #[serde(default)]
    pub high_contrast_mode: bool,
    #[serde(default)]
    pub verbose_status: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    /// How to apply theme injection: "invasive" (uses !important) or "css-vars" (sets CSS variables)
    #[serde(default = "default_webview_theme_mode")]
    pub webview_theme_mode: String,
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
    /// Keybindings
    #[serde(default = "default_keybindings")]
    pub keybindings: KeyMap,
    /// UI Customization
    #[serde(default)]
    pub ui: UiConfig,
    /// Network Configuration
    #[serde(default)]
    pub network: NetworkConfig,
    /// Logging Configuration
    #[serde(default)]
    pub log: LogConfig,
    #[serde(default)]
    pub accessibility: AccessibilityConfig,
}

fn default_webview_theme_injection() -> String {
    // Default to not injecting theme into WebView content.
    // Unknown/absent config -> treat as "none" (do not inject).
    "none".to_string()
}

fn default_webview_zoom() -> u32 {
    120
}

fn default_webview_theme_mode() -> String {
    // Default to invasive mode to preserve current behavior
    "invasive".to_string()
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

fn default_keybindings() -> KeyMap {
    let mut map = std::collections::HashMap::new();
    map.insert("q".to_string(), Action::Quit);
    map.insert("escape".to_string(), Action::Back);
    map.insert("ctrl+r".to_string(), Action::FocusSearch);
    map.insert("ctrl+m".to_string(), Action::CycleSearchMode);
    map.insert("ctrl+s".to_string(), Action::CycleSortOption);
    map.insert("o".to_string(), Action::ToggleSortOrder);
    map.insert("j".to_string(), Action::ScrollDown);
    map.insert("k".to_string(), Action::ScrollUp);
    map.insert("g".to_string(), Action::ScrollToTop);
    map.insert("b".to_string(), Action::ToggleBookmark);
    map.insert("shift+b".to_string(), Action::ShowBookmarks);
    map.insert("shift+h".to_string(), Action::ShowHistory);
    map.insert("shift+x".to_string(), Action::ClearHistory);
    map.insert("t".to_string(), Action::OpenThemeEditor);
    map.insert("shift+l".to_string(), Action::ShowLogViewer);
    map
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
            webview_theme_mode: default_webview_theme_mode(),
            soft_wrap_max_run: default_soft_wrap_max_run(),
            window_width: 980.0,
            window_height: 720.0,
            keybindings: default_keybindings(),
            ui: Default::default(),
            network: Default::default(),
            log: Default::default(),
            accessibility: Default::default(),
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
                    Ok(mut config) => {
                        tracing::info!("Loaded config from {}", path.display());

                        // Merge default keybindings
                        // If a key is missing in user config, add the default one.
                        // If user wants to unbind a key, they should map it to Action::None.
                        let defaults = default_keybindings();
                        for (key, action) in defaults {
                            config.keybindings.entry(key).or_insert(action);
                        }

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

    pub fn save(&self) {
        self.save_to(PathBuf::from("config.ron"));
    }

    pub fn save_to(&self, path: PathBuf) {
        // Try to read existing config to preserve comments
        let existing_content = fs::read_to_string(&path).unwrap_or_default();

        if existing_content.is_empty() {
            // Fallback to standard serialization if file doesn't exist or is empty
            let pretty = ron::ser::PrettyConfig::default()
                .depth_limit(2)
                .separate_tuple_members(true)
                .enumerate_arrays(true);

            match ron::ser::to_string_pretty(self, pretty) {
                Ok(content) => match fs::write(&path, content) {
                    Ok(_) => {
                        tracing::info!("Saved config to {}", path.display());
                    }
                    Err(e) => {
                        tracing::error!("Failed to write config to {}: {}", path.display(), e);
                    }
                },
                Err(e) => {
                    tracing::error!("Failed to serialize config: {}", e);
                }
            }
            return;
        }

        // Helper to replace value in RON content
        // Matches `key: value` or `key: "value"`
        let mut new_content = existing_content.clone();

        let replace_str = |content: &mut String, key: &str, value: &str| {
            let re = RegexBuilder::new(&format!(r#"(\s*{}\s*:\s*)"[^"]*""#, regex::escape(key)))
                .build()
                .unwrap();
            *content = re
                .replace_all(content, format!(r#"${{1}}"{}""#, value))
                .to_string();
        };

        let replace_val = |content: &mut String, key: &str, value: String| {
            let re = RegexBuilder::new(&format!(r#"(\s*{}\s*:\s*)[^,\s)]+"#, regex::escape(key)))
                .build()
                .unwrap();
            *content = re
                .replace_all(content, format!(r#"${{1}}{}"#, value))
                .to_string();
        };

        replace_str(&mut new_content, "font_sans", &self.font_sans);
        replace_str(&mut new_content, "font_serif", &self.font_serif);
        replace_str(&mut new_content, "font_mono", &self.font_mono);
        replace_str(&mut new_content, "theme_name", &self.theme_name);
        replace_str(&mut new_content, "theme_file", &self.theme_file);
        replace_val(
            &mut new_content,
            "webview_zoom",
            self.webview_zoom.to_string(),
        );
        replace_str(
            &mut new_content,
            "webview_theme_injection",
            &self.webview_theme_injection,
        );
        replace_str(
            &mut new_content,
            "webview_theme_mode",
            &self.webview_theme_mode,
        );
        replace_val(
            &mut new_content,
            "soft_wrap_max_run",
            self.soft_wrap_max_run.to_string(),
        );
        // Floating point numbers might need specific formatting, but to_string() is usually fine for RON
        replace_val(
            &mut new_content,
            "window_width",
            format!("{:.1}", self.window_width),
        );
        replace_val(
            &mut new_content,
            "window_height",
            format!("{:.1}", self.window_height),
        );

        match fs::write(&path, new_content) {
            Ok(_) => {
                tracing::info!("Updated config at {} (preserving comments)", path.display());
            }
            Err(e) => {
                tracing::error!("Failed to update config at {}: {}", path.display(), e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_save_preserves_comments() {
        use std::io::Write;

        // Create a temporary config file with comments
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("config_test_comments.ron");

        let initial_content = r#"(
    // This is a comment about fonts
    font_sans: "Test Sans",
    font_serif: "Test Serif",
    font_mono: "Test Mono",

    // Theme settings
    theme_name: "Old Theme",
    theme_file: "./themes",

    // Zoom level
    webview_zoom: 100,

    // Injection mode
    webview_theme_injection: "none",

    soft_wrap_max_run: 20,
    window_width: 800.0,
    window_height: 600.0,
)"#;

        {
            let mut file = fs::File::create(&config_path).unwrap();
            file.write_all(initial_content.as_bytes()).unwrap();
        }

        // Load config manually (since load() logic is complex with paths)
        let mut config: AppConfig = ron::from_str(initial_content).unwrap();

        // Modify values
        config.webview_theme_injection = "both".to_string();
        config.webview_zoom = 150;

        // Save to the temp path
        config.save_to(config_path.clone());

        // Read back
        let new_content = fs::read_to_string(&config_path).unwrap();

        // Verify values updated
        assert!(new_content.contains("webview_theme_injection: \"both\""));
        assert!(new_content.contains("webview_zoom: 150"));

        // Verify comments preserved
        assert!(new_content.contains("// This is a comment about fonts"));
        assert!(new_content.contains("// Theme settings"));
        assert!(new_content.contains("// Zoom level"));
        assert!(new_content.contains("// Injection mode"));

        // Cleanup
        let _ = fs::remove_file(config_path);
    }
}
