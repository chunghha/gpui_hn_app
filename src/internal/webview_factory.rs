use crate::config::AppConfig;
use crate::internal::webview::make_init_script;
use gpui::{AppContext, Context, Entity, Window};
use gpui_component::theme::ActiveTheme;
use gpui_component::webview::WebView;

/// Creates a WebView with platform-specific initialization.
pub fn create_webview<V>(
    window: &mut Window,
    cx: &mut Context<V>,
    config: &AppConfig,
) -> Entity<WebView> {
    cx.new(|cx| {
        let builder = gpui_component::wry::WebViewBuilder::new();

        // Get theme and zoom configuration
        let zoom_level = config.webview_zoom;

        // Get theme colors
        let theme = cx.theme();
        let is_dark_theme = theme.is_dark();
        let colors = theme.colors;

        // Convert GPUI Hsla colors to CSS hex strings using utility
        let bg_color = crate::utils::theme::hsla_to_hex(colors.background);
        let fg_color = crate::utils::theme::hsla_to_hex(colors.foreground);
        let link_color = crate::utils::theme::hsla_to_hex(colors.info);

        tracing::info!(
            "Initializing webview with zoom: {}%, dark mode: {}, bg: {}, fg: {}",
            zoom_level,
            is_dark_theme,
            bg_color,
            fg_color
        );

        // Build initialization script
        let init_script = make_init_script(
            config,
            is_dark_theme,
            &bg_color,
            &fg_color,
            &link_color,
            zoom_level,
        );
        let builder = builder.with_initialization_script(&init_script);

        // Platform-specific WebView creation
        #[cfg(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        ))]
        let wry_webview = {
            use raw_window_handle::HasWindowHandle;
            let window_handle = window.window_handle().expect("No window handle");
            builder.build_as_child(&window_handle).unwrap()
        };

        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "ios",
            target_os = "android"
        )))]
        let wry_webview = {
            use gpui_component::wry::WebViewBuilderExtUnix;
            use gtk::prelude::*;
            let fixed = gtk::Fixed::builder().build();
            fixed.show_all();
            builder.build_gtk(&fixed).unwrap()
        };

        WebView::new(wry_webview, window, cx)
    })
}
