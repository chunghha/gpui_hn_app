use crate::api::StoryListType;
use crate::state::AppState;
use gpui::{Entity, IntoElement, MouseButton, SharedString, div, prelude::*};
use gpui_component::theme::ActiveTheme;

/// Header component - simple builder for header UI
pub fn render_header(
    app_state: Entity<AppState>,
    title: SharedString,
    font_serif: SharedString,
    font_sans: SharedString,
    current_list: StoryListType,
    colors: gpui_component::ThemeColor,
    is_dark: bool,
) -> impl IntoElement {
    let app_state_for_theme_toggle = app_state.clone();

    div()
        .flex()
        .flex_col()
        // Header bar
        .child(
            div()
                .flex()
                .items_center()
                .px_4()
                .py_2()
                .bg(colors.accent)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_3xl()
                                .font_family(font_serif.clone())
                                .font_weight(gpui::FontWeight::BOLD)
                                .text_color(colors.accent_foreground)
                                .child(title),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(colors.accent_foreground)
                                .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .justify_end()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .cursor_pointer()
                                .on_mouse_down(MouseButton::Left, move |_, _window, cx| {
                                    let current_config_name = app_state_for_theme_toggle
                                        .read(cx)
                                        .config
                                        .theme_name
                                        .clone();

                                    let computed_name = crate::utils::theme::toggle_dark_light(
                                        &current_config_name,
                                        Some(cx.theme().is_dark()),
                                    );

                                    let desired_shared: SharedString =
                                        gpui::SharedString::from(computed_name.clone());

                                    if let Some(theme) = gpui_component::ThemeRegistry::global(cx)
                                        .themes()
                                        .get(&desired_shared)
                                        .cloned()
                                    {
                                        tracing::info!(
                                            target: "gpui_component::theme::registry",
                                            "Reload active theme: \"{}\"",
                                            computed_name
                                        );
                                        gpui_component::Theme::global_mut(cx).apply_config(&theme);
                                        app_state_for_theme_toggle.update(cx, |state, cx| {
                                            state.config.theme_name = computed_name.clone();
                                            cx.notify();
                                        });
                                    } else {
                                        tracing::warn!(
                                            "Requested theme '{}' not found in ThemeRegistry",
                                            computed_name
                                        );
                                    }
                                })
                                .child(if is_dark {
                                    "\u{2600}\u{fe0f}"
                                } else {
                                    "\u{1F319}"
                                }),
                        ),
                ),
        )
        // Tabs bar
        .child(
            div()
                .flex()
                .items_center()
                .px_4()
                .py_2()
                .font_family(font_sans)
                .bg(colors.background)
                .border_b_1()
                .border_color(colors.border)
                .children(
                    [
                        StoryListType::Best,
                        StoryListType::Top,
                        StoryListType::New,
                        StoryListType::Ask,
                        StoryListType::Show,
                        StoryListType::Job,
                    ]
                    .into_iter()
                    .map(|list_type| {
                        let app_state_clone = app_state.clone();
                        div()
                            .flex_1()
                            .flex()
                            .justify_center()
                            .cursor_pointer()
                            .py_1()
                            .font_weight(if current_list == list_type {
                                gpui::FontWeight::BOLD
                            } else {
                                gpui::FontWeight::NORMAL
                            })
                            .text_color(if current_list == list_type {
                                colors.primary
                            } else {
                                colors.foreground
                            })
                            .child(format!("{}", list_type))
                            .on_mouse_down(MouseButton::Left, move |_, _window, cx| {
                                AppState::fetch_stories(app_state_clone.clone(), list_type, cx);
                            })
                    }),
                ),
        )
}
