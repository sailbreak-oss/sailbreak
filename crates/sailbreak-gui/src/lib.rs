//! A read-only GPUI control-center shell for Sailbreak.
//!
//! The GUI deliberately has no hardware mutation path. It renders the state
//! reported by [`lctrl_hal::Hal`] and marks every capability according to the
//! core availability value, including its explanation.

use std::{cell::RefCell, collections::BTreeMap, env, rc::Rc, sync::Arc};

use gpui::{
    App, Bounds, Context, Div, Entity, FocusHandle, IntoElement, Render, Stateful, Subscription,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use lctrl_core::{Availability, CapabilitySet, HardwareInfo, LctrlError, Platform, Result};
use lctrl_hal::Hal;
use proto_ui_gpui::{
    BridgeError, DispatchOutcome, FocusOperationResult, InputKind, InputSource, ProtoButtonHost,
    ProtoButtonState, ProtoSelectHost, ProtoSelectSnapshot, ProtoSeparatorHost,
    ProtoSeparatorSnapshot, ProtoTabsHost, ProtoTextareaHost, ProtoTextareaSnapshot,
    ProtoToggleHost, ProtoToggleSnapshot, SelectContentProps, SelectDispatchOutcome,
    SelectItemProps, SelectRootProps, SelectTriggerProps, SelectValueProps, SeparatorProps,
    ShadcnButtonSize, ShadcnButtonVariant, TabsActivationMode, TabsContentProps, TabsListProps,
    TabsOrientation, TabsRootProps, TabsSnapshot, TabsTriggerProps, TextareaProps,
    ToggleDispatchOutcome, ToggleProps, ToggleSize, ToggleVariant,
};

mod proto_surface;
pub use proto_surface::{
    AccessibleProjection, overlay_surface_element, project_a11y, select_content_element,
    select_item_element, select_value_element,
};

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
    Refresh,
    RunCommand(&'static [&'static str]),
    SaveProfile,
    ApplyProfile,
}

/// Platform composition root used by the interactive dashboard.
pub trait GuiController: Send + Sync {
    fn refresh(&self) -> Result<DashboardSnapshot>;
    fn execute(&self, args: &[&str]) -> Result<String>;
    fn save_profile(&self, _source: &str) -> Result<String> {
        Err(LctrlError::Unsupported {
            feature: "gui.profile-persistence".into(),
        })
    }
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
const SIDEBAR_CONTENT_IDS: [&str; 7] = [
    "sidebar-content-0",
    "sidebar-content-1",
    "sidebar-content-2",
    "sidebar-content-3",
    "sidebar-content-4",
    "sidebar-content-5",
    "sidebar-content-6",
];
const SECTION_VALUES: [&str; 7] = [
    "overview",
    "power",
    "performance",
    "devices",
    "bios",
    "tuning",
    "diagnostics",
];
const SIDEBAR_TABS_ROOT_ID: &str = "sidebar-tabs-root";
const SIDEBAR_TABS_LIST_ID: &str = "sidebar-tabs-list";

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
const TUNING_SECTION_INDEX: usize = 5;
const TUNING_PROFILE_EDITOR_ID: &str = "tuning-profile-editor";
const TUNING_PROFILE_SAVE_ID: &str = "tuning-profile-save";
const TUNING_PROFILE_APPLY_ID: &str = "tuning-profile-apply";
const TUNING_PROFILE_DRAFT: &str = r#"schema = 1

[profile]
name = "sailbreak-gui"
description = "Sailbreak tuning profile draft"

[goal]
"#;
const MAIN_HEADER_SEPARATOR_ID: &str = "main-header-separator";
const ACTION_SEPARATOR_ID: &str = "action-separator";
const CAPABILITY_SEPARATOR_ID: &str = "capability-separator";
const PERFORMANCE_MODE_SELECT_ROOT_ID: &str = "performance-mode-select";
const PERFORMANCE_MODE_SELECT_TRIGGER_ID: &str = "performance-mode-trigger";
const PERFORMANCE_MODE_SELECT_VALUE_ID: &str = "performance-mode-value";
const PERFORMANCE_MODE_SELECT_CONTENT_ID: &str = "performance-mode-content";
const TUNING_PROFILE_SELECT_ROOT_ID: &str = "tuning-profile-select";
const TUNING_PROFILE_SELECT_TRIGGER_ID: &str = "tuning-profile-trigger";
const TUNING_PROFILE_SELECT_VALUE_ID: &str = "tuning-profile-value";
const TUNING_PROFILE_SELECT_CONTENT_ID: &str = "tuning-profile-content";
const POWER_SCHEME_SELECT_ROOT_ID: &str = "power-scheme-select";
const POWER_SCHEME_SELECT_TRIGGER_ID: &str = "power-scheme-trigger";
const POWER_SCHEME_SELECT_VALUE_ID: &str = "power-scheme-value";
const POWER_SCHEME_SELECT_CONTENT_ID: &str = "power-scheme-content";

fn unavailable_select_root_props() -> SelectRootProps {
    SelectRootProps {
        disabled: true,
        ..SelectRootProps::default()
    }
}

fn unavailable_select_item(value: &str) -> SelectItemProps {
    SelectItemProps {
        value: value.to_owned(),
        text_value: value.to_owned(),
        disabled: true,
        ..SelectItemProps::default()
    }
}

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
    separator_host: Option<ProtoSeparatorHost>,
    textarea_host: Option<ProtoTextareaHost>,
    textarea_snapshot: Option<ProtoTextareaSnapshot>,
    textarea_error: Option<String>,
    tabs_host: Option<ProtoTabsHost>,
    tabs_snapshot: Option<TabsSnapshot>,
    tabs_error: Option<String>,
    performance_select_host: Option<ProtoSelectHost>,
    performance_select_snapshot: Option<ProtoSelectSnapshot>,
    tuning_select_host: Option<ProtoSelectHost>,
    tuning_select_snapshot: Option<ProtoSelectSnapshot>,
    power_select_host: Option<ProtoSelectHost>,
    power_select_snapshot: Option<ProtoSelectSnapshot>,
    select_error: Option<String>,
    error: Option<String>,
}

