use crate::internal::scroll::ScrollState;
use crate::state::AppState;
use gpui::{
    Context, Entity, FocusHandle, IntoElement, MouseButton, Render, Window, div, prelude::*,
};
use gpui_component::menu::ContextMenuExt;
use gpui_component::theme::ActiveTheme;

/// BookmarkListView - renders list of bookmarked stories
pub struct BookmarkListView {
    app_state: Entity<AppState>,
    scroll_state: ScrollState,
    focus_handle: FocusHandle,
}

impl BookmarkListView {
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

impl Render for BookmarkListView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.read(cx);
        let bookmarks = app_state.bookmarks.get_all();
        let _ = app_state; // Release borrow

        let scroll_y = self.scroll_state.scroll_y;
        let colors = cx.theme().colors;

        div()
            .track_focus(&self.focus_handle)
            .flex()
            .size_full()
            .overflow_hidden()
            .on_scroll_wheel(cx.listener(|this, event: &gpui::ScrollWheelEvent, _, cx| {
                let delta_pixels = event.delta.pixel_delta(gpui::px(1.0)).y;
                let delta_y: f32 = delta_pixels.into();
                this.scroll_state.scroll_by(-delta_y);
                cx.notify();
            }))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .relative()
                    .top(gpui::px(-scroll_y))
                    .p_2()
                    .gap_2()
                    .child(
                        div()
                            .p_2()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(colors.foreground)
                                    .child(format!("Bookmarks ({})", bookmarks.len())),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors.muted_foreground)
                                    .child("Press 'b' to remove bookmark"),
                            ),
                    )
                    .children(bookmarks.iter().map(|bookmark| {
                        let app_state_entity = self.app_state.clone();
                        bookmark_item(
                            bookmark.id,
                            bookmark.title.clone().unwrap_or_default(),
                            bookmark.url.clone(),
                            colors.background.into(),
                            colors.foreground.into(),
                            colors.border.into(),
                            app_state_entity,
                        )
                    }))
                    .when(bookmarks.is_empty(), |this| {
                        this.child(
                            div()
                                .p_8()
                                .flex()
                                .flex_col()
                                .items_center()
                                .justify_center()
                                .gap_4()
                                .child(
                                    div()
                                        .text_xl()
                                        .text_color(colors.muted_foreground)
                                        .child("No bookmarks yet"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(colors.muted_foreground)
                                        .child("Press 'b' on any story to bookmark it"),
                                ),
                        )
                    }),
            )
    }
}

fn bookmark_item(
    id: u32,
    title: String,
    url: Option<String>,
    surface_color: gpui::Rgba,
    text_color: gpui::Rgba,
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
                .flex()
                .items_start()
                .justify_between()
                .child(
                    div()
                        .text_base()
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .text_color(text_color)
                        .child(title.clone()),
                )
                .child(
                    div()
                        .text_color(gpui::rgb(0x00FF_D700)) // Gold color for star
                        .child("★"),
                ),
        )
        .context_menu(move |menu, _window, _cx| {
            let app_state_bookmark = app_state.clone();
            let app_state_stories_nav = app_state.clone();
            let app_state_history_nav = app_state.clone();
            let title_bookmark = title.clone();
            let url_bookmark = url.clone();
            let url_browser = url.clone();

            menu.item(
                gpui_component::menu::PopupMenuItem::new("Remove Bookmark").on_click(
                    move |_, _, cx| {
                        AppState::toggle_bookmark_by_data(
                            app_state_bookmark.clone(),
                            id,
                            Some(title_bookmark.clone()),
                            url_bookmark.clone(),
                            cx,
                        );
                    },
                ),
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
                gpui_component::menu::PopupMenuItem::new("Go to Stories").on_click(
                    move |_, _, cx| {
                        AppState::show_stories(app_state_stories_nav.clone(), cx);
                    },
                ),
            )
            .item(
                gpui_component::menu::PopupMenuItem::new("Go to History").on_click(
                    move |_, _, cx| {
                        AppState::show_history(app_state_history_nav.clone(), cx);
                    },
                ),
            )
        })
}
