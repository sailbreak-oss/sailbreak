//! A read-only GPUI control-center shell for Sailbreak.
//!
//! The GUI deliberately has no hardware mutation path. It renders the state
//! reported by [`lctrl_hal::Hal`] and marks every capability according to the
//! core availability value, including its explanation.

use std::{cell::RefCell, env, rc::Rc, sync::Arc};

use gpui::{
    App, Bounds, Context, Div, IntoElement, Render, Stateful, Window, WindowBounds, WindowOptions,
    div, prelude::*, px, rgb, size,
};
use lctrl_core::{Availability, CapabilitySet, HardwareInfo, LctrlError, Platform, Result};
use lctrl_hal::Hal;
use proto_ui_gpui::{
    BridgeError, DispatchOutcome, InputKind, InputSource, ProtoButtonHost, ProtoButtonState,
    ProtoToggleHost, ProtoToggleSnapshot, ShadcnButtonSize, ShadcnButtonVariant,
    ToggleDispatchOutcome, ToggleProps, ToggleSize, ToggleVariant,
};

mod proto_surface;
pub use proto_surface::{AccessibleProjection, project_a11y};

// Palette tokens: a cool instrument-panel base, with one signal color and one
// caution color. Keeping these in one place makes the dashboard's visual
// language intentional and easy to retune without scattering literals.
const INK: u32 = 0x0b1118;
const SURFACE: u32 = 0x111b24;
const SURFACE_RAISED: u32 = 0x172531;
const RULE: u32 = 0x2a3b48;
const TEXT: u32 = 0xd8e4e8;
const MUTED: u32 = 0x7f98a2;
const SIGNAL: u32 = 0x54d6c7;
const CAUTION: u32 = 0xf2b66d;
const UNAVAILABLE: u32 = 0x53636c;
const WINDOW_WIDTH: f32 = 1240.0;
const WINDOW_HEIGHT: f32 = 820.0;
const SIDEBAR_WIDTH: f32 = 238.0;
const MARKER_SIZE: f32 = 8.0;
const FEATURE_COLUMN_WIDTH: f32 = 190.0;
const AVAILABILITY_COLUMN_WIDTH: f32 = 112.0;

/// Immutable data consumed by the dashboard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DashboardSnapshot {
    /// The platform reported by the hardware abstraction layer.
    pub platform: Platform,
    /// Product identity reported by the hardware abstraction layer.
    pub hardware: HardwareInfo,
    /// Capabilities and their availability explanations.
    pub capabilities: CapabilitySet,
    /// Human-readable status for the safety banner.
    pub status_message: String,
}

impl DashboardSnapshot {
    /// Query the real platform, hardware identity, and capability set.
    ///
    /// Errors are returned unchanged from `lctrl-core`; this keeps callers from
    /// accidentally displaying a successful-looking snapshot for a failed
    /// query.
    pub fn from_hal(hal: &dyn Hal) -> Result<Self> {
        let platform = hal.platform();
        let hardware = hal.hardware_info()?;
        let capabilities = hal.capabilities()?;
        Ok(Self {
            platform,
            hardware,
            capabilities,
            status_message: "Live hardware status synchronized".to_string(),
        })
    }

