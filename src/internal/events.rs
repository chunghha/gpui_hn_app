use crate::state::ViewMode;
use gpui::{AppContext, Context, KeyDownEvent};

/// Scroll step size in pixels for j/k navigation
const SCROLL_STEP: f32 = 50.0;

/// Handle keyboard events for the HN Layout
pub fn handle_key_down(
    viewer: &mut crate::internal::layout::HnLayout,
    event: &KeyDownEvent,
    window: &mut gpui::Window,
    cx: &mut Context<crate::internal::layout::HnLayout>,
) {
    let key = event.keystroke.key.as_str();

    // Construct key string for config lookup
    let modifiers = event.keystroke.modifiers;
    let modifier_parts: Vec<&str> = [
        (modifiers.control, "ctrl"),
        (modifiers.alt, "alt"),
        (modifiers.shift, "shift"),
        (modifiers.platform, "cmd"),
    ]
    .iter()
    .filter_map(|(active, name)| active.then_some(*name))
    .collect();

    let key_string = if modifier_parts.is_empty() {
        key.to_string()
    } else {
        format!("{}+{}", modifier_parts.join("+"), key)
    };

    // Debug logging
    tracing::debug!(
        "Key pressed: '{}', modifiers: {:?}, resolved: '{}'",
        key,
        event.keystroke.modifiers,
        key_string
    );

    // Handle Cmd+Q (quit) - platform modifier is Cmd on Mac, Ctrl on Windows/Linux
    // We keep this as a hardcoded fallback/override for safety
    if event.keystroke.modifiers.platform && key == "q" {
        tracing::debug!("Quit application (Cmd/Ctrl+Q)");
        cx.quit();
        return;
    }

    // Resolve action from config
    let app_state = viewer.app_state.read(cx);
    let action = app_state
        .config
        .keybindings
        .get(&key_string)
        .unwrap_or(&crate::config::Action::None);

    tracing::debug!("Resolved action: {:?}", action);

    match action {
        crate::config::Action::Quit => {
            cx.quit();
        }
        crate::config::Action::Back => {
            let view_mode = app_state.view_mode.clone();
            match view_mode {
                ViewMode::Bookmarks
                | ViewMode::History
                | ViewMode::ThemeEditor
                | ViewMode::LogViewer => {
                    tracing::debug!("Back action - returning to List view");
                    crate::state::AppState::show_stories(viewer.app_state.clone(), cx);
                    cx.notify();
                }
                _ => {}
            }
        }
        crate::config::Action::FocusSearch => {
            tracing::debug!("Focus search");
            crate::state::AppState::trigger_search_focus(viewer.app_state.clone(), cx);
            cx.notify();
        }
        crate::config::Action::CycleSearchMode => {
            tracing::debug!("Cycle search mode");
            let current_mode = app_state.search_mode;
            let next_mode = match current_mode {
                crate::state::SearchMode::Title => crate::state::SearchMode::Comments,
                crate::state::SearchMode::Comments => crate::state::SearchMode::Both,
                crate::state::SearchMode::Both => crate::state::SearchMode::Title,
            };
            crate::state::AppState::set_search_mode(viewer.app_state.clone(), next_mode, cx);
            cx.notify();
        }
        crate::config::Action::CycleSortOption => {
            tracing::debug!("Cycle sort option");
            let current_option = app_state.sort_option;
            let next_option = match current_option {
                crate::state::SortOption::Score => crate::state::SortOption::Comments,
                crate::state::SortOption::Comments => crate::state::SortOption::Time,
                crate::state::SortOption::Time => crate::state::SortOption::Score,
            };
            crate::state::AppState::set_sort_option(viewer.app_state.clone(), next_option, cx);
            cx.notify();
        }
        crate::config::Action::ToggleSortOrder => {
            tracing::debug!("Toggle sort order");
            crate::state::AppState::toggle_sort_order(viewer.app_state.clone(), cx);
            cx.notify();
        }
        crate::config::Action::ScrollDown => {
            tracing::debug!("Scroll down");
            let view_mode = app_state.view_mode.clone();
            match view_mode {
                ViewMode::List => {
                    viewer.story_list_view().update(cx, |view, _| {
                        view.scroll_by(SCROLL_STEP);
                    });
                }
                ViewMode::Story(_) => {
                    viewer.story_detail_view().update(cx, |view, _| {
                        view.scroll_by(SCROLL_STEP);
                    });
                }
                ViewMode::Bookmarks => {
                    viewer.bookmark_list_view().update(cx, |view, _| {
                        view.scroll_by(SCROLL_STEP);
                    });
                }
                ViewMode::History => {
                    viewer.history_list_view().update(cx, |view, _| {
                        view.scroll_by(SCROLL_STEP);
                    });
                }
                ViewMode::Webview(_) | ViewMode::ThemeEditor | ViewMode::LogViewer => {
                    // WebView, ThemeEditor, and LogViewer handle their own scrolling
                }
            }
            cx.notify();
        }
        crate::config::Action::ScrollUp => {
            tracing::debug!("Scroll up");
            let view_mode = app_state.view_mode.clone();
            match view_mode {
                ViewMode::List => {
                    viewer.story_list_view().update(cx, |view, _| {
                        view.scroll_by(-SCROLL_STEP);
                    });
                }
                ViewMode::Story(_) => {
                    viewer.story_detail_view().update(cx, |view, _| {
                        view.scroll_by(-SCROLL_STEP);
                    });
                }
                ViewMode::Bookmarks => {
                    viewer.bookmark_list_view().update(cx, |view, _| {
                        view.scroll_by(-SCROLL_STEP);
                    });
                }
                ViewMode::History => {
                    viewer.history_list_view().update(cx, |view, _| {
                        view.scroll_by(-SCROLL_STEP);
                    });
                }
                ViewMode::Webview(_) | ViewMode::ThemeEditor | ViewMode::LogViewer => {
                    // WebView, ThemeEditor, and LogViewer handle their own scrolling
                }
            }
            cx.notify();
        }
        crate::config::Action::ScrollToTop => {
            tracing::debug!("Scroll to top");
            let view_mode = app_state.view_mode.clone();
            match view_mode {
                ViewMode::List => {
                    viewer.story_list_view().update(cx, |view, _| {
                        view.scroll_to_top();
                    });
                }
                ViewMode::Story(_) => {
                    viewer.story_detail_view().update(cx, |view, _| {
                        view.scroll_to_top();
                    });
                }
                ViewMode::Bookmarks => {
                    viewer.bookmark_list_view().update(cx, |view, _| {
                        view.scroll_to_top();
                    });
                }
                ViewMode::History => {
                    viewer.history_list_view().update(cx, |view, _| {
                        view.scroll_to_top();
                    });
                }
                ViewMode::Webview(_) | ViewMode::ThemeEditor | ViewMode::LogViewer => {
                    // WebView, ThemeEditor, and LogViewer handle their own scrolling
                }
            }
            cx.notify();
        }
        crate::config::Action::ToggleBookmark => {
            let view_mode = app_state.view_mode.clone();
            tracing::debug!("Toggle bookmark - current view_mode: {:?}", view_mode);
            match view_mode {
                ViewMode::Story(_) => {
                    tracing::debug!("Toggling bookmark for current story");
                    crate::state::AppState::toggle_bookmark(viewer.app_state.clone(), cx);
                    cx.notify();
                }
                _ => {
                    tracing::debug!("Cannot bookmark - not in Story view");
                }
            }
        }
        crate::config::Action::ShowBookmarks => {
            tracing::debug!("Show bookmarks");
            crate::state::AppState::show_bookmarks(viewer.app_state.clone(), cx);
            cx.notify();
        }
        crate::config::Action::ShowHistory => {
            tracing::debug!("Show history");
            crate::state::AppState::show_history(viewer.app_state.clone(), cx);
            cx.notify();
        }
        crate::config::Action::ClearHistory => {
            tracing::debug!("Clear history");
            let view_mode = app_state.view_mode.clone();
            match view_mode {
                ViewMode::History => {
                    crate::state::AppState::clear_history(viewer.app_state.clone(), cx);
                    cx.notify();
                }
                _ => {
                    // Only allow clearing history in History view
                }
            }
        }
        crate::config::Action::None => {
            // Do nothing
        }
        crate::config::Action::OpenThemeEditor => {
            tracing::info!("Opening theme editor");
            // Re-initialize theme editor to pick up current theme colors
            viewer.theme_editor_view = cx
                .new(|cx| crate::internal::ui::ThemeEditorView::new(viewer.app_state.clone(), cx));

            viewer.theme_editor_view.update(cx, |view, _cx| {
                window.focus(&view.focus_handle);
            });

            viewer.app_state.update(cx, |state, cx| {
                state.view_mode = ViewMode::ThemeEditor;
                cx.notify();
            });
        }
        crate::config::Action::ShowLogViewer => {
            tracing::info!("Opening log viewer");
            viewer.app_state.update(cx, |state, cx| {
                state.view_mode = ViewMode::LogViewer;
                cx.notify();
            });
        }
        crate::config::Action::ShowKeyboardHelp => {
            tracing::debug!("Toggle keyboard help overlay");
            viewer.app_state.update(cx, |state, cx| {
                state.show_keyboard_help = !state.show_keyboard_help;
                cx.notify();
            });
        }
    }
}
