use std::ops::Range;

use gpui::accesskit::ActionData;
use gpui::{
    AccessibleAction, App, Bounds, ClipboardItem, Context, CursorStyle, Div, Element, ElementId,
    ElementInputHandler, Entity, EntityInputHandler, EventEmitter, FocusHandle, Focusable,
    GlobalElementId, KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Orientation, PaintQuad, Pixels, Point, Role, ShapedLine, SharedString, Stateful,
    Style, TextRun, Toggled, UTF16Selection, UnderlineStyle, Window, div, fill, point, prelude::*,
    px, relative, rgb, rgba, size,
};
use proto_ui_gpui::{
    A11ySnapshot, ButtonStyle, ColorValue, DialogCloseSnapshot, DialogContentSnapshot,
    DialogDescriptionSnapshot, DialogFooterSnapshot, DialogHeaderSnapshot, DialogMaskSnapshot,
    DialogTitleSnapshot, DialogTriggerSnapshot, DropdownContentSnapshot, DropdownItemSnapshot,
    DropdownTriggerSnapshot, PlacementSnapshot, ProtoButtonState, ProtoSeparatorSnapshot,
    ProtoTextareaSnapshot, ProtoToggleSnapshot, SelectContentSnapshot, SelectItemSnapshot,
    SelectTriggerSnapshot, SelectValueSnapshot, SeparatorOrientation, TabsContentSnapshot,
    TabsListSnapshot, TabsTriggerSnapshot, TextControlEvent, TextControlEventType,
    TextControlSelection, TextControlSelectionDirection, ViewEpoch,
};

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
    action_element(
        id,
        label,
        &state.style,
        state.a11y.as_ref(),
        None,
        on_a11y_click,
    )
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
        None,
        on_a11y_click,
    )
}

/// Project one Tabs Trigger through a caller-owned GPUI focus handle.
pub fn tab_trigger_element(
    id: &'static str,
    label: &'static str,
    state: &TabsTriggerSnapshot,
    focus_handle: &FocusHandle,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    action_element(
        id,
        label,
        &state.resolved_style,
        state.a11y.as_ref(),
        Some(focus_handle),
        on_a11y_click,
    )
}
/// Project a Select Trigger through a caller-owned native focus handle.
pub fn select_trigger_element(
    id: &'static str,
    label: &'static str,
    state: &SelectTriggerSnapshot,
    focus_handle: Option<&FocusHandle>,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    action_element(
        id,
        label,
        &state.resolved_style,
        state.a11y.as_ref(),
        focus_handle,
        on_a11y_click,
    )
}

/// Project the Proto-owned display value into a native trigger child.
pub fn select_value_element(id: &'static str, state: &SelectValueSnapshot) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .child(state.display_value.clone())
}

/// Project a Select Content portal from the last Rust-owned placement fact.
/// No layout or paint callback calls JavaScript.
pub fn select_content_element(id: &'static str, state: &SelectContentSnapshot) -> Stateful<Div> {
    let mut element = state.placement.as_ref().map_or_else(
        || div().id(id).debug_selector(move || id.to_owned()),
        |placement| overlay_surface_element(id, placement),
    );
    if let Some(a11y) = state.a11y.as_ref() {
        element = apply_a11y(element, a11y, |_, _, _| {});
    }
    if !state.present {
        element = element.opacity(0.0);
    }
    element
}

/// Project one Select option and its selected indicator through the same
/// Proto-resolved action styling used by the trigger.
pub fn select_item_element(
    id: &'static str,
    state: &SelectItemSnapshot,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    let element = action_element(
        id,
        state.label.clone(),
        &state.resolved_style,
        state.a11y.as_ref(),
        None,
        on_a11y_click,
    );
    if state.selected_indicator {
        element.child("✓")
    } else {
        element
    }
}

/// Project a Dropdown Trigger through a caller-owned native focus handle.
pub fn dropdown_trigger_element(
    id: &'static str,
    label: &'static str,
    state: &DropdownTriggerSnapshot,
    focus_handle: Option<&FocusHandle>,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    action_element(
        id,
        label,
        &state.resolved_style,
        state.a11y.as_ref(),
        focus_handle,
        on_a11y_click,
    )
}

/// Project a Dropdown Content portal from the last Rust-owned placement fact.
/// No layout or paint callback calls JavaScript.
pub fn dropdown_content_element(
    id: &'static str,
    state: &DropdownContentSnapshot,
) -> Stateful<Div> {
    let mut element = state.placement.as_ref().map_or_else(
        || div().id(id).debug_selector(move || id.to_owned()),
        |placement| overlay_surface_element(id, placement),
    );
    if let Some(a11y) = state.a11y.as_ref() {
        element = apply_a11y(element, a11y, |_, _, _| {});
    }
    if !state.present {
        element = element.opacity(0.0);
    }
    element
}