    /// Build the conservative state used by the standalone executable until a
    /// platform HAL is supplied by the controller.
    #[must_use]
    pub fn unavailable(platform: Platform, status_message: impl Into<String>) -> Self {
        Self {
            platform,
            hardware: HardwareInfo::default(),
            capabilities: CapabilitySet::new(platform),
            status_message: status_message.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DashboardAction {
    SelectSection(usize),
    Refresh,
    RunCommand(&'static [&'static str]),
}

/// Platform composition root used by the interactive dashboard.
pub trait GuiController: Send + Sync {
    fn refresh(&self) -> Result<DashboardSnapshot>;
    fn execute(&self, args: &[&str]) -> Result<String>;
}

struct StaticController {
    snapshot: DashboardSnapshot,
}

impl GuiController for StaticController {
    fn refresh(&self) -> Result<DashboardSnapshot> {
        Ok(self.snapshot.clone())
    }

    fn execute(&self, _args: &[&str]) -> Result<String> {
        Err(LctrlError::Unsupported {
            feature: "gui.command-controller".into(),
        })
    }
}

const SIDEBAR_BUTTON_IDS: [&str; 7] = [
    "sidebar-section-0",
    "sidebar-section-1",
    "sidebar-section-2",
    "sidebar-section-3",
    "sidebar-section-4",
    "sidebar-section-5",
    "sidebar-section-6",
];

const ACTION_BUTTONS: [(&str, &str, ShadcnButtonVariant, ShadcnButtonSize); 7] = [
    (
        "refresh-status",
        "REFRESH STATUS",
        ShadcnButtonVariant::Default,
        ShadcnButtonSize::Sm,
    ),
    (
        "daemon-status",
        "DAEMON STATUS",
        ShadcnButtonVariant::Outline,
        ShadcnButtonSize::Sm,
    ),
    (
        "battery-status",
        "BATTERY STATUS",
        ShadcnButtonVariant::Secondary,
        ShadcnButtonSize::Sm,
    ),
    (
        "thermal-sensors",
        "THERMAL SENSORS",
        ShadcnButtonVariant::Secondary,
        ShadcnButtonSize::Sm,
    ),
    (
        "power-schemes",
        "POWER SCHEMES",
        ShadcnButtonVariant::Secondary,
        ShadcnButtonSize::Sm,
    ),
    (
        "diagnostics",
        "DIAGNOSTICS",
        ShadcnButtonVariant::Destructive,
        ShadcnButtonSize::Sm,
    ),
    (
        "magicbay-detect",
        "MAGICBAY DETECT",
        ShadcnButtonVariant::Destructive,
        ShadcnButtonSize::Sm,
    ),
];
const PERFORMANCE_PREVIEW_TOGGLE_ID: &str = "performance-preview";

fn performance_preview_props(active: bool) -> ToggleProps {
    ToggleProps {
        variant: ToggleVariant::Outline,
        size: ToggleSize::Sm,
        active: Some(active),
        default_active: false,
        disabled: false,
    }
}

struct ProtoUiState {
    host: Option<ProtoButtonHost>,
    toggle_host: Option<ProtoToggleHost>,
    error: Option<String>,
}

impl ProtoUiState {
    fn new() -> Self {
        match Self::build() {
            Ok((host, toggle_host)) => Self {
                host: Some(host),
                toggle_host: Some(toggle_host),
                error: None,
            },
            Err(error) => Self {
                host: None,
                toggle_host: None,
                error: Some(error.to_string()),
            },
        }
    }

    fn build() -> std::result::Result<(ProtoButtonHost, ProtoToggleHost), BridgeError> {
        let mut host = ProtoButtonHost::new()?;
        for (index, _) in SECTION_NAMES.iter().enumerate() {
            let variant = if index == 0 {
                ShadcnButtonVariant::Secondary
            } else {
                ShadcnButtonVariant::Ghost
            };
            host.register_button(
                SIDEBAR_BUTTON_IDS[index],
                SECTION_BUTTON_LABELS[index],
                variant,
                ShadcnButtonSize::Sm,
            )?;
        }
        for (id, label, variant, size) in ACTION_BUTTONS {
            host.register_button(id, label, variant, size)?;
        }

        let mut toggle_host = ProtoToggleHost::new()?;
        toggle_host.register(
            PERFORMANCE_PREVIEW_TOGGLE_ID,
            "PERFORMANCE PREVIEW",
            performance_preview_props(false),
        )?;
        Ok((host, toggle_host))
    }

    fn unavailable_error(&self) -> BridgeError {
        BridgeError::Runtime {
            detail: self
                .error
                .clone()
                .unwrap_or_else(|| "Proto UI host is unavailable".to_owned()),
        }
    }

    fn button(&self, id: &str) -> Option<&ProtoButtonState> {
        self.host.as_ref()?.button(id)
    }

    fn toggle(&self, id: &str) -> std::result::Result<ProtoToggleSnapshot, BridgeError> {
        self.toggle_host
            .as_ref()
            .ok_or_else(|| self.unavailable_error())?
            .snapshot(id)
    }

    fn dispatch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) -> std::result::Result<DispatchOutcome, BridgeError> {
        let unavailable = self.unavailable_error();
        self.host
            .as_mut()
            .ok_or(unavailable)?
            .dispatch(id, kind, source, detail)
    }

    fn dispatch_toggle(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) -> std::result::Result<ToggleDispatchOutcome, BridgeError> {
        let unavailable = self.unavailable_error();
        self.toggle_host
            .as_mut()
            .ok_or(unavailable)?
            .dispatch(id, kind, source, detail)
    }

    fn set_toggle_active(
        &mut self,
        id: &str,
        active: bool,
    ) -> std::result::Result<(), BridgeError> {
        let unavailable = self.unavailable_error();
        self.toggle_host
            .as_mut()
            .ok_or(unavailable)?
            .set_props(id, performance_preview_props(active))?;
        Ok(())
    }

    fn set_variant(
        &mut self,
        id: &str,
        variant: ShadcnButtonVariant,
    ) -> std::result::Result<DispatchOutcome, BridgeError> {
        let unavailable = self.unavailable_error();
        self.host
            .as_mut()
            .ok_or(unavailable)?
            .set_variant(id, variant)
    }
}

const DAEMON_STATUS_COMMAND: &[&str] = &["daemon", "status"];
const PERFORMANCE_DRY_RUN_COMMAND: &[&str] = &["--dry-run", "perf", "mode", "performance"];
const BATTERY_STATUS_COMMAND: &[&str] = &["battery", "status"];
const THERMAL_SENSORS_COMMAND: &[&str] = &["perf", "temp"];
const POWER_SCHEMES_COMMAND: &[&str] = &["power", "scheme", "list"];
const DIAGNOSTICS_COMMAND: &[&str] = &["scan", "list"];
const MAGICBAY_COMMAND: &[&str] = &["magicbay", "detect"];

const SECTION_NAMES: [(&str, &str); 7] = [
    ("01", "OVERVIEW"),
    ("02", "POWER"),
    ("03", "PERFORMANCE"),
    ("04", "DEVICES"),
    ("05", "BIOS"),
    ("06", "TUNING"),
    ("07", "DIAGNOSTICS"),
];

const SECTION_BUTTON_LABELS: [&str; 7] = [
    "01  OVERVIEW",
    "02  POWER",
    "03  PERFORMANCE",
    "04  DEVICES",
    "05  BIOS",
    "06  TUNING",
    "07  DIAGNOSTICS",
];
fn action_button(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    id: &'static str,
    label: &'static str,
    action: DashboardAction,
) -> Stateful<Div> {
    let Some(state) = dashboard.proto.button(id) else {
        return unavailable_button(id, label);
    };

    let dashboard_entity = cx.entity().downgrade();
    proto_surface::button_element(id, label, state, move |_, _, cx| {
        dashboard_entity
            .update(cx, |this, cx| {
                this.handle_accessible_proto_click(id, action);
                cx.notify();
            })
            .ok();
    })
    .on_hover(cx.listener(move |this, hovered, _, cx| {
        let kind = if *hovered {
            InputKind::PointerEnter
        } else {
            InputKind::PointerLeave
        };
        this.dispatch_proto(id, kind, InputSource::Mouse);
        cx.notify();
    }))
    .on_mouse_down(
        gpui::MouseButton::Left,
        cx.listener(move |this, _, _, cx| {
            this.dispatch_proto(id, InputKind::Focus, InputSource::Mouse);
            this.dispatch_proto(id, InputKind::PointerDown, InputSource::Mouse);
            cx.notify();
        }),
    )
    .on_mouse_up(
        gpui::MouseButton::Left,
        cx.listener(move |this, _, _, cx| {
            this.dispatch_proto(id, InputKind::PointerUp, InputSource::Mouse);
            cx.notify();
        }),
    )
    .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, _, cx| {
        this.dispatch_proto(id, InputKind::Focus, InputSource::Keyboard);
        this.dispatch_proto_with_detail(
            id,
            InputKind::KeyDown,
            InputSource::Keyboard,
            Some(serde_json::json!({ "key": event.keystroke.key.clone() })),
        );
        cx.notify();
    }))
    .on_key_up(cx.listener(move |this, event: &gpui::KeyUpEvent, _, cx| {
        this.dispatch_proto_with_detail(
            id,
            InputKind::KeyUp,
            InputSource::Keyboard,
            Some(serde_json::json!({ "key": event.keystroke.key.clone() })),
        );
        cx.notify();
    }))
    .on_click(cx.listener(move |this, event, _, cx| {
        let source = match event {
            gpui::ClickEvent::Mouse(_) => InputSource::Mouse,
            gpui::ClickEvent::Keyboard(_) => InputSource::Keyboard,
            gpui::ClickEvent::Touch(_) => InputSource::Touch,
        };
        this.handle_proto_click(id, action, source);
        cx.notify();
    }))
}

