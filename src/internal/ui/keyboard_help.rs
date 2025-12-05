use crate::config::Action;
use crate::state::ViewMode;
use gpui::{Context, Entity, IntoElement, Render, Window, div, prelude::*};
use gpui_component::theme::ActiveTheme;

use crate::state::AppState;

/// Keyboard Help Overlay - shows context-sensitive keyboard shortcuts
pub struct KeyboardHelpOverlay {
    app_state: Entity<AppState>,
}

impl KeyboardHelpOverlay {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        cx.observe(&app_state, |_, _, cx| cx.notify()).detach();
        Self { app_state }
    }
}

impl Render for KeyboardHelpOverlay {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.read(cx);
        let show = app_state.show_keyboard_help;
        let view_mode = app_state.view_mode.clone();
        let keybindings = app_state.config.keybindings.clone();
        let _ = app_state;

        if !show {
            return div();
        }

        let colors = cx.theme().colors;

        // Get context-specific shortcuts
        let shortcuts = get_shortcuts_for_context(&view_mode, &keybindings);

        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(colors.background.opacity(0.9))
            .on_mouse_down(gpui::MouseButton::Left, {
                let app_state = self.app_state.clone();
                move |_, _, cx| {
                    app_state.update(cx, |state, cx| {
                        state.show_keyboard_help = false;
                        cx.notify();
                    });
                }
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p_6()
                    .rounded_lg()
                    .bg(colors.secondary)
                    .border_1()
                    .border_color(colors.border)
                    .max_w(gpui::px(800.0))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .mb_6()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(colors.foreground)
                                    .child("Keyboard Shortcuts"),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(colors.muted_foreground)
                                    .child(format!("Context: {:?}", view_mode)),
                            ),
                    )
                    .child({
                        let midpoint = shortcuts.len().div_ceil(2);
                        let (left_col, right_col) = shortcuts.split_at(midpoint);

                        div()
                            .flex()
                            .gap_8()
                            .child(
                                // Left Column
                                div().flex_1().flex().flex_col().gap_2().children(
                                    left_col
                                        .iter()
                                        .map(|(key, desc)| render_shortcut_row(key, desc, &colors)),
                                ),
                            )
                            .child(
                                // Right Column
                                div().flex_1().flex().flex_col().gap_2().children(
                                    right_col
                                        .iter()
                                        .map(|(key, desc)| render_shortcut_row(key, desc, &colors)),
                                ),
                            )
                    })
                    .child(
                        div()
                            .mt_6()
                            .text_sm()
                            .text_color(colors.muted_foreground)
                            .child("Press ? or click anywhere to close"),
                    ),
            )
    }
}

fn get_shortcuts_for_context(
    view_mode: &ViewMode,
    keybindings: &crate::config::KeyMap,
) -> Vec<(String, String)> {
    // Build a reverse map: Action -> Key
    let mut action_to_key: std::collections::HashMap<Action, String> =
        std::collections::HashMap::new();
    for (key, action) in keybindings {
        action_to_key.insert(action.clone(), key.clone());
    }

    let mut shortcuts = Vec::new();

    // Global shortcuts (always shown)
    add_shortcut(&mut shortcuts, &action_to_key, Action::Quit, "Quit app");
    add_shortcut(
        &mut shortcuts,
        &action_to_key,
        Action::ShowKeyboardHelp,
        "Show this help",
    );
    add_shortcut(&mut shortcuts, &action_to_key, Action::Back, "Go back");

    // Navigation shortcuts
    add_shortcut(
        &mut shortcuts,
        &action_to_key,
        Action::ScrollDown,
        "Scroll down",
    );
    add_shortcut(
        &mut shortcuts,
        &action_to_key,
        Action::ScrollUp,
        "Scroll up",
    );
    add_shortcut(
        &mut shortcuts,
        &action_to_key,
        Action::ScrollToTop,
        "Scroll to top",
    );

    // Context-specific shortcuts
    match view_mode {
        ViewMode::List => {
            add_shortcut(
                &mut shortcuts,
                &action_to_key,
                Action::FocusSearch,
                "Focus search",
            );
            add_shortcut(
                &mut shortcuts,
                &action_to_key,
                Action::CycleSearchMode,
                "Cycle search mode",
            );
            add_shortcut(
                &mut shortcuts,
                &action_to_key,
                Action::CycleSortOption,
                "Cycle sort option",
            );
            add_shortcut(
                &mut shortcuts,
                &action_to_key,
                Action::ToggleSortOrder,
                "Toggle sort order",
            );
            add_shortcut(
                &mut shortcuts,
                &action_to_key,
                Action::ToggleBookmark,
                "Toggle bookmark",
            );
            add_shortcut(
                &mut shortcuts,
                &action_to_key,
                Action::ShowBookmarks,
                "Show bookmarks",
            );
            add_shortcut(
                &mut shortcuts,
                &action_to_key,
                Action::ShowHistory,
                "Show history",
            );
        }
        ViewMode::Story(_) => {
            add_shortcut(
                &mut shortcuts,
                &action_to_key,
                Action::ToggleBookmark,
                "Toggle bookmark",
            );
        }
        ViewMode::Bookmarks | ViewMode::History => {
            add_shortcut(
                &mut shortcuts,
                &action_to_key,
                Action::Back,
                "Return to stories",
            );
        }
        _ => {}
    }

    // Utility shortcuts
    add_shortcut(
        &mut shortcuts,
        &action_to_key,
        Action::OpenThemeEditor,
        "Open theme editor",
    );
    add_shortcut(
        &mut shortcuts,
        &action_to_key,
        Action::ShowLogViewer,
        "Show log viewer",
    );

    shortcuts
}

fn add_shortcut(
    shortcuts: &mut Vec<(String, String)>,
    action_to_key: &std::collections::HashMap<Action, String>,
    action: Action,
    description: &str,
) {
    if let Some(key) = action_to_key.get(&action) {
        shortcuts.push((key.clone(), description.to_string()));
    }
}

fn render_shortcut_row(
    key: &str,
    desc: &str,
    colors: &gpui_component::ThemeColor,
) -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_center()
        .py_2()
        .border_b_1()
        .border_color(colors.border.opacity(0.3))
        .child(
            div()
                .text_base()
                .text_color(colors.foreground)
                .child(desc.to_string()),
        )
        .child(
            div()
                .px_2()
                .py_1()
                .rounded_md()
                .bg(colors.secondary)
                .text_sm()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(colors.foreground)
                .child(key.to_string()),
        )
}
