use crate::state::AppState;
use crate::utils::theme_export::Rgb;
use gpui::{
    Context, Entity, IntoElement, Render, SharedString, Styled, Window, div, prelude::*, px,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::slider::{Slider, SliderEvent, SliderState};
use gpui_component::{ActiveTheme, h_flex, v_flex};

pub struct ThemeEditorView {
    app_state: Entity<AppState>,
    // Color being edited
    background_color: Rgb,
    foreground_color: Rgb,
    accent_color: Rgb,
    // Slider states for background RGB
    bg_r_slider: Entity<SliderState>,
    bg_g_slider: Entity<SliderState>,
    bg_b_slider: Entity<SliderState>,
    // Slider states for foreground RGB
    fg_r_slider: Entity<SliderState>,
    fg_g_slider: Entity<SliderState>,
    fg_b_slider: Entity<SliderState>,
    // Slider states for accent RGB
    ac_r_slider: Entity<SliderState>,
    ac_g_slider: Entity<SliderState>,
    ac_b_slider: Entity<SliderState>,
    // Theme name
    theme_name: String,
    pub focus_handle: gpui::FocusHandle,
    theme_name_focus_handle: gpui::FocusHandle,
}

impl ThemeEditorView {
    pub fn new(app_state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
        // Initialize with current theme colors
        let theme = cx.theme();
        let colors = &theme.colors;

        let background_color = Rgb::from_hex(&crate::utils::theme::hsla_to_hex(colors.background))
            .unwrap_or(Rgb::new(16, 15, 15));
        let foreground_color = Rgb::from_hex(&crate::utils::theme::hsla_to_hex(colors.foreground))
            .unwrap_or(Rgb::new(206, 205, 195));
        let accent_color = Rgb::from_hex(&crate::utils::theme::hsla_to_hex(colors.accent))
            .unwrap_or(Rgb::new(36, 131, 123));

        let bg_r_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(255.0)
                .default_value(background_color.r as f32)
        });
        let bg_g_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(255.0)
                .default_value(background_color.g as f32)
        });
        let bg_b_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(255.0)
                .default_value(background_color.b as f32)
        });
        let fg_r_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(255.0)
                .default_value(foreground_color.r as f32)
        });
        let fg_g_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(255.0)
                .default_value(foreground_color.g as f32)
        });
        let fg_b_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(255.0)
                .default_value(foreground_color.b as f32)
        });
        let ac_r_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(255.0)
                .default_value(accent_color.r as f32)
        });
        let ac_g_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(255.0)
                .default_value(accent_color.g as f32)
        });
        let ac_b_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(255.0)
                .default_value(accent_color.b as f32)
        });

        let view = Self {
            app_state,
            background_color,
            foreground_color,
            accent_color,
            bg_r_slider,
            bg_g_slider,
            bg_b_slider,
            fg_r_slider,
            fg_g_slider,
            fg_b_slider,
            ac_r_slider,
            ac_g_slider,
            ac_b_slider,
            theme_name: "Custom Theme".to_string(),
            focus_handle: cx.focus_handle(),
            theme_name_focus_handle: cx.focus_handle(),
        };

        // Subscribe to background color slider changes
        cx.subscribe(
            &view.bg_r_slider,
            move |this, _, _event: &SliderEvent, cx| {
                this.update_background_color(cx);
            },
        )
        .detach();
        cx.subscribe(
            &view.bg_g_slider,
            move |this, _, _event: &SliderEvent, cx| {
                this.update_background_color(cx);
            },
        )
        .detach();
        cx.subscribe(
            &view.bg_b_slider,
            move |this, _, _event: &SliderEvent, cx| {
                this.update_background_color(cx);
            },
        )
        .detach();

        // Subscribe to foreground color slider changes
        cx.subscribe(
            &view.fg_r_slider,
            move |this, _, _event: &SliderEvent, cx| {
                this.update_foreground_color(cx);
            },
        )
        .detach();
        cx.subscribe(
            &view.fg_g_slider,
            move |this, _, _event: &SliderEvent, cx| {
                this.update_foreground_color(cx);
            },
        )
        .detach();
        cx.subscribe(
            &view.fg_b_slider,
            move |this, _, _event: &SliderEvent, cx| {
                this.update_foreground_color(cx);
            },
        )
        .detach();

        // Subscribe to accent color slider changes
        cx.subscribe(
            &view.ac_r_slider,
            move |this, _, _event: &SliderEvent, cx| {
                this.update_accent_color(cx);
            },
        )
        .detach();
        cx.subscribe(
            &view.ac_g_slider,
            move |this, _, _event: &SliderEvent, cx| {
                this.update_accent_color(cx);
            },
        )
        .detach();
        cx.subscribe(
            &view.ac_b_slider,
            move |this, _, _event: &SliderEvent, cx| {
                this.update_accent_color(cx);
            },
        )
        .detach();

        view
    }

    fn update_background_color(&mut self, cx: &mut Context<Self>) {
        let r = self.bg_r_slider.read(cx).value().start() as u8;
        let g = self.bg_g_slider.read(cx).value().start() as u8;
        let b = self.bg_b_slider.read(cx).value().start() as u8;
        self.background_color = Rgb::new(r, g, b);
        cx.notify();
    }

    fn update_foreground_color(&mut self, cx: &mut Context<Self>) {
        let r = self.fg_r_slider.read(cx).value().start() as u8;
        let g = self.fg_g_slider.read(cx).value().start() as u8;
        let b = self.fg_b_slider.read(cx).value().start() as u8;
        self.foreground_color = Rgb::new(r, g, b);
        cx.notify();
    }

    fn update_accent_color(&mut self, cx: &mut Context<Self>) {
        let r = self.ac_r_slider.read(cx).value().start() as u8;
        let g = self.ac_g_slider.read(cx).value().start() as u8;
        let b = self.ac_b_slider.read(cx).value().start() as u8;
        self.accent_color = Rgb::new(r, g, b);
        cx.notify();
    }

    fn color_picker_section(
        &self,
        label: SharedString,
        color: Rgb,
        r_slider: &Entity<SliderState>,
        g_slider: &Entity<SliderState>,
        b_slider: &Entity<SliderState>,
    ) -> impl IntoElement {
        let hex = color.to_hex();

        v_flex()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(label),
                    )
                    .child(
                        div()
                            .w(px(80.0))
                            .h(px(30.0))
                            .rounded_md()
                            .border_1()
                            .border_color(gpui::rgb(0x666666))
                            .bg(gpui::rgb(u32::from_str_radix(&hex[1..], 16).unwrap_or(0))),
                    )
                    .child(div().text_xs().text_color(gpui::rgb(0x888888)).child(hex)),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(div().w(px(15.0)).text_xs().child("R"))
                    .child(Slider::new(r_slider).flex_1())
                    .child(
                        div()
                            .w(px(30.0))
                            .text_xs()
                            .text_right()
                            .child(format!("{}", color.r)),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(div().w(px(15.0)).text_xs().child("G"))
                    .child(Slider::new(g_slider).flex_1())
                    .child(
                        div()
                            .w(px(30.0))
                            .text_xs()
                            .text_right()
                            .child(format!("{}", color.g)),
                    ),
            )
            .child(
                h_flex()
                    .gap_2()
                    .child(div().w(px(15.0)).text_xs().child("B"))
                    .child(Slider::new(b_slider).flex_1())
                    .child(
                        div()
                            .w(px(30.0))
                            .text_xs()
                            .text_right()
                            .child(format!("{}", color.b)),
                    ),
            )
    }
}