/// Project one Dropdown menu item through Proto-resolved action styling.
pub fn dropdown_item_element(
    id: &'static str,
    state: &DropdownItemSnapshot,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    action_element(
        id,
        state.label.clone(),
        &state.resolved_style,
        state.a11y.as_ref(),
        None,
        on_a11y_click,
    )
}
/// Project a Dialog Trigger through the caller-owned native focus handle.
pub fn dialog_trigger_element(
    id: &'static str,
    label: &'static str,
    state: &DialogTriggerSnapshot,
    focus_handle: Option<&FocusHandle>,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    action_element(
        id,
        label,
        &state.resolved_style,
        state.a11y.as_ref(),
        focus_handle,
        on_a11y_click,
    )
}

/// Project the modal mask into the native GPUI overlay layer. The mask keeps
/// its geometry host-owned and is intentionally independent of dialog content
/// placement; Proto only supplies its resolved style and presence state.
pub fn dialog_mask_element(id: &'static str, state: &DialogMaskSnapshot) -> Stateful<Div> {
    let mut element = div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .absolute()
        .left(px(0.))
        .top(px(0.))
        .w_full()
        .h_full()
        .bg(rgba(0x00000080));
    if let Some(a11y) = state.a11y.as_ref() {
        element = apply_a11y(element, a11y, |_, _, _| {});
    }
    if !state.present {
        element = element.opacity(0.0);
    }
    element
}

/// Project centered dialog content from the last Rust-owned placement fact.
/// No layout or paint callback calls JavaScript.
pub fn dialog_content_element(
    id: &'static str,
    state: &DialogContentSnapshot,
    focus_handle: Option<&FocusHandle>,
) -> Stateful<Div> {
    let mut element = state.placement.as_ref().map_or_else(
        || div().id(id).debug_selector(move || id.to_owned()),
        |placement| overlay_surface_element(id, placement),
    );
    if let Some(handle) = focus_handle {
        element = element.track_focus(handle);
    }
    element = element
        .rounded(px(state.resolved_style.radius))
        .border_1()
        .border_color(to_hsla(state.resolved_style.border))
        .bg(to_hsla(state.resolved_style.background))
        .p_6();
    if let Some(a11y) = state.a11y.as_ref() {
        element = apply_a11y(element, a11y, |_, _, _| {});
    }
    if !state.present {
        element = element.opacity(0.0);
    }
    element
}

pub fn dialog_title_element(id: &'static str, state: &DialogTitleSnapshot) -> Stateful<Div> {
    let mut element = div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .text_lg()
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .child(state.label.clone());
    if let Some(a11y) = state.a11y.as_ref() {
        element = apply_a11y(element, a11y, |_, _, _| {});
    }
    element
}

pub fn dialog_description_element(
    id: &'static str,
    state: &DialogDescriptionSnapshot,
) -> Stateful<Div> {
    let mut element = div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .text_sm()
        .text_color(rgb(0x7f98a2))
        .child(state.label.clone());
    if let Some(a11y) = state.a11y.as_ref() {
        element = apply_a11y(element, a11y, |_, _, _| {});
    }
    element
}

pub fn dialog_close_element(
    id: &'static str,
    state: &DialogCloseSnapshot,
    focus_handle: Option<&FocusHandle>,
    on_a11y_click: impl FnMut(Option<&ActionData>, &mut Window, &mut App) + 'static,
) -> Stateful<Div> {
    action_element(
        id,
        state.label.clone(),
        &state.resolved_style,
        state.a11y.as_ref(),
        focus_handle,
        on_a11y_click,
    )
}

pub fn dialog_header_element(id: &'static str, state: &DialogHeaderSnapshot) -> Stateful<Div> {
    let element = div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .flex_col();
    match state.a11y.as_ref() {
        Some(a11y) => apply_a11y(element, a11y, |_, _, _| {}),
        None => element,
    }
}

pub fn dialog_footer_element(id: &'static str, state: &DialogFooterSnapshot) -> Stateful<Div> {
    let element = div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .flex_row();
    match state.a11y.as_ref() {
        Some(a11y) => apply_a11y(element, a11y, |_, _, _| {}),
        None => element,
    }
}

pub fn tab_list_element(id: &'static str, state: &TabsListSnapshot) -> Stateful<Div> {
    let element = div().id(id).debug_selector(move || id.to_owned());
    match state.a11y.as_ref() {
        Some(a11y) => apply_a11y(element, a11y, |_, _, _| {}),
        None => element,
    }
}

pub fn tab_panel_element(id: &'static str, state: &TabsContentSnapshot) -> Stateful<Div> {
    let element = div().id(id).debug_selector(move || id.to_owned());
    match state.a11y.as_ref() {
        Some(a11y) => apply_a11y(element, a11y, |_, _, _| {}),
        None => element,
    }
}