impl ProtoUiState {
    fn new() -> Self {
        let (host, toggle_host, separator_host) = match Self::build_core() {
            Ok(hosts) => hosts,
            Err(error) => {
                return Self {
                    host: None,
                    toggle_host: None,
                    separator_host: None,
                    textarea_host: None,
                    textarea_snapshot: None,
                    textarea_error: None,
                    tabs_host: None,
                    tabs_snapshot: None,
                    tabs_error: None,
                    performance_select_host: None,
                    performance_select_snapshot: None,
                    tuning_select_host: None,
                    tuning_select_snapshot: None,
                    power_select_host: None,
                    power_select_snapshot: None,
                    select_error: None,
                    error: Some(error.to_string()),
                };
            }
        };
        let (textarea_host, textarea_snapshot, textarea_error) = match Self::build_textarea() {
            Ok((host, snapshot)) => (Some(host), Some(snapshot), None),
            Err(error) => (None, None, Some(error.to_string())),
        };
        let (tabs_host, tabs_snapshot, tabs_error) = match Self::build_tabs() {
            Ok((host, snapshot)) => (Some(host), Some(snapshot), None),
            Err(error) => (None, None, Some(error.to_string())),
        };
        let (performance_select_host, performance_select_snapshot) =
            match Self::build_performance_select() {
                Ok((host, snapshot)) => (Some(host), Some(snapshot)),
                Err(_) => (None, None),
            };
        let (tuning_select_host, tuning_select_snapshot) = match Self::build_tuning_select() {
            Ok((host, snapshot)) => (Some(host), Some(snapshot)),
            Err(_) => (None, None),
        };
        let (power_select_host, power_select_snapshot) = match Self::build_power_select() {
            Ok((host, snapshot)) => (Some(host), Some(snapshot)),
            Err(_) => (None, None),
        };
        Self {
            host: Some(host),
            toggle_host: Some(toggle_host),
            separator_host: Some(separator_host),
            textarea_host,
            textarea_snapshot,
            textarea_error,
            tabs_host,
            tabs_snapshot,
            tabs_error,
            performance_select_host,
            performance_select_snapshot,
            tuning_select_host,
            tuning_select_snapshot,
            power_select_host,
            power_select_snapshot,
            select_error: None,
            error: None,
        }
    }

    fn build_core()
    -> std::result::Result<(ProtoButtonHost, ProtoToggleHost, ProtoSeparatorHost), BridgeError>
    {
        let mut host = ProtoButtonHost::new()?;
        for (id, label, variant, size) in ACTION_BUTTONS {
            host.register_button(id, label, variant, size)?;
        }
        host.register_button(
            TUNING_PROFILE_SAVE_ID,
            "SAVE PROFILE",
            ShadcnButtonVariant::Default,
            ShadcnButtonSize::Sm,
        )?;
        host.register_button(
            TUNING_PROFILE_APPLY_ID,
            "APPLY DRY-RUN",
            ShadcnButtonVariant::Outline,
            ShadcnButtonSize::Sm,
        )?;

        let mut toggle_host = ProtoToggleHost::new()?;
        toggle_host.register(
            PERFORMANCE_PREVIEW_TOGGLE_ID,
            "PERFORMANCE PREVIEW",
            performance_preview_props(false),
        )?;
        let mut separator_host = ProtoSeparatorHost::new()?;
        for id in [
            MAIN_HEADER_SEPARATOR_ID,
            ACTION_SEPARATOR_ID,
            CAPABILITY_SEPARATOR_ID,
        ] {
            separator_host.register(id, SeparatorProps::default())?;
        }
        Ok((host, toggle_host, separator_host))
    }

    fn build_textarea()
    -> std::result::Result<(ProtoTextareaHost, ProtoTextareaSnapshot), BridgeError> {
        let mut textarea_host = ProtoTextareaHost::new()?;
        textarea_host.register(
            TUNING_PROFILE_EDITOR_ID,
            "Tuning profile DSL",
            TextareaProps {
                default_value: TUNING_PROFILE_DRAFT.to_owned(),
                placeholder: "schema-v1 profile TOML".to_owned(),
                rows: 12,
                name: "tuning-profile".to_owned(),
                ..TextareaProps::default()
            },
        )?;
        let textarea_snapshot = textarea_host.snapshot(TUNING_PROFILE_EDITOR_ID)?;
        Ok((textarea_host, textarea_snapshot))
    }

    fn build_tabs() -> std::result::Result<(ProtoTabsHost, TabsSnapshot), BridgeError> {
        let mut host = ProtoTabsHost::new()?;
        host.register_root(
            SIDEBAR_TABS_ROOT_ID,
            "Dashboard sections",
            TabsRootProps {
                default_value: SECTION_VALUES[0].to_owned(),
                orientation: TabsOrientation::Vertical,
                activation_mode: TabsActivationMode::Automatic,
                ..TabsRootProps::default()
            },
        )?;
        host.register_list(
            SIDEBAR_TABS_LIST_ID,
            SIDEBAR_TABS_ROOT_ID,
            TabsListProps {
                loop_navigation: true,
                a11y_label: "Dashboard sections".to_owned(),
                ..TabsListProps::default()
            },
        )?;
        for index in 0..SECTION_VALUES.len() {
            host.register_trigger(
                SIDEBAR_BUTTON_IDS[index],
                SECTION_BUTTON_LABELS[index],
                SIDEBAR_TABS_LIST_ID,
                TabsTriggerProps {
                    value: SECTION_VALUES[index].to_owned(),
                    disabled: false,
                },
            )?;
            host.register_content(
                SIDEBAR_CONTENT_IDS[index],
                SIDEBAR_TABS_ROOT_ID,
                TabsContentProps {
                    value: SECTION_VALUES[index].to_owned(),
                    keep_mounted: false,
                },
            )?;
        }
        host.setup()?;
        let snapshot = host.snapshot()?;
        Ok((host, snapshot))
    }
    fn build_performance_select()
    -> std::result::Result<(ProtoSelectHost, ProtoSelectSnapshot), BridgeError> {
        let mut host = ProtoSelectHost::new()?;
        host.register_root(
            PERFORMANCE_MODE_SELECT_ROOT_ID,
            "Performance mode",
            unavailable_select_root_props(),
        )?;
        host.register_trigger(
            PERFORMANCE_MODE_SELECT_TRIGGER_ID,
            PERFORMANCE_MODE_SELECT_ROOT_ID,
            SelectTriggerProps::default(),
        )?;
        host.register_value(
            PERFORMANCE_MODE_SELECT_VALUE_ID,
            PERFORMANCE_MODE_SELECT_ROOT_ID,
            SelectValueProps {
                placeholder: "Performance mode".to_owned(),
            },
        )?;
        host.register_content(
            PERFORMANCE_MODE_SELECT_CONTENT_ID,
            PERFORMANCE_MODE_SELECT_ROOT_ID,
            SelectContentProps::default(),
        )?;
        for value in ["balanced", "quiet", "performance", "geek"] {
            host.register_item(
                format!("{PERFORMANCE_MODE_SELECT_ROOT_ID}-{value}"),
                value.to_uppercase(),
                PERFORMANCE_MODE_SELECT_CONTENT_ID,
                unavailable_select_item(value),
            )?;
        }
        host.setup()?;
        let snapshot = host.snapshot()?;
        Ok((host, snapshot))
    }

