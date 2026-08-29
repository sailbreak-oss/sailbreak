//! A read-only GPUI control-center shell for Sailbreak.
//!
//! The GUI deliberately has no hardware mutation path. It renders the state
//! reported by [`lctrl_hal::Hal`] and marks every capability according to the
//! core availability value, including its explanation.

use std::{cell::RefCell, env, rc::Rc, sync::Arc};

use gpui::{
    App, Application, Bounds, Context, Div, IntoElement, Render, Stateful, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use lctrl_core::{Availability, CapabilitySet, HardwareInfo, LctrlError, Platform, Result};
use lctrl_hal::Hal;

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
fn action_button(
    cx: &mut Context<Dashboard>,
    id: &'static str,
    label: &'static str,
    action: DashboardAction,
    color: u32,
) -> Stateful<Div> {
    div()
        .id(id)
        .debug_selector(move || id.to_owned())
        .cursor_pointer()
        .focusable()
        .hover(|this| this.bg(rgb(SURFACE_RAISED)))
        .active(|this| this.bg(rgb(RULE)))
        .border_1()
        .border_color(rgb(color))
        .px_3()
        .py_2()
        .text_xs()
        .text_color(rgb(color))
        .on_click(cx.listener(move |this, _, _, cx| {
            this.apply_action(action);
            cx.notify();
        }))
        .child(label)
}

fn action_bar(cx: &mut Context<Dashboard>) -> impl IntoElement {
    div().flex().flex_row().flex_wrap().gap_2().children([
        action_button(
            cx,
            "refresh-status",
            "REFRESH STATUS",
            DashboardAction::Refresh,
            SIGNAL,
        ),
        action_button(
            cx,
            "daemon-status",
            "DAEMON STATUS",
            DashboardAction::RunCommand(DAEMON_STATUS_COMMAND),
            CAUTION,
        ),
        action_button(
            cx,
            "battery-status",
            "BATTERY STATUS",
            DashboardAction::RunCommand(BATTERY_STATUS_COMMAND),
            SIGNAL,
        ),
        action_button(
            cx,
            "thermal-sensors",
            "THERMAL SENSORS",
            DashboardAction::RunCommand(THERMAL_SENSORS_COMMAND),
            SIGNAL,
        ),
        action_button(
            cx,
            "power-schemes",
            "POWER SCHEMES",
            DashboardAction::RunCommand(POWER_SCHEMES_COMMAND),
            SIGNAL,
        ),
        action_button(
            cx,
            "diagnostics",
            "DIAGNOSTICS",
            DashboardAction::RunCommand(DIAGNOSTICS_COMMAND),
            CAUTION,
        ),
        action_button(
            cx,
            "magicbay-detect",
            "MAGICBAY DETECT",
            DashboardAction::RunCommand(MAGICBAY_COMMAND),
            CAUTION,
        ),
        action_button(
            cx,
            "dry-run-performance",
            "DRY-RUN PERFORMANCE",
            DashboardAction::RunCommand(PERFORMANCE_DRY_RUN_COMMAND),
            MUTED,
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
    Application::new().run(move |cx: &mut App| {
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
        Self {
            snapshot,
            controller,
            active_section: 0,
        }
    }

    fn apply_action(&mut self, action: DashboardAction) {
        match action {
            DashboardAction::SelectSection(index) => {
                self.active_section = index.min(SECTION_NAMES.len() - 1);
                self.snapshot.status_message =
                    format!("Section {} selected", SECTION_NAMES[self.active_section].1);
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
            .child(sidebar(self.active_section, cx))
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
                    .child(action_bar(cx))
                    .child(capability_matrix(&snapshot.capabilities))
                    .child(telemetry_panel(snapshot)),
            )
    }
}

fn sidebar(active_section: usize, cx: &mut Context<Dashboard>) -> impl IntoElement {
    div()
        .w(px(238.0))
        .flex_none()
        .flex()
        .flex_col()
        .bg(rgb(SURFACE))
        .border_r_1()
        .w(px(SIDEBAR_WIDTH))
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
                        .child("INTERACTIVE CONSOLE 0.1"),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .pt_5()
                .children(
                    SECTION_NAMES
                        .into_iter()
                        .enumerate()
                        .map(|(index, (number, label))| {
                            let active = index == active_section;
                            div()
                                .id(("sidebar-section", index))
                                .debug_selector(|| format!("sidebar-section-{index}"))
                                .cursor_pointer()
                                .focusable()
                                .hover(|this| this.bg(rgb(SURFACE_RAISED)))
                                .active(|this| this.bg(rgb(RULE)))
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_3()
                                .px_3()
                                .py_2()
                                .bg(if active {
                                    rgb(SURFACE_RAISED)
                                } else {
                                    rgb(SURFACE)
                                })
                                .border_l_2()
                                .border_color(if active { rgb(SIGNAL) } else { rgb(SURFACE) })
                                .text_color(if active { rgb(SIGNAL) } else { rgb(MUTED) })
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.apply_action(DashboardAction::SelectSection(index));
                                    cx.notify();
                                }))
                                .child(div().text_xs().child(number))
                                .child(div().text_sm().child(label))
                        }),
                )
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
                                .text_color(rgb(CAUTION))
                                .child("●  CLICK-READY"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(UNAVAILABLE))
                                .child("Writes remain guarded by\nCLI safety semantics."),
                        ),
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
}
