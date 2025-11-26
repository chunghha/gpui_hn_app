use crate::api::StoryListType;
use crate::internal::markdown::{MarkdownStyle, render_markdown};
use crate::internal::scroll::ScrollState;
use crate::internal::webview::make_init_script;
use crate::state::{AppState, ViewMode};
use gpui::{Context, Entity, FocusHandle, SharedString, Window, div, prelude::*};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::slider::{Slider, SliderEvent, SliderState};
use gpui_component::theme::ActiveTheme;
use gpui_component::webview::WebView;

pub struct HnLayout {
    title: SharedString,
    pub(crate) app_state: Entity<AppState>,
    pub(crate) scroll_state: ScrollState,
    pub(crate) list_scroll_state: ScrollState,
    webview: Entity<WebView>,
    current_webview_url: Option<String>,
    current_zoom_level: u32,
    slider_state: Entity<SliderState>,
    focus_handle: FocusHandle,
}

impl HnLayout {
    pub fn new(app_state: Entity<AppState>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        window
            .observe(&app_state, cx, |_entity, window, _cx| window.refresh())
            .detach();

        // Trigger initial fetch
        AppState::fetch_stories(app_state.clone(), StoryListType::Best, cx);

        // Initialize WebView with platform-specific WRY builder
        let webview = cx.new(|cx| {
            let builder = gpui_component::wry::WebViewBuilder::new();

            // Get theme and zoom configuration
            let config = app_state.read(cx).config.clone();
            let zoom_level = config.webview_zoom;

            // Get theme colors
            let theme = cx.theme();
            let is_dark_theme = theme.is_dark();
            let colors = theme.colors;

            // Convert GPUI Hsla colors to CSS hex strings using utility
            let bg_color = crate::utils::theme::hsla_to_hex(colors.background);
            let fg_color = crate::utils::theme::hsla_to_hex(colors.foreground);
            let link_color = crate::utils::theme::hsla_to_hex(colors.info);

            tracing::info!(
                "Initializing webview with zoom: {}%, dark mode: {}, bg: {}, fg: {}",
                zoom_level,
                is_dark_theme,
                bg_color,
                fg_color
            );

            // Build initialization script using helper in the webview module.
            // The helper centralizes the JS creation and the decision logic for whether to
            // inject themed CSS into the page. It receives already-formatted hex color
            // strings and the zoom level.
            let init_script = make_init_script(
                &config,
                is_dark_theme,
                &bg_color,
                &fg_color,
                &link_color,
                zoom_level,
            );
            let builder = builder.with_initialization_script(&init_script);

            #[cfg(any(
                target_os = "windows",
                target_os = "macos",
                target_os = "ios",
                target_os = "android"
            ))]
            let wry_webview = {
                use raw_window_handle::HasWindowHandle;
                let window_handle = window.window_handle().expect("No window handle");
                builder.build_as_child(&window_handle).unwrap()
            };

            #[cfg(not(any(
                target_os = "windows",
                target_os = "macos",
                target_os = "ios",
                target_os = "android"
            )))]
            let wry_webview = {
                use gpui_component::wry::WebViewBuilderExtUnix;
                use gtk::prelude::*;
                let fixed = gtk::Fixed::builder().build();
                fixed.show_all();
                builder.build_gtk(&fixed).unwrap()
            };

            WebView::new(wry_webview, window, cx)
        });

        // Create slider state for zoom control (50% to 250%)
        let initial_zoom = app_state.read(cx).config.webview_zoom;
        let slider_state = cx.new(|_| {
            SliderState::new()
                .min(50.0)
                .max(250.0)
                .default_value(initial_zoom as f32)
                .step(5.0)
        });

        // Subscribe to slider changes
        let app_state_for_slider = app_state.clone();
        cx.subscribe(&slider_state, move |_this, _, event: &SliderEvent, cx| {
            let SliderEvent::Change(value) = event;
            let zoom = value.start() as u32;
            AppState::set_zoom_level(app_state_for_slider.clone(), zoom, cx);
            // Don't update current_zoom_level here - let render() detect and apply the change
            cx.notify();
        })
        .detach();

        let focus_handle = cx.focus_handle();

        Self {
            title: "Hacker News".into(),
            app_state: app_state.clone(),
            scroll_state: ScrollState::new(),
            list_scroll_state: ScrollState::new(),
            webview,
            current_webview_url: None,
            current_zoom_level: app_state.read(cx).config.webview_zoom,
            slider_state,
            focus_handle,
        }
    }
}