    fn build_tuning_select()
    -> std::result::Result<(ProtoSelectHost, ProtoSelectSnapshot), BridgeError> {
        let mut host = ProtoSelectHost::new()?;
        host.register_root(
            TUNING_PROFILE_SELECT_ROOT_ID,
            "Tuning profile",
            unavailable_select_root_props(),
        )?;
        host.register_trigger(
            TUNING_PROFILE_SELECT_TRIGGER_ID,
            TUNING_PROFILE_SELECT_ROOT_ID,
            SelectTriggerProps::default(),
        )?;
        host.register_value(
            TUNING_PROFILE_SELECT_VALUE_ID,
            TUNING_PROFILE_SELECT_ROOT_ID,
            SelectValueProps {
                placeholder: "Tuning profile".to_owned(),
            },
        )?;
        host.register_content(
            TUNING_PROFILE_SELECT_CONTENT_ID,
            TUNING_PROFILE_SELECT_ROOT_ID,
            SelectContentProps::default(),
        )?;
        for value in ["sailbreak-gui", "balanced", "quiet", "performance"] {
            host.register_item(
                format!("{TUNING_PROFILE_SELECT_ROOT_ID}-{value}"),
                value.to_uppercase(),
                TUNING_PROFILE_SELECT_CONTENT_ID,
                unavailable_select_item(value),
            )?;
        }
        host.setup()?;
        let snapshot = host.snapshot()?;
        Ok((host, snapshot))
    }

    fn build_power_select()
    -> std::result::Result<(ProtoSelectHost, ProtoSelectSnapshot), BridgeError> {
        let mut host = ProtoSelectHost::new()?;
        host.register_root(
            POWER_SCHEME_SELECT_ROOT_ID,
            "Power scheme",
            unavailable_select_root_props(),
        )?;
        host.register_trigger(
            POWER_SCHEME_SELECT_TRIGGER_ID,
            POWER_SCHEME_SELECT_ROOT_ID,
            SelectTriggerProps::default(),
        )?;
        host.register_value(
            POWER_SCHEME_SELECT_VALUE_ID,
            POWER_SCHEME_SELECT_ROOT_ID,
            SelectValueProps {
                placeholder: "Power scheme".to_owned(),
            },
        )?;
        host.register_content(
            POWER_SCHEME_SELECT_CONTENT_ID,
            POWER_SCHEME_SELECT_ROOT_ID,
            SelectContentProps::default(),
        )?;
        for value in ["balanced", "power-saver", "high-performance"] {
            host.register_item(
                format!("{POWER_SCHEME_SELECT_ROOT_ID}-{value}"),
                value.to_uppercase(),
                POWER_SCHEME_SELECT_CONTENT_ID,
                unavailable_select_item(value),
            )?;
        }
        host.setup()?;
        let snapshot = host.snapshot()?;
        Ok((host, snapshot))
    }

    fn unavailable_error(&self) -> BridgeError {
        BridgeError::Runtime {
            detail: self
                .error
                .clone()
                .unwrap_or_else(|| "Proto UI host is unavailable".to_owned()),
        }
    }

    fn textarea_unavailable_error(&self) -> BridgeError {
        BridgeError::Runtime {
            detail: self
                .textarea_error
                .clone()
                .unwrap_or_else(|| "Proto UI textarea host is unavailable".to_owned()),
        }
    }

