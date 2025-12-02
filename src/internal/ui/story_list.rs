use crate::internal::scroll::ScrollState;
use crate::internal::ui::constants::STORY_ITEM_HEIGHT;
use crate::state::AppState;
use gpui::{
    Context, Entity, FocusHandle, IntoElement, MouseButton, Render, Window, div, prelude::*,
};
use gpui_component::menu::ContextMenuExt;
use gpui_component::theme::ActiveTheme;

/// StoryListView - renders story list with infinite scroll
pub struct StoryListView {
    app_state: Entity<AppState>,
    scroll_state: ScrollState,
    focus_handle: FocusHandle,
    search_focus_handle: FocusHandle,
    history_index: Option<usize>,
}

impl StoryListView {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        // Observe app_state for changes
        cx.observe(&app_state, |_, _, cx| cx.notify()).detach();

        // Restore saved scroll position
        let saved_scroll_y = app_state.read(cx).get_scroll_position();
        let mut scroll_state = ScrollState::new();
        scroll_state.scroll_y = saved_scroll_y;

        Self {
            app_state,
            scroll_state,
            focus_handle: cx.focus_handle(),
            search_focus_handle: cx.focus_handle(),
            history_index: None,
        }
    }

    pub fn scroll_by(&mut self, delta: f32) {
        self.scroll_state.scroll_by(delta);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_state.scroll_to_top();
    }
}

