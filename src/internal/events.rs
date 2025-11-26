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
    tracing::debug!("Key pressed: '{}'", key);

    // Handle Cmd+Q (quit) - platform modifier is Cmd on Mac, Ctrl on Windows/Linux
    if event.keystroke.modifiers.platform && key == "q" {
        tracing::debug!("Quit application (Cmd/Ctrl+Q)");
        cx.quit();
        return;
    }

    // Handle single-key shortcuts (j, k, g)
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
                ViewMode::Webview(_) => {
                    // WebView handles its own scrolling
                }
            }
            cx.notify();
        }
        _ => {
            // Other keys - do nothing
        }
    }
}