    fn tabs_unavailable_error(&self) -> BridgeError {
        BridgeError::Runtime {
            detail: self
                .tabs_error
                .clone()
                .unwrap_or_else(|| "Proto UI Tabs host is unavailable".to_owned()),
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

    fn separator(&self, id: &str) -> std::result::Result<ProtoSeparatorSnapshot, BridgeError> {
        self.separator_host
            .as_ref()
            .ok_or_else(|| self.unavailable_error())?
            .snapshot(id)
    }

    fn textarea_snapshot(&self) -> Option<&ProtoTextareaSnapshot> {
        self.textarea_snapshot.as_ref()
    }

    fn tabs_snapshot(&self) -> Option<&TabsSnapshot> {
        self.tabs_snapshot.as_ref()
    }

    fn refresh_tabs(&mut self) -> std::result::Result<(), BridgeError> {
        let unavailable = self.tabs_unavailable_error();
        let host = self.tabs_host.as_mut().ok_or(unavailable)?;
        self.tabs_snapshot = Some(host.snapshot()?);
        Ok(())
    }

    fn selected_section_index(&self) -> usize {
        let selected = self
            .tabs_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.root.as_ref())
            .map(|root| root.value.as_str());
        selected
            .and_then(|value| {
                SECTION_VALUES
                    .iter()
                    .position(|candidate| *candidate == value)
            })
            .unwrap_or(0)
    }

    fn active_tab_id(&self) -> Option<String> {
        self.tabs_snapshot
            .as_ref()?
            .triggers
            .iter()
            .find(|trigger| trigger.focused)
            .map(|trigger| trigger.id.clone())
    }

    fn set_tab_focus_ready(
        &mut self,
        id: &str,
        ready: bool,
    ) -> std::result::Result<(), BridgeError> {
        let unavailable = self.tabs_unavailable_error();
        self.tabs_host
            .as_mut()
            .ok_or(unavailable)?
            .set_focus_ready(id, ready)
    }

    fn focus_tab(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> std::result::Result<FocusOperationResult, BridgeError> {
        let unavailable = self.tabs_unavailable_error();
        let result = self
            .tabs_host
            .as_mut()
            .ok_or(unavailable)?
            .focus_with_source(id, source)?;
        self.refresh_tabs()?;
        Ok(result)
    }

    fn blur_tab(&mut self, id: &str) -> std::result::Result<(), BridgeError> {
        let unavailable = self.tabs_unavailable_error();
        self.tabs_host
            .as_mut()
            .ok_or(unavailable)?
            .blur(id, InputSource::Programmatic)?;
        self.refresh_tabs()
    }

    fn dispatch_tab_key(&mut self, key: &str) -> std::result::Result<(), BridgeError> {
        let unavailable = self.tabs_unavailable_error();
        self.tabs_host
            .as_mut()
            .ok_or(unavailable)?
            .dispatch_key(key)?;
        self.refresh_tabs()
    }

    fn press_tab(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> std::result::Result<bool, BridgeError> {
        let unavailable = self.tabs_unavailable_error();
        let outcome = self
            .tabs_host
            .as_mut()
            .ok_or(unavailable)?
            .press_commit(id, source)?;
        self.refresh_tabs()?;
        Ok(outcome.click_count == 1)
    }

    fn dispatch_tab_input(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
    ) -> std::result::Result<(), BridgeError> {
        let unavailable = self.tabs_unavailable_error();
        self.tabs_host
            .as_mut()
            .ok_or(unavailable)?
            .dispatch_trigger(id, kind, source, None)?;
        self.refresh_tabs()
    }

    fn select_snapshot(&self, root_id: &str) -> Option<&ProtoSelectSnapshot> {
        match root_id {
            PERFORMANCE_MODE_SELECT_ROOT_ID => self.performance_select_snapshot.as_ref(),
            TUNING_PROFILE_SELECT_ROOT_ID => self.tuning_select_snapshot.as_ref(),
            POWER_SCHEME_SELECT_ROOT_ID => self.power_select_snapshot.as_ref(),
            _ => None,
        }
    }

    fn select_host_mut(&mut self, root_id: &str) -> Option<&mut ProtoSelectHost> {
        match root_id {
            PERFORMANCE_MODE_SELECT_ROOT_ID => self.performance_select_host.as_mut(),
            TUNING_PROFILE_SELECT_ROOT_ID => self.tuning_select_host.as_mut(),
            POWER_SCHEME_SELECT_ROOT_ID => self.power_select_host.as_mut(),
            _ => None,
        }
    }

    fn refresh_select(&mut self, root_id: &str) -> std::result::Result<(), BridgeError> {
        let Some(host) = self.select_host_mut(root_id) else {
            return Err(self.select_unavailable_error());
        };
        let snapshot = host.snapshot()?;
        match root_id {
            PERFORMANCE_MODE_SELECT_ROOT_ID => {
                self.performance_select_snapshot = Some(snapshot);
            }
            TUNING_PROFILE_SELECT_ROOT_ID => {
                self.tuning_select_snapshot = Some(snapshot);
            }
            POWER_SCHEME_SELECT_ROOT_ID => {
                self.power_select_snapshot = Some(snapshot);
            }
            _ => {}
        }
        Ok(())
    }

    fn select_unavailable_error(&self) -> BridgeError {
        BridgeError::Runtime {
            detail: self
                .select_error
                .clone()
                .unwrap_or_else(|| "Proto UI Select host is unavailable".to_owned()),
        }
    }

    fn dispatch_select(
        &mut self,
        root_id: &str,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) -> std::result::Result<SelectDispatchOutcome, BridgeError> {
        let Some(host) = self.select_host_mut(root_id) else {
            return Err(self.select_unavailable_error());
        };
        let outcome = host.dispatch(id, kind, source, detail)?;
        self.refresh_select(root_id)?;
        Ok(outcome)
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

    fn dispatch_textarea(
        &mut self,
        event: &proto_surface::ProtoTextareaNativeEvent,
    ) -> std::result::Result<(), BridgeError> {
        let (id, epoch, text, selection) = match event {
            proto_surface::ProtoTextareaNativeEvent::Text {
                id,
                epoch,
                event,
                selection,
            } => (id.as_str(), *epoch, event.clone(), *selection),
        };
        let unavailable = self.textarea_unavailable_error();
        let host = self.textarea_host.as_mut().ok_or(unavailable)?;
        host.dispatch_text_with_selection_at_epoch(id, epoch, text, selection)?;
        self.textarea_snapshot = Some(host.snapshot(id)?);
        Ok(())
    }

    fn dispatch_textarea_change(&mut self, id: &str) -> std::result::Result<(), BridgeError> {
        let unavailable = self.textarea_unavailable_error();
        let host = self.textarea_host.as_mut().ok_or(unavailable)?;
        host.change(id)?;
        self.textarea_snapshot = Some(host.snapshot(id)?);
        Ok(())
    }

    fn dispatch_textarea_focus(
        &mut self,
        id: &str,
        focused: bool,
        source: InputSource,
    ) -> std::result::Result<(), BridgeError> {
        let unavailable = self.textarea_unavailable_error();
        let host = self.textarea_host.as_mut().ok_or(unavailable)?;
        let kind = if focused {
            InputKind::Focus
        } else {
            InputKind::Blur
        };
        host.dispatch(id, kind, source, None)?;
        self.textarea_snapshot = Some(host.snapshot(id)?);
        Ok(())
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

fn separator_view(dashboard: &Dashboard, id: &'static str) -> Stateful<Div> {
    match dashboard.proto.separator(id) {
        Ok(snapshot) => proto_surface::separator_element(id, &snapshot),
        Err(_) => div().id(id).h(px(1.)).w_full().bg(rgb(UNAVAILABLE)),
    }
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
fn selector_trigger_view(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    root_id: &'static str,
    trigger_id: &'static str,
    label: &'static str,
) -> Stateful<Div> {
    let Some(snapshot) = dashboard
        .proto
        .select_snapshot(root_id)
        .and_then(|snapshot| snapshot.trigger.as_ref())
    else {
        return unavailable_button(trigger_id, label);
    };
    let dashboard_entity = cx.entity().downgrade();
    proto_surface::select_trigger_element(trigger_id, label, snapshot, None, move |_, _, cx| {
        dashboard_entity
            .update(cx, |this, cx| {
                if let Err(error) = this.proto.dispatch_select(
                    root_id,
                    trigger_id,
                    InputKind::PressCommit,
                    InputSource::Accessibility,
                    None,
                ) {
                    this.snapshot.status_message = format!("Proto UI Select failed: {error}");
                }
                cx.notify();
            })
            .ok();
    })
    .on_click(cx.listener(move |this, event, _, cx| {
        let source = match event {
            gpui::ClickEvent::Mouse(_) => InputSource::Mouse,
            gpui::ClickEvent::Keyboard(_) => InputSource::Keyboard,
            gpui::ClickEvent::Touch(_) => InputSource::Touch,
        };
        if let Err(error) =
            this.proto
                .dispatch_select(root_id, trigger_id, InputKind::PressCommit, source, None)
        {
            this.snapshot.status_message = format!("Proto UI Select failed: {error}");
        }
        cx.notify();
    }))
}

/// Disabled Select surfaces for performance mode, tuning profile, and power
/// scheme. The hardware snapshot has no typed current readback for these
/// channels yet, so the selectors remain honestly disabled instead of
/// inventing state; enabling them is the final-composition migration.
fn selectors_strip(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> impl IntoElement {
    div().flex().flex_row().flex_wrap().gap_2().children([
        selector_trigger_view(
            dashboard,
            cx,
            PERFORMANCE_MODE_SELECT_ROOT_ID,
            PERFORMANCE_MODE_SELECT_TRIGGER_ID,
            "Performance mode",
        ),
        selector_trigger_view(
            dashboard,
            cx,
            TUNING_PROFILE_SELECT_ROOT_ID,
            TUNING_PROFILE_SELECT_TRIGGER_ID,
            "Tuning profile",
        ),
        selector_trigger_view(
            dashboard,
            cx,
            POWER_SCHEME_SELECT_ROOT_ID,
            POWER_SCHEME_SELECT_TRIGGER_ID,
            "Power scheme",
        ),
    ])
}

fn tuning_panel(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> impl IntoElement {
    let Some(snapshot) = dashboard.proto.textarea_snapshot() else {
        let detail = dashboard.proto.textarea_error.as_deref().map_or_else(
            || "Tuning profile editor unavailable".to_owned(),
            |error| format!("Tuning profile editor unavailable: {error}"),
        );
        return div()
            .border_1()
            .border_color(rgb(UNAVAILABLE))
            .p_4()
            .text_sm()
            .text_color(rgb(UNAVAILABLE))
            .child(detail);
    };
    let Some(input) = dashboard.textarea_input.clone() else {
        return div()
            .border_1()
            .border_color(rgb(UNAVAILABLE))
            .p_4()
            .text_sm()
            .text_color(rgb(UNAVAILABLE))
            .child("Tuning profile editor is not mounted");
    };
    let save = action_button(
        dashboard,
        cx,
        TUNING_PROFILE_SAVE_ID,
        "SAVE PROFILE",
        DashboardAction::SaveProfile,
    );
    let apply = action_button(
        dashboard,
        cx,
        TUNING_PROFILE_APPLY_ID,
        "APPLY DRY-RUN",
        DashboardAction::ApplyProfile,
    );
    div()
        .flex()
        .flex_col()
        .gap_3()
        .min_h_0()
        .child(
            div()
                .flex()
                .flex_row()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(SIGNAL))
                        .child("TUNING / SCHEMA-V1 PROFILE DSL"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(format!("{} UTF-8 bytes", snapshot.native_value.len())),
                ),
        )
        .child(proto_surface::textarea_element(
            TUNING_PROFILE_EDITOR_ID,
            input,
            snapshot,
        ))
        .child(div().flex().flex_row().gap_2().child(save).child(apply))
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
        proto_surface::bind_textarea_keys(cx);
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
    textarea_input: Option<Entity<proto_surface::ProtoTextareaInput>>,
    textarea_subscription: Option<Subscription>,
    textarea_focus_subscription: Option<Subscription>,
    textarea_blur_subscription: Option<Subscription>,
    tab_focus_handles: BTreeMap<&'static str, FocusHandle>,
    tab_focus_subscriptions: Vec<Subscription>,
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
        let active_section = proto.selected_section_index();
        Self {
            snapshot,
            controller,
            active_section,
            proto,
            textarea_input: None,
            textarea_subscription: None,
            textarea_focus_subscription: None,
            textarea_blur_subscription: None,
            tab_focus_handles: BTreeMap::new(),
            tab_focus_subscriptions: Vec::new(),
        }
    }

    fn ensure_tab_focus_handles(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.tab_focus_handles.is_empty() {
            return;
        }
        for id in SIDEBAR_BUTTON_IDS {
            let handle = cx.focus_handle();
            let focus_subscription = cx.on_focus(&handle, window, move |this, window, cx| {
                let source = if window.last_input_was_keyboard() {
                    InputSource::Keyboard
                } else {
                    InputSource::Mouse
                };
                this.handle_tab_focus(id, source, cx);
                cx.notify();
            });
            let blur_subscription = cx.on_blur(&handle, window, move |this, _, cx| {
                this.handle_tab_blur(id, cx);
                cx.notify();
            });
            if let Err(error) = self.proto.set_tab_focus_ready(id, true) {
                self.snapshot.status_message = format!("Proto UI tab focus unavailable: {error}");
            }
            self.tab_focus_handles.insert(id, handle);
            self.tab_focus_subscriptions.push(focus_subscription);
            self.tab_focus_subscriptions.push(blur_subscription);
        }
    }

    fn ensure_textarea_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.textarea_input.is_some() {
            return;
        }
        let Some(snapshot) = self.proto.textarea_snapshot().cloned() else {
            return;
        };
        let input = cx.new(|cx| proto_surface::ProtoTextareaInput::from_snapshot(&snapshot, cx));
        let textarea_subscription = cx.subscribe(
            &input,
            |this, _, event: &proto_surface::ProtoTextareaNativeEvent, cx| {
                this.handle_textarea_native_event(event, cx);
                cx.notify();
            },
        );
        let focus_handle = input.read(cx).focus_handle();
        let textarea_focus_subscription = cx.on_focus(&focus_handle, window, |this, window, cx| {
            let source = if window.last_input_was_keyboard() {
                InputSource::Keyboard
            } else {
                InputSource::Mouse
            };
            this.handle_textarea_focus(TUNING_PROFILE_EDITOR_ID, true, source, cx);
            cx.notify();
        });
        let blur_input = input.clone();
        let textarea_blur_subscription = cx.on_blur(&focus_handle, window, move |this, _, cx| {
            let changed = blur_input.update(cx, |input, _| input.take_dirty());
            this.handle_textarea_blur(TUNING_PROFILE_EDITOR_ID, changed, cx);
            cx.notify();
        });
        self.textarea_input = Some(input);
        self.textarea_subscription = Some(textarea_subscription);
        self.textarea_focus_subscription = Some(textarea_focus_subscription);
        self.textarea_blur_subscription = Some(textarea_blur_subscription);
    }

    fn sync_textarea_input(&mut self, cx: &mut Context<Self>) {
        let Some(snapshot) = self.proto.textarea_snapshot().cloned() else {
            return;
        };
        let Some(input) = self.textarea_input.as_ref() else {
            return;
        };
        input.update(cx, |input, _| input.sync_snapshot(&snapshot));
    }

    fn handle_textarea_native_event(
        &mut self,
        event: &proto_surface::ProtoTextareaNativeEvent,
        cx: &mut Context<Self>,
    ) {
        if let Err(error) = self.proto.dispatch_textarea(event) {
            self.snapshot.status_message = format!("Profile editor input failed: {error}");
        }
        self.sync_textarea_input(cx);
    }

    fn handle_textarea_focus(
        &mut self,
        id: &str,
        focused: bool,
        source: InputSource,
        cx: &mut Context<Self>,
    ) {
        if let Err(error) = self.proto.dispatch_textarea_focus(id, focused, source) {
            self.snapshot.status_message = format!("Profile editor focus failed: {error}");
        }
        self.sync_textarea_input(cx);
    }

    fn handle_textarea_blur(&mut self, id: &str, changed: bool, cx: &mut Context<Self>) {
        if changed && let Err(error) = self.proto.dispatch_textarea_change(id) {
            self.snapshot.status_message = format!("Profile editor change failed: {error}");
        }
        self.handle_textarea_focus(id, false, InputSource::Programmatic, cx);
    }

    fn sync_active_section_from_tabs(&mut self) {
        let next = self.proto.selected_section_index();
        if next != self.active_section {
            self.active_section = next;
            self.snapshot.status_message =
                format!("Section {} selected", SECTION_NAMES[self.active_section].1);
        }
    }

    fn handle_tab_focus(&mut self, id: &str, source: InputSource, _cx: &mut Context<Self>) {
        match self.proto.focus_tab(id, source) {
            Ok(FocusOperationResult::Accepted) => self.sync_active_section_from_tabs(),
            Ok(FocusOperationResult::NotReady) => {
                self.snapshot.status_message = format!("Proto UI tab focus is not ready: {id}");
            }
            Ok(FocusOperationResult::Rejected) => {
                self.snapshot.status_message = format!("Proto UI tab focus was rejected: {id}");
            }
            Err(error) => {
                self.snapshot.status_message = format!("Proto UI tab focus failed: {error}");
            }
        }
    }

    fn handle_tab_blur(&mut self, id: &str, _cx: &mut Context<Self>) {
        if let Err(error) = self.proto.blur_tab(id) {
            self.snapshot.status_message = format!("Proto UI tab blur failed: {error}");
        }
    }

    fn handle_tab_key(&mut self, key: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(error) = self.proto.dispatch_tab_key(key) {
            self.snapshot.status_message = format!("Proto UI tab navigation failed: {error}");
            return;
        }
        self.sync_active_section_from_tabs();
        if let Some(id) = self.proto.active_tab_id()
            && let Some(handle) = self.tab_focus_handles.get(id.as_str())
        {
            window.focus(handle, cx);
        }
    }

    fn handle_tab_press(&mut self, id: &str, source: InputSource) {
        match self.proto.press_tab(id, source) {
            Ok(true) => self.sync_active_section_from_tabs(),
            Ok(false) => {}
            Err(error) => {
                self.snapshot.status_message = format!("Proto UI tab activation failed: {error}");
            }
        }
    }

    fn profile_source(&self) -> Result<String> {
        self.proto
            .textarea_snapshot()
            .map(|snapshot| snapshot.native_value.clone())
            .ok_or_else(|| LctrlError::Unsupported {
                feature: "gui.tuning-profile-editor".into(),
            })
    }

    fn validated_profile_name(source: &str) -> Result<String> {
        lctrl_tune::parse_profile_toml(source, lctrl_tune::ProfileOrigin::User)
            .map(|document| document.profile.name.as_str().to_owned())
    }

    fn save_profile(&mut self) {
        let result = self.profile_source().and_then(|source| {
            let name = Self::validated_profile_name(&source)?;
            self.controller
                .save_profile(&source)
                .map(|saved_name| (name, saved_name))
        });
        self.snapshot.status_message = match result {
            Ok((name, saved_name)) if name == saved_name => {
                format!("Tuning profile {name} saved")
            }
            Ok((name, saved_name)) => {
                format!("Tuning profile {name} saved as {saved_name}")
            }
            Err(error) => format!("Profile save failed: {error}"),
        };
    }

    fn apply_profile(&mut self) {
        let result = self.profile_source().and_then(|source| {
            let name = Self::validated_profile_name(&source)?;
            let saved_name = self.controller.save_profile(&source)?;
            if saved_name != name {
                return Err(LctrlError::InvalidArgument {
                    detail: format!("profile saved as {saved_name}, expected {name}"),
                });
            }
            let args = ["--dry-run", "tune", "profile", "apply", name.as_str()];
            self.controller.execute(&args)
        });
        self.snapshot.status_message = match result {
            Ok(message) => message.trim().to_owned(),
            Err(error) => format!("Profile apply failed: {error}"),
        };
    }

    fn apply_action(&mut self, action: DashboardAction) {
        match action {
            DashboardAction::Refresh => match self.controller.refresh() {
                Ok(mut snapshot) => {
                    snapshot.status_message = "Status refreshed".into();
                    self.snapshot = snapshot;
                }
                Err(error) => {
                    self.snapshot.status_message = format!("Refresh failed: {error}");
                }
            },
            DashboardAction::SaveProfile => self.save_profile(),
            DashboardAction::ApplyProfile => self.apply_profile(),
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

fn tab_navigation_key(key: &str) -> Option<&'static str> {
    match key {
        "left" | "ArrowLeft" => Some("ArrowLeft"),
        "right" | "ArrowRight" => Some("ArrowRight"),
        "up" | "ArrowUp" => Some("ArrowUp"),
        "down" | "ArrowDown" => Some("ArrowDown"),
        "home" | "Home" => Some("Home"),
        "end" | "End" => Some("End"),
        _ => None,
    }
}

fn tab_action(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    id: &'static str,
    label: &'static str,
) -> Stateful<Div> {
    let Some(state) = dashboard
        .proto
        .tabs_snapshot()
        .and_then(|snapshot| snapshot.triggers.iter().find(|trigger| trigger.id == id))
    else {
        return unavailable_button(id, label);
    };
    let disabled = state.disabled;
    let Some(focus_handle) = dashboard.tab_focus_handles.get(id).cloned() else {
        return unavailable_button(id, label);
    };
    let dashboard_entity = cx.entity().downgrade();
    let mouse_focus = focus_handle.clone();
    proto_surface::tab_trigger_element(id, label, state, &focus_handle, move |_, _, cx| {
        dashboard_entity
            .update(cx, |this, cx| {
                this.handle_tab_press(id, InputSource::Accessibility);
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
        if let Err(error) = this.proto.dispatch_tab_input(id, kind, InputSource::Mouse) {
            this.snapshot.status_message = format!("Proto UI tab hover failed: {error}");
        }
        cx.notify();
    }))
    .on_mouse_down(
        gpui::MouseButton::Left,
        cx.listener(move |this, _, window, cx| {
            if !disabled {
                window.focus(&mouse_focus, cx);
            }
            if let Err(error) =
                this.proto
                    .dispatch_tab_input(id, InputKind::PointerDown, InputSource::Mouse)
            {
                this.snapshot.status_message = format!("Proto UI tab press failed: {error}");
            }
            cx.notify();
        }),
    )
    .on_mouse_up(
        gpui::MouseButton::Left,
        cx.listener(move |this, _, _, cx| {
            if let Err(error) =
                this.proto
                    .dispatch_tab_input(id, InputKind::PointerUp, InputSource::Mouse)
            {
                this.snapshot.status_message = format!("Proto UI tab release failed: {error}");
            }
            cx.notify();
        }),
    )
    .on_key_down(
        cx.listener(move |this, event: &gpui::KeyDownEvent, window, cx| {
            if let Some(key) = tab_navigation_key(&event.keystroke.key) {
                this.handle_tab_key(key, window, cx);
                cx.notify();
            }
        }),
    )
    .on_click(cx.listener(move |this, event, _, cx| {
        let source = match event {
            gpui::ClickEvent::Mouse(_) => InputSource::Mouse,
            gpui::ClickEvent::Keyboard(_) => InputSource::Keyboard,
            gpui::ClickEvent::Touch(_) => InputSource::Touch,
        };
        this.handle_tab_press(id, source);
        cx.notify();
    }))
}
fn section_panel(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> Stateful<Div> {
    let index = dashboard.active_section.min(SECTION_VALUES.len() - 1);
    let content_id = SIDEBAR_CONTENT_IDS[index];
    let mut panel = dashboard
        .proto
        .tabs_snapshot()
        .and_then(|snapshot| {
            snapshot
                .contents
                .iter()
                .find(|content| content.id == content_id && content.present)
        })
        .map_or_else(
            || div().id(content_id),
            |content| proto_surface::tab_panel_element(content_id, content),
        )
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .p_6()
        .gap_5()
        .child(identity_header(&dashboard.snapshot))
        .child(separator_view(dashboard, MAIN_HEADER_SEPARATOR_ID))
        .child(safety_banner(&dashboard.snapshot.status_message))
        .child(separator_view(dashboard, ACTION_SEPARATOR_ID))
        .child(action_bar(dashboard, cx))
        .child(selectors_strip(dashboard, cx))
        .child(separator_view(dashboard, CAPABILITY_SEPARATOR_ID));
    if index == TUNING_SECTION_INDEX {
        panel = panel.child(tuning_panel(dashboard, cx));
    } else {
        panel = panel
            .child(capability_matrix(&dashboard.snapshot.capabilities))
            .child(telemetry_panel(&dashboard.snapshot));
    }
    panel
}

impl Render for Dashboard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_tab_focus_handles(window, cx);
        if self.active_section == TUNING_SECTION_INDEX {
            self.ensure_textarea_input(window, cx);
        }
        div()
            .size_full()
            .flex()
            .flex_row()
            .bg(rgb(INK))
            .text_color(rgb(TEXT))
            .font_family("Iosevka, IBM Plex Mono, monospace")
            .child(sidebar(self, cx))
            .child(section_panel(self, cx))
    }
}

fn sidebar(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> impl IntoElement {
    let tabs_unavailable = dashboard.proto.error.is_some() || dashboard.proto.tabs_error.is_some();
    let session_label = if tabs_unavailable {
        "●  PROTO UI UNAVAILABLE"
    } else {
        "●  CLICK-READY"
    };
    let session_color = if tabs_unavailable {
        UNAVAILABLE
    } else {
        CAUTION
    };
    let list = dashboard
        .proto
        .tabs_snapshot()
        .map_or_else(
            || div().id(SIDEBAR_TABS_LIST_ID),
            |snapshot| proto_surface::tab_list_element(SIDEBAR_TABS_LIST_ID, &snapshot.list),
        )
        .flex()
        .flex_col()
        .gap_2()
        .pt_5();
    let sections = SIDEBAR_BUTTON_IDS
        .into_iter()
        .enumerate()
        .map(|(index, id)| {
            tab_action(dashboard, cx, id, SECTION_BUTTON_LABELS[index])
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
        .child(list.children(sections))
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
    fn dashboard_selectors_are_present_disabled_and_never_reach_controller() {
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

        for (root_id, trigger_id) in [
            (
                PERFORMANCE_MODE_SELECT_ROOT_ID,
                PERFORMANCE_MODE_SELECT_TRIGGER_ID,
            ),
            (
                TUNING_PROFILE_SELECT_ROOT_ID,
                TUNING_PROFILE_SELECT_TRIGGER_ID,
            ),
            (POWER_SCHEME_SELECT_ROOT_ID, POWER_SCHEME_SELECT_TRIGGER_ID),
        ] {
            let snapshot = dashboard
                .proto
                .select_snapshot(root_id)
                .expect("selector is present");
            assert!(snapshot.root.disabled);
            assert!(
                snapshot
                    .trigger
                    .as_ref()
                    .expect("trigger is present")
                    .disabled
            );
            assert_eq!(snapshot.trigger.as_ref().expect("trigger").id, trigger_id);
            assert!(snapshot.items.iter().all(|item| item.disabled));
            assert!(!snapshot.root.open);
        }

        // A disabled selection never emits a semantic signal and never reaches
        // the GuiController, which remains the only bridge to CLI actions.
        let outcome = dashboard
            .proto
            .dispatch_select(
                PERFORMANCE_MODE_SELECT_ROOT_ID,
                &format!("{PERFORMANCE_MODE_SELECT_ROOT_ID}-performance"),
                InputKind::PressCommit,
                InputSource::Accessibility,
                None,
            )
            .expect("dispatch succeeds");
        assert_eq!(outcome.item_select_count, 0);
        assert_eq!(outcome.value_change_count, 0);
        assert_eq!(controller.calls.load(Ordering::SeqCst), 0);
    }
    #[test]
    fn dashboard_navigation_is_derived_from_proto_tabs_selection() {
        let snapshot = DashboardSnapshot::unavailable(Platform::Linux, "test");
        let mut dashboard = Dashboard::new(snapshot);

        dashboard.handle_tab_press(SIDEBAR_BUTTON_IDS[2], InputSource::Mouse);

        assert_eq!(dashboard.active_section, 2);
        let tabs = dashboard.proto.tabs_snapshot().expect("tabs snapshot");
        assert!(tabs.contents[2].present);
        assert!(!tabs.contents[0].present);
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

    #[test]
    fn tuning_editor_registers_lazily_for_the_tuning_section() {
        let dashboard = Dashboard::new(DashboardSnapshot::unavailable(Platform::Linux, "initial"));

        let snapshot = dashboard
            .proto
            .textarea_snapshot()
            .expect("tuning editor is registered");
        assert_eq!(snapshot.id, TUNING_PROFILE_EDITOR_ID);
        assert_eq!(SECTION_NAMES[TUNING_SECTION_INDEX].1, "TUNING");
        assert!(snapshot.value.starts_with("schema = 1"));
        // The native entity is composed lazily, only when the Tuning section
        // is rendered, so other sections never host a text input.
        assert!(dashboard.textarea_input.is_none());
    }

    #[test]
    fn save_validation_rejects_non_schema_v1_drafts() {
        let dashboard = Dashboard::new(DashboardSnapshot::unavailable(Platform::Linux, "initial"));
        let snapshot = dashboard
            .proto
            .textarea_snapshot()
            .cloned()
            .expect("tuning editor is registered");
        assert_eq!(
            Dashboard::validated_profile_name(&snapshot.native_value)
                .expect("draft is a valid schema-v1 profile"),
            "sailbreak-gui"
        );

        let unsupported_schema = "schema = 2\n[profile]\nname = \"x\"\n";
        assert!(Dashboard::validated_profile_name(unsupported_schema).is_err());
        let missing_name = "[profile]\ndescription = \"no name\"\n";
        assert!(Dashboard::validated_profile_name(missing_name).is_err());
    }

    #[test]
    fn save_profile_routes_through_the_controller_gateway() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct SaveRecorder {
            saved: AtomicUsize,
        }

        impl GuiController for SaveRecorder {
            fn refresh(&self) -> Result<DashboardSnapshot> {
                Ok(DashboardSnapshot::unavailable(Platform::Linux, "refreshed"))
            }

            fn execute(&self, _args: &[&str]) -> Result<String> {
                panic!("save must not run a command")
            }

            fn save_profile(&self, source: &str) -> Result<String> {
                let document =
                    lctrl_tune::parse_profile_toml(source, lctrl_tune::ProfileOrigin::User)?;
                self.saved.fetch_add(1, Ordering::SeqCst);
                Ok(document.profile.name.as_str().to_owned())
            }
        }

        let controller = Arc::new(SaveRecorder {
            saved: AtomicUsize::new(0),
        });
        let mut dashboard = Dashboard::with_controller(
            DashboardSnapshot::unavailable(Platform::Linux, "initial"),
            controller.clone(),
        );

        dashboard.save_profile();

        assert_eq!(controller.saved.load(Ordering::SeqCst), 1);
        assert_eq!(
            dashboard.snapshot.status_message,
            "Tuning profile sailbreak-gui saved"
        );
    }

    #[test]
    fn apply_profile_uses_the_exact_dry_run_gateway_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct Recorder {
            calls: AtomicUsize,
            saves: AtomicUsize,
        }

        impl GuiController for Recorder {
            fn refresh(&self) -> Result<DashboardSnapshot> {
                Ok(DashboardSnapshot::unavailable(Platform::Linux, "refreshed"))
            }

            fn execute(&self, args: &[&str]) -> Result<String> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(format!("executed {}", args.join(" ")))
            }

            fn save_profile(&self, source: &str) -> Result<String> {
                let name = Dashboard::validated_profile_name(source)?;
                self.saves.fetch_add(1, Ordering::SeqCst);
                Ok(name)
            }
        }

        let controller = Arc::new(Recorder {
            calls: AtomicUsize::new(0),
            saves: AtomicUsize::new(0),
        });
        let mut dashboard = Dashboard::with_controller(
            DashboardSnapshot::unavailable(Platform::Linux, "initial"),
            controller.clone(),
        );

        dashboard.apply_profile();

        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
        assert_eq!(controller.saves.load(Ordering::SeqCst), 1);
        assert_eq!(
            dashboard.snapshot.status_message,
            "executed --dry-run tune profile apply sailbreak-gui"
        );
    }
}