impl Render for ThemeEditorView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let theme = cx.theme();
        let colors = &theme.colors;

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
                            })
                    )
                    .child(
                        div()
                            .text_xl()
                            .font_weight(gpui::FontWeight::BOLD)
                            .child("Theme Editor")
                    )
            )
            .child(
                h_flex()
                    .gap_6()
                    .flex_1()
                    .child(
                        v_flex()
                            .w(px(400.0))
                            .gap_6()
                            .child(
                                v_flex()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child("Theme Name"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .bg(colors.background)
                                            .border_1()
                                            .border_color(if self.theme_name_focus_handle.is_focused(window) {
                                                colors.accent
                                            } else {
                                                colors.border
                                            })
                                            .rounded_md()
                                            .p_2()
                                            .track_focus(&self.theme_name_focus_handle)
                                            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _window, cx| {
                                                let keystroke = &event.keystroke;
                                                if keystroke.modifiers.platform
                                                    || keystroke.modifiers.control
                                                    || keystroke.modifiers.alt
                                                {
                                                    return;
                                                }

                                                if keystroke.key == "backspace" {
                                                    this.theme_name.pop();
                                                    cx.notify();
                                                } else if keystroke.key.len() == 1 {
                                                    this.theme_name.push_str(&keystroke.key);
                                                    cx.notify();
                                                }
                                            }))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .child(if self.theme_name.is_empty() {
                                                        "Enter theme name...".to_string()
                                                    } else {
                                                        self.theme_name.clone()
                                                    })
                                                    .text_color(if self.theme_name.is_empty() {
                                                        colors.muted_foreground
                                                    } else {
                                                        colors.foreground
                                                    }),
                                            ),
                                    ),
                            )
                            .child(self.color_picker_section(
                                "Background Color".into(),
                                self.background_color,
                                &self.bg_r_slider,
                                &self.bg_g_slider,
                                &self.bg_b_slider,
                            ))
                            .child(self.color_picker_section(
                                "Foreground Color".into(),
                                self.foreground_color,
                                &self.fg_r_slider,
                                &self.fg_g_slider,
                                &self.fg_b_slider,
                            ))
                            .child(self.color_picker_section(
                                "Accent Color".into(),
                                self.accent_color,
                                &self.ac_r_slider,
                                &self.ac_g_slider,
                                &self.ac_b_slider,
                            ))
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new("btn-save")
                                            .primary()
                                            .label("Save Theme")
                                            .on_click(cx.listener(|this, _, _window, cx| {
                                                let palette = crate::utils::theme_export::build_theme_palette(
                                                    this.background_color,
                                                    this.foreground_color,
                                                    this.accent_color,
                                                );

                                                // Determine save path
                                                let app_config = this.app_state.read(cx).config.clone();
                                                let theme_dir = std::path::PathBuf::from(app_config.theme_file)
                                                    .parent()
                                                    .map(|p| p.to_path_buf())
                                                    .unwrap_or_else(|| std::path::PathBuf::from("./themes"));

                                                // Create directory if it doesn't exist
                                                let _ = std::fs::create_dir_all(&theme_dir);

                                                let filename = format!("{}.json", this.theme_name.replace(" ", "_").to_lowercase());
                                                let path = theme_dir.join(filename);

                                                match crate::utils::theme_export::export_theme_to_json(
                                                    &this.theme_name,
                                                    if this.background_color.r < 128 { "dark" } else { "light" },
                                                    &palette,
                                                    &path,
                                                ) {
                                                    Ok(_) => {
                                                        tracing::info!("Theme saved to {}", path.display());
                                                        // TODO: Show success notification
                                                    },
                                                    Err(e) => {
                                                        tracing::error!("Failed to save theme: {}", e);
                                                        // TODO: Show error notification
                                                    }
                                                }
                                            }))
                                    )
                                    .child(
                                        Button::new("btn-export")
                                            .label("Export JSON")
                                            .on_click(cx.listener(|this, _, _window, cx| {
                                                 let palette = crate::utils::theme_export::build_theme_palette(
                                                    this.background_color,
                                                    this.foreground_color,
                                                    this.accent_color,
                                                );

                                                // For now, export behaves same as save but logs explicitly as export
                                                // In a real app, this might open a file dialog
                                                let app_config = this.app_state.read(cx).config.clone();
                                                let theme_dir = std::path::PathBuf::from(app_config.theme_file)
                                                    .parent()
                                                    .map(|p| p.to_path_buf())
                                                    .unwrap_or_else(|| std::path::PathBuf::from("./themes"));

                                                let _ = std::fs::create_dir_all(&theme_dir);
                                                let filename = format!("{}_export.json", this.theme_name.replace(" ", "_").to_lowercase());
                                                let path = theme_dir.join(filename);

                                                match crate::utils::theme_export::export_theme_to_json(
                                                    &this.theme_name,
                                                    if this.background_color.r < 128 { "dark" } else { "light" },
                                                    &palette,
                                                    &path,
                                                ) {
                                                    Ok(_) => {
                                                        tracing::info!("Theme exported to {}", path.display());
                                                    },
                                                    Err(e) => {
                                                        tracing::error!("Failed to export theme: {}", e);
                                                    }
                                                }
                                            }))
                                    )
                            )
                    )
                    .child(
                        div()
                            .flex_1()
                            .rounded_lg()
                            .border_1()
                            .border_color(gpui::rgb(0x333333))
                            .bg(gpui::rgb(u32::from_str_radix(&self.background_color.to_hex()[1..], 16).unwrap_or(0)))
                            .p_6()
                            .child(
                                v_flex()
                                    .gap_4()
                                    .child(
                                        div()
                                            .text_xl()
                                            .font_weight(gpui::FontWeight::BOLD)
                                            .text_color(gpui::rgb(u32::from_str_radix(&self.foreground_color.to_hex()[1..], 16).unwrap_or(0xFFFFFF)))
                                            .child("Theme Preview")
                                    )
                                    .child(
                                        div()
                                            .text_color(gpui::rgb(u32::from_str_radix(&self.foreground_color.to_hex()[1..], 16).unwrap_or(0xFFFFFF)))
                                            .child("This is how your text will look with the selected foreground color.")
                                    )
                                    .child(
                                        div()
                                            .p_3()
                                            .rounded_md()
                                            .bg(gpui::rgb(u32::from_str_radix(&self.accent_color.to_hex()[1..], 16).unwrap_or(0)))
                                            .text_color(gpui::rgb(u32::from_str_radix(&self.background_color.to_hex()[1..], 16).unwrap_or(0)))
                                            .child("Accent color example")
                                    )
                            )
                    )
            )
    }
}
