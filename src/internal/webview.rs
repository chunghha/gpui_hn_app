use crate::config::AppConfig;

/// Utilities for building the WebView initialization script.
///
/// This module intentionally focuses on constructing the JavaScript that gets
/// injected into the WebView on initialization. The existing platform-specific
/// builder and the actual `WebView::new(...)` call remain the responsibility of
/// the caller (e.g. `layout.rs`) so that platform-dependent window-handle
/// logic stays local to the place where the WebView is created.
///
/// Public API:
/// - `make_init_script(...) -> String` — builds the JS initialization script
///   which applies zoom and (optionally) a scoped theme stylesheet into the
///   page. The caller supplies already-formatted hex color strings for the
///   theme colors (e.g. `#RRGGBB`).
///
/// Example usage (pseudocode):
/// ```ignore
/// let init = gpui_hn_app::webview::make_init_script(&config, is_dark, &bg, &fg, &link, zoom);
/// builder = builder.with_initialization_script(&init);
/// ```
pub fn make_init_script(
    config: &AppConfig,
    is_dark_theme: bool,
    bg_color: &str,
    fg_color: &str,
    link_color: &str,
    zoom_level: u32,
) -> String {
    // Prepare values and JSON-encode them so they're safe to interpolate into JS.
    let zoom_str = format!("{}%", zoom_level);
    let zoom_js = serde_json::to_string(&zoom_str).unwrap_or_else(|_| "\"100%\"".to_string());
    let is_dark_js = if is_dark_theme { "true" } else { "false" };
    let bg_js = serde_json::to_string(&bg_color).unwrap_or_else(|_| "\"#000000\"".to_string());
    let fg_js = serde_json::to_string(&fg_color).unwrap_or_else(|_| "\"#000000\"".to_string());
    let link_js = serde_json::to_string(&link_color).unwrap_or_else(|_| "\"#0000FF\"".to_string());

    // Decide whether to inject theme styles based on config string and current theme darkness.
    // Accepts case-insensitive values: "none", "light", "dark", "both".
    // Unknown/invalid values are treated as "none" (do not inject) to avoid modifying pages.
    let inject_theme = {
        let mode = config.webview_theme_injection.to_lowercase();
        match mode.as_str() {
            "none" => false,
            "light" => !is_dark_theme,
            "dark" => is_dark_theme,
            "both" => true,
            _ => false,
        }
    };

    // Build the JS snippet that conditionally injects a scoped stylesheet.
    // We scope the CSS to a small set of common content selectors and also set
    // the `html` background to cover padding/margin regions.
    let style_block = if inject_theme {
        // Keep the block as a JS fragment that uses the already-JSON-encoded values.
        r#"
                var style = document.createElement('style');
                style.setAttribute('data-gpui-theme', '1');
                style.textContent =
                    // Apply theme background to the page root so margin/padding areas are covered
                    'html {' +
                    '    background-color: ' + bgColor + ' !important;' +
                    '    min-height: 100%;' +
                    '}' +
                    // Keep body transparent so html background is visible in padding areas
                    'body {' +
                    '    background-color: transparent !important;' +
                    '}' +
                    // Scoped styling for typical article/content containers for contrast
                    'main, article, .content, .post, .post-content, #content {' +
                    '    background-color: ' + bgColor + ' !important;' +
                    '    color: ' + fgColor + ' !important;' +
                    '}' +
                    'a, a:link, a:visited {' +
                    '    color: ' + linkColor + ' !important;' +
                    '}' ;
                document.head.appendChild(style);
        "#
        .to_string()
    } else {
        String::new()
    };

    // Construct the final initialization script. It always applies zoom. The style_block
    // is injected only when enabled by the config + theme darkness rules.
    let init_script = format!(
        r#"
            (function() {{
                var zoom = {zoom};
                var isDarkTheme = {is_dark};
                var bgColor = {bg};
                var fgColor = {fg};
                var linkColor = {link};

                var applyStyles = function() {{
                    // Apply zoom to body or html, whichever exists.
                    if (document.body) {{
                        document.body.style.zoom = zoom;
                    }} else if (document.documentElement) {{
                        document.documentElement.style.zoom = zoom;
                    }}

                    // Conditionally inject a scoped stylesheet to align WebView content with app theme.
                    {style_block}
                }};

                if (document.readyState === 'loading') {{
                    document.addEventListener('DOMContentLoaded', applyStyles);
                }} else {{
                    applyStyles();
                }}
            }})();
        "#,
        zoom = zoom_js,
        is_dark = is_dark_js,
        bg = bg_js,
        fg = fg_js,
        link = link_js,
        style_block = style_block
    );

    init_script
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_config(theme_injection: &str) -> AppConfig {
        AppConfig {
            font_sans: "Test Sans".to_string(),
            font_serif: "Test Serif".to_string(),
            font_mono: "Test Mono".to_string(),
            theme_name: "Test Theme".to_string(),
            theme_file: "./themes".to_string(),
            webview_zoom: 120,
            webview_theme_injection: theme_injection.to_string(),
            soft_wrap_max_run: 20,
            window_width: 980.0,
            window_height: 720.0,
        }
    }

    #[test]
    fn test_zoom_always_included() {
        let config = mock_config("none");
        let script = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 100);

        // Verify zoom is included
        assert!(script.contains("var zoom = \"100%\""));
        assert!(script.contains("document.body.style.zoom = zoom"));
    }

    #[test]
    fn test_zoom_levels() {
        let config = mock_config("none");

        let script_100 = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 100);
        assert!(script_100.contains("\"100%\""));

        let script_120 = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120);
        assert!(script_120.contains("\"120%\""));

        let script_150 = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 150);
        assert!(script_150.contains("\"150%\""));
    }

    #[test]
    fn test_theme_injection_none() {
        let config = mock_config("none");

        // Should not inject theme for either dark or light
        let script_dark = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120);
        assert!(!script_dark.contains("createElement('style')"));

        let script_light = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120);
        assert!(!script_light.contains("createElement('style')"));
    }

    #[test]
    fn test_theme_injection_light_only() {
        let config = mock_config("light");

        // Should inject for light theme only
        let script_light = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120);
        assert!(script_light.contains("createElement('style')"));
        assert!(script_light.contains("data-gpui-theme"));

        // Should NOT inject for dark theme
        let script_dark = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120);
        assert!(!script_dark.contains("createElement('style')"));
    }

    #[test]
    fn test_theme_injection_dark_only() {
        let config = mock_config("dark");

        // Should inject for dark theme only
        let script_dark = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120);
        assert!(script_dark.contains("createElement('style')"));
        assert!(script_dark.contains("data-gpui-theme"));

        // Should NOT inject for light theme
        let script_light = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120);
        assert!(!script_light.contains("createElement('style')"));
    }

    #[test]
    fn test_theme_injection_both() {
        let config = mock_config("both");

        // Should inject for both dark and light themes
        let script_dark = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120);
        assert!(script_dark.contains("createElement('style')"));

        let script_light = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120);
        assert!(script_light.contains("createElement('style')"));
    }

    #[test]
    fn test_theme_injection_invalid_mode() {
        let config = mock_config("invalid_mode");

        // Unknown modes should be treated as "none"
        let script = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120);
        assert!(!script.contains("createElement('style')"));
    }

    #[test]
    fn test_color_values_in_script() {
        let config = mock_config("both");
        let script = make_init_script(&config, true, "#123456", "#ABCDEF", "#FF00FF", 120);

        // Verify colors are JSON-encoded and present
        assert!(script.contains("var bgColor = \"#123456\""));
        assert!(script.contains("var fgColor = \"#ABCDEF\""));
        assert!(script.contains("var linkColor = \"#FF00FF\""));
    }

    #[test]
    fn test_dark_theme_flag() {
        let config = mock_config("none");

        let script_dark = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120);
        assert!(script_dark.contains("var isDarkTheme = true"));

        let script_light = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120);
        assert!(script_light.contains("var isDarkTheme = false"));
    }

    #[test]
    fn test_css_selectors_present() {
        let config = mock_config("both");
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120);

        // Verify CSS selectors are included when theme injection is enabled
        assert!(script.contains("html {"));
        assert!(script.contains("body {"));
        assert!(script.contains("main, article, .content, .post, .post-content, #content {"));
        assert!(script.contains("a, a:link, a:visited {"));
    }

    #[test]
    fn test_script_has_iife_structure() {
        let config = mock_config("none");
        let script = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120);

        // Verify the script has IIFE (Immediately Invoked Function Expression) structure
        assert!(script.contains("(function() {"));
        assert!(script.contains("})();"));
    }

    #[test]
    fn test_dom_ready_handling() {
        let config = mock_config("none");
        let script = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120);

        // Verify DOMContentLoaded event handling
        assert!(script.contains("if (document.readyState === 'loading')"));
        assert!(script.contains("document.addEventListener('DOMContentLoaded', applyStyles)"));
        assert!(script.contains("applyStyles();"));
    }

    #[test]
    fn test_case_insensitive_injection_mode() {
        // Test that uppercase "BOTH" works
        let config_upper = mock_config("BOTH");
        let script_upper =
            make_init_script(&config_upper, true, "#000000", "#FFFFFF", "#0000FF", 120);
        assert!(script_upper.contains("createElement('style')"));

        // Test that mixed case "Dark" works
        let config_mixed = mock_config("Dark");
        let script_mixed =
            make_init_script(&config_mixed, true, "#000000", "#FFFFFF", "#0000FF", 120);
        assert!(script_mixed.contains("createElement('style')"));
    }
}
