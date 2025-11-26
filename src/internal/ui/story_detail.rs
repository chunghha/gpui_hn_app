use crate::internal::markdown::{MarkdownStyle, render_markdown, soft_wrap};
use crate::internal::models::Comment;
use crate::internal::scroll::ScrollState;
use crate::state::{AppState, ViewMode};
use gpui::{
    Context, Entity, FocusHandle, IntoElement, MouseButton, Render, SharedString, Window, div,
    prelude::*,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::theme::{ActiveTheme, ThemeColor};

/// StoryDetailView - renders story detail with comments
pub struct StoryDetailView {
    app_state: Entity<AppState>,
    scroll_state: ScrollState,
    focus_handle: FocusHandle,
}

impl StoryDetailView {
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

impl Render for StoryDetailView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.read(cx);
        let colors = cx.theme().colors;
        let font_sans = app_state.config.font_sans.clone();
        let font_mono = app_state.config.font_mono.clone();
        // Capture config so we can drop the read guard early but still read config values.
        let config = app_state.config.clone();

        // Extract story from view mode
        let story = match &app_state.view_mode {
            ViewMode::Story(s) => s.clone(),
            _ => {
                let _ = app_state;
                return div(); // Should not happen
            }
        };

        let selected_story_content = app_state.selected_story_content.clone();
        let selected_story_content_loading = app_state.selected_story_content_loading;
        let comments = app_state.comments.clone();
        let comments_loading = app_state.comments_loading;
        let _ = app_state; // Release borrow

        let scroll_y = self.scroll_state.scroll_y;

        div()
            .track_focus(&self.focus_handle)
            .flex()
            .size_full()
            .overflow_hidden()
            .on_scroll_wheel(
                cx.listener(|this, event: &gpui::ScrollWheelEvent, _window, cx| {
                    let delta_pixels = event.delta.pixel_delta(gpui::px(1.0)).y;
                    let delta_y: f32 = delta_pixels.into();
                    this.scroll_state.scroll_by(-delta_y);
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
                    .p_6()
                    .gap_4()
                    .bg(colors.background)
                    .child({
                        let app_state_entity = self.app_state.clone();
                        Button::new("btn-primary")
                            .primary()
                            .label("← Back")
                            .on_click(move |_, _w, cx| {
                                AppState::clear_selection(app_state_entity.clone(), cx);
                            })
                    })
                    // Metadata row
                    .child(render_story_header(
                        &story,
                        &colors,
                        &config,
                        self.app_state.clone(),
                    ))
                    // Fetched content area
                    .child({
                        div()
                            .flex()
                            .flex_col()
                            .mt_2()
                            .child(if selected_story_content_loading {
                                div()
                                    .p_4()
                                    .text_sm()
                                    .text_color(colors.foreground)
                                    .child("Loading page content...")
                            } else if let Some(ref text) = selected_story_content {
                                let style = MarkdownStyle {
                                    text_color: colors.foreground,
                                    link_color: colors.info,
                                    code_bg_color: colors.secondary,
                                    font_sans: font_sans.clone().into(),
                                    font_mono: font_mono.clone().into(),
                                };

                                // Wrap rendered markdown in a full-width flex row with wrapping
                                // Preprocess the fetched text with soft_wrap to ensure
                                // long runs don't overflow and can wrap inside the layout.
                                let preprocessed = soft_wrap(text, config.soft_wrap_max_run);
                                div()
                                    .p_4()
                                    .bg(colors.background)
                                    .rounded_md()
                                    .border_1()
                                    .border_color(colors.border)
                                    .text_base()
                                    .line_height(gpui::rems(1.5))
                                    .text_color(colors.foreground)
                                    .child(render_markdown(&preprocessed, style, &config))
                            } else if story.url.is_some() {
                                div()
                                    .p_4()
                                    .text_sm()
                                    .text_color(colors.foreground)
                                    .child("Failed to load content")
                            } else {
                                div()
                                    .p_4()
                                    .text_sm()
                                    .text_color(colors.foreground)
                                    .child("No URL available for this story")
                            })
                    })
                    // Comments section
                    .child(render_comments_list(
                        &story,
                        &comments,
                        comments_loading,
                        &colors,
                        font_mono.clone().into(),
                        config.soft_wrap_max_run,
                    )),
            )
    }
}

fn render_story_header(
    story: &crate::internal::models::Story,
    colors: &ThemeColor,
    config: &crate::config::AppConfig,
    app_state: Entity<AppState>,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .w_full()
                .flex()
                .flex_row()
                .flex_wrap()
                .text_3xl()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(colors.foreground)
                .child(div().flex().flex_wrap().child(soft_wrap(
                    &story.title.clone().unwrap_or_default(),
                    config.soft_wrap_max_run,
                ))),
        )
        // Metadata row
        .child(
            div()
                .flex()
                .gap_4()
                .text_base()
                .text_color(colors.foreground)
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .items_center()
                        .child("⭐")
                        .child(format!("{}", story.score.unwrap_or(0))),
                )
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .items_center()
                        .child("👤")
                        .child(story.by.clone().unwrap_or_default()),
                )
                .when_some(story.time, |this, time| {
                    this.child(
                        div()
                            .flex()
                            .gap_1()
                            .items_center()
                            .child("🕒")
                            .child(crate::utils::datetime::format_timestamp(&time)),
                    )
                })
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .items_center()
                        .child("💬")
                        .child(format!("{}", story.descendants.unwrap_or(0))),
                ),
        )
        // Story URL if available
        .when_some(story.url.clone(), |this, url| {
            let app_state_entity = app_state.clone();
            let url_clone = url.clone();
            this.child(
                div()
                    .text_lg()
                    .text_color(colors.info)
                    .cursor_pointer()
                    .child(soft_wrap(&url, config.soft_wrap_max_run))
                    .on_mouse_down(MouseButton::Left, move |_, _w, cx| {
                        AppState::show_webview(app_state_entity.clone(), url_clone.clone(), cx);
                    }),
            )
        })
}

