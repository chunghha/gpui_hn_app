/*
Extracted webview controls into a reusable UI component.

This module provides a small helper to render the bottom-right webview controls
that were previously in `HnLayout::render`. It keeps the existing look/behavior
but makes the UI reusable so the layout can be tidier.

Usage:
    render_webview_controls(app_state_entity, &slider_state_entity, current_zoom, colors)
*/

use crate::state::AppState;
use gpui::{Entity, IntoElement, div, prelude::*};
use gpui_component::slider::{Slider, SliderState};
use gpui_component::theme::ThemeColor;

/// Render the webview controls row used when the WebView is visible.
///
/// - `app_state` is kept so individual controls can dispatch actions if needed
///   (the current implementation only reads the zoom value from the passed-in
///   `zoom_value` and uses the provided `slider_state` for the slider widget).
/// - `slider_state` is the shared state backing the `Slider` widget.
/// - `zoom_value` is the current integer zoom percent (e.g. 100 for 100%).
/// - `colors` is the theme color palette to style the control bar.
///
/// This returns an `impl IntoElement` so it can be composed inline in layouts.
pub fn render_webview_controls(
    _app_state: Entity<AppState>,
    slider_state: &Entity<SliderState>,
    zoom_value: u32,
    colors: ThemeColor,
) -> impl IntoElement {
    // Keep layout and styling consistent with previous inline implementation.
    div()
        .flex()
        .items_center()
        .gap_3()
        .p_3()
        .bg(colors.background)
        .border_1()
        .border_color(colors.border)
        .rounded_md()
        .child(div().text_sm().text_color(colors.foreground).child("Zoom:"))
        .child(Slider::new(slider_state))
        .child(
            div()
                .text_sm()
                .text_color(colors.foreground)
                .child(format!("{}%", zoom_value)),
        )
}