fn toggle_action(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    id: &'static str,
    label: &'static str,
    action: DashboardAction,
) -> Stateful<Div> {
    let Ok(state) = dashboard.proto.toggle(id) else {
        return unavailable_button(id, label);
    };
    let dashboard_entity = cx.entity().downgrade();
    proto_surface::toggle_element(id, label, &state, move |_, _, cx| {
        dashboard_entity
            .update(cx, |this, cx| {
                this.handle_accessible_proto_toggle(id, action);
                cx.notify();
            })
            .ok();
    })
    .on_hover(cx.listener(move |this, hovered, _, cx| {
        let kind = if *hovered {
            InputKind::PointerEnter
        } else {
            InputKind::PointerLeave
        };
        this.dispatch_proto_toggle(id, kind, InputSource::Mouse, None);
        cx.notify();
    }))
    .on_mouse_down(
        gpui::MouseButton::Left,
        cx.listener(move |this, _, _, cx| {
            this.dispatch_proto_toggle(id, InputKind::Focus, InputSource::Mouse, None);
            this.dispatch_proto_toggle(id, InputKind::PointerDown, InputSource::Mouse, None);
            cx.notify();
        }),
    )
    .on_mouse_up(
        gpui::MouseButton::Left,
        cx.listener(move |this, _, _, cx| {
            this.dispatch_proto_toggle(id, InputKind::PointerUp, InputSource::Mouse, None);
            cx.notify();
        }),
    )
    .on_key_down(cx.listener(move |this, event: &gpui::KeyDownEvent, _, cx| {
        this.dispatch_proto_toggle(id, InputKind::Focus, InputSource::Keyboard, None);
        this.dispatch_proto_toggle(
            id,
            InputKind::KeyDown,
            InputSource::Keyboard,
            Some(serde_json::json!({ "key": event.keystroke.key.clone() })),
        );
        cx.notify();
    }))
    .on_key_up(cx.listener(move |this, event: &gpui::KeyUpEvent, _, cx| {
        this.dispatch_proto_toggle(
            id,
            InputKind::KeyUp,
            InputSource::Keyboard,
            Some(serde_json::json!({ "key": event.keystroke.key.clone() })),
        );
        cx.notify();
    }))
    .on_click(cx.listener(move |this, event, _, cx| {
        let source = match event {
            gpui::ClickEvent::Mouse(_) => InputSource::Mouse,
            gpui::ClickEvent::Keyboard(_) => InputSource::Keyboard,
            gpui::ClickEvent::Touch(_) => InputSource::Touch,
        };
        this.handle_proto_toggle(id, action, source);
        cx.notify();
    }))
}

