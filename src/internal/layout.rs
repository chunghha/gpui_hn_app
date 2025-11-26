use crate::api::StoryListType;
use crate::internal::ui::{StoryDetailView, StoryListView, render_header, render_webview_controls};
use crate::state::{AppState, ViewMode};
use gpui::{prelude::*, *};
use gpui_component::ActiveTheme;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::slider::{SliderEvent, SliderState};
use gpui_component::webview::WebView;

pub struct HnLayout {
    title: SharedString,
    pub app_state: Entity<AppState>,
    webview: Entity<WebView>,
    current_webview_url: Option<String>,
    current_zoom_level: u32,
    slider_state: Entity<SliderState>,
    focus_handle: FocusHandle,
    story_list_view: Entity<StoryListView>,
    story_detail_view: Entity<StoryDetailView>,
}

impl HnLayout {
    pub fn new(
        app_state: Entity<AppState>,
        story_list_view: Entity<StoryListView>,
        story_detail_view: Entity<StoryDetailView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        window
            .observe(&app_state, cx, |_entity, window, _cx| window.refresh())
            .detach();

        // Trigger initial fetch
        AppState::fetch_stories(app_state.clone(), StoryListType::Best, cx);

        // Initialize WebView using factory
        let config = app_state.read(cx).config.clone();
        let webview = crate::internal::webview_factory::create_webview(window, cx, &config);

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
            cx.notify();
        })
        .detach();

        let focus_handle = cx.focus_handle();

        Self {
            title: "Hacker News".into(),
            app_state,
            webview,
            current_webview_url: None,
            current_zoom_level: initial_zoom,
            slider_state,
            focus_handle,
            story_list_view,
            story_detail_view,
        }
    }
    pub fn story_list_view(&self) -> Entity<StoryListView> {
        self.story_list_view.clone()
    }

    pub fn story_detail_view(&self) -> Entity<StoryDetailView> {
        self.story_detail_view.clone()
    }
}

impl Render for HnLayout {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors;
        let app_state = self.app_state.read(cx);
        let (current_list, view_mode, font_sans, font_serif, injection_mode) = (
            app_state.current_list,
            app_state.view_mode.clone(),
            app_state.config.font_sans.clone(),
            app_state.config.font_serif.clone(),
            app_state.config.webview_theme_injection.clone(),
        );
        let loading = app_state.loading;
        let _ = app_state;

        // Handle WebView visibility
        if let ViewMode::Webview(url) = &view_mode {
            let url_clone = url.clone();
            let webview_zoom = self.app_state.read(cx).config.webview_zoom;

            if self.current_webview_url.as_ref() != Some(&url_clone) {
                self.current_webview_url = Some(url_clone.clone());
                self.webview
                    .update(cx, |webview, _| webview.load_url(&url_clone));
            }
            if self.current_zoom_level != webview_zoom {
                self.current_zoom_level = webview_zoom;
                let zoom_script = format!("document.body.style.zoom = '{}%';", webview_zoom);
                self.webview
                    .update(cx, |webview, _| webview.evaluate_script(&zoom_script).ok());
            }
            self.webview.update(cx, |webview, _| webview.show());
        } else {
            self.current_webview_url = None;
            self.webview.update(cx, |webview, _| webview.hide());
        }

        div()
            .track_focus(&self.focus_handle)
            .id("root")
            .flex()
            .flex_col()
            .size_full()
            .bg(colors.background)
            .font_family(font_serif.clone())
            .overflow_hidden()
            .on_key_down(cx.listener(crate::internal::events::handle_key_down))
            .child(render_header(
                self.app_state.clone(),
                self.title.clone(),
                font_serif.into(),
                font_sans.clone().into(),
                current_list,
                colors,
                cx.theme().is_dark(),
            ))
            .child(match view_mode {
                ViewMode::List => div().flex_1().child(self.story_list_view.clone()),
                ViewMode::Story(_) => div().flex_1().child(self.story_detail_view.clone()),
                ViewMode::Webview(_) => {
                    let app_state_entity = self.app_state.clone();
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
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
                        .child(render_webview_controls(
                            self.app_state.clone(),
                            &self.slider_state,
                            self.current_zoom_level,
                            injection_mode,
                            colors,
                        ))
                }
            })
            .when(loading, |this| {
                this.child(
                    div()
                        .absolute()
                        .bottom_4()
                        .right_4()
                        .p_2()
                        .rounded_md()
                        .bg(colors.accent)
                        .text_color(colors.accent_foreground)
                        .child("Loading..."),
                )
            })
    }
}