/// Project a contentless Proto Separator while keeping surrounding layout host-owned.
pub fn separator_element(id: &'static str, state: &ProtoSeparatorSnapshot) -> Stateful<Div> {
    let mut element = div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .flex_shrink_0()
        .bg(to_hsla(state.color));
    element = match state.orientation {
        SeparatorOrientation::Horizontal => element.h(px(1.)).w_full(),
        SeparatorOrientation::Vertical => element.w(px(1.)).h_full(),
    };
    if let Some(a11y) = state.session.a11y.as_ref() {
        element = apply_a11y(element, a11y, |_, _, _| {});
    }
    element
}

/// Materialize a precomputed Rust overlay placement in GPUI's absolute layer.
/// Callers order siblings by `OverlayHost::layer_order_of`; no JavaScript runs
/// during layout, prepaint, or paint.
pub fn overlay_surface_element(id: &'static str, placement: &PlacementSnapshot) -> Stateful<Div> {
    let rect = placement.floating_rect;
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .absolute()
        .left(px(rect.x))
        .top(px(rect.y))
        .w(px(rect.width))
        .h(px(rect.height))
}

fn action_element(
    id: &'static str,
    label: impl IntoElement,
    style: &ButtonStyle,
    a11y: Option<&A11ySnapshot>,
    focus_handle: Option<&FocusHandle>,
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
        .font_weight(gpui::FontWeight::MEDIUM);
    element = if let Some(handle) = focus_handle {
        element.track_focus(handle)
    } else {
        element.focusable()
    };
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
    pub hidden: bool,
    pub orientation: Option<Orientation>,
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
        hidden: snapshot.hidden,
        orientation: snapshot
            .orientation
            .as_deref()
            .and_then(|orientation| match orientation {
                "horizontal" => Some(Orientation::Horizontal),
                "vertical" => Some(Orientation::Vertical),
                _ => None,
            }),
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
    if projection.hidden {
        return element;
    }
    if projection.role != Role::Unknown {
        element = element.role(projection.role);
    }
    if !projection.label.is_empty() {
        element = element.aria_label(projection.label);
    }
    if let Some(orientation) = projection.orientation {
        element = element.aria_orientation(orientation);
    }
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

gpui::actions!(
    proto_textarea,
    [
        Backspace,
        Delete,
        Left,
        Right,
        SelectLeft,
        SelectRight,
        SelectAll,
        Home,
        End,
        ShiftHome,
        ShiftEnd,
        Enter,
        Copy,
        Cut,
        Paste,
    ]
);

/// Register the keyboard actions consumed by the native textarea entity.
///
/// The context keeps these bindings local to the editor; the Proto runtime
/// remains the semantic owner of the value and receives only data events.
pub(crate) fn bind_textarea_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some("ProtoTextarea")),
        KeyBinding::new("delete", Delete, Some("ProtoTextarea")),
        KeyBinding::new("left", Left, Some("ProtoTextarea")),
        KeyBinding::new("right", Right, Some("ProtoTextarea")),
        KeyBinding::new("shift-left", SelectLeft, Some("ProtoTextarea")),
        KeyBinding::new("shift-right", SelectRight, Some("ProtoTextarea")),
        KeyBinding::new("cmd-a", SelectAll, Some("ProtoTextarea")),
        KeyBinding::new("ctrl-a", SelectAll, Some("ProtoTextarea")),
        KeyBinding::new("home", Home, Some("ProtoTextarea")),
        KeyBinding::new("end", End, Some("ProtoTextarea")),
        KeyBinding::new("shift-home", ShiftHome, Some("ProtoTextarea")),
        KeyBinding::new("shift-end", ShiftEnd, Some("ProtoTextarea")),
        KeyBinding::new("enter", Enter, Some("ProtoTextarea")),
        KeyBinding::new("cmd-c", Copy, Some("ProtoTextarea")),
        KeyBinding::new("ctrl-c", Copy, Some("ProtoTextarea")),
        KeyBinding::new("cmd-x", Cut, Some("ProtoTextarea")),
        KeyBinding::new("ctrl-x", Cut, Some("ProtoTextarea")),
        KeyBinding::new("cmd-v", Paste, Some("ProtoTextarea")),
        KeyBinding::new("ctrl-v", Paste, Some("ProtoTextarea")),
    ]);
}

/// A data-only notification emitted by the native input entity.
///
/// GPUI handles and callbacks never leave this module. Dashboard subscribes to
/// this event and forwards its value/selection through `ProtoTextareaHost`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProtoTextareaNativeEvent {
    Text {
        id: String,
        epoch: ViewEpoch,
        event: TextControlEvent,
        selection: TextControlSelection,
    },
}