fn unavailable_button(id: &'static str, label: &'static str) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .px_3()
        .py_2()
        .text_xs()
        .text_color(rgb(UNAVAILABLE))
        .child(label)
}
fn action_bar(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> impl IntoElement {
    div().flex().flex_row().flex_wrap().gap_2().children([
        action_button(
            dashboard,
            cx,
            "refresh-status",
            "REFRESH STATUS",
            DashboardAction::Refresh,
        ),
        action_button(
            dashboard,
            cx,
            "daemon-status",
            "DAEMON STATUS",
            DashboardAction::RunCommand(DAEMON_STATUS_COMMAND),
        ),
        action_button(
            dashboard,
            cx,
            "battery-status",
            "BATTERY STATUS",
            DashboardAction::RunCommand(BATTERY_STATUS_COMMAND),
        ),
        action_button(
            dashboard,
            cx,
            "thermal-sensors",
            "THERMAL SENSORS",
            DashboardAction::RunCommand(THERMAL_SENSORS_COMMAND),
        ),
        action_button(
            dashboard,
            cx,
            "power-schemes",
            "POWER SCHEMES",
            DashboardAction::RunCommand(POWER_SCHEMES_COMMAND),
        ),
        action_button(
            dashboard,
            cx,
            "diagnostics",
            "DIAGNOSTICS",
            DashboardAction::RunCommand(DIAGNOSTICS_COMMAND),
        ),
        action_button(
            dashboard,
            cx,
            "magicbay-detect",
            "MAGICBAY DETECT",
            DashboardAction::RunCommand(MAGICBAY_COMMAND),
        ),
        toggle_action(
            dashboard,
            cx,
            PERFORMANCE_PREVIEW_TOGGLE_ID,
            "PERFORMANCE PREVIEW",
            DashboardAction::RunCommand(PERFORMANCE_DRY_RUN_COMMAND),
        ),
    ])
}

/// Open the Sailbreak dashboard with a read-only snapshot.
///
/// A real Wayland or X11 session is required on Linux. Calling this from SSH
/// or another headless session returns a normal channel error rather than
/// crashing after `open_window` fails.
pub fn run(snapshot: DashboardSnapshot) -> Result<()> {
    let controller = Arc::new(StaticController {
        snapshot: snapshot.clone(),
    });
    run_with_controller(snapshot, controller)
}

/// Open the dashboard with a controller that executes the shared CLI surface.
pub fn run_with_controller(
    snapshot: DashboardSnapshot,
    controller: Arc<dyn GuiController>,
) -> Result<()> {
    if cfg!(target_os = "linux")
        && !desktop_session_available(
            env::var_os("DISPLAY").as_deref(),
            env::var_os("WAYLAND_DISPLAY").as_deref(),
        )
    {
        return Err(LctrlError::ChannelUnavailable {
            channel: "desktop display (DISPLAY or WAYLAND_DISPLAY)".into(),
        });
    }

    let failure = Rc::new(RefCell::new(None::<String>));
    let reported_failure = Rc::clone(&failure);
    gpui_platform::application().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT)), cx);
        if let Err(error) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            move |_, cx| cx.new(|_| Dashboard::with_controller(snapshot, controller)),
        ) {
            *reported_failure.borrow_mut() = Some(error.to_string());
            cx.quit();
        } else {
            cx.activate(true);
        }
    });
    if let Some(error) = failure.borrow_mut().take() {
        Err(LctrlError::ChannelUnavailable {
            channel: format!("desktop window: {error}"),
        })
    } else {
        Ok(())
    }
}

fn desktop_session_available(
    display: Option<&std::ffi::OsStr>,
    wayland: Option<&std::ffi::OsStr>,
) -> bool {
    [display, wayland]
        .into_iter()
        .flatten()
        .any(|value| !value.is_empty())
}

