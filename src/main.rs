use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

mod api;
mod config;
mod layout;
mod models;
mod scroll;
mod state;
mod webview;

use layout::HnLayout;
use state::AppState;

use std::process;

fn main() {
    tracing_subscriber::fmt::try_init().ok();

    Application::new().run(move |cx: &mut App| {
        gpui_component::theme::init(cx);

        // Load app config (includes preferred theme name and optional theme_file)
        let app_config = config::AppConfig::load();
        let theme_name = gpui::SharedString::from(app_config.theme_name.clone());

        // Determine a directory to watch for themes.
        // If config specifies a file, watch its parent directory; otherwise watch the configured path.
        let configured_path = std::path::PathBuf::from(app_config.theme_file.clone());
        let watch_path = if configured_path.is_file() {
            configured_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::path::PathBuf::from("./themes"))
        } else {
            configured_path
        };

        if let Err(err) = gpui_component::ThemeRegistry::watch_dir(watch_path, cx, move |cx| {
            if let Some(theme) = gpui_component::ThemeRegistry::global(cx)
                .themes()
                .get(&theme_name)
                .cloned()
            {
                gpui_component::Theme::global_mut(cx).apply_config(&theme);
            }
        }) {
            tracing::error!("Failed to watch themes directory: {}", err);
        }

        let bounds = Bounds::centered(
            None,
            size(px(app_config.window_width), px(app_config.window_height)),
            cx,
        );

        let app_state = AppState::new(app_config, cx);

        let result = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |window, cx| cx.new(move |cx| HnLayout::new(app_state.clone(), window, cx)),
        );

        if let Err(e) = result {
            tracing::error!(error = %e, "gpui-hn-app: failed to open window");
            process::exit(1);
        }
    });
}