impl Render for StoryListView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scroll_y = self.scroll_state.scroll_y;
        let viewport_height: f32 = window.viewport_size().height.into();

        // Calculate visible range first (requires mutable borrow)
        let (start_idx, end_idx) = self.app_state.update(cx, |state, _| {
            // Also save current scroll position while we have mutable access
            state.save_scroll_position(scroll_y);
            state.calculate_visible_range(scroll_y, viewport_height, STORY_ITEM_HEIGHT)
        });

        // Now read state (immutable borrow)
        let app_state_read = self.app_state.read(cx);
        let stories = app_state_read.get_filtered_sorted_stories();
        let loading = app_state_read.loading;
        let loading_more = app_state_read.loading_more;
        let all_loaded = app_state_read.all_stories_loaded;
        let search_query = app_state_read.search_query.clone();
        let search_mode = app_state_read.search_mode;
        let sort_option = app_state_read.sort_option;
        let sort_order = app_state_read.sort_order;
        let regex_error = app_state_read.regex_error.clone();
        let should_focus = app_state_read.should_focus_search;
        let _ = app_state_read; // Release borrow

        if should_focus {
            window.focus(&self.search_focus_handle);
            let app_state = self.app_state.clone();
            cx.on_next_frame(window, move |_, _, cx| {
                AppState::consume_search_focus(app_state, cx);
            });
        }

        let colors = cx.theme().colors;

        div()
            .flex()
            .flex_col()
            .size_full()
            .child(
                // Search Bar & Status
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .bg(colors.secondary)
                    .border_b_1()
                    .border_color(colors.border)
                    .p_2()
                    .gap_2()
                    .child(
                        // Input Area
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .bg(colors.background)
                            .border_1()
                            .border_color(if self.search_focus_handle.is_focused(window) {
                                colors.accent
                            } else {
                                colors.border
                            })
                            .rounded_md()
                            .p_1()
                            .track_focus(&self.search_focus_handle)
                            .on_key_down(cx.listener(
                                |this, event: &gpui::KeyDownEvent, _window, cx| {
                                    let keystroke = &event.keystroke;
                                    if keystroke.modifiers.platform
                                        || keystroke.modifiers.control
                                        || keystroke.modifiers.alt
                                    {
                                        return;
                                    }

                                    if keystroke.key == "enter" {
                                        let query = this.app_state.read(cx).search_query.clone();
                                        if !query.is_empty() {
                                            this.app_state.update(cx, |state, _| {
                                                state.search_history.add(query);
                                            });
                                            this.history_index = None;
                                        }
                                        return;
                                    }

                                    if keystroke.key == "up" {
                                        let history =
                                            this.app_state.read(cx).search_history.get_all();
                                        if !history.is_empty() {
                                            let new_index = match this.history_index {
                                                Some(i) => (i + 1).min(history.len() - 1),
                                                None => 0,
                                            };
                                            this.history_index = Some(new_index);
                                            let query = history[new_index].clone();
                                            AppState::set_search_query(
                                                this.app_state.clone(),
                                                query,
                                                cx,
                                            );
                                        }
                                        return;
                                    }

                                    if keystroke.key == "down" {
                                        let history =
                                            this.app_state.read(cx).search_history.get_all();
                                        if let Some(i) = this.history_index {
                                            if i > 0 {
                                                let new_index = i - 1;
                                                this.history_index = Some(new_index);
                                                let query = history[new_index].clone();
                                                AppState::set_search_query(
                                                    this.app_state.clone(),
                                                    query,
                                                    cx,
                                                );
                                            } else {
                                                this.history_index = None;
                                                AppState::set_search_query(
                                                    this.app_state.clone(),
                                                    "".to_string(),
                                                    cx,
                                                );
                                            }
                                        }
                                        return;
                                    }

                                    let mut query = this.app_state.read(cx).search_query.clone();

                                    if keystroke.key == "backspace" {
                                        query.pop();
                                        AppState::set_search_query(
                                            this.app_state.clone(),
                                            query,
                                            cx,
                                        );
                                        this.history_index = None;
                                    } else if keystroke.key.len() == 1 {
                                        query.push_str(&keystroke.key);
                                        AppState::set_search_query(
                                            this.app_state.clone(),
                                            query,
                                            cx,
                                        );
                                        this.history_index = None;
                                    }
                                },
                            ))
                            .child(div().text_sm().text_color(colors.foreground).child(
                                if search_query.is_empty() {
                                    "Type to search... (Ctrl+R for Regex)".to_string()
                                } else {
                                    search_query.clone()
                                },
                            )),
                    )
                    .child(
                        // Status Bar
                        div()
                            .flex()
                            .justify_between()
                            .text_xs()
                            .text_color(colors.foreground)
                            .child(
                                div()
                                    .flex()
                                    .gap_2()
                                    .child(format!("Mode: {:?}", search_mode))
                                    .child(format!("Sort: {:?} ({:?})", sort_option, sort_order)),
                            )
                            .child(if let Some(err) = regex_error {
                                div()
                                    .text_color(gpui::rgb(0xFF0000))
                                    .child(format!("Regex Error: {}", err))
                            } else {
                                div()
                            }),
                    ),
            )
            .child(
                div()
                    .track_focus(&self.focus_handle)
                    .flex()
                    .size_full()
                    .overflow_hidden()
                    .on_scroll_wheel(cx.listener(
                        |this, event: &gpui::ScrollWheelEvent, window, cx| {
                            let delta_pixels = event.delta.pixel_delta(gpui::px(1.0)).y;
                            let delta_y: f32 = delta_pixels.into();
                            this.scroll_state.scroll_by(-delta_y);

                            // Check if we're near the bottom to load more stories
                            let estimated_height =
                                (this.app_state.read(cx).stories.len() as f32) * 88.0;
                            let viewport_height: f32 = window.viewport_size().height.into();

                            if this.scroll_state.scroll_y
                                > estimated_height - viewport_height - 200.0
                            {
                                let app_state = this.app_state.read(cx);
                                let should_load = !app_state.loading_more
                                    && !app_state.all_stories_loaded
                                    && !app_state.loading;
                                let _ = app_state;

                                if should_load {
                                    let entity = this.app_state.clone();
                                    let foreground = cx.foreground_executor().clone();
                                    let mut async_cx = cx.to_async();
                                    foreground
                                        .spawn(async move {
                                            AppState::fetch_more_stories(entity, &mut async_cx)
                                                .await;
                                        })
                                        .detach();
                                }
                            }

                            cx.notify();
                        },
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w_full()
                            .relative()
                            .child({
                                // Only render visible stories
                                let visible_stories: Vec<_> = stories
                                    .iter()
                                    .skip(start_idx)
                                    .take(end_idx - start_idx)
                                    .collect();

                                // Offset to compensate for hidden items above
                                let top_offset = -scroll_y + (start_idx as f32 * STORY_ITEM_HEIGHT);

                                div().top(gpui::px(top_offset)).p_2().gap_2().children(
                                    visible_stories.iter().map(|story| {
                                        let app_state_entity = self.app_state.clone();
                                        let bookmarks = &self.app_state.read(cx).bookmarks;
                                        let is_bookmarked = bookmarks.is_bookmarked(story.id);
                                        story_item(
                                            story.id,
                                            story.title.clone().unwrap_or_default(),
                                            story.url.clone(),
                                            story.score.unwrap_or(0),
                                            story.descendants.unwrap_or(0),
                                            is_bookmarked,
                                            colors.background.into(),
                                            colors.foreground.into(),
                                            colors.foreground.into(),
                                            colors.border.into(),
                                            app_state_entity,
                                        )
                                    }),
                                )
                            })
                            .when(loading && stories.is_empty(), |this| {
                                this.child(
                                    div()
                                        .p_4()
                                        .flex()
                                        .justify_center()
                                        .text_color(colors.foreground)
                                        .child("Loading stories..."),
                                )
                            })
                            .when(loading_more, |this| {
                                this.child(
                                    div()
                                        .p_4()
                                        .flex()
                                        .justify_center()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(colors.foreground)
                                        .child("Loading more stories..."),
                                )
                            })
                            .when(all_loaded && !stories.is_empty(), |this| {
                                this.child(
                                    div()
                                        .p_4()
                                        .flex()
                                        .justify_center()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .text_color(colors.foreground)
                                        .child("• End of list - No more stories •"),
                                )
                            }),
                    ),
            )
    }
}