/// GPUI view for the technical control-center dashboard.
pub struct Dashboard {
    snapshot: DashboardSnapshot,
    controller: Arc<dyn GuiController>,
    active_section: usize,
    proto: ProtoUiState,
}

impl Dashboard {
    #[must_use]
    pub fn new(snapshot: DashboardSnapshot) -> Self {
        let controller = Arc::new(StaticController {
            snapshot: snapshot.clone(),
        });
        Self::with_controller(snapshot, controller)
    }

    fn with_controller(snapshot: DashboardSnapshot, controller: Arc<dyn GuiController>) -> Self {
        let proto = ProtoUiState::new();
        let snapshot = match &proto.error {
            Some(error) => DashboardSnapshot {
                status_message: format!("Proto UI host unavailable: {error}"),
                ..snapshot
            },
            None => snapshot,
        };
        Self {
            snapshot,
            controller,
            active_section: 0,
            proto,
        }
    }

    fn apply_action(&mut self, action: DashboardAction) {
        match action {
            DashboardAction::SelectSection(index) => {
                let next_section = index.min(SECTION_NAMES.len() - 1);
                let previous_section = self.active_section;
                self.active_section = next_section;
                let mut variant_error = None;
                if previous_section != next_section {
                    if let Err(error) = self.proto.set_variant(
                        SIDEBAR_BUTTON_IDS[previous_section],
                        ShadcnButtonVariant::Ghost,
                    ) {
                        variant_error = Some(error.to_string());
                    }
                    if let Err(error) = self.proto.set_variant(
                        SIDEBAR_BUTTON_IDS[next_section],
                        ShadcnButtonVariant::Secondary,
                    ) {
                        variant_error = Some(error.to_string());
                    }
                }
                self.snapshot.status_message = match variant_error {
                    Some(error) => format!("Proto UI action failed: {error}"),
                    None => format!("Section {} selected", SECTION_NAMES[self.active_section].1),
                };
            }
            DashboardAction::Refresh => match self.controller.refresh() {
                Ok(mut snapshot) => {
                    snapshot.status_message = "Status refreshed".into();
                    self.snapshot = snapshot;
                }
                Err(error) => {
                    self.snapshot.status_message = format!("Refresh failed: {error}");
                }
            },
            DashboardAction::RunCommand(args) => match self.controller.execute(args) {
                Ok(message) => {
                    self.snapshot.status_message = message.trim().to_owned();
                }
                Err(error) => {
                    self.snapshot.status_message = format!("Action failed: {error}");
                }
            },
        }
    }

    fn dispatch_proto(&mut self, id: &str, kind: InputKind, source: InputSource) -> bool {
        self.dispatch_proto_with_detail(id, kind, source, None)
    }

    fn dispatch_proto_with_detail(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) -> bool {
        match self.proto.dispatch(id, kind, source, detail) {
            Ok(outcome) => {
                if let Some(diagnostic) = outcome.diagnostics.last() {
                    self.snapshot.status_message = format!("Proto UI: {}", diagnostic.detail);
                }
                outcome.click_emitted
            }
            Err(error) => {
                self.snapshot.status_message = format!("Proto UI action failed: {error}");
                false
            }
        }
    }

    fn dispatch_proto_toggle(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) -> bool {
        match self.proto.dispatch_toggle(id, kind, source, detail) {
            Ok(outcome) => {
                if let Some(diagnostic) = outcome.diagnostics.last() {
                    self.snapshot.status_message = format!("Proto UI: {}", diagnostic.detail);
                }
                outcome.active_change_count == 1
            }
            Err(error) => {
                self.snapshot.status_message = format!("Proto UI action failed: {error}");
                false
            }
        }
    }

    fn handle_proto_click(&mut self, id: &str, action: DashboardAction, source: InputSource) {
        if self.dispatch_proto(id, InputKind::PressCommit, source) {
            self.apply_action(action);
        }
    }

    fn handle_accessible_proto_click(&mut self, id: &str, action: DashboardAction) {
        self.handle_proto_click(id, action, InputSource::Accessibility);
    }

    fn handle_proto_toggle(&mut self, id: &str, action: DashboardAction, source: InputSource) {
        let Ok(current) = self.proto.toggle(id) else {
            self.snapshot.status_message = "Proto UI toggle is unavailable".to_owned();
            return;
        };
        let next_active = !current.active;
        if !self.dispatch_proto_toggle(id, InputKind::PressCommit, source, None) {
            return;
        }

        let command_succeeded = if next_active {
            match action {
                DashboardAction::RunCommand(args) => match self.controller.execute(args) {
                    Ok(message) => {
                        self.snapshot.status_message = message.trim().to_owned();
                        true
                    }
                    Err(error) => {
                        self.snapshot.status_message = format!("Action failed: {error}");
                        false
                    }
                },
                other => {
                    self.apply_action(other);
                    true
                }
            }
        } else {
            self.snapshot.status_message = "Performance preview cleared".to_owned();
            true
        };

        if command_succeeded && let Err(error) = self.proto.set_toggle_active(id, next_active) {
            self.snapshot.status_message = format!("Proto UI action failed: {error}");
        }
    }