impl Render for HnLayout {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors;

        // Read app_state once to get all needed data and clone what's needed
        let (
            stories,
            loading,
            current_list,
            view_mode,
            font_sans,
            font_serif,
            font_mono,
            comments,
            comments_loading,
            selected_story_content,
            selected_story_content_loading,
        ) = {
            let app_state = self.app_state.read(cx);
            (
                app_state.stories.clone(),
                app_state.loading,
                app_state.current_list,
                app_state.view_mode.clone(),
                app_state.config.font_sans.clone(),
                app_state.config.font_serif.clone(),
                app_state.config.font_mono.clone(),
                app_state.comments.clone(),
                app_state.comments_loading,
                app_state.selected_story_content.clone(),
                app_state.selected_story_content_loading,
            )
        };
        // app_state borrow is now dropped

        // Handle WebView visibility based on view mode
        match &view_mode {
            ViewMode::Webview(url) => {
                let url_clone = url.clone();

                // Get current zoom level from app state
                let webview_zoom = self.app_state.read(cx).config.webview_zoom;

                // Only load if URL changed to avoid reloading on every render
                if self.current_webview_url.as_ref() != Some(&url_clone) {
                    tracing::info!("Loading new URL: {}", url_clone);
                    self.current_webview_url = Some(url_clone.clone());
                    self.webview.update(cx, move |webview, _| {
                        webview.load_url(&url_clone);
                        webview.show();
                    });
                    self.current_zoom_level = webview_zoom;
                } else {
                    tracing::debug!("URL unchanged, ensuring webview is visible: {}", url_clone);
                    // Ensure it's visible even if URL didn't change (e.g. coming back from hidden)
                    self.webview.update(cx, |webview, _| {
                        webview.show();
                    });
                }

                // Apply zoom if it changed via slider
                if self.current_zoom_level != webview_zoom {
                    tracing::debug!("Applying zoom level change: {}%", webview_zoom);
                    self.current_zoom_level = webview_zoom;
                    let zoom_script = format!(
                        r#"
                        (function() {{
                            if (document.body) {{
                                document.body.style.zoom = '{}%';
                            }} else if (document.documentElement) {{
                                document.documentElement.style.zoom = '{}%';
                            }}
                        }})();
                        "#,
                        webview_zoom, webview_zoom
                    );
                    self.webview.update(cx, move |webview, _| {
                        let _ = webview.evaluate_script(&zoom_script);
                    });
                }
            }
            _ => {
                // If we were previously showing a URL, clear it so next time we load fresh
                if self.current_webview_url.is_some() {
                    tracing::info!("Clearing webview state");
                    self.current_webview_url = None;
                    self.webview.update(cx, |webview, _| {
                        webview.load_url("about:blank");
                        webview.hide();
                    });
                } else {
                    self.webview.update(cx, |webview, _| {
                        webview.hide();
                    });
                }
            }
        }

        // Root container with keyboard event handling
        let mut root = div()
            .track_focus(&self.focus_handle)
            .id("root")
            .flex()
            .flex_col()
            .size_full()
            .bg(colors.background)
            .font_family(font_serif.clone())
            .on_key_down(cx.listener(crate::internal::events::handle_key_down));

        // Prepare a clone of app_state for the theme toggle handler
        let app_state_for_theme_toggle = self.app_state.clone();

        // Header
        root = root.child(
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
                                .child(self.title.clone()),
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
                                .on_mouse_down(gpui::MouseButton::Left, move |_, _window, cx| {
                                    // Use the shared utility to compute the new theme name by flipping
                                    // the Dark/Light token based on the configured name and runtime hint.
                                    let current_config_name = app_state_for_theme_toggle
                                        .read(cx)
                                        .config
                                        .theme_name
                                        .clone();

                                    // `toggle_dark_light` returns the new theme name.
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
                                        // Registry-style informational log to match existing logs.
                                        tracing::info!(
                                            target: "gpui_component::theme::registry",
                                            "Reload active theme: \"{}\"",
                                            computed_name
                                        );

                                        // Apply the theme immediately.
                                        gpui_component::Theme::global_mut(cx).apply_config(&theme);

                                        // Persist selection to app state so other parts of the app
                                        // (and future restarts when config is saved) can read it.
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
                                .child(if cx.theme().is_dark() {
                                    // Show sun when in dark mode (clicking will switch to light)
                                    "\u{2600}\u{fe0f}"
                                } else {
                                    // Show moon when in light mode (clicking will switch to dark)
                                    "\u{1F319}"
                                }),
                        ),
                ),
        );

