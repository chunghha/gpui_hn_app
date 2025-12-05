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
    current_url: Option<&str>,
) -> String {
    // Prepare values and JSON-encode them so they're safe to interpolate into JS.
    let zoom_str = format!("{}%", zoom_level);
    let zoom_js = serde_json::to_string(&zoom_str).unwrap_or_else(|_| "\"100%\"".to_string());
    let is_dark_js = match is_dark_theme {
        true => "true",
        false => "false",
    };
    let bg_js = serde_json::to_string(&bg_color).unwrap_or_else(|_| "\"#000000\"".to_string());
    let fg_js = serde_json::to_string(&fg_color).unwrap_or_else(|_| "\"#000000\"".to_string());
    let link_js = serde_json::to_string(&link_color).unwrap_or_else(|_| "\"#0000FF\"".to_string());

    // Decide whether to inject theme styles based on config string and current theme darkness.
    // Accepts case-insensitive values: "none", "light", "dark", "both".
    // Unknown/invalid values are treated as "none" (do not inject) to avoid modifying pages.
    let inject_theme = {
        let mode = config.webview_theme_injection.to_lowercase();
        let mode_enabled = match mode.as_str() {
            "none" => false,
            "light" => !is_dark_theme,
            "dark" => is_dark_theme,
            "both" => true,
            _ => false,
        };

        // Check domain whitelist if enabled
        match (mode_enabled, config.webview_trusted_domains.is_empty()) {
            (false, _) => false,
            (true, true) => true, // No whitelist configured, allow all
            (true, false) => {
                // Whitelist configured, check if domain is trusted
                current_url
                    .map(|url| {
                        let domain = url
                            .trim_start_matches("https://")
                            .trim_start_matches("http://")
                            .split('/')
                            .next()
                            .unwrap_or("");
                        config.webview_trusted_domains.iter().any(|trusted| {
                            domain == trusted.as_str() || domain.ends_with(&format!(".{}", trusted))
                        })
                    })
                    .unwrap_or(false)
            }
        }
    };

    // Determine theming mode: invasive (!important) or non-invasive (CSS vars)
    let use_css_vars = config.webview_theme_mode.to_lowercase() == "css-vars";

    // Build the JS snippet that conditionally injects a scoped stylesheet.
    let style_block = match inject_theme {
        true => match use_css_vars {
            true => {
                // Non-invasive CSS variable mode - sets custom properties without forcing
                r#"
                var style = document.createElement('style');
                style.setAttribute('data-gpui-theme', 'css-vars');
                var css = '';

                // Set CSS custom properties on :root that sites can optionally use
                css += ':root {' +
                    '--gpui-bg-color: ' + bgColor + ';' +
                    '--gpui-fg-color: ' + fgColor + ';' +
                    '--gpui-link-color: ' + linkColor + ';' +
                '}';

                // Gentle application without !important - respects existing site styles
                css += 'body:not([style*="background"]) {' +
                    'background-color: var(--gpui-bg-color);' +
                    'color: var(--gpui-fg-color);' +
                '}';

                css += 'a:not([style*="color"]) {' +
                    'color: var(--gpui-link-color);' +
                    '}' +
                '}';

                // Content containers can also optionally pick up the variables
                css += 
                    'main:not([style*="background"]), ' +
                    'article:not([style*="background"]), ' +
                    '.content:not([style*="background"]) {' +
                    'background-color: var(--gpui-bg-color);' +
                    'color: var(--gpui-fg-color);' +
                '}';

                style.textContent = css;
                document.head.appendChild(style);
            "#
                .to_string()
            }
            false => {
                // Invasive mode - uses !important with background detection (current enhanced behavior)
                r#"
                // Helper function to check if a color is transparent
                function isTransparent(color) {
                    if (!color || color === 'transparent') return true;
                    if (color === 'rgba(0, 0, 0, 0)') return true;
                    if (color.startsWith('rgba')) {
                        // Check if alpha channel is 0
                        var match = color.match(/rgba?\([^)]+,\s*([0-9.]+)\)/);
                        return match && parseFloat(match[1]) === 0;
                    }
                    return false;
                }

                var style = document.createElement('style');
                style.setAttribute('data-gpui-theme', 'invasive');
                var css = '';

                // Check if html element has a background image
                var htmlStyle = getComputedStyle(document.documentElement);
                var htmlBgImage = htmlStyle.backgroundImage;
                var htmlBgColor = htmlStyle.backgroundColor;

                // Only apply to html if it has no background image
                if (!htmlBgImage || htmlBgImage === 'none') {
                    css += 'html { background-color: ' + bgColor + ' !important; min-height: 100%; }';
                }

                // Check body background
                if (document.body) {
                    var bodyStyle = getComputedStyle(document.body);
                    var bodyBgImage = bodyStyle.backgroundImage;
                    var bodyBgColor = bodyStyle.backgroundColor;

                    // Only apply to body if it has no background image and is transparent
                    if ((!bodyBgImage || bodyBgImage === 'none') && isTransparent(bodyBgColor)) {
                        css += 'body { background-color: transparent !important; }';
                    }
                }

                // Apply to content containers using :not() selectors to avoid overriding
                // elements with inline styles, background classes, or existing background images
                css += 
                    'main:not([style*="background"]):not([class*="bg-"]), ' +
                    'article:not([style*="background"]):not([class*="bg-"]), ' +
                    '.content:not([style*="background"]):not([class*="bg-"]), ' +
                    '.post:not([style*="background"]):not([class*="bg-"]), ' +
                    '.post-content:not([style*="background"]):not([class*="bg-"]), ' +
                    '#content:not([style*="background"]):not([class*="bg-"]) { ' +
                    '    background-color: ' + bgColor + ' !important; ' +
                    '    color: ' + fgColor + ' !important; ' +
                    '}';

                // Apply link colors with more specificity
                css += 
                    'a, a:link, a:visited { ' +
                    '    color: ' + linkColor + ' !important; ' +
                    '}';

                style.textContent = css;
                document.head.appendChild(style);
            "#
                .to_string()
            }
        },
        false => String::new(),
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
            webview_theme_mode: "invasive".to_string(),
            webview_trusted_domains: Vec::new(),
            soft_wrap_max_run: 20,
            window_width: 980.0,
            window_height: 720.0,
            keybindings: Default::default(),
            ui: Default::default(),
            network: Default::default(),
            log: Default::default(),
            accessibility: Default::default(),
        }
    }

    #[test]
    fn test_zoom_always_included() {
        let config = mock_config("none");
        let script = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 100, None);

        // Verify zoom is included
        assert!(script.contains("var zoom = \"100%\""));
        assert!(script.contains("document.body.style.zoom = zoom"));
    }

    #[test]
    fn test_zoom_levels() {
        let config = mock_config("none");

        let script_100 =
            make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 100, None);
        assert!(script_100.contains("\"100%\""));

        let script_120 =
            make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120, None);
        assert!(script_120.contains("\"120%\""));

        let script_150 =
            make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 150, None);
        assert!(script_150.contains("\"150%\""));
    }

    #[test]
    fn test_theme_injection_none() {
        let config = mock_config("none");

        // Should not inject theme for either dark or light
        let script_dark =
            make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);
        assert!(!script_dark.contains("createElement('style')"));

        let script_light =
            make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120, None);
        assert!(!script_light.contains("createElement('style')"));
    }

    #[test]
    fn test_theme_injection_light_only() {
        let config = mock_config("light");

        // Should inject for light theme only
        let script_light =
            make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120, None);
        assert!(script_light.contains("createElement('style')"));
        assert!(script_light.contains("data-gpui-theme"));

        // Should NOT inject for dark theme
        let script_dark =
            make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);
        assert!(!script_dark.contains("createElement('style')"));
    }

    #[test]
    fn test_theme_injection_dark_only() {
        let config = mock_config("dark");

        // Should inject for dark theme only
        let script_dark =
            make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);
        assert!(script_dark.contains("createElement('style')"));
        assert!(script_dark.contains("data-gpui-theme"));

        // Should NOT inject for light theme
        let script_light =
            make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120, None);
        assert!(!script_light.contains("createElement('style')"));
    }

    #[test]
    fn test_theme_injection_both() {
        let config = mock_config("both");

        // Should inject for both dark and light themes
        let script_dark =
            make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);
        assert!(script_dark.contains("createElement('style')"));

        let script_light =
            make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120, None);
        assert!(script_light.contains("createElement('style')"));
    }

    #[test]
    fn test_theme_injection_invalid_mode() {
        let config = mock_config("invalid_mode");

        // Unknown modes should be treated as "none"
        let script = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120, None);
        assert!(!script.contains("createElement('style')"));
    }

    #[test]
    fn test_color_values_in_script() {
        let config = mock_config("both");
        let script = make_init_script(&config, true, "#123456", "#ABCDEF", "#FF00FF", 120, None);

        // Verify colors are JSON-encoded and present
        assert!(script.contains("var bgColor = \"#123456\""));
        assert!(script.contains("var fgColor = \"#ABCDEF\""));
        assert!(script.contains("var linkColor = \"#FF00FF\""));
    }

    #[test]
    fn test_dark_theme_flag() {
        let config = mock_config("none");

        let script_dark =
            make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);
        assert!(script_dark.contains("var isDarkTheme = true"));

        let script_light =
            make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120, None);
        assert!(script_light.contains("var isDarkTheme = false"));
    }

    #[test]
    fn test_css_selectors_present() {
        let config = mock_config("both");
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // Verify CSS selectors are included when theme injection is enabled
        assert!(script.contains("html {"));
        assert!(script.contains("body {"));
        // Check for enhanced selectors with :not() clauses
        assert!(script.contains("main:not([style*=\"background\"]):not([class*=\"bg-\"])"));
        assert!(script.contains("article:not([style*=\"background\"]):not([class*=\"bg-\"])"));
        assert!(script.contains("a, a:link, a:visited {"));
    }

    #[test]
    fn test_script_has_iife_structure() {
        let config = mock_config("none");
        let script = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120, None);

        // Verify the script has IIFE (Immediately Invoked Function Expression) structure
        assert!(script.contains("(function() {"));
        assert!(script.contains("})();"));
    }

    #[test]
    fn test_dom_ready_handling() {
        let config = mock_config("none");
        let script = make_init_script(&config, false, "#FFFFFF", "#000000", "#0000FF", 120, None);

        // Verify DOMContentLoaded event handling
        assert!(script.contains("if (document.readyState === 'loading')"));
        assert!(script.contains("document.addEventListener('DOMContentLoaded', applyStyles)"));
        assert!(script.contains("applyStyles();"));
    }

    #[test]
    fn test_case_insensitive_injection_mode() {
        // Test that uppercase "BOTH" works
        let config_upper = mock_config("BOTH");
        let script_upper = make_init_script(
            &config_upper,
            true,
            "#000000",
            "#FFFFFF",
            "#0000FF",
            120,
            None,
        );
        assert!(script_upper.contains("createElement('style')"));

        // Test that mixed case "Dark" works
        let config_mixed = mock_config("Dark");
        let script_mixed = make_init_script(
            &config_mixed,
            true,
            "#000000",
            "#FFFFFF",
            "#0000FF",
            120,
            None,
        );
        assert!(script_mixed.contains("createElement('style')"));
    }

    #[test]
    fn test_background_image_detection() {
        let config = mock_config("both");
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // Verify background image detection is included
        assert!(script.contains("getComputedStyle(document.documentElement)"));
        assert!(script.contains("htmlBgImage"));
        assert!(script.contains("bodyBgImage"));
        assert!(script.contains("htmlBgImage === 'none'"));
    }

    #[test]
    fn test_transparency_detection_helper() {
        let config = mock_config("both");
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // Verify isTransparent helper function is included
        assert!(script.contains("function isTransparent(color)"));
        assert!(script.contains("color === 'transparent'"));
        assert!(script.contains("rgba(0, 0, 0, 0)"));
    }

    #[test]
    fn test_selective_selector_targeting() {
        let config = mock_config("both");
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // Verify :not() selectors are used to avoid overriding existing styles
        assert!(script.contains(":not([style*=\"background\"])"));
        assert!(script.contains(":not([class*=\"bg-\"])"));

        // Check that content selectors use the selective targeting
        assert!(script.contains(".content:not([style*=\"background\"]):not([class*=\"bg-\"])"));
        assert!(script.contains(".post:not([style*=\"background\"]):not([class*=\"bg-\"])"));
    }

    #[test]
    fn test_css_vars_mode() {
        let mut config = mock_config("both");
        config.webview_theme_mode = "css-vars".to_string();
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // Verify CSS variables are set
        assert!(script.contains("'data-gpui-theme', 'css-vars'"));
        assert!(script.contains(":root {"));
        assert!(script.contains("--gpui-bg-color:"));
        assert!(script.contains("--gpui-fg-color:"));
        assert!(script.contains("--gpui-link-color:"));
    }

    #[test]
    fn test_invasive_mode_default() {
        let config = mock_config("both");
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // Verify invasive mode is used by default
        assert!(script.contains("'data-gpui-theme', 'invasive'"));
        assert!(script.contains("!important"));
    }

    #[test]
    fn test_css_vars_no_important() {
        let mut config = mock_config("both");
        config.webview_theme_mode = "css-vars".to_string();
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // Verify CSS vars mode does NOT use !important for theme colors
        assert!(!script.contains("background-color:' + bgColor + ' !important"));
        assert!(!script.contains("color:' + fgColor + ' !important"));
    }

    #[test]
    fn test_css_vars_root_selector() {
        let mut config = mock_config("both");
        config.webview_theme_mode = "css-vars".to_string();
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // Verify :root selector is used for CSS variables
        assert!(script.contains(":root {"));
        assert!(script.contains("var(--gpui-bg-color)"));
        assert!(script.contains("var(--gpui-fg-color)"));
        assert!(script.contains("var(--gpui-link-color)"));
    }

    #[test]
    fn test_css_vars_respects_injection_mode() {
        let mut config = mock_config("none");
        config.webview_theme_mode = "css-vars".to_string();
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // If injection is "none", CSS vars should not be injected either
        assert!(!script.contains(":root"));
        assert!(!script.contains("--gpui-bg-color"));
    }

    #[test]
    fn test_css_vars_case_insensitive() {
        let mut config = mock_config("both");
        config.webview_theme_mode = "CSS-VARS".to_string();
        let script = make_init_script(&config, true, "#000000", "#FFFFFF", "#0000FF", 120, None);

        // Verify mode detection is case-insensitive
        assert!(script.contains("'data-gpui-theme', 'css-vars'"));
        assert!(script.contains(":root {"));
    }
}