    fn handle_accessible_proto_toggle(&mut self, id: &str, action: DashboardAction) {
        self.handle_proto_toggle(id, action, InputSource::Accessibility);
    }
}

impl Render for Dashboard {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = &self.snapshot;
        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(rgb(INK))
            .text_color(rgb(TEXT))
            .font_family("Iosevka, IBM Plex Mono, monospace")
            .child(sidebar(self, cx))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .p_6()
                    .gap_5()
                    .child(identity_header(snapshot))
                    .child(safety_banner(&snapshot.status_message))
                    .child(action_bar(self, cx))
                    .child(capability_matrix(&snapshot.capabilities))
                    .child(telemetry_panel(snapshot)),
            )
    }
}

fn sidebar(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> impl IntoElement {
    let session_label = if dashboard.proto.error.is_some() {
        "●  PROTO UI UNAVAILABLE"
    } else {
        "●  CLICK-READY"
    };
    let session_color = if dashboard.proto.error.is_some() {
        UNAVAILABLE
    } else {
        CAUTION
    };
    let sections = SIDEBAR_BUTTON_IDS
        .into_iter()
        .enumerate()
        .map(|(index, id)| {
            action_button(
                dashboard,
                cx,
                id,
                SECTION_BUTTON_LABELS[index],
                DashboardAction::SelectSection(index),
            )
            .w_full()
            .justify_start()
            .gap_3()
        });

    div()
        .w(px(SIDEBAR_WIDTH))
        .flex_none()
        .flex()
        .flex_col()
        .bg(rgb(SURFACE))
        .border_r_1()
        .border_color(rgb(RULE))
        .p_5()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .pb_6()
                .border_b_1()
                .border_color(rgb(RULE))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(SIGNAL))
                        .child("SAILBREAK / CONTROL"),
                )
                .child(
                    div()
                        .text_xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("CENTER"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child("INTERACTIVE CONSOLE 0.1.1"),
                ),
        )
        .child(div().flex().flex_col().gap_2().pt_5().children(sections))
        .child(
            div()
                .mt_6()
                .pt_5()
                .border_t_1()
                .border_color(rgb(RULE))
                .flex()
                .flex_col()
                .gap_2()
                .child(div().text_xs().text_color(rgb(MUTED)).child("SESSION"))
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(session_color))
                        .child(session_label),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(UNAVAILABLE))
                        .child("Writes remain guarded by\nCLI safety semantics."),
                ),
        )
}

fn identity_header(snapshot: &DashboardSnapshot) -> impl IntoElement {
    let product = snapshot
        .hardware
        .product_name
        .as_deref()
        .unwrap_or("Hardware identity pending");
    let family = snapshot
        .hardware
        .family
        .as_deref()
        .unwrap_or("Unknown family");
    let bios = snapshot
        .hardware
        .bios_version
        .as_deref()
        .unwrap_or("BIOS not reported");
    let platform = platform_name(snapshot.platform);

    div()
        .flex()
        .flex_row()
        .justify_between()
        .items_end()
        .pb_4()
        .border_b_1()
        .border_color(rgb(RULE))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(SIGNAL))
                        .child("SYSTEM / OVERVIEW"),
                )
                .child(
                    div()
                        .text_2xl()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child(product.to_string()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(MUTED))
                        .child(format!("{family}  ·  {platform}")),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .items_end()
                .gap_1()
                .text_xs()
                .child(div().text_color(rgb(MUTED)).child("FIRMWARE"))
                .child(div().text_color(rgb(TEXT)).child(bios.to_string())),
        )
}

fn safety_banner(status_message: &str) -> impl IntoElement {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .py_3()
        .bg(rgb(SURFACE_RAISED))
        .border_1()
        .border_color(rgb(CAUTION))
        .child(div().w(px(MARKER_SIZE)).h(px(MARKER_SIZE)).bg(rgb(CAUTION)))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(CAUTION))
                        .child("SAFETY GATE / NO MUTATIONS"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(TEXT))
                        .child(status_message.to_string()),
                ),
        )
}

