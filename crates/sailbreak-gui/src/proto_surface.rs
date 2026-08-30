use gpui::{Div, Stateful, div, prelude::*, px, rgb};
use proto_ui_gpui::{ColorValue, ProtoButtonState};

/// Project a Proto UI Button snapshot into the native GPUI surface.
///
/// The Button state and style are produced by Proto UI. This function only
/// translates the already-resolved style intent into GPUI's native element
/// styling and keeps the Slot label owned by Sailbreak.
pub fn button_element(
    id: &'static str,
    label: &'static str,
    state: &ProtoButtonState,
) -> Stateful<Div> {
    let style = &state.style;
    let mut element = div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .flex()
        .flex_row()
        .items_center()
        .justify_center()
        .gap(px(style.gap))
        .h(px(style.height))
        .px(px(style.padding_x))
        .rounded(px(style.radius))
        .border_1()
        .border_color(to_hsla(style.border))
        .bg(to_hsla(style.background))
        .text_color(to_hsla(style.foreground))
        .text_sm()
        .font_weight(gpui::FontWeight::MEDIUM)
        .focusable();

    if style.pointer_events_none {
        element = element.cursor_not_allowed();
    } else {
        element = element.cursor_pointer();
    }
    if style.opacity < 1.0 {
        element = element.opacity(style.opacity);
    }
    if style.underline {
        element = element.underline();
    }
    if style.translate_y != 0.0 {
        element = element.relative().top(px(style.translate_y));
    }
    if let Some(ring) = style.ring {
        element = element.border_2().border_color(to_hsla(ring));
    }

    element.child(label)
}

fn to_hsla(color: ColorValue) -> gpui::Hsla {
    let mut rgba = rgb(color.rgb);
    rgba.a = color.alpha;
    rgba.into()
}