#[allow(clippy::too_many_arguments)]
fn story_item(
    id: u32,
    title: String,
    url: Option<String>,
    score: u32,
    comments: u32,
    is_bookmarked: bool,
    surface_color: gpui::Rgba,
    text_color: gpui::Rgba,
    meta_text_color: gpui::Rgba,
    border_color: gpui::Rgba,
    app_state: Entity<AppState>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .p_3()
        .gap_2()
        .bg(surface_color)
        .border_1()
        .border_color(border_color)
        .rounded_md()
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, {
            let app_state_click = app_state.clone();
            move |_, _window, cx| {
                AppState::select_story(app_state_click.clone(), id, cx);
            }
        })
        .child(
            div()
                .text_base()
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(text_color)
                .child(title.clone()),
        )
        .child(
            div()
                .flex()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .gap_4()
                        .text_sm()
                        .text_color(meta_text_color)
                        .child(
                            div()
                                .flex()
                                .gap_1()
                                .items_center()
                                .child("⭐")
                                .child(format!("{}", score)),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_1()
                                .items_center()
                                .child("💬")
                                .child(format!("{}", comments)),
                        ),
                )
                .when(is_bookmarked, |this| {
                    this.child(
                        div()
                            .text_color(gpui::rgb(0xFFD700)) // Gold color
                            .child("★"),
                    )
                }),
        )
        .context_menu(move |menu, _window, _cx| {
            let app_state_bookmark = app_state.clone();
            let app_state_bookmarks_nav = app_state.clone();
            let app_state_history_nav = app_state.clone();
            let title_bookmark = title.clone();
            let url_bookmark = url.clone();
            let url_browser = url.clone();

            menu.item(
                gpui_component::menu::PopupMenuItem::new(if is_bookmarked {
                    "Remove Bookmark"
                } else {
                    "Bookmark"
                })
                .on_click(move |_, _, cx| {
                    tracing::debug!("Context menu: Bookmark toggle clicked for story {}", id);
                    AppState::toggle_bookmark_by_data(
                        app_state_bookmark.clone(),
                        id,
                        Some(title_bookmark.clone()),
                        url_bookmark.clone(),
                        cx,
                    );
                }),
            )
            .separator()
            .item(
                gpui_component::menu::PopupMenuItem::new("Open in Browser").on_click(
                    move |_, _, cx| {
                        if let Some(url) = &url_browser {
                            cx.open_url(url);
                        }
                    },
                ),
            )
            .separator()
            .item(
                gpui_component::menu::PopupMenuItem::new("Go to Bookmarks").on_click(
                    move |_, _, cx| {
                        tracing::debug!("Context menu: Go to Bookmarks clicked");
                        AppState::show_bookmarks(app_state_bookmarks_nav.clone(), cx);
                    },
                ),
            )
            .item(
                gpui_component::menu::PopupMenuItem::new("Go to History").on_click(
                    move |_, _, cx| {
                        tracing::debug!("Context menu: Go to History clicked");
                        AppState::show_history(app_state_history_nav.clone(), cx);
                    },
                ),
            )
        })
}
