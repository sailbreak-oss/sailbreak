use gpui::accesskit::{ActionData, Role};
use gpui::{AccessibleAction, App, Div, Stateful, Toggled, Window, div, prelude::*, px, rgb};
use proto_ui_gpui::{A11ySnapshot, ButtonStyle, ColorValue, ProtoButtonState, ProtoToggleSnapshot};

/// Project a Proto UI Button snapshot into the native GPUI surface.
///
/// The Button state and style are produced by Proto UI. This function only
/// translates the already-resolved style intent into GPUI's native element
/// styling and keeps the Slot label owned by Sailbreak.
pub fn button_element(
    id: &'static str,
    label: &'static str,
    state: &ProtoButtonState,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    action_element(id, label, &state.style, state.a11y.as_ref(), on_a11y_click)
}

/// Project a Proto UI Toggle snapshot through the same native action surface.
pub fn toggle_element(
    id: &'static str,
    label: &'static str,
    state: &ProtoToggleSnapshot,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    action_element(
        id,
        label,
        &state.resolved_style,
        state.a11y.as_ref(),
        on_a11y_click,
    )
}

fn action_element(
    id: &'static str,
    label: &'static str,
    style: &ButtonStyle,
    a11y: Option<&A11ySnapshot>,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
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
    if let Some(snapshot) = a11y {
        element = apply_a11y(element, snapshot, on_a11y_click);
    }
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

/// Native AccessKit projection for one resolved Proto UI a11y snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessibleProjection {
    pub role: Role,
    pub label: String,
    pub disabled: bool,
    pub selected: Option<bool>,
    pub toggled: Option<Toggled>,
    pub action: Option<AccessibleAction>,
}

/// Map a resolved `A11ySnapshot` to the pinned GPUI AccessKit surface.
pub fn project_a11y(snapshot: &A11ySnapshot) -> AccessibleProjection {
    AccessibleProjection {
        role: role_for(&snapshot.role),
        label: snapshot.name.clone(),
        disabled: snapshot.disabled,
        selected: snapshot.selected,
        toggled: snapshot
            .toggled
            .map(|value| if value { Toggled::True } else { Toggled::False }),
        action: snapshot
            .actions
            .iter()
            .any(|action| action == "activate")
            .then_some(AccessibleAction::Click),
    }
}

fn apply_a11y(
    mut element: Stateful<Div>,
    snapshot: &A11ySnapshot,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let projection = project_a11y(snapshot);
    element = element.role(projection.role).aria_label(projection.label);
    if let Some(selected) = projection.selected {
        element = element.aria_selected(selected);
    }
    if let Some(toggled) = projection.toggled {
        element = element.aria_toggled(toggled);
    }
    if projection.disabled {
        element = element.a11y_synthetic_children(|builder| {
            builder.parent_node().set_disabled();
        });
    }
    if projection.action == Some(AccessibleAction::Click) {
        element = element.on_a11y_action(AccessibleAction::Click, on_a11y_click);
    }
    element
}

fn role_for(role: &str) -> Role {
    match role {
        "button" => Role::Button,
        _ => Role::Unknown,
    }
}