/// Native text buffer and input capabilities for one Proto Textarea.
///
/// This entity deliberately owns only the platform-facing buffer, UTF-16
/// selection, marked range, focus handle, and rendering cache. The semantic
/// value is accepted from `ProtoTextareaHost` after every event dispatch.
pub(crate) struct ProtoTextareaInput {
    id: String,
    epoch: ViewEpoch,
    content: String,
    placeholder: String,
    selection: TextControlSelection,
    marked_range: Option<Range<usize>>, // UTF-16 offsets, as required by GPUI.
    disabled: bool,
    read_only: bool,
    composing: bool,
    rows: u32,
    focus_handle: FocusHandle,
    selecting: bool,
    dirty: bool,
    last_layout: Vec<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
}

impl ProtoTextareaInput {
    pub(crate) fn from_snapshot(snapshot: &ProtoTextareaSnapshot, cx: &mut Context<Self>) -> Self {
        let content = snapshot.native_value.clone();
        Self {
            id: snapshot.id.clone(),
            epoch: snapshot.session.projection.view_epoch,
            content,
            placeholder: snapshot.placeholder.clone(),
            selection: snapshot.selection.clamp(utf16_len(&snapshot.native_value)),
            marked_range: None,
            disabled: snapshot.disabled,
            read_only: snapshot.read_only,
            composing: snapshot.composing,
            rows: snapshot.rows.max(1),
            focus_handle: cx.focus_handle(),
            selecting: false,
            dirty: false,
            last_layout: Vec::new(),
            last_bounds: None,
        }
    }

    pub(crate) fn focus_handle(&self) -> FocusHandle {
        self.focus_handle.clone()
    }

    pub(crate) fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    /// Project the host's semantic/native snapshot back into this entity.
    ///
    /// `native_value` is intentional here: a controlled Proto value may defer
    /// replacement while IME text is marked, and resetting that buffer during a
    /// composition round would corrupt the platform selection.
    pub(crate) fn sync_snapshot(&mut self, snapshot: &ProtoTextareaSnapshot) {
        self.epoch = snapshot.session.projection.view_epoch;
        self.content = snapshot.native_value.clone();
        self.placeholder = snapshot.placeholder.clone();
        self.selection = snapshot.selection.clamp(utf16_len(&self.content));
        self.disabled = snapshot.disabled;
        self.read_only = snapshot.read_only;
        self.composing = snapshot.composing;
        self.rows = snapshot.rows.max(1);
        if !self.composing {
            self.marked_range = None;
        } else if let Some(marked_range) = self.marked_range.as_mut() {
            let length = utf16_len(&self.content);
            marked_range.start = marked_range.start.min(length);
            marked_range.end = marked_range.end.min(length);
            if marked_range.start >= marked_range.end {
                self.marked_range = None;
            }
        }
        self.last_layout.clear();
        self.last_bounds = None;
    }

    fn cursor_utf16(&self) -> usize {
        if self.selection.direction == TextControlSelectionDirection::Backward {
            self.selection.start
        } else {
            self.selection.end
        }
    }

    fn selected_range_utf8(&self) -> Range<usize> {
        let selection = self.selection.clamp(utf16_len(&self.content));
        range_from_utf16(&self.content, selection.start..selection.end)
    }

    fn cursor_utf8(&self) -> usize {
        offset_from_utf16(&self.content, self.cursor_utf16())
    }

    fn set_caret_utf8(&mut self, offset: usize) {
        self.selection = TextControlSelection::caret(offset_to_utf16(&self.content, offset));
    }

    fn extend_selection_utf8(&mut self, offset: usize) {
        let current = self.selection.clamp(utf16_len(&self.content));
        let cursor = self.cursor_utf16();
        let anchor = if current.direction == TextControlSelectionDirection::Backward {
            current.end
        } else if current.direction == TextControlSelectionDirection::Forward
            || current.start != current.end
        {
            current.start
        } else {
            cursor
        };
        let next = offset_to_utf16(&self.content, offset);
        self.selection = if next == anchor {
            TextControlSelection::caret(next)
        } else if next < anchor {
            TextControlSelection::range(next, anchor)
                .with_direction(TextControlSelectionDirection::Backward)
        } else {
            TextControlSelection::range(anchor, next)
                .with_direction(TextControlSelectionDirection::Forward)
        };
    }

    fn emit_text(
        &self,
        event_type: TextControlEventType,
        data: Option<String>,
        input_type: Option<&str>,
        composing: bool,
        cx: &mut Context<Self>,
    ) {
        let event = TextControlEvent {
            event_type,
            value: self.content.clone(),
            composing,
            data,
            input_type: input_type.map(str::to_owned),
        };
        cx.emit(ProtoTextareaNativeEvent::Text {
            id: self.id.clone(),
            epoch: self.epoch,
            event,
            selection: self.selection,
        });
    }

