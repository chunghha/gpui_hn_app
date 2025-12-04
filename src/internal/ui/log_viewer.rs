use crate::log_buffer::LogBuffer;
use crate::state::AppState;
use gpui::{
    Entity, InteractiveElement, IntoElement, ParentElement as _, Render, Styled, Window, div, px,
};
use gpui_component::button::Button;
use gpui_component::{ActiveTheme, h_flex, v_flex};

pub struct LogViewerView {
    app_state: Entity<AppState>,
    log_buffer: LogBuffer,
    focus_handle: gpui::FocusHandle,
}

impl LogViewerView {
    pub fn new(
        app_state: Entity<AppState>,
        log_buffer: LogBuffer,
        cx: &mut gpui::Context<Self>,
    ) -> Self {
        Self {
            app_state,
            log_buffer,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for LogViewerView {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let theme = cx.theme();
        let colors = &theme.colors;
        let log_lines = self.log_buffer.get_lines();

        div()
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .p_6()
            .gap_6()
            .bg(colors.background)
            .text_color(colors.foreground)
            .child(
                h_flex()
                    .gap_4()
                    .items_center()
                    .child(
                        Button::new("btn-back")
                            .label("← Back")
                            .on_click(move |_, _w, cx| {
                                AppState::show_stories(app_state.clone(), cx);
                            }),
                    )
                    .child(
                        div()
                            .text_xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("Log Viewer"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors.muted_foreground)
                            .child(format!("{} log entries", log_lines.len())),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .gap_1()
                    .overflow_y_hidden()
                    .p_4()
                    .bg(colors.overlay)
                    .border_1()
                    .border_color(colors.border)
                    .rounded_md()
                    .children(log_lines.iter().map(|line| {
                        let (level_color, level_text) = match () {
                            _ if line.contains("[ERROR]") => (colors.danger, "ERROR"),
                            _ if line.contains("[WARN]") => (colors.warning, "WARN"),
                            _ if line.contains("[INFO]") => (colors.info, "INFO"),
                            _ if line.contains("[DEBUG]") => (colors.accent, "DEBUG"),
                            _ => (colors.foreground, "TRACE"),
                        };

                        div()
                            .flex()
                            .items_start()
                            .gap_2()
                            .p_1()
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(level_color)
                                    .w(px(60.0))
                                    .child(level_text),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors.foreground)
                                    .flex_1()
                                    .child(line.clone()),
                            )
                    })),
            )
    }
}