        // Tabs bar
        root = root.child(
            div()
                .flex()
                .items_center()
                .px_4()
                .py_2()
                .font_family(font_sans.clone())
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
                        let app_state_clone = self.app_state.clone();
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
                            .on_mouse_down(gpui::MouseButton::Left, move |_, _window, cx| {
                                AppState::fetch_stories(app_state_clone.clone(), list_type, cx);
                            })
                    }),
                ),
        );

        // Main content
        root = root.child(match view_mode {
            ViewMode::Story(story) => {
                let scroll_y = self.scroll_state.scroll_y;

                div()
                    .flex()
                    .size_full()
                    .overflow_hidden()
                    .on_scroll_wheel(cx.listener(
                        |this, event: &gpui::ScrollWheelEvent, _window, cx| {
                            let delta_pixels = event.delta.pixel_delta(gpui::px(1.0)).y;
                            let delta_y: f32 = delta_pixels.into();
                            this.scroll_state.scroll_by(-delta_y); // Inverted for natural Mac scrolling
                            cx.notify();
                        },
                    ))
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
                            .child(
                                div()
                                    .text_3xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(colors.foreground)
                                    .child(story.title.clone().unwrap_or_default()),
                            )
                            // Metadata row with icons
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
                                            div().flex().gap_1().items_center().child("🕒").child(
                                                crate::utils::datetime::format_timestamp(&time),
                                            ),
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
                                let app_state_entity = self.app_state.clone();
                                let url_clone = url.clone();
                                this.child(
                                    div()
                                        .text_lg()
                                        .text_color(colors.info)
                                        .cursor_pointer()
                                        .child(url)
                                        .on_mouse_down(
                                            gpui::MouseButton::Left,
                                            move |_, _w, cx| {
                                                AppState::show_webview(
                                                    app_state_entity.clone(),
                                                    url_clone.clone(),
                                                    cx,
                                                );
                                            },
                                        ),
                                )
                            })
                            // Fetched content area
                            .child({
                                div().flex().flex_col().mt_2().child(
                                    if selected_story_content_loading {
                                        div()
                                            .p_4()
                                            .text_sm()
                                            .text_color(colors.foreground)
                                            .child("Loading page content...")
                                    } else if let Some(text) = selected_story_content {
                                        let style = MarkdownStyle {
                                            text_color: colors.foreground,
                                            link_color: colors.info,
                                            code_bg_color: colors.secondary, // Use secondary color for code blocks
                                            font_sans: font_sans.clone().into(),
                                            font_mono: font_mono.clone().into(),
                                        };

                                        div()
                                            .p_4()
                                            .bg(colors.background)
                                            .rounded_md()
                                            .border_1()
                                            .border_color(colors.border)
                                            .text_base()
                                            .line_height(gpui::rems(1.5))
                                            .text_color(colors.foreground)
                                            // Removed max_h and overflow_hidden to allow full content reading
                                            .child(render_markdown(&text, style))
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
                                    },
                                )
                            })
                            // Comments section
                            .child(
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
                                            .child(format!(
                                                "Comments ({})",
                                                story.descendants.unwrap_or(0)
                                            )),
                                    )
                                    .child(if comments_loading {
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
                                        // Pass font_mono (from config) down to render_comment so comment text
                                        // can use monospace where appropriate.
                                        div().flex().flex_col().gap_2().children(
                                            comments.iter().map(|comment| {
                                                render_comment(comment, &colors, font_mono.clone())
                                            }),
                                        )
                                    }),
                            ),
                    )
            }
            ViewMode::Webview(_url) => {
                let app_state_entity = self.app_state.clone();
                let webview_zoom = self.app_state.read(cx).config.webview_zoom;

                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .p_6()
                    .gap_4()
                    .child(
                        Button::new("btn-primary")
                            .primary()
                            .label("← Back")
                            .on_click(move |_, _w, cx| {
                                AppState::hide_webview(app_state_entity.clone(), cx);
                            }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .border_1()
                            .border_color(colors.border)
                            .child(self.webview.clone()),
                    )
                    .child(
                        // Zoom slider at bottom
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .p_3()
                            .bg(colors.background)
                            .border_1()
                            .border_color(colors.border)
                            .rounded_md()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors.foreground)
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .font_family(font_sans.clone())
                                    .child("Zoom:"),
                            )
                            .child(
                                Slider::new(&self.slider_state)
                                    .w(gpui::px(200.0))
                                    .bg(colors.primary)
                                    .text_color(colors.primary_foreground)
                                    .font_family(font_sans.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors.foreground)
                                    .font_family(font_sans.clone())
                                    .child(format!("{}%", webview_zoom)),
                            ),
                    )
            }
            ViewMode::List => {
                let list_scroll_y = self.list_scroll_state.scroll_y;

                div()
                    .flex()
                    .size_full()
                    .overflow_hidden()
                    .on_scroll_wheel(cx.listener(
                        |this, event: &gpui::ScrollWheelEvent, window, cx| {
                            let delta_pixels = event.delta.pixel_delta(gpui::px(1.0)).y;
                            let delta_y: f32 = delta_pixels.into();
                            this.list_scroll_state.scroll_by(-delta_y); // Inverted for natural Mac scrolling

                            // Check if we're near the bottom to load more stories
                            let estimated_height =
                                (this.app_state.read(cx).stories.len() as f32) * 88.0;

                            // Use actual window height
                            let viewport_height: f32 = window.viewport_size().height.into();

                            if this.list_scroll_state.scroll_y
                                > estimated_height - viewport_height - 200.0
                            {
                                let entity = this.app_state.clone();
                                let foreground = cx.foreground_executor().clone();
                                let mut async_cx = cx.to_async();
                                foreground
                                    .spawn(async move {
                                        AppState::fetch_more_stories(entity, &mut async_cx).await;
                                    })
                                    .detach();
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
                            .top(gpui::px(-list_scroll_y))
                            .p_2()
                            .gap_2()
                            .children(stories.iter().map(|story| {
                                let app_state_entity = self.app_state.clone();
                                let data = StoryViewData {
                                    id: story.id,
                                    title: story.title.clone().unwrap_or_default(),
                                    score: story.score.unwrap_or(0),
                                    comments: story.descendants.unwrap_or(0),
                                    surface_color: colors.background.into(),
                                    text_color: colors.foreground.into(),
                                    meta_text_color: colors.foreground.into(),
                                    border_color: colors.border.into(),
                                    accent_color: colors.accent.into(),
                                    selected: false,
                                    app_state: app_state_entity,
                                };
                                story_item(data)
                            })),
                    )
            }
        });

        // Loading indicator
        if loading {
            root = root.child(div().p_4().child("Loading..."));
        }

        root
    }
}

