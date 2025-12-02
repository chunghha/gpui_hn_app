use crate::state::ViewMode;
use gpui::{Context, KeyDownEvent};

/// Scroll step size in pixels for j/k navigation
const SCROLL_STEP: f32 = 50.0;

/// Handle keyboard events for the HN Layout
pub fn handle_key_down(
    viewer: &mut crate::internal::layout::HnLayout,
    event: &KeyDownEvent,
    _window: &mut gpui::Window,
    cx: &mut Context<crate::internal::layout::HnLayout>,
) {
    let key = event.keystroke.key.as_str();

    // Debug logging
    tracing::debug!(
        "Key pressed: '{}', modifiers: {:?}",
        key,
        event.keystroke.modifiers
    );

    // Handle Cmd+Q (quit) - platform modifier is Cmd on Mac, Ctrl on Windows/Linux
    if event.keystroke.modifiers.platform && key == "q" {
        tracing::debug!("Quit application (Cmd/Ctrl+Q)");
        cx.quit();
        return;
    }

    // Handle single-key shortcuts (j, k, g, b, B, H, X)
    // Handle Escape key to return to List view from Bookmarks/History
    if key == "escape" {
        let view_mode = viewer.app_state.read(cx).view_mode.clone();
        match view_mode {
            ViewMode::Bookmarks | ViewMode::History => {
                tracing::debug!("Escape key - returning to List view");
                crate::state::AppState::show_stories(viewer.app_state.clone(), cx);
                cx.notify();
            }
            _ => {}
        }
        return;
    }

    // Handle Ctrl+R (Focus Search)
    if event.keystroke.modifiers.control && key == "r" {
        tracing::debug!("Focus search (Ctrl+R)");
        crate::state::AppState::trigger_search_focus(viewer.app_state.clone(), cx);
        cx.notify();
        return;
    }

    // Handle Ctrl+M (Cycle Search Mode)
    if event.keystroke.modifiers.control && key == "m" {
        tracing::debug!("Cycle search mode (Ctrl+M)");
        let current_mode = viewer.app_state.read(cx).search_mode;
        let next_mode = match current_mode {
            crate::state::SearchMode::Title => crate::state::SearchMode::Comments,
            crate::state::SearchMode::Comments => crate::state::SearchMode::Both,
            crate::state::SearchMode::Both => crate::state::SearchMode::Title,
        };
        crate::state::AppState::set_search_mode(viewer.app_state.clone(), next_mode, cx);
        cx.notify();
        return;
    }

    // Handle Ctrl+S (Cycle Sort Option)
    if event.keystroke.modifiers.control && key == "s" {
        tracing::debug!("Cycle sort option (Ctrl+S)");
        let current_option = viewer.app_state.read(cx).sort_option;
        let next_option = match current_option {
            crate::state::SortOption::Score => crate::state::SortOption::Comments,
            crate::state::SortOption::Comments => crate::state::SortOption::Time,
            crate::state::SortOption::Time => crate::state::SortOption::Score,
        };
        crate::state::AppState::set_sort_option(viewer.app_state.clone(), next_option, cx);
        cx.notify();
        return;
    }

    // Handle O (Toggle Sort Order)
    if key == "o" {
        tracing::debug!("Toggle sort order (o)");
        crate::state::AppState::toggle_sort_order(viewer.app_state.clone(), cx);
        cx.notify();
        return;
    }

    match key {
        "j" => {
            tracing::debug!("Scroll down (j)");
            let view_mode = viewer.app_state.read(cx).view_mode.clone();
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
                ViewMode::Webview(_) => {
                    // WebView handles its own scrolling
                }
            }
            cx.notify();
        }
        "k" => {
            tracing::debug!("Scroll up (k)");
            let view_mode = viewer.app_state.read(cx).view_mode.clone();
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
                ViewMode::Webview(_) => {
                    // WebView handles its own scrolling
                }
            }
            cx.notify();
        }
        "g" => {
            tracing::debug!("Scroll to top (g)");
            let view_mode = viewer.app_state.read(cx).view_mode.clone();
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
                ViewMode::Webview(_) => {
                    // WebView handles its own scrolling
                }
            }
            cx.notify();
        }
        "b" => {
            if event.keystroke.modifiers.shift {
                tracing::debug!("Show bookmarks (Shift+B)");
                crate::state::AppState::show_bookmarks(viewer.app_state.clone(), cx);
                cx.notify();
            } else {
                let view_mode = viewer.app_state.read(cx).view_mode.clone();
                tracing::debug!("Toggle bookmark (b) - current view_mode: {:?}", view_mode);
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
        }
        "h" => {
            if event.keystroke.modifiers.shift {
                tracing::debug!("Show history (Shift+H)");
                crate::state::AppState::show_history(viewer.app_state.clone(), cx);
                cx.notify();
            }
        }
        "x" => {
            if event.keystroke.modifiers.shift {
                tracing::debug!("Clear history (Shift+X)");
                let view_mode = viewer.app_state.read(cx).view_mode.clone();
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
        }
        _ => {
            // Other keys - do nothing
        }
    }
}