fn capability_matrix(capabilities: &CapabilitySet) -> impl IntoElement {
    let rows = capabilities.features.iter().map(|(feature, capability)| {
        let (label, color) = availability_presentation(capability.availability);
        let detail = capability
            .detail
            .as_deref()
            .unwrap_or("No additional detail reported");
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_3()
            .px_3()
            .py_3()
            .border_b_1()
            .border_color(rgb(RULE))
            .child(div().w(px(MARKER_SIZE)).h(px(MARKER_SIZE)).bg(color))
            .child(
                div()
                    .w(px(FEATURE_COLUMN_WIDTH))
                    .flex_none()
                    .text_sm()
                    .child(feature.clone()),
            )
            .child(
                div()
                    .w(px(AVAILABILITY_COLUMN_WIDTH))
                    .flex_none()
                    .text_xs()
                    .text_color(color)
                    .child(label),
            )
            .child(
                div()
                    .flex_1()
                    .text_xs()
                    .text_color(rgb(MUTED))
                    .child(detail.to_string()),
            )
    });

    div()
        .flex()
        .flex_col()
        .min_h_0()
        .overflow_hidden()
        .border_1()
        .border_color(rgb(RULE))
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .items_center()
                .px_4()
                .py_3()
                .bg(rgb(SURFACE_RAISED))
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::BOLD)
                        .child("CAPABILITY MATRIX"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(format!("{} SIGNALS", capabilities.features.len())),
                ),
        )
        .child(div().flex().flex_col().children(rows).when(
            capabilities.features.is_empty(),
            |this| {
                this.child(
                    div()
                        .px_3()
                        .py_4()
                        .text_sm()
                        .text_color(rgb(UNAVAILABLE))
                        .child("No capability claims available; awaiting a platform HAL."),
                )
            },
        ))
}

fn telemetry_panel(snapshot: &DashboardSnapshot) -> impl IntoElement {
    let availability_count = snapshot
        .capabilities
        .features
        .values()
        .filter(|capability| capability.availability == Availability::Available)
        .count();
    let limited_count = snapshot
        .capabilities
        .features
        .values()
        .filter(|capability| capability.availability == Availability::Limited)
        .count();
    let unavailable_count = snapshot
        .capabilities
        .features
        .values()
        .filter(|capability| capability.availability == Availability::Unavailable)
        .count();

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(div().text_xs().text_color(rgb(SIGNAL)).child("TELEMETRY / READ PATH"))
        .child(
            div()
                .flex()
                .flex_row()
                .gap_3()
                .children([
                    metric("AVAILABLE", availability_count, SIGNAL),
                    metric("LIMITED", limited_count, CAUTION),
                    metric("UNAVAILABLE", unavailable_count, UNAVAILABLE),
                ]),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child("This shell never presents a disabled operation as executed. Hardware writes require an explicit controller dispatcher."),
        )
}

fn metric(label: &str, value: usize, color: u32) -> impl IntoElement {
    div()
        .flex_1()
        .flex()
        .flex_col()
        .gap_1()
        .p_4()
        .bg(rgb(SURFACE))
        .border_1()
        .border_color(rgb(RULE))
        .child(
            div()
                .text_2xl()
                .font_weight(gpui::FontWeight::BOLD)
                .text_color(rgb(color))
                .child(value.to_string()),
        )
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child(label.to_string()),
        )
}

fn platform_name(platform: Platform) -> &'static str {
    match platform {
        Platform::Windows => "WINDOWS",
        Platform::Linux => "LINUX",
    }
}

