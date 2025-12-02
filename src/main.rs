use gpui::{App, Application, Bounds, WindowBounds, WindowOptions, prelude::*, px, size};

mod api;
mod bookmarks;
mod cache;
mod config;
mod history;
mod internal;
mod search;
mod state;
mod utils;

use crate::internal::layout::HnLayout;
use crate::internal::ui::{BookmarkListView, HistoryListView, StoryDetailView, StoryListView};
use crate::state::AppState;

use std::process;

/// Initialize file-based logging with daily rotation
fn init_logging() {
    let logs_dir = std::path::PathBuf::from("./logs");
    std::fs::create_dir_all(&logs_dir).ok();

    let file_appender = tracing_appender::rolling::daily(logs_dir, "gpui-hn-app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Leak the guard to keep it alive for the entire program lifetime
    // This is necessary to ensure logs are flushed properly
    Box::leak(Box::new(guard));

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

fn main() {
    init_logging();

    // Initialize Tokio runtime to support async operations in background tasks
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");
    let _guard = runtime.enter();

    Application::new().run(move |cx: &mut App| {
        gpui_component::theme::init(cx);

        // Load app config (includes preferred theme name and optional theme_file)
        let app_config = config::AppConfig::load();
        let theme_name = gpui::SharedString::from(app_config.theme_name.clone());

        // Determine a directory to watch for themes.
        // If config specifies a file, watch its parent directory; otherwise watch the configured path.
        let configured_path = std::path::PathBuf::from(app_config.theme_file.clone());
        let watch_path = match configured_path.is_file() {
            true => configured_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::path::PathBuf::from("./themes")),
            false => configured_path,
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
            move |window, cx| {
                cx.new(move |cx| {
                    let story_list_view = cx.new(|cx| StoryListView::new(app_state.clone(), cx));
                    let story_detail_view =
                        cx.new(|cx| StoryDetailView::new(app_state.clone(), cx));
                    let bookmark_list_view =
                        cx.new(|cx| BookmarkListView::new(app_state.clone(), cx));
                    let history_list_view =
                        cx.new(|cx| HistoryListView::new(app_state.clone(), cx));
                    HnLayout::new(
                        app_state.clone(),
                        story_list_view,
                        story_detail_view,
                        bookmark_list_view,
                        history_list_view,
                        window,
                        cx,
                    )
                })
            },
        );

        if let Err(e) = result {
            tracing::error!(error = %e, "gpui-hn-app: failed to open window");
            process::exit(1);
        }
    });
}