    fn emit_composition_commit(&self, data: Option<String>, cx: &mut Context<Self>) {
        self.emit_text(
            TextControlEventType::CompositionEnd,
            data.clone(),
            None,
            false,
            cx,
        );
        self.emit_text(
            TextControlEventType::Input,
            data,
            Some("insertCompositionText"),
            false,
            cx,
        );
    }

    fn replace_selection(
        &mut self,
        new_text: &str,
        input_type: &'static str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || self.read_only {
            return;
        }
        let selected = self.selected_range_utf8();
        let was_composing = self.marked_range.take().is_some();
        if selected.start == selected.end && new_text.is_empty() {
            window.play_system_bell();
            return;
        }
        self.content.replace_range(selected.clone(), new_text);
        self.dirty = true;
        self.set_caret_utf8(selected.start + new_text.len());
        self.composing = false;
        if was_composing {
            self.emit_composition_commit(Some(new_text.to_owned()), cx);
        } else {
            self.emit_text(
                TextControlEventType::Input,
                Some(new_text.to_owned()),
                Some(input_type),
                false,
                cx,
            );
        }
        cx.notify();
    }

    fn backspace(&mut self, _: &Backspace, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.read_only {
            return;
        }
        if self.selection.start == self.selection.end {
            let cursor = self.cursor_utf8();
            if cursor == 0 {
                window.play_system_bell();
                return;
            }
            let start = previous_boundary(&self.content, cursor);
            self.selection = TextControlSelection::range(
                offset_to_utf16(&self.content, start),
                offset_to_utf16(&self.content, cursor),
            );
        }
        self.replace_selection("", "deleteContentBackward", window, cx);
    }

    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.read_only {
            return;
        }
        if self.selection.start == self.selection.end {
            let cursor = self.cursor_utf8();
            if cursor >= self.content.len() {
                window.play_system_bell();
                return;
            }
            let end = next_boundary(&self.content, cursor);
            self.selection = TextControlSelection::range(
                offset_to_utf16(&self.content, cursor),
                offset_to_utf16(&self.content, end),
            );
        }
        self.replace_selection("", "deleteContentForward", window, cx);
    }

    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection.start != self.selection.end {
            let start =
                range_from_utf16(&self.content, self.selection.start..self.selection.end).start;
            self.set_caret_utf8(start);
        } else {
            self.set_caret_utf8(previous_boundary(&self.content, self.cursor_utf8()));
        }
        cx.notify();
    }

    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection.start != self.selection.end {
            let end = range_from_utf16(&self.content, self.selection.start..self.selection.end).end;
            self.set_caret_utf8(end);
        } else {
            self.set_caret_utf8(next_boundary(&self.content, self.cursor_utf8()));
        }
        cx.notify();
    }

    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        let offset = previous_boundary(&self.content, self.cursor_utf8());
        self.extend_selection_utf8(offset);
        cx.notify();
    }

    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        let offset = next_boundary(&self.content, self.cursor_utf8());
        self.extend_selection_utf8(offset);
        cx.notify();
    }

    fn select_all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.selection = TextControlSelection::range(0, utf16_len(&self.content))
            .with_direction(TextControlSelectionDirection::Forward);
        cx.notify();
    }

    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.set_caret_utf8(line_start(&self.content, self.cursor_utf8()));
        cx.notify();
    }

    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        self.set_caret_utf8(line_end(&self.content, self.cursor_utf8()));
        cx.notify();
    }

    fn shift_home(&mut self, _: &ShiftHome, _: &mut Window, cx: &mut Context<Self>) {
        self.extend_selection_utf8(line_start(&self.content, self.cursor_utf8()));
        cx.notify();
    }

    fn shift_end(&mut self, _: &ShiftEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.extend_selection_utf8(line_end(&self.content, self.cursor_utf8()));
        cx.notify();
    }

    fn enter(&mut self, _: &Enter, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_selection("\n", "insertLineBreak", window, cx);
    }

    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        if self.selection.start != self.selection.end {
            let range = self.selected_range_utf8();
            cx.write_to_clipboard(ClipboardItem::new_string(self.content[range].to_owned()));
        }
    }

    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>) {
        if self.selection.start != self.selection.end && !self.disabled && !self.read_only {
            let range = self.selected_range_utf8();
            cx.write_to_clipboard(ClipboardItem::new_string(
                self.content[range.clone()].to_owned(),
            ));
            self.replace_selection("", "deleteByCut", window, cx);
        }
    }

    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.read_only {
            return;
        }
        if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
            // Preserve line breaks; flattening paste would make a multiline
            // profile impossible to author.
            self.replace_selection(&text, "insertFromPaste", window, cx);
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        window.focus(&self.focus_handle, cx);
        let offset = self.index_for_mouse_position(event.position, window);
        if event.modifiers.shift {
            self.extend_selection_utf8(offset);
        } else {
            self.set_caret_utf8(offset);
        }
        self.selecting = true;
        cx.notify();
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.selecting = false;
    }

    fn on_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.selecting {
            self.extend_selection_utf8(self.index_for_mouse_position(event.position, window));
            cx.notify();
        }
    }

    fn index_for_mouse_position(&self, position: Point<Pixels>, window: &Window) -> usize {
        let Some(bounds) = self.last_bounds else {
            return 0;
        };
        if self.content.is_empty() || self.last_layout.is_empty() {
            return 0;
        }
        let line_height = window.line_height();
        let mut line_index = 0;
        let mut top = bounds.top();
        while line_index + 1 < self.last_layout.len() && position.y >= top + line_height {
            line_index += 1;
            top += line_height;
        }
        if position.y < bounds.top() {
            line_index = 0;
        }
        let line_index = line_index.min(self.last_layout.len().saturating_sub(1));
        let line_ranges = line_ranges(&self.content);
        let Some(line_range) = line_ranges.get(line_index) else {
            return self.content.len();
        };
        let x = position.x - bounds.left();
        let local = if x <= px(0.) {
            0
        } else {
            self.last_layout[line_index]
                .closest_index_for_x(x)
                .min(line_range.end - line_range.start)
        };
        line_range.start + local
    }
}