fn render_comments_list(
    story: &crate::internal::models::Story,
    comments: &[Comment],
    loading: bool,
    colors: &ThemeColor,
    font_mono: SharedString,
    max_run: usize,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .mt_4()
        .gap_2()
        .child(
            div()
                .text_lg()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(colors.foreground)
                .child(format!("Comments ({})", story.descendants.unwrap_or(0))),
        )
        .child(if loading {
            div()
                .p_4()
                .text_sm()
                .text_color(colors.foreground)
                .child("Loading comments...")
        } else if comments.is_empty() {
            div()
                .p_4()
                .text_sm()
                .text_color(colors.foreground)
                .child("No comments yet")
        } else {
            div().flex().flex_col().gap_2().children(
                comments
                    .iter()
                    .map(|comment| render_comment(comment, colors, font_mono.clone(), max_run)),
            )
        })
}

fn render_comment(
    comment: &Comment,
    colors: &ThemeColor,
    font_mono: SharedString,
    max_run: usize,
) -> impl IntoElement {
    let display_text = if comment.deleted {
        "[deleted]".to_string()
    } else {
        match &comment.text {
            Some(text) => match html2text::from_read(text.as_bytes(), 80) {
                Ok(plain_text) => plain_text,
                Err(_) => "[failed to parse comment]".to_string(),
            },
            None => "".to_string(),
        }
    };

    div()
        .flex()
        .flex_col()
        .p_3()
        .bg(colors.background)
        .border_1()
        .border_color(colors.border)
        .rounded_md()
        .gap_2()
        .child(
            div()
                .flex()
                .gap_3()
                .text_sm()
                .text_color(colors.foreground)
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .items_center()
                        .child("👤")
                        .child(comment.by.clone().unwrap_or_else(|| "N/A".to_string())),
                )
                .when_some(comment.time, |this, time| {
                    this.child(
                        div()
                            .flex()
                            .gap_1()
                            .items_center()
                            .child("🕒")
                            .child(crate::utils::datetime::format_timestamp(&time)),
                    )
                }),
        )
        .child(
            div()
                .text_base()
                .line_height(gpui::rems(1.4))
                .text_color(colors.foreground)
                .font_family(font_mono)
                .child(soft_wrap(&display_text, max_run)),
        )
}
