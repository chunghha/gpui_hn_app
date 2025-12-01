use crate::internal::scroll::ScrollState;
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
}

impl StoryListView {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        // Observe app_state for changes
        cx.observe(&app_state, |_, _, cx| cx.notify()).detach();

        Self {
            app_state,
            scroll_state: ScrollState::new(),
            focus_handle: cx.focus_handle(),
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.read(cx);
        let stories = app_state.stories.clone();
        let loading = app_state.loading;
        let loading_more = app_state.loading_more;
        let all_loaded = app_state.all_stories_loaded;
        let _ = app_state; // Release borrow

        let scroll_y = self.scroll_state.scroll_y;
        let colors = cx.theme().colors;

        div()
            .track_focus(&self.focus_handle)
            .flex()
            .size_full()
            .overflow_hidden()
            .on_scroll_wheel(
                cx.listener(|this, event: &gpui::ScrollWheelEvent, window, cx| {
                    let delta_pixels = event.delta.pixel_delta(gpui::px(1.0)).y;
                    let delta_y: f32 = delta_pixels.into();
                    this.scroll_state.scroll_by(-delta_y);

                    // Check if we're near the bottom to load more stories
                    let estimated_height = (this.app_state.read(cx).stories.len() as f32) * 88.0;
                    let viewport_height: f32 = window.viewport_size().height.into();

                    if this.scroll_state.scroll_y > estimated_height - viewport_height - 200.0 {
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
                                    AppState::fetch_more_stories(entity, &mut async_cx).await;
                                })
                                .detach();
                        }
                    }

                    cx.notify();
                }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .relative()
                    .top(gpui::px(-scroll_y))
                    .p_2()
                    .gap_2()
                    .children(stories.iter().map(|story| {
                        let app_state_entity = self.app_state.clone();
                        let is_bookmarked = app_state.bookmarks.is_bookmarked(story.id);
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
                    }))
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