impl EventEmitter<ProtoTextareaNativeEvent> for ProtoTextareaInput {}

impl Focusable for ProtoTextareaInput {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EntityInputHandler for ProtoTextareaInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let length = utf16_len(&self.content);
        let range_utf16 = range_utf16.start.min(length)..range_utf16.end.min(length);
        let range = range_from_utf16(&self.content, range_utf16.clone());
        actual_range.replace(range_to_utf16(&self.content, &range));
        Some(self.content[range].to_owned())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let selection = self.selection.clamp(utf16_len(&self.content));
        Some(UTF16Selection {
            range: selection.start..selection.end,
            reversed: selection.direction == TextControlSelectionDirection::Backward,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked_range.clone()
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if self.marked_range.take().is_some() {
            self.composing = false;
            self.emit_composition_commit(None, cx);
            cx.notify();
        }
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || self.read_only {
            return;
        }
        let marked = self.marked_range.take();
        let was_composing = marked.is_some() || self.composing;
        let range_utf16 = range_utf16.or(marked).unwrap_or_else(|| {
            let selection = self.selection.clamp(utf16_len(&self.content));
            selection.start..selection.end
        });
        let range = range_from_utf16(&self.content, range_utf16);
        self.content.replace_range(range.clone(), new_text);
        self.dirty = true;
        self.set_caret_utf8(range.start + new_text.len());
        self.composing = false;
        if was_composing {
            self.emit_composition_commit(Some(new_text.to_owned()), cx);
        } else {
            self.emit_text(
                TextControlEventType::Input,
                Some(new_text.to_owned()),
                Some("insertText"),
                false,
                cx,
            );
        }
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || self.read_only {
            return;
        }
        let previous_marked = self.marked_range.take();
        let was_composing = previous_marked.is_some();
        let range_utf16 = range_utf16.or(previous_marked).unwrap_or_else(|| {
            let cursor = self.cursor_utf16();
            cursor..cursor
        });
        let range = range_from_utf16(&self.content, range_utf16);
        let start = range.start;
        self.content.replace_range(range, new_text);
        self.dirty = true;
        let start_utf16 = offset_to_utf16(&self.content, start);
        let end_utf16 = start_utf16 + utf16_len(new_text);
        self.marked_range = (!new_text.is_empty()).then_some(start_utf16..end_utf16);
        self.selection = new_selected_range_utf16
            .map(|range| {
                TextControlSelection::range(
                    (start_utf16 + range.start).min(utf16_len(&self.content)),
                    (start_utf16 + range.end).min(utf16_len(&self.content)),
                )
            })
            .unwrap_or_else(|| TextControlSelection::caret(end_utf16));
        self.composing = true;
        self.emit_text(
            if was_composing {
                TextControlEventType::CompositionUpdate
            } else {
                TextControlEventType::CompositionStart
            },
            Some(new_text.to_owned()),
            None,
            true,
            cx,
        );
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let bounds = self.last_bounds.unwrap_or(bounds);
        let range = range_from_utf16(&self.content, range_utf16);
        let ranges = line_ranges(&self.content);
        let (line_index, line_range) = ranges
            .iter()
            .enumerate()
            .find(|(_, line)| range.start <= line.end && range.start >= line.start)
            .unwrap_or((0, ranges.first()?));
        let line = self.last_layout.get(line_index)?;
        let x = line.x_for_index(range.start.saturating_sub(line_range.start));
        Some(Bounds::new(
            point(
                bounds.left() + x,
                bounds.top() + window.line_height() * line_index as f32,
            ),
            size(px(1.), window.line_height()),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        window: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(offset_to_utf16(
            &self.content,
            self.index_for_mouse_position(point, window),
        ))
    }
}

impl gpui::Render for ProtoTextareaInput {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .min_h_0()
            .key_context("ProtoTextarea")
            .track_focus(&self.focus_handle())
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_all))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::shift_home))
            .on_action(cx.listener(Self::shift_end))
            .on_action(cx.listener(Self::enter))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_action(cx.listener(Self::paste))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .child(ProtoTextareaText { input: cx.entity() })
    }
}

