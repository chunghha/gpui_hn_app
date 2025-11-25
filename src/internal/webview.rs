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