fn render_comment(
    comment: &crate::internal::models::Comment,
    colors: &gpui_component::ThemeColor,
    font_mono: String,
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
                .child(display_text),
        )
}

struct StoryViewData {
    id: u32,
    title: String,
    score: u32,
    comments: u32,
    surface_color: gpui::Rgba,
    text_color: gpui::Rgba,
    meta_text_color: gpui::Rgba,
    border_color: gpui::Rgba,
    accent_color: gpui::Rgba,
    selected: bool,
    app_state: Entity<AppState>,
}

fn story_item(data: StoryViewData) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .p_3()
        .gap_2()
        .bg(data.surface_color)
        .border_1()
        .border_color(data.border_color)
        .rounded_md()
        .cursor_pointer()
        .border_color(if data.selected {
            data.accent_color
        } else {
            data.border_color
        })
        .on_mouse_down(gpui::MouseButton::Left, move |_, _w, cx| {
            AppState::select_story(data.app_state.clone(), data.id, cx);
        })
        .child(
            div()
                .text_lg()
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(data.text_color)
                .child(data.title),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .text_base()
                .text_color(data.meta_text_color)
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .items_center()
                        .child("⭐")
                        .child(format!("{}", data.score)),
                )
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .items_center()
                        .child("💬")
                        .child(format!("{}", data.comments)),
                ),
        )
}