struct ProtoTextareaText {
    input: Entity<ProtoTextareaInput>,
}

struct ProtoTextareaPrepaint {
    lines: Vec<ShapedLine>,
    selection: Vec<PaintQuad>,
    cursor: Option<PaintQuad>,
}

impl IntoElement for ProtoTextareaText {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ProtoTextareaText {
    type RequestLayoutState = ();
    type PrepaintState = ProtoTextareaPrepaint;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let input = self.input.read(cx);
        let line_count = input
            .content
            .split('\n')
            .count()
            .max(input.rows.max(1) as usize);
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = (window.line_height() * line_count as f32).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let content = input.content.clone();
        let placeholder = input.placeholder.clone();
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        let is_placeholder = content.is_empty();
        let display = if is_placeholder {
            placeholder.clone()
        } else {
            content.clone()
        };
        let display_ranges = line_ranges(&display);
        let marked = if is_placeholder {
            None
        } else {
            input
                .marked_range
                .as_ref()
                .map(|range| range_from_utf16(&content, range.clone()))
        };
        let text_color = style.color;
        let placeholder_color = rgb(0x7f98a2).into();
        let lines = display
            .split('\n')
            .enumerate()
            .map(|(line_index, line_text)| {
                let line_start = display_ranges
                    .get(line_index)
                    .map_or(0, |range| range.start);
                let line_end = line_start + line_text.len();
                let color = if is_placeholder {
                    placeholder_color
                } else {
                    text_color
                };
                let base_run = |len: usize, underline: Option<UnderlineStyle>| TextRun {
                    len,
                    font: style.font(),
                    color,
                    background_color: None,
                    underline,
                    strikethrough: None,
                };
                let mut runs = Vec::new();
                if let Some(marked) = marked.as_ref() {
                    let marked_start = marked.start.max(line_start).min(line_end);
                    let marked_end = marked.end.max(marked_start).min(line_end);
                    if marked_start > line_start {
                        runs.push(base_run(marked_start - line_start, None));
                    }
                    if marked_end > marked_start {
                        runs.push(base_run(
                            marked_end - marked_start,
                            Some(UnderlineStyle {
                                color: Some(color),
                                thickness: px(1.),
                                wavy: false,
                            }),
                        ));
                    }
                    if line_end > marked_end {
                        runs.push(base_run(line_end - marked_end, None));
                    }
                } else {
                    runs.push(base_run(line_text.len(), None));
                }
                window.text_system().shape_line(
                    SharedString::from(line_text.to_owned()),
                    font_size,
                    &runs,
                    None,
                )
            })
            .collect::<Vec<_>>();

        let mut selection = Vec::new();
        if !is_placeholder && input.selection.start != input.selection.end {
            let selected = input.selected_range_utf8();
            for (line_index, range) in line_ranges(&content).iter().enumerate() {
                let start = selected.start.max(range.start).min(range.end);
                let end = selected.end.max(start).min(range.end);
                if end > start {
                    let Some(line) = lines.get(line_index) else {
                        continue;
                    };
                    selection.push(fill(
                        Bounds::new(
                            point(
                                bounds.left() + line.x_for_index(start - range.start),
                                bounds.top() + line_height * line_index as f32,
                            ),
                            size(
                                (line.x_for_index(end - range.start)
                                    - line.x_for_index(start - range.start))
                                .max(px(2.)),
                                line_height,
                            ),
                        ),
                        rgba(0x54d6c744),
                    ));
                }
            }
        }