fn availability_presentation(availability: Availability) -> (&'static str, gpui::Hsla) {
    match availability {
        Availability::Available => ("AVAILABLE", rgb(SIGNAL).into()),
        Availability::Limited => ("LIMITED", rgb(CAUTION).into()),
        Availability::Unavailable => ("UNAVAILABLE", rgb(UNAVAILABLE).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lctrl_core::{CapabilitySet, HardwareInfo, LctrlError};

    struct ReadOnlyHal {
        capabilities: CapabilitySet,
    }

    impl Hal for ReadOnlyHal {
        fn platform(&self) -> Platform {
            self.capabilities.platform
        }

        fn hardware_info(&self) -> Result<HardwareInfo> {
            Ok(HardwareInfo {
                product_name: Some("Test Control Center".into()),
                family: Some("Test Family".into()),
                bios_version: Some("0.1".into()),
            })
        }

        fn capabilities(&self) -> Result<CapabilitySet> {
            Ok(self.capabilities.clone())
        }
    }

    #[test]
    fn snapshot_preserves_unavailable_capability_detail() {
        let mut capabilities = CapabilitySet::new(Platform::Linux);
        capabilities
            .record(
                "battery.threshold",
                Availability::Unavailable,
                Some("arbitrary percentage writes are unavailable".into()),
            )
            .expect("valid capability id");
        let snapshot =
            DashboardSnapshot::from_hal(&ReadOnlyHal { capabilities }).expect("read succeeds");
        let capability = snapshot
            .capabilities
            .get("battery.threshold")
            .expect("capability is retained");

        assert_eq!(capability.availability, Availability::Unavailable);
        assert_eq!(
            capability.detail.as_deref(),
            Some("arbitrary percentage writes are unavailable")
        );
        assert_ne!(capability.availability, Availability::Available);
    }

    #[test]
    fn from_hal_keeps_core_errors_strict() {
        struct FailingHal;
        impl Hal for FailingHal {
            fn platform(&self) -> Platform {
                Platform::Linux
            }
            fn hardware_info(&self) -> Result<HardwareInfo> {
                Err(LctrlError::Unsupported {
                    feature: "hardware identity".into(),
                })
            }
            fn capabilities(&self) -> Result<CapabilitySet> {
                panic!("capabilities must not be queried after hardware failure")
            }
        }

        let error =
            DashboardSnapshot::from_hal(&FailingHal).expect_err("core error must propagate");
        assert!(
            matches!(error, LctrlError::Unsupported { feature } if feature == "hardware identity")
        );
    }

    #[test]
    fn desktop_session_requires_nonempty_display_or_wayland_socket() {
        use std::ffi::OsStr;

        assert!(!desktop_session_available(None, None));
        assert!(!desktop_session_available(Some(OsStr::new("")), None));
        assert!(desktop_session_available(Some(OsStr::new(":0")), None));
        assert!(desktop_session_available(
            None,
            Some(OsStr::new("wayland-0"))
        ));
    }

    #[test]
    fn dashboard_navigation_action_updates_active_section() {
        let snapshot = DashboardSnapshot::unavailable(Platform::Linux, "test");
        let mut dashboard = Dashboard::new(snapshot);

        dashboard.apply_action(DashboardAction::SelectSection(2));

        assert_eq!(dashboard.active_section, 2);
        assert_eq!(
            dashboard.snapshot.status_message,
            "Section PERFORMANCE selected"
        );
    }
    #[test]
    fn run_command_action_surfaces_controller_output() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct Recorder {
            calls: AtomicUsize,
        }

        impl GuiController for Recorder {
            fn refresh(&self) -> Result<DashboardSnapshot> {
                Ok(DashboardSnapshot::unavailable(Platform::Linux, "refreshed"))
            }

            fn execute(&self, args: &[&str]) -> Result<String> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(format!("executed {}", args.join(" ")))
            }
        }

        let controller = Arc::new(Recorder {
            calls: AtomicUsize::new(0),
        });
        let mut dashboard = Dashboard::with_controller(
            DashboardSnapshot::unavailable(Platform::Linux, "initial"),
            controller.clone(),
        );

        dashboard.apply_action(DashboardAction::RunCommand(&["battery", "status"]));

        assert_eq!(dashboard.snapshot.status_message, "executed battery status");
    }

    #[test]
    fn accessibility_click_uses_the_single_proto_activation_gateway() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct Recorder {
            calls: AtomicUsize,
        }

        impl GuiController for Recorder {
            fn refresh(&self) -> Result<DashboardSnapshot> {
                Ok(DashboardSnapshot::unavailable(Platform::Linux, "refreshed"))
            }

            fn execute(&self, args: &[&str]) -> Result<String> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(format!("executed {}", args.join(" ")))
            }
        }

        let controller = Arc::new(Recorder {
            calls: AtomicUsize::new(0),
        });
        let mut dashboard = Dashboard::with_controller(
            DashboardSnapshot::unavailable(Platform::Linux, "initial"),
            controller.clone(),
        );

        dashboard.handle_accessible_proto_click(
            "battery-status",
            DashboardAction::RunCommand(BATTERY_STATUS_COMMAND),
        );

        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            dashboard
                .proto
                .button("battery-status")
                .unwrap()
                .click_count,
            1
        );
    }

    #[test]
    fn performance_preview_toggle_uses_runtime_state_and_cli_gateway() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct Recorder {
            calls: AtomicUsize,
        }

        impl GuiController for Recorder {
            fn refresh(&self) -> Result<DashboardSnapshot> {
                Ok(DashboardSnapshot::unavailable(Platform::Linux, "refreshed"))
            }

            fn execute(&self, args: &[&str]) -> Result<String> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(format!("executed {}", args.join(" ")))
            }
        }

        let controller = Arc::new(Recorder {
            calls: AtomicUsize::new(0),
        });
        let mut dashboard = Dashboard::with_controller(
            DashboardSnapshot::unavailable(Platform::Linux, "initial"),
            controller.clone(),
        );

        assert!(
            !dashboard
                .proto
                .toggle(PERFORMANCE_PREVIEW_TOGGLE_ID)
                .unwrap()
                .active
        );
        dashboard.handle_accessible_proto_toggle(
            PERFORMANCE_PREVIEW_TOGGLE_ID,
            DashboardAction::RunCommand(PERFORMANCE_DRY_RUN_COMMAND),
        );
        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
        assert!(
            dashboard
                .proto
                .toggle(PERFORMANCE_PREVIEW_TOGGLE_ID)
                .unwrap()
                .active
        );

        dashboard.handle_accessible_proto_toggle(
            PERFORMANCE_PREVIEW_TOGGLE_ID,
            DashboardAction::RunCommand(PERFORMANCE_DRY_RUN_COMMAND),
        );
        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
        assert!(
            !dashboard
                .proto
                .toggle(PERFORMANCE_PREVIEW_TOGGLE_ID)
                .unwrap()
                .active
        );
    }
}