        let cursor_offset = if input.selection.direction == TextControlSelectionDirection::Backward
        {
            input.selection.start
        } else {
            input.selection.end
        };
        let cursor_utf8 = offset_from_utf16(&content, cursor_offset);
        let (cursor_line, cursor_in_line) = cursor_line_and_offset(&content, cursor_utf8);
        let cursor_line = cursor_line.min(lines.len().saturating_sub(1));
        let cursor_x = lines
            .get(cursor_line)
            .map_or(px(0.), |line| line.x_for_index(cursor_in_line));
        let cursor = (input.focus_handle.is_focused(window) && !input.disabled).then(|| {
            fill(
                Bounds::new(
                    point(
                        bounds.left() + cursor_x,
                        bounds.top() + line_height * cursor_line as f32,
                    ),
                    size(px(1.5), line_height),
                ),
                text_color,
            )
        });
        ProtoTextareaPrepaint {
            lines,
            selection,
            cursor,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.input.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
            cx,
        );
        for quad in prepaint.selection.drain(..) {
            window.paint_quad(quad);
        }
        let line_height = window.line_height();
        let lines = std::mem::take(&mut prepaint.lines);
        for (line_index, line) in lines.iter().enumerate() {
            let _ = line.paint(
                point(
                    bounds.left(),
                    bounds.top() + line_height * line_index as f32,
                ),
                line_height,
                gpui::TextAlign::Left,
                None,
                window,
                cx,
            );
        }
        if let Some(cursor) = prepaint.cursor.take() {
            window.paint_quad(cursor);
        }
        self.input.update(cx, |input, _| {
            input.last_layout = lines;
            input.last_bounds = Some(bounds);
        });
    }
}

/// Compose a native GPUI Textarea around a resolved Proto snapshot.
pub(crate) fn textarea_element(
    id: &'static str,
    input: Entity<ProtoTextareaInput>,
    snapshot: &ProtoTextareaSnapshot,
) -> Stateful<Div> {
    let border = if snapshot.focused || snapshot.focus_visible {
        rgb(0x54d6c7)
    } else {
        rgb(0x2a3b48)
    };
    let mut element = div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .w_full()
        .min_h_0()
        .flex()
        .flex_col()
        .overflow_hidden()
        .rounded(px(4.))
        .border_1()
        .border_color(border)
        .bg(rgb(0x111b24))
        .text_color(rgb(0xd8e4e8))
        .child(input);
    if snapshot.disabled {
        element = element.opacity(0.55).cursor_not_allowed();
    }
    if let Some(a11y) = snapshot.a11y.as_ref() {
        element = apply_a11y(element, a11y, |_, _, _| {})
            .role(Role::MultilineTextInput)
            .aria_value(snapshot.native_value.clone())
            .aria_placeholder(snapshot.placeholder.clone());
    }
    element
}

fn utf16_len(value: &str) -> usize {
    value.encode_utf16().count()
}

fn offset_from_utf16(value: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    let mut utf8 = 0;
    for ch in value.chars() {
        if utf16 >= offset {
            break;
        }
        utf16 += ch.len_utf16();
        utf8 += ch.len_utf8();
    }
    utf8
}

fn offset_to_utf16(value: &str, offset: usize) -> usize {
    let mut utf16 = 0;
    let mut utf8 = 0;
    for ch in value.chars() {
        if utf8 >= offset {
            break;
        }
        utf8 += ch.len_utf8();
        utf16 += ch.len_utf16();
    }
    utf16
}

fn range_from_utf16(value: &str, range: Range<usize>) -> Range<usize> {
    offset_from_utf16(value, range.start)..offset_from_utf16(value, range.end)
}

fn range_to_utf16(value: &str, range: &Range<usize>) -> Range<usize> {
    offset_to_utf16(value, range.start)..offset_to_utf16(value, range.end)
}

fn previous_boundary(value: &str, offset: usize) -> usize {
    value[..offset.min(value.len())]
        .char_indices()
        .last()
        .map_or(0, |(index, _)| index)
}

fn next_boundary(value: &str, offset: usize) -> usize {
    value[offset.min(value.len())..]
        .chars()
        .next()
        .map_or(value.len(), |ch| offset.min(value.len()) + ch.len_utf8())
}

fn line_ranges(value: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for (index, ch) in value.char_indices() {
        if ch == '\n' {
            ranges.push(start..index);
            start = index + ch.len_utf8();
        }
    }
    ranges.push(start..value.len());
    ranges
}

fn line_start(value: &str, offset: usize) -> usize {
    let offset = offset.min(value.len());
    value[..offset].rfind('\n').map_or(0, |index| index + 1)
}

fn line_end(value: &str, offset: usize) -> usize {
    let offset = offset.min(value.len());
    value[offset..]
        .find('\n')
        .map_or(value.len(), |index| offset + index)
}

fn cursor_line_and_offset(value: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(value.len());
    let line_index = value[..offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let line_start = line_start(value, offset);
    (line_index, offset.saturating_sub(line_start))
}

fn role_for(role: &str) -> Role {
    match role {
        "button" => Role::Button,
        "textbox" => Role::TextInput,
        "separator" => Role::Splitter,
        "tab" => Role::Tab,
        "tablist" => Role::TabList,
        "tabpanel" => Role::TabPanel,
        "menu" => Role::Menu,
        "menuitem" => Role::MenuItem,
        "dialog" => Role::Dialog,
        "alertdialog" => Role::AlertDialog,
        _ => Role::Unknown,
    }
}
