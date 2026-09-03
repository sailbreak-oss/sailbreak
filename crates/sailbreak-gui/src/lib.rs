//! A read-only GPUI control-center shell for Sailbreak.
//!
//! The GUI deliberately has no hardware mutation path. It renders the state
//! reported by [`lctrl_hal::Hal`] and marks every capability according to the
//! core availability value, including its explanation.

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    env,
    rc::Rc,
    sync::Arc,
};

use gpui::{
    App, Bounds, Context, Div, Entity, FocusHandle, IntoElement, Render, Stateful, Subscription,
    Window, WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};
use lctrl_core::{Availability, CapabilitySet, HardwareInfo, LctrlError, Platform, Result};
use lctrl_hal::Hal;
use proto_ui_gpui::{
    BridgeError, CheckboxProps, DialogCloseProps, DialogContentProps, DialogDescriptionProps,
    DialogDispatchOutcome, DialogFooterProps, DialogHeaderProps, DialogMaskProps, DialogRootProps,
    DialogSnapshot, DialogTitleProps, DialogTriggerProps, DispatchOutcome, DropdownContentProps,
    DropdownDispatchOutcome, DropdownItemProps, DropdownRootProps, DropdownSnapshot,
    DropdownTriggerProps, FocusOperationResult, HoverCardContentProps, HoverCardDispatchOutcome,
    HoverCardRootProps, HoverCardSnapshot, HoverCardTriggerProps, InputKind, InputSource,
    OverlayRect, ProtoButtonHost, ProtoButtonState, ProtoCheckboxHost, ProtoCheckboxSnapshot,
    ProtoDialogHost, ProtoDropdownHost, ProtoHoverCardHost, ProtoSelectHost, ProtoSelectSnapshot,
    ProtoSeparatorHost, ProtoSeparatorSnapshot, ProtoSwitchHost, ProtoSwitchSnapshot,
    ProtoTabsHost, ProtoTextareaHost, ProtoTextareaSnapshot, ProtoToggleHost, ProtoToggleSnapshot,
    SelectContentProps, SelectDispatchOutcome, SelectItemProps, SelectRootProps,
    SelectTriggerProps, SelectValueProps, SeparatorProps, ShadcnButtonSize, ShadcnButtonVariant,
    SwitchProps, TabsActivationMode, TabsContentProps, TabsListProps, TabsOrientation,
    TabsRootProps, TabsSnapshot, TabsTriggerProps, TextareaProps, ToggleDispatchOutcome,
    ToggleProps, ToggleSize, ToggleVariant,
};

mod proto_surface;
pub use proto_surface::{
    AccessibleProjection, dialog_close_element, dialog_content_element, dialog_description_element,
    dialog_footer_element, dialog_header_element, dialog_mask_element, dialog_title_element,
    dialog_trigger_element, dropdown_content_element, dropdown_item_element,
    dropdown_trigger_element, hover_card_content_element, hover_card_trigger_element,
    overlay_surface_element, project_a11y, select_content_element, select_item_element,
    select_value_element,
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
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiosWriteRequest {
    pub setting: String,
    pub current_value: String,
    pub requested_value: String,
}

impl BiosWriteRequest {
    #[must_use]
    pub fn recovery_command(&self) -> String {
        format!(
            "sailbreak bios set {} {} --save --yes",
            self.setting, self.current_value
        )
    }
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
    /// Return one typed BIOS read/modify/write request when the platform can
    /// prove the current value. Without typed readback the GUI leaves the BIOS
    /// confirmation unavailable rather than inventing a setting.
    fn bios_write_request(&self) -> Result<BiosWriteRequest> {
        Err(LctrlError::Unsupported {
            feature: "gui.bios.write-readback".into(),
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

const CAPABILITY_SWITCH_ID: &str = "capability-readback-switch";
const CAPABILITY_CHECKBOX_ID: &str = "capability-readback-checkbox";
const CAPABILITY_SWITCH_LABEL: &str = "SWITCH / READBACK UNAVAILABLE";
const CAPABILITY_CHECKBOX_LABEL: &str = "CHECKBOX / READBACK UNAVAILABLE";
const EMPTY_HOVER_CARD_FEATURE_ID: &str = "capability-detail-unavailable";

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

const STATUS_ACTIONS_DROPDOWN_ROOT_ID: &str = "status-actions-dropdown";
const STATUS_ACTIONS_DROPDOWN_TRIGGER_ID: &str = "status-actions-trigger";
const STATUS_ACTIONS_DROPDOWN_CONTENT_ID: &str = "status-actions-content";
const STATUS_ACTIONS_DROPDOWN_ITEM_IDS: [&str; 3] =
    ["daemon-status", "battery-status", "thermal-sensors"];
const SYSTEM_ACTIONS_DROPDOWN_ROOT_ID: &str = "system-actions-dropdown";
const SYSTEM_ACTIONS_DROPDOWN_TRIGGER_ID: &str = "system-actions-trigger";
const SYSTEM_ACTIONS_DROPDOWN_CONTENT_ID: &str = "system-actions-content";
const SYSTEM_ACTIONS_DROPDOWN_ITEM_IDS: [&str; 3] =
    ["power-schemes", "diagnostics", "magicbay-detect"];

const BIOS_SECTION_INDEX: usize = 4;
const BIOS_DIALOG_ROOT_ID: &str = "bios-write-dialog";
const BIOS_DIALOG_TRIGGER_ID: &str = "bios-write-trigger";
const BIOS_DIALOG_MASK_ID: &str = "bios-write-mask";
const BIOS_DIALOG_CONTENT_ID: &str = "bios-write-content";
const BIOS_DIALOG_TITLE_ID: &str = "bios-write-title";
const BIOS_DIALOG_DESCRIPTION_ID: &str = "bios-write-description";
const BIOS_DIALOG_CANCEL_ID: &str = "bios-write-cancel";
const BIOS_DIALOG_CONFIRM_ID: &str = "bios-write-confirm";
const BIOS_DIALOG_HEADER_ID: &str = "bios-write-header";
const BIOS_DIALOG_FOOTER_ID: &str = "bios-write-footer";
const PROFILE_DIALOG_ROOT_ID: &str = "profile-apply-dialog";
const PROFILE_DIALOG_MASK_ID: &str = "profile-apply-mask";
const PROFILE_DIALOG_CONTENT_ID: &str = "profile-apply-content";
const PROFILE_DIALOG_TITLE_ID: &str = "profile-apply-title";
const PROFILE_DIALOG_DESCRIPTION_ID: &str = "profile-apply-description";
const PROFILE_DIALOG_CANCEL_ID: &str = "profile-apply-cancel";
const PROFILE_DIALOG_CONFIRM_ID: &str = "profile-apply-confirm";
const PROFILE_DIALOG_HEADER_ID: &str = "profile-apply-header";
const PROFILE_DIALOG_FOOTER_ID: &str = "profile-apply-footer";

#[derive(Clone, Copy)]
struct DropdownActionSpec {
    id: &'static str,
    label: &'static str,
    value: &'static str,
    capability: Option<&'static str>,
    action: DashboardAction,
}

const STATUS_ACTION_SPECS: [DropdownActionSpec; 3] = [
    DropdownActionSpec {
        id: "daemon-status",
        label: "DAEMON STATUS",
        value: "daemon-status",
        capability: None,
        action: DashboardAction::RunCommand(DAEMON_STATUS_COMMAND),
    },
    DropdownActionSpec {
        id: "battery-status",
        label: "BATTERY STATUS",
        value: "battery-status",
        capability: Some("battery.status"),
        action: DashboardAction::RunCommand(BATTERY_STATUS_COMMAND),
    },
    DropdownActionSpec {
        id: "thermal-sensors",
        label: "THERMAL SENSORS",
        value: "thermal-sensors",
        capability: Some("perf.temp"),
        action: DashboardAction::RunCommand(THERMAL_SENSORS_COMMAND),
    },
];

const SYSTEM_ACTION_SPECS: [DropdownActionSpec; 3] = [
    DropdownActionSpec {
        id: "power-schemes",
        label: "POWER SCHEMES",
        value: "power-schemes",
        capability: Some("power.scheme"),
        action: DashboardAction::RunCommand(POWER_SCHEMES_COMMAND),
    },
    DropdownActionSpec {
        id: "diagnostics",
        label: "DIAGNOSTICS",
        value: "diagnostics",
        capability: Some("diagnostics.inventory"),
        action: DashboardAction::RunCommand(DIAGNOSTICS_COMMAND),
    },
    DropdownActionSpec {
        id: "magicbay-detect",
        label: "MAGICBAY DETECT",
        value: "magicbay-detect",
        capability: Some("magicbay.inventory"),
        action: DashboardAction::RunCommand(MAGICBAY_COMMAND),
    },
];

fn capability_available(capabilities: &CapabilitySet, capability: Option<&str>) -> bool {
    capability.is_none_or(|id| {
        capabilities
            .get(id)
            .is_some_and(|entry| entry.availability != Availability::Unavailable)
    })
}

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

type HoverCardRegistry = (
    BTreeMap<String, ProtoHoverCardHost>,
    BTreeMap<String, HoverCardSnapshot>,
);

struct ProtoUiState {
    host: Option<ProtoButtonHost>,
    toggle_host: Option<ProtoToggleHost>,
    switch_host: Option<ProtoSwitchHost>,
    switch_snapshot: Option<ProtoSwitchSnapshot>,
    checkbox_host: Option<ProtoCheckboxHost>,
    checkbox_snapshot: Option<ProtoCheckboxSnapshot>,
    separator_host: Option<ProtoSeparatorHost>,
    hover_card_hosts: BTreeMap<String, ProtoHoverCardHost>,
    hover_card_snapshots: BTreeMap<String, HoverCardSnapshot>,
    hover_card_error: Option<String>,
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
    status_dropdown_host: Option<ProtoDropdownHost>,
    status_dropdown_snapshot: Option<DropdownSnapshot>,
    system_dropdown_host: Option<ProtoDropdownHost>,
    system_dropdown_snapshot: Option<DropdownSnapshot>,
    profile_dialog_host: Option<ProtoDialogHost>,
    profile_dialog_snapshot: Option<DialogSnapshot>,
    profile_dialog_error: Option<String>,
    bios_dialog_host: Option<ProtoDialogHost>,
    bios_dialog_snapshot: Option<DialogSnapshot>,
    bios_dialog_error: Option<String>,
    bios_write_request: Option<BiosWriteRequest>,
    select_error: Option<String>,
    dropdown_error: Option<String>,
    error: Option<String>,
}

impl ProtoUiState {
    fn new(capabilities: &CapabilitySet) -> Self {
        let (host, toggle_host, mut switch_host, mut checkbox_host, separator_host) =
            match Self::build_core() {
                Ok(hosts) => hosts,
                Err(error) => {
                    return Self {
                        host: None,
                        toggle_host: None,
                        switch_host: None,
                        switch_snapshot: None,
                        checkbox_host: None,
                        checkbox_snapshot: None,
                        separator_host: None,
                        hover_card_hosts: BTreeMap::new(),
                        hover_card_snapshots: BTreeMap::new(),
                        hover_card_error: None,
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
                        status_dropdown_host: None,
                        status_dropdown_snapshot: None,
                        system_dropdown_host: None,
                        system_dropdown_snapshot: None,
                        profile_dialog_host: None,
                        profile_dialog_snapshot: None,
                        profile_dialog_error: None,
                        bios_dialog_host: None,
                        bios_dialog_snapshot: None,
                        bios_dialog_error: None,
                        bios_write_request: None,
                        select_error: None,
                        dropdown_error: None,
                        error: Some(error.to_string()),
                    };
                }
            };
        let switch_snapshot = switch_host.snapshot(CAPABILITY_SWITCH_ID).ok();
        let checkbox_snapshot = checkbox_host.snapshot(CAPABILITY_CHECKBOX_ID).ok();
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
        let (status_dropdown_host, status_dropdown_snapshot, status_error) =
            match Self::build_status_dropdown(capabilities) {
                Ok((host, snapshot)) => (Some(host), Some(snapshot), None),
                Err(error) => (None, None, Some(error.to_string())),
            };
        let (system_dropdown_host, system_dropdown_snapshot, system_error) =
            match Self::build_system_dropdown(capabilities) {
                Ok((host, snapshot)) => (Some(host), Some(snapshot), None),
                Err(error) => (None, None, Some(error.to_string())),
            };
        let dropdown_error = status_error.or(system_error);
        let (profile_dialog_host, profile_dialog_snapshot, profile_dialog_error) =
            match Self::build_profile_dialog() {
                Ok((host, snapshot)) => (Some(host), Some(snapshot), None),
                Err(error) => (None, None, Some(error.to_string())),
            };
        let (hover_card_hosts, hover_card_snapshots, hover_card_error) =
            match Self::build_hover_card(capabilities) {
                Ok((hosts, snapshots)) => (hosts, snapshots, None),
                Err(error) => (BTreeMap::new(), BTreeMap::new(), Some(error.to_string())),
            };
        Self {
            host: Some(host),
            toggle_host: Some(toggle_host),
            switch_host: Some(switch_host),
            switch_snapshot,
            checkbox_host: Some(checkbox_host),
            checkbox_snapshot,
            separator_host: Some(separator_host),
            hover_card_hosts,
            hover_card_snapshots,
            hover_card_error,
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
            status_dropdown_host,
            status_dropdown_snapshot,
            system_dropdown_host,
            system_dropdown_snapshot,
            profile_dialog_host,
            profile_dialog_snapshot,
            profile_dialog_error,
            bios_dialog_host: None,
            bios_dialog_snapshot: None,
            bios_dialog_error: None,
            bios_write_request: None,
            select_error: None,
            dropdown_error,
            error: None,
        }
    }
    fn build_profile_dialog() -> std::result::Result<(ProtoDialogHost, DialogSnapshot), BridgeError>
    {
        let mut host = ProtoDialogHost::new()?;
        host.register_root(
            PROFILE_DIALOG_ROOT_ID,
            "Confirm tuning profile apply",
            DialogRootProps {
                alert: true,
                a11y_label: "Confirm tuning profile apply".to_owned(),
                ..DialogRootProps::default()
            },
        )?;
        host.register_trigger(
            TUNING_PROFILE_APPLY_ID,
            PROFILE_DIALOG_ROOT_ID,
            DialogTriggerProps::default(),
        )?;
        host.register_mask(
            PROFILE_DIALOG_MASK_ID,
            PROFILE_DIALOG_ROOT_ID,
            DialogMaskProps::default(),
        )?;
        host.register_content(
            PROFILE_DIALOG_CONTENT_ID,
            PROFILE_DIALOG_ROOT_ID,
            DialogContentProps,
        )?;
        host.register_title(
            PROFILE_DIALOG_TITLE_ID,
            "Apply tuning profile?",
            PROFILE_DIALOG_CONTENT_ID,
            DialogTitleProps,
        )?;
        host.register_description(
            PROFILE_DIALOG_DESCRIPTION_ID,
            "This writes the validated profile through the CLI safety gate. Recovery: sailbreak tune restore.",
            PROFILE_DIALOG_CONTENT_ID,
            DialogDescriptionProps,
        )?;
        host.register_close(
            PROFILE_DIALOG_CANCEL_ID,
            "CANCEL",
            PROFILE_DIALOG_CONTENT_ID,
            DialogCloseProps::default(),
        )?;
        host.register_close(
            PROFILE_DIALOG_CONFIRM_ID,
            "APPLY WITH --YES",
            PROFILE_DIALOG_CONTENT_ID,
            DialogCloseProps::default(),
        )?;
        host.register_header(
            PROFILE_DIALOG_HEADER_ID,
            PROFILE_DIALOG_CONTENT_ID,
            DialogHeaderProps,
        )?;
        host.register_footer(
            PROFILE_DIALOG_FOOTER_ID,
            PROFILE_DIALOG_CONTENT_ID,
            DialogFooterProps,
        )?;
        host.setup()?;
        let snapshot = host.snapshot()?;
        Ok((host, snapshot))
    }
    fn build_bios_dialog(
        request: &BiosWriteRequest,
    ) -> std::result::Result<(ProtoDialogHost, DialogSnapshot), BridgeError> {
        if request.setting.trim().is_empty()
            || request.current_value.trim().is_empty()
            || request.requested_value.trim().is_empty()
        {
            return Err(BridgeError::InvalidIdentity {
                kind: "BIOS dialog requires typed current and requested values".to_owned(),
            });
        }
        let mut host = ProtoDialogHost::new()?;
        host.register_root(
            BIOS_DIALOG_ROOT_ID,
            "Confirm BIOS write",
            DialogRootProps {
                alert: true,
                a11y_label: "Confirm BIOS write".to_owned(),
                ..DialogRootProps::default()
            },
        )?;
        host.register_trigger(
            BIOS_DIALOG_TRIGGER_ID,
            BIOS_DIALOG_ROOT_ID,
            DialogTriggerProps::default(),
        )?;
        host.register_mask(
            BIOS_DIALOG_MASK_ID,
            BIOS_DIALOG_ROOT_ID,
            DialogMaskProps::default(),
        )?;
        host.register_content(
            BIOS_DIALOG_CONTENT_ID,
            BIOS_DIALOG_ROOT_ID,
            DialogContentProps,
        )?;
        host.register_title(
            BIOS_DIALOG_TITLE_ID,
            "Write BIOS setting?",
            BIOS_DIALOG_CONTENT_ID,
            DialogTitleProps,
        )?;
        let description = format!(
            "BIOS setting {} changes from {} to {}. The effect may require a reboot. Recovery: {}",
            request.setting,
            request.current_value,
            request.requested_value,
            request.recovery_command()
        );
        host.register_description(
            BIOS_DIALOG_DESCRIPTION_ID,
            description,
            BIOS_DIALOG_CONTENT_ID,
            DialogDescriptionProps,
        )?;
        host.register_close(
            BIOS_DIALOG_CANCEL_ID,
            "CANCEL",
            BIOS_DIALOG_CONTENT_ID,
            DialogCloseProps::default(),
        )?;
        host.register_close(
            BIOS_DIALOG_CONFIRM_ID,
            "WRITE WITH --YES",
            BIOS_DIALOG_CONTENT_ID,
            DialogCloseProps::default(),
        )?;
        host.register_header(
            BIOS_DIALOG_HEADER_ID,
            BIOS_DIALOG_CONTENT_ID,
            DialogHeaderProps,
        )?;
        host.register_footer(
            BIOS_DIALOG_FOOTER_ID,
            BIOS_DIALOG_CONTENT_ID,
            DialogFooterProps,
        )?;
        host.setup()?;
        let snapshot = host.snapshot()?;
        Ok((host, snapshot))
    }

    fn attach_bios_dialog(&mut self, controller: &dyn GuiController) {
        match controller.bios_write_request() {
            Ok(request) => match Self::build_bios_dialog(&request) {
                Ok((host, snapshot)) => {
                    self.bios_write_request = Some(request);
                    self.bios_dialog_host = Some(host);
                    self.bios_dialog_snapshot = Some(snapshot);
                    self.bios_dialog_error = None;
                }
                Err(error) => {
                    self.bios_dialog_error = Some(error.to_string());
                }
            },
            Err(error) => {
                self.bios_dialog_error = Some(error.to_string());
            }
        }
    }

    fn profile_dialog_snapshot(&self) -> Option<&DialogSnapshot> {
        self.profile_dialog_snapshot.as_ref()
    }

    fn bios_dialog_snapshot(&self) -> Option<&DialogSnapshot> {
        self.bios_dialog_snapshot.as_ref()
    }

    fn dialog_host_mut(&mut self, id: &str) -> Option<&mut ProtoDialogHost> {
        if matches!(
            id,
            TUNING_PROFILE_APPLY_ID
                | PROFILE_DIALOG_ROOT_ID
                | PROFILE_DIALOG_MASK_ID
                | PROFILE_DIALOG_CONTENT_ID
                | PROFILE_DIALOG_TITLE_ID
                | PROFILE_DIALOG_DESCRIPTION_ID
                | PROFILE_DIALOG_CANCEL_ID
                | PROFILE_DIALOG_CONFIRM_ID
                | PROFILE_DIALOG_HEADER_ID
                | PROFILE_DIALOG_FOOTER_ID
        ) {
            self.profile_dialog_host.as_mut()
        } else if matches!(
            id,
            BIOS_DIALOG_TRIGGER_ID
                | BIOS_DIALOG_ROOT_ID
                | BIOS_DIALOG_MASK_ID
                | BIOS_DIALOG_CONTENT_ID
                | BIOS_DIALOG_TITLE_ID
                | BIOS_DIALOG_DESCRIPTION_ID
                | BIOS_DIALOG_CANCEL_ID
                | BIOS_DIALOG_CONFIRM_ID
                | BIOS_DIALOG_HEADER_ID
                | BIOS_DIALOG_FOOTER_ID
        ) {
            self.bios_dialog_host.as_mut()
        } else {
            None
        }
    }

    fn is_dialog_id(id: &str) -> bool {
        matches!(
            id,
            TUNING_PROFILE_APPLY_ID
                | PROFILE_DIALOG_ROOT_ID
                | PROFILE_DIALOG_MASK_ID
                | PROFILE_DIALOG_CONTENT_ID
                | PROFILE_DIALOG_TITLE_ID
                | PROFILE_DIALOG_DESCRIPTION_ID
                | PROFILE_DIALOG_CANCEL_ID
                | PROFILE_DIALOG_CONFIRM_ID
                | PROFILE_DIALOG_HEADER_ID
                | PROFILE_DIALOG_FOOTER_ID
                | BIOS_DIALOG_TRIGGER_ID
                | BIOS_DIALOG_ROOT_ID
                | BIOS_DIALOG_MASK_ID
                | BIOS_DIALOG_CONTENT_ID
                | BIOS_DIALOG_TITLE_ID
                | BIOS_DIALOG_DESCRIPTION_ID
                | BIOS_DIALOG_CANCEL_ID
                | BIOS_DIALOG_CONFIRM_ID
                | BIOS_DIALOG_HEADER_ID
                | BIOS_DIALOG_FOOTER_ID
        )
    }

    fn refresh_profile_dialog(&mut self) -> std::result::Result<(), BridgeError> {
        let Some(host) = self.profile_dialog_host.as_mut() else {
            return Err(BridgeError::Runtime {
                detail: self
                    .profile_dialog_error
                    .clone()
                    .unwrap_or_else(|| "Proto UI profile Dialog is unavailable".to_owned()),
            });
        };
        self.profile_dialog_snapshot = Some(host.snapshot()?);
        Ok(())
    }

    fn refresh_bios_dialog(&mut self) -> std::result::Result<(), BridgeError> {
        let Some(host) = self.bios_dialog_host.as_mut() else {
            return Err(BridgeError::Runtime {
                detail: self
                    .bios_dialog_error
                    .clone()
                    .unwrap_or_else(|| "Proto UI BIOS Dialog is unavailable".to_owned()),
            });
        };
        self.bios_dialog_snapshot = Some(host.snapshot()?);
        Ok(())
    }

    fn dialog_modal_blocking(&mut self) -> bool {
        self.profile_dialog_host
            .as_mut()
            .and_then(|host| host.modal_blocking().ok())
            .unwrap_or(false)
            || self
                .bios_dialog_host
                .as_mut()
                .and_then(|host| host.modal_blocking().ok())
                .unwrap_or(false)
    }

    fn set_dialog_focus_ready(
        &mut self,
        id: &str,
        ready: bool,
    ) -> std::result::Result<(), BridgeError> {
        let Some(host) = self.dialog_host_mut(id) else {
            return Err(BridgeError::Runtime {
                detail: format!("Dialog focus target unavailable: {id}"),
            });
        };
        host.set_focus_ready(id, ready)
    }

    fn focus_dialog(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> std::result::Result<FocusOperationResult, BridgeError> {
        let Some(host) = self.dialog_host_mut(id) else {
            return Ok(FocusOperationResult::Rejected);
        };
        let result = host.focus_with_source(id, source)?;
        if id.starts_with("bios-") {
            self.refresh_bios_dialog()?;
        } else {
            self.refresh_profile_dialog()?;
        }
        Ok(result)
    }

    fn blur_dialog(&mut self, id: &str) -> std::result::Result<(), BridgeError> {
        let Some(host) = self.dialog_host_mut(id) else {
            return Ok(());
        };
        host.blur(id, InputSource::Programmatic)?;
        if id.starts_with("bios-") {
            self.refresh_bios_dialog()?;
        } else {
            self.refresh_profile_dialog()?;
        }
        Ok(())
    }

    fn dispatch_dialog_key(
        &mut self,
        id: &str,
        key: &str,
    ) -> std::result::Result<DialogDispatchOutcome, BridgeError> {
        let Some(host) = self.dialog_host_mut(id) else {
            return Err(BridgeError::Runtime {
                detail: format!("Dialog keyboard target unavailable: {id}"),
            });
        };
        let outcome = host.dispatch_key(key)?;
        if id.starts_with("bios-") {
            self.refresh_bios_dialog()?;
        } else {
            self.refresh_profile_dialog()?;
        }
        Ok(outcome)
    }

    fn press_profile_trigger(
        &mut self,
        source: InputSource,
    ) -> std::result::Result<DialogDispatchOutcome, BridgeError> {
        let host = self
            .profile_dialog_host
            .as_mut()
            .ok_or_else(|| BridgeError::Runtime {
                detail: self
                    .profile_dialog_error
                    .clone()
                    .unwrap_or_else(|| "Proto UI profile Dialog is unavailable".to_owned()),
            })?;
        let outcome = host.press_trigger(TUNING_PROFILE_APPLY_ID, source)?;
        self.refresh_profile_dialog()?;
        Ok(outcome)
    }

    fn press_profile_close(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> std::result::Result<DialogDispatchOutcome, BridgeError> {
        let host = self
            .profile_dialog_host
            .as_mut()
            .ok_or_else(|| BridgeError::Runtime {
                detail: self
                    .profile_dialog_error
                    .clone()
                    .unwrap_or_else(|| "Proto UI profile Dialog is unavailable".to_owned()),
            })?;
        let outcome = host.press_close(id, source)?;
        self.refresh_profile_dialog()?;
        Ok(outcome)
    }

    fn press_bios_trigger(
        &mut self,
        source: InputSource,
    ) -> std::result::Result<DialogDispatchOutcome, BridgeError> {
        let host = self
            .bios_dialog_host
            .as_mut()
            .ok_or_else(|| BridgeError::Runtime {
                detail: self
                    .bios_dialog_error
                    .clone()
                    .unwrap_or_else(|| "Proto UI BIOS Dialog is unavailable".to_owned()),
            })?;
        let outcome = host.press_trigger(BIOS_DIALOG_TRIGGER_ID, source)?;
        self.refresh_bios_dialog()?;
        Ok(outcome)
    }

    fn dispatch_dialog_input(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) -> std::result::Result<DialogDispatchOutcome, BridgeError> {
        let Some(host) = self.dialog_host_mut(id) else {
            return Err(BridgeError::Runtime {
                detail: format!("Dialog input target unavailable: {id}"),
            });
        };
        let outcome = host.dispatch(id, kind, source, detail)?;
        if id.starts_with("bios-") {
            self.refresh_bios_dialog()?;
        } else {
            self.refresh_profile_dialog()?;
        }
        Ok(outcome)
    }

    fn press_bios_close(
        &mut self,
        id: &str,
        source: InputSource,
    ) -> std::result::Result<DialogDispatchOutcome, BridgeError> {
        let host = self
            .bios_dialog_host
            .as_mut()
            .ok_or_else(|| BridgeError::Runtime {
                detail: self
                    .bios_dialog_error
                    .clone()
                    .unwrap_or_else(|| "Proto UI BIOS Dialog is unavailable".to_owned()),
            })?;
        let outcome = host.press_close(id, source)?;
        self.refresh_bios_dialog()?;
        Ok(outcome)
    }

    fn build_core() -> std::result::Result<
        (
            ProtoButtonHost,
            ProtoToggleHost,
            ProtoSwitchHost,
            ProtoCheckboxHost,
            ProtoSeparatorHost,
        ),
        BridgeError,
    > {
        let mut host = ProtoButtonHost::new()?;
        host.register_button(
            "refresh-status",
            "REFRESH STATUS",
            ShadcnButtonVariant::Default,
            ShadcnButtonSize::Sm,
        )?;
        host.register_button(
            TUNING_PROFILE_SAVE_ID,
            "SAVE PROFILE",
            ShadcnButtonVariant::Default,
            ShadcnButtonSize::Sm,
        )?;

        let mut toggle_host = ProtoToggleHost::new()?;
        toggle_host.register(
            PERFORMANCE_PREVIEW_TOGGLE_ID,
            "PERFORMANCE PREVIEW",
            performance_preview_props(false),
        )?;

        let mut switch_host = ProtoSwitchHost::new()?;
        switch_host.register_root(
            CAPABILITY_SWITCH_ID,
            CAPABILITY_SWITCH_LABEL,
            SwitchProps {
                checked: None,
                default_checked: false,
                disabled: true,
            },
        )?;
        switch_host.register_thumb(
            format!("{CAPABILITY_SWITCH_ID}-thumb"),
            CAPABILITY_SWITCH_ID,
        )?;

        let mut checkbox_host = ProtoCheckboxHost::new()?;
        checkbox_host.register_root(
            CAPABILITY_CHECKBOX_ID,
            CAPABILITY_CHECKBOX_LABEL,
            CheckboxProps {
                checked: None,
                default_checked: false,
                disabled: true,
                indeterminate: None,
                default_indeterminate: false,
            },
        )?;
        checkbox_host.register_indicator(
            format!("{CAPABILITY_CHECKBOX_ID}-indicator"),
            CAPABILITY_CHECKBOX_ID,
        )?;

        let mut separator_host = ProtoSeparatorHost::new()?;
        for id in [
            MAIN_HEADER_SEPARATOR_ID,
            ACTION_SEPARATOR_ID,
            CAPABILITY_SEPARATOR_ID,
        ] {
            separator_host.register(id, SeparatorProps::default())?;
        }
        Ok((
            host,
            toggle_host,
            switch_host,
            checkbox_host,
            separator_host,
        ))
    }

    fn build_status_dropdown(
        capabilities: &CapabilitySet,
    ) -> std::result::Result<(ProtoDropdownHost, DropdownSnapshot), BridgeError> {
        Self::build_action_dropdown(
            STATUS_ACTIONS_DROPDOWN_ROOT_ID,
            STATUS_ACTIONS_DROPDOWN_TRIGGER_ID,
            STATUS_ACTIONS_DROPDOWN_CONTENT_ID,
            "Status actions",
            &STATUS_ACTION_SPECS,
            capabilities,
        )
    }

    fn build_system_dropdown(
        capabilities: &CapabilitySet,
    ) -> std::result::Result<(ProtoDropdownHost, DropdownSnapshot), BridgeError> {
        Self::build_action_dropdown(
            SYSTEM_ACTIONS_DROPDOWN_ROOT_ID,
            SYSTEM_ACTIONS_DROPDOWN_TRIGGER_ID,
            SYSTEM_ACTIONS_DROPDOWN_CONTENT_ID,
            "System actions",
            &SYSTEM_ACTION_SPECS,
            capabilities,
        )
    }

    fn build_action_dropdown(
        root_id: &'static str,
        trigger_id: &'static str,
        content_id: &'static str,
        label: &'static str,
        specs: &[DropdownActionSpec],
        capabilities: &CapabilitySet,
    ) -> std::result::Result<(ProtoDropdownHost, DropdownSnapshot), BridgeError> {
        let mut host = ProtoDropdownHost::new()?;
        host.register_root(root_id, label, DropdownRootProps::default())?;
        host.register_trigger(
            trigger_id,
            root_id,
            DropdownTriggerProps {
                indicator: true,
                ..DropdownTriggerProps::default()
            },
        )?;
        host.register_content(content_id, root_id, DropdownContentProps::default())?;
        for spec in specs {
            host.register_item(
                spec.id,
                spec.label,
                content_id,
                DropdownItemProps {
                    value: spec.value.to_owned(),
                    text_value: spec.label.to_owned(),
                    disabled: !capability_available(capabilities, spec.capability),
                    ..DropdownItemProps::default()
                },
            )?;
        }
        host.setup()?;
        let snapshot = host.snapshot()?;
        Ok((host, snapshot))
    }

    fn build_hover_card(
        capabilities: &CapabilitySet,
    ) -> std::result::Result<HoverCardRegistry, BridgeError> {
        if capabilities.features.is_empty() {
            return Err(BridgeError::InvalidIdentity {
                kind: "hover card graph requires a capability detail".to_owned(),
            });
        }
        let mut hosts = BTreeMap::new();
        let mut snapshots = BTreeMap::new();
        for (feature, capability) in &capabilities.features {
            let mut host = ProtoHoverCardHost::new()?;
            let root = format!("hover-card:{feature}");
            host.register_root(
                root.clone(),
                feature.clone(),
                HoverCardRootProps::default().with_delays(400, 200),
            )?;
            host.register_trigger(
                format!("{root}-trigger"),
                &root,
                HoverCardTriggerProps::default(),
            )?;
            let accessible = capability_detail_accessibility(capability);
            host.register_content_with_slot(
                format!("{root}-content"),
                &root,
                accessible,
                HoverCardContentProps::default(),
            )?;
            host.setup()?;
            let snapshot = host.snapshot()?;
            hosts.insert(feature.clone(), host);
            snapshots.insert(feature.clone(), snapshot);
        }
        Ok((hosts, snapshots))
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

    fn switch_snapshot(&self) -> Option<&ProtoSwitchSnapshot> {
        self.switch_snapshot.as_ref()
    }

    fn checkbox_snapshot(&self) -> Option<&ProtoCheckboxSnapshot> {
        self.checkbox_snapshot.as_ref()
    }

    fn dispatch_switch(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) -> std::result::Result<proto_ui_gpui::SwitchDispatchOutcome, BridgeError> {
        let unavailable = self.unavailable_error();
        let host = self.switch_host.as_mut().ok_or(unavailable)?;
        let outcome = host.dispatch(id, kind, source, detail)?;
        self.switch_snapshot = Some(host.snapshot(id)?);
        Ok(outcome)
    }

    fn dispatch_checkbox(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) -> std::result::Result<proto_ui_gpui::CheckboxDispatchOutcome, BridgeError> {
        let unavailable = self.unavailable_error();
        let host = self.checkbox_host.as_mut().ok_or(unavailable)?;
        let outcome = host.dispatch(id, kind, source, detail)?;
        self.checkbox_snapshot = Some(host.snapshot(id)?);
        Ok(outcome)
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

    fn dropdown_snapshot(&self, root_id: &str) -> Option<&DropdownSnapshot> {
        match root_id {
            STATUS_ACTIONS_DROPDOWN_ROOT_ID => self.status_dropdown_snapshot.as_ref(),
            SYSTEM_ACTIONS_DROPDOWN_ROOT_ID => self.system_dropdown_snapshot.as_ref(),
            _ => None,
        }
    }

    fn dropdown_host_mut(&mut self, root_id: &str) -> Option<&mut ProtoDropdownHost> {
        match root_id {
            STATUS_ACTIONS_DROPDOWN_ROOT_ID => self.status_dropdown_host.as_mut(),
            SYSTEM_ACTIONS_DROPDOWN_ROOT_ID => self.system_dropdown_host.as_mut(),
            _ => None,
        }
    }

    fn refresh_dropdown(&mut self, root_id: &str) -> std::result::Result<(), BridgeError> {
        let unavailable = self.dropdown_unavailable_error();
        let Some(host) = self.dropdown_host_mut(root_id) else {
            return Err(unavailable);
        };
        let snapshot = host.snapshot()?;
        match root_id {
            STATUS_ACTIONS_DROPDOWN_ROOT_ID => self.status_dropdown_snapshot = Some(snapshot),
            SYSTEM_ACTIONS_DROPDOWN_ROOT_ID => self.system_dropdown_snapshot = Some(snapshot),
            _ => {}
        }
        Ok(())
    }

    fn dropdown_unavailable_error(&self) -> BridgeError {
        BridgeError::Runtime {
            detail: self
                .dropdown_error
                .clone()
                .unwrap_or_else(|| "Proto UI Dropdown host is unavailable".to_owned()),
        }
    }

    fn dispatch_dropdown(
        &mut self,
        root_id: &str,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) -> std::result::Result<DropdownDispatchOutcome, BridgeError> {
        let unavailable = self.dropdown_unavailable_error();
        let Some(host) = self.dropdown_host_mut(root_id) else {
            return Err(unavailable);
        };
        let outcome = host.dispatch(id, kind, source, detail)?;
        self.refresh_dropdown(root_id)?;
        Ok(outcome)
    }

    fn set_dropdown_focus_ready(
        &mut self,
        root_id: &str,
        trigger_id: &str,
        ready: bool,
    ) -> std::result::Result<(), BridgeError> {
        let unavailable = self.dropdown_unavailable_error();
        let Some(host) = self.dropdown_host_mut(root_id) else {
            return Err(unavailable);
        };
        host.set_focus_ready(trigger_id, ready)
    }

    fn focus_dropdown(
        &mut self,
        root_id: &str,
        trigger_id: &str,
        source: InputSource,
    ) -> std::result::Result<FocusOperationResult, BridgeError> {
        let unavailable = self.dropdown_unavailable_error();
        let Some(host) = self.dropdown_host_mut(root_id) else {
            return Err(unavailable);
        };
        let result = host.focus_with_source(trigger_id, source)?;
        self.refresh_dropdown(root_id)?;
        Ok(result)
    }

    fn blur_dropdown(
        &mut self,
        root_id: &str,
        trigger_id: &str,
    ) -> std::result::Result<(), BridgeError> {
        let unavailable = self.dropdown_unavailable_error();
        let Some(host) = self.dropdown_host_mut(root_id) else {
            return Err(unavailable);
        };
        host.blur(trigger_id, InputSource::Programmatic)?;
        self.refresh_dropdown(root_id)
    }

    fn dispatch_dropdown_key(
        &mut self,
        root_id: &str,
        key: &str,
    ) -> std::result::Result<DropdownDispatchOutcome, BridgeError> {
        let unavailable = self.dropdown_unavailable_error();
        let Some(host) = self.dropdown_host_mut(root_id) else {
            return Err(unavailable);
        };
        let outcome = host.dispatch_key(key)?;
        self.refresh_dropdown(root_id)?;
        Ok(outcome)
    }

    fn hover_card_trigger_id(&self, feature: &str) -> String {
        format!("hover-card:{feature}-trigger")
    }

    fn hover_card_content_id(&self, feature: &str) -> String {
        format!("hover-card:{feature}-content")
    }

    fn hover_card_snapshot_for(&self, feature: &str) -> Option<&HoverCardSnapshot> {
        self.hover_card_snapshots.get(feature)
    }

    fn dispatch_hover_card(
        &mut self,
        feature: &str,
        kind: InputKind,
        source: InputSource,
    ) -> std::result::Result<HoverCardDispatchOutcome, BridgeError> {
        let unavailable = self.hover_card_unavailable_error();
        let trigger_id = self.hover_card_trigger_id(feature);
        let Some(host) = self.hover_card_hosts.get_mut(feature) else {
            return Err(unavailable);
        };
        let outcome = host.dispatch(&trigger_id, kind, source, None)?;
        self.refresh_hover_card(feature)?;
        Ok(outcome)
    }

    fn set_hover_card_geometry(
        &mut self,
        feature: &str,
        anchor_rect: OverlayRect,
        floating_size: (f32, f32),
        viewport: OverlayRect,
    ) -> std::result::Result<(), BridgeError> {
        let unavailable = self.hover_card_unavailable_error();
        let Some(host) = self.hover_card_hosts.get_mut(feature) else {
            return Err(unavailable);
        };
        host.set_anchor_geometry(anchor_rect, floating_size, viewport)?;
        self.hover_card_snapshots
            .insert(feature.to_owned(), host.snapshot()?);
        Ok(())
    }

    fn advance_all_hover_cards(
        &mut self,
        milliseconds: u64,
    ) -> std::result::Result<(), BridgeError> {
        if milliseconds == 0 {
            return Ok(());
        }
        let features: Vec<String> = self.hover_card_hosts.keys().cloned().collect();
        for feature in features {
            let unavailable = self.hover_card_unavailable_error();
            let Some(host) = self.hover_card_hosts.get_mut(&feature) else {
                return Err(unavailable);
            };
            host.advance_time(milliseconds)?;
            self.hover_card_snapshots.insert(feature, host.snapshot()?);
        }
        Ok(())
    }

    fn set_hover_focus_ready(
        &mut self,
        feature: &str,
        ready: bool,
    ) -> std::result::Result<(), BridgeError> {
        let unavailable = self.hover_card_unavailable_error();
        let trigger_id = self.hover_card_trigger_id(feature);
        let Some(host) = self.hover_card_hosts.get_mut(feature) else {
            return Err(unavailable);
        };
        host.set_focus_ready(&trigger_id, ready)
    }

    fn focus_hover_card(
        &mut self,
        feature: &str,
        source: InputSource,
    ) -> std::result::Result<FocusOperationResult, BridgeError> {
        let unavailable = self.hover_card_unavailable_error();
        let trigger_id = self.hover_card_trigger_id(feature);
        let Some(host) = self.hover_card_hosts.get_mut(feature) else {
            return Err(unavailable);
        };
        let result = host.focus_with_source(&trigger_id, source)?;
        self.refresh_hover_card(feature)?;
        Ok(result)
    }

    fn blur_hover_card(&mut self, feature: &str) -> std::result::Result<(), BridgeError> {
        let unavailable = self.hover_card_unavailable_error();
        let trigger_id = self.hover_card_trigger_id(feature);
        let Some(host) = self.hover_card_hosts.get_mut(feature) else {
            return Err(unavailable);
        };
        host.blur(&trigger_id, InputSource::Programmatic)?;
        self.refresh_hover_card(feature)
    }

    /// Any capability card currently interacting (Proto-owned hover/focus
    /// facts). The GUI frame pump advances the virtual clock only while at
    /// least one card is interacting, so the bridge delays evaluate against
    /// real time without any Rust semantic timer or show/hide state.
    fn hover_interacting(&self) -> bool {
        self.hover_card_snapshots.values().any(|snapshot| {
            snapshot
                .trigger
                .as_ref()
                .is_some_and(|trigger| trigger.hovered || trigger.focused)
                || snapshot
                    .content
                    .as_ref()
                    .is_some_and(|content| content.open || content.present)
        })
    }

    fn hover_card_unavailable_error(&self) -> BridgeError {
        BridgeError::Runtime {
            detail: self
                .hover_card_error
                .clone()
                .unwrap_or_else(|| "Proto UI Hover Card host is unavailable".to_owned()),
        }
    }

    fn refresh_hover_card(&mut self, feature: &str) -> std::result::Result<(), BridgeError> {
        let unavailable = self.hover_card_unavailable_error();
        let Some(host) = self.hover_card_hosts.get_mut(feature) else {
            return Err(unavailable);
        };
        self.hover_card_snapshots
            .insert(feature.to_owned(), host.snapshot()?);
        Ok(())
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

fn capability_detail_accessibility(capability: &lctrl_core::Capability) -> String {
    match capability.availability {
        Availability::Available => capability
            .detail
            .clone()
            .unwrap_or_else(|| "Capability is available on this platform".to_owned()),
        Availability::Limited => capability
            .detail
            .clone()
            .unwrap_or_else(|| "Capability is limited on this platform".to_owned()),
        Availability::Unavailable => capability
            .detail
            .clone()
            .unwrap_or_else(|| "Capability is unavailable on this platform".to_owned()),
    }
}

fn button_action(id: &str) -> Option<DashboardAction> {
    match id {
        "refresh-status" => Some(DashboardAction::Refresh),
        TUNING_PROFILE_SAVE_ID => Some(DashboardAction::SaveProfile),
        _ => None,
    }
}
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
    proto_surface::button_surface(
        id,
        label,
        state,
        None,
        cx,
        move |this, kind, source, detail| {
            this.dispatch_proto_with_detail(id, kind, source, detail);
        },
        move |this, source| this.handle_proto_click(id, action, source),
    )
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
    proto_surface::toggle_surface(
        id,
        label,
        &state,
        None,
        cx,
        move |this, kind, source, detail| {
            this.dispatch_proto_toggle(id, kind, source, detail);
        },
        move |this, source| {
            this.handle_proto_toggle(id, action, source);
        },
    )
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
fn dropdown_specs(root_id: &str) -> &'static [DropdownActionSpec] {
    match root_id {
        STATUS_ACTIONS_DROPDOWN_ROOT_ID => &STATUS_ACTION_SPECS,
        SYSTEM_ACTIONS_DROPDOWN_ROOT_ID => &SYSTEM_ACTION_SPECS,
        _ => &[],
    }
}

fn dropdown_item_ids(root_id: &str) -> &'static [&'static str] {
    match root_id {
        STATUS_ACTIONS_DROPDOWN_ROOT_ID => &STATUS_ACTIONS_DROPDOWN_ITEM_IDS,
        SYSTEM_ACTIONS_DROPDOWN_ROOT_ID => &SYSTEM_ACTIONS_DROPDOWN_ITEM_IDS,
        _ => &[],
    }
}

fn dropdown_content_id(root_id: &str) -> Option<&'static str> {
    match root_id {
        STATUS_ACTIONS_DROPDOWN_ROOT_ID => Some(STATUS_ACTIONS_DROPDOWN_CONTENT_ID),
        SYSTEM_ACTIONS_DROPDOWN_ROOT_ID => Some(SYSTEM_ACTIONS_DROPDOWN_CONTENT_ID),
        _ => None,
    }
}

fn dropdown_trigger_id(root_id: &str) -> Option<&'static str> {
    match root_id {
        STATUS_ACTIONS_DROPDOWN_ROOT_ID => Some(STATUS_ACTIONS_DROPDOWN_TRIGGER_ID),
        SYSTEM_ACTIONS_DROPDOWN_ROOT_ID => Some(SYSTEM_ACTIONS_DROPDOWN_TRIGGER_ID),
        _ => None,
    }
}

fn dropdown_trigger_view(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    root_id: &'static str,
    trigger_id: &'static str,
    label: &'static str,
) -> Stateful<Div> {
    let Some(snapshot) = dashboard
        .proto
        .dropdown_snapshot(root_id)
        .and_then(|snapshot| snapshot.trigger.as_ref())
    else {
        return unavailable_button(trigger_id, label);
    };
    let Some(focus_handle) = dashboard.dropdown_focus_handles.get(trigger_id).cloned() else {
        return unavailable_button(trigger_id, label);
    };
    proto_surface::dropdown_trigger_surface(
        trigger_id,
        label,
        snapshot,
        focus_handle,
        cx,
        move |this, kind, source, detail| {
            this.forward_dropdown_input(root_id, trigger_id, kind, source, detail);
        },
        move |this, source| this.handle_dropdown_trigger(root_id, trigger_id, source),
    )
}

fn dropdown_item_view(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    root_id: &'static str,
    item_id: &'static str,
    action: DashboardAction,
) -> Stateful<Div> {
    let Some(item) = dashboard
        .proto
        .dropdown_snapshot(root_id)
        .and_then(|snapshot| snapshot.items.iter().find(|item| item.id == item_id))
    else {
        return unavailable_button(item_id, item_id);
    };
    proto_surface::dropdown_item_surface(
        item_id,
        item,
        cx,
        move |this, kind, source, detail| {
            this.forward_dropdown_input(root_id, item_id, kind, source, detail);
        },
        move |this, source| {
            this.handle_dropdown_item(root_id, item_id, action, source);
        },
        move |this, key, _, _| this.handle_dropdown_key(root_id, key),
    )
}

fn dropdown_group_view(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    root_id: &'static str,
    trigger_id: &'static str,
    trigger_label: &'static str,
) -> Stateful<Div> {
    let trigger = dropdown_trigger_view(dashboard, cx, root_id, trigger_id, trigger_label);
    let Some(content_id) = dropdown_content_id(root_id) else {
        return trigger;
    };
    let Some(content) = dashboard
        .proto
        .dropdown_snapshot(root_id)
        .and_then(|snapshot| snapshot.content.as_ref())
    else {
        return trigger;
    };
    let ids = dropdown_item_ids(root_id);
    let specs = dropdown_specs(root_id);
    let mut items = Vec::new();
    if let Some(snapshot) = dashboard.proto.dropdown_snapshot(root_id) {
        for (index, item) in snapshot.items.iter().enumerate() {
            let Some(item_id) = ids.get(index).copied() else {
                continue;
            };
            let Some(spec) = specs.get(index) else {
                continue;
            };
            if item.id != item_id {
                continue;
            }
            items.push(dropdown_item_view(
                dashboard,
                cx,
                root_id,
                item_id,
                spec.action,
            ));
        }
    }
    let mut content_element = proto_surface::dropdown_content_element(content_id, content);
    content_element = content_element.children(items);
    div()
        .id(format!("{trigger_id}-group"))
        .flex()
        .flex_col()
        .gap_1()
        .child(trigger)
        .child(content_element)
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
        dropdown_group_view(
            dashboard,
            cx,
            STATUS_ACTIONS_DROPDOWN_ROOT_ID,
            STATUS_ACTIONS_DROPDOWN_TRIGGER_ID,
            "STATUS ACTIONS",
        ),
        dropdown_group_view(
            dashboard,
            cx,
            SYSTEM_ACTIONS_DROPDOWN_ROOT_ID,
            SYSTEM_ACTIONS_DROPDOWN_TRIGGER_ID,
            "SYSTEM ACTIONS",
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
    proto_surface::select_trigger_surface(
        trigger_id,
        label,
        snapshot,
        cx,
        move |this, kind, source, detail| {
            this.forward_select_input(root_id, trigger_id, kind, source, detail);
        },
        move |this, source| this.handle_select_trigger(root_id, trigger_id, source),
    )
}

fn selector_group_view(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    root_id: &'static str,
    trigger_id: &'static str,
    value_id: &'static str,
    content_id: &'static str,
    label: &'static str,
) -> Stateful<Div> {
    let trigger = selector_trigger_view(dashboard, cx, root_id, trigger_id, label);
    let Some(snapshot) = dashboard.proto.select_snapshot(root_id) else {
        return trigger;
    };
    let value = snapshot
        .value
        .as_ref()
        .map(|value| proto_surface::select_value_element(value_id, value));
    let mut content = snapshot.content.as_ref().map_or_else(
        || div().id(content_id),
        |state| proto_surface::select_content_element(content_id, state),
    );
    let items = snapshot.items.iter().map(|item| {
        let item_id = item.id.clone();
        let dispatch_id = item_id.clone();
        let commit_id = item_id.clone();
        proto_surface::select_item_surface(
            item_id,
            item,
            cx,
            move |this, kind, source, detail| {
                this.forward_select_input(root_id, &dispatch_id, kind, source, detail);
            },
            move |this, source| {
                this.forward_select_input(
                    root_id,
                    &commit_id,
                    InputKind::PressCommit,
                    source,
                    None,
                );
            },
        )
    });
    content = content.children(items);
    let mut group = div()
        .id(format!("{trigger_id}-group"))
        .flex()
        .flex_col()
        .gap_1()
        .child(trigger);
    if let Some(value) = value {
        group = group.child(value);
    }
    group.child(content)
}

/// Disabled Select surfaces for performance mode, tuning profile, and power
/// scheme. The hardware snapshot has no typed current readback for these
/// channels yet, so the selectors remain honestly disabled instead of
/// inventing state; enabling them is the final-composition migration.
fn selectors_strip(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> impl IntoElement {
    div().flex().flex_row().flex_wrap().gap_2().children([
        selector_group_view(
            dashboard,
            cx,
            PERFORMANCE_MODE_SELECT_ROOT_ID,
            PERFORMANCE_MODE_SELECT_TRIGGER_ID,
            PERFORMANCE_MODE_SELECT_VALUE_ID,
            PERFORMANCE_MODE_SELECT_CONTENT_ID,
            "Performance mode",
        ),
        selector_group_view(
            dashboard,
            cx,
            TUNING_PROFILE_SELECT_ROOT_ID,
            TUNING_PROFILE_SELECT_TRIGGER_ID,
            TUNING_PROFILE_SELECT_VALUE_ID,
            TUNING_PROFILE_SELECT_CONTENT_ID,
            "Tuning profile",
        ),
        selector_group_view(
            dashboard,
            cx,
            POWER_SCHEME_SELECT_ROOT_ID,
            POWER_SCHEME_SELECT_TRIGGER_ID,
            POWER_SCHEME_SELECT_VALUE_ID,
            POWER_SCHEME_SELECT_CONTENT_ID,
            "Power scheme",
        ),
    ])
}

fn profile_dialog_trigger_view(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
) -> Stateful<Div> {
    let Some(snapshot) = dashboard.proto.profile_dialog_snapshot() else {
        return unavailable_button(TUNING_PROFILE_APPLY_ID, "APPLY PROFILE UNAVAILABLE");
    };
    let Some(trigger) = snapshot.trigger.as_ref() else {
        return unavailable_button(TUNING_PROFILE_APPLY_ID, "APPLY PROFILE UNAVAILABLE");
    };
    let Some(focus_handle) = dashboard
        .dialog_focus_handles
        .get(TUNING_PROFILE_APPLY_ID)
        .cloned()
    else {
        return unavailable_button(TUNING_PROFILE_APPLY_ID, "APPLY PROFILE UNAVAILABLE");
    };
    proto_surface::dialog_trigger_surface(
        TUNING_PROFILE_APPLY_ID,
        trigger,
        focus_handle,
        cx,
        move |this, kind, source, detail| {
            this.forward_dialog_input(TUNING_PROFILE_APPLY_ID, kind, source, detail);
        },
        move |this, source| {
            this.open_profile_dialog(source);
        },
        move |this, key, _, _| {
            if key.eq_ignore_ascii_case("escape") {
                this.handle_dialog_key(PROFILE_DIALOG_CONTENT_ID, "Escape");
            }
        },
    )
}

fn bios_dialog_trigger_view(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> Stateful<Div> {
    let Some(snapshot) = dashboard.proto.bios_dialog_snapshot() else {
        return unavailable_button(BIOS_DIALOG_TRIGGER_ID, "BIOS WRITE UNAVAILABLE");
    };
    let Some(trigger) = snapshot.trigger.as_ref() else {
        return unavailable_button(BIOS_DIALOG_TRIGGER_ID, "BIOS WRITE UNAVAILABLE");
    };
    let Some(focus_handle) = dashboard
        .dialog_focus_handles
        .get(BIOS_DIALOG_TRIGGER_ID)
        .cloned()
    else {
        return unavailable_button(BIOS_DIALOG_TRIGGER_ID, "BIOS WRITE UNAVAILABLE");
    };
    proto_surface::dialog_trigger_surface(
        BIOS_DIALOG_TRIGGER_ID,
        trigger,
        focus_handle,
        cx,
        move |this, kind, source, detail| {
            this.forward_dialog_input(BIOS_DIALOG_TRIGGER_ID, kind, source, detail);
        },
        move |this, source| {
            this.open_bios_dialog(source);
        },
        move |this, key, _, _| {
            if key.eq_ignore_ascii_case("escape") {
                this.handle_dialog_key(BIOS_DIALOG_CONTENT_ID, "Escape");
            }
        },
    )
}
fn dialog_close_view(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    snapshot: &DialogSnapshot,
    id: &'static str,
    bios: bool,
    confirm: bool,
) -> Stateful<Div> {
    let Some(close) = snapshot.closes.iter().find(|close| close.id == id) else {
        return unavailable_button(id, "UNAVAILABLE");
    };
    let focus_handle = dashboard.dialog_focus_handles.get(id).cloned();
    proto_surface::dialog_close_surface(
        id,
        close,
        focus_handle,
        cx,
        move |this, kind, source, detail| {
            this.forward_dialog_input(id, kind, source, detail);
        },
        move |this, _source| {
            if bios {
                if confirm {
                    this.confirm_bios_dialog();
                } else {
                    this.cancel_bios_dialog();
                }
            } else if confirm {
                this.confirm_profile_dialog();
            } else {
                this.cancel_profile_dialog();
            }
        },
    )
}

fn dialog_surface_view(
    dashboard: &Dashboard,
    cx: &mut Context<Dashboard>,
    snapshot: &DialogSnapshot,
    bios: bool,
) -> Stateful<Div> {
    let (content_id, title_id, description_id, header_id, footer_id, cancel_id, confirm_id) =
        if bios {
            (
                BIOS_DIALOG_CONTENT_ID,
                BIOS_DIALOG_TITLE_ID,
                BIOS_DIALOG_DESCRIPTION_ID,
                BIOS_DIALOG_HEADER_ID,
                BIOS_DIALOG_FOOTER_ID,
                BIOS_DIALOG_CANCEL_ID,
                BIOS_DIALOG_CONFIRM_ID,
            )
        } else {
            (
                PROFILE_DIALOG_CONTENT_ID,
                PROFILE_DIALOG_TITLE_ID,
                PROFILE_DIALOG_DESCRIPTION_ID,
                PROFILE_DIALOG_HEADER_ID,
                PROFILE_DIALOG_FOOTER_ID,
                PROFILE_DIALOG_CANCEL_ID,
                PROFILE_DIALOG_CONFIRM_ID,
            )
        };
    let Some(content) = snapshot.content.as_ref() else {
        return unavailable_button(content_id, "DIALOG CONTENT UNAVAILABLE");
    };
    let focus_handle = dashboard.dialog_focus_handles.get(content_id);
    let mut content_element =
        proto_surface::dialog_content_element(content_id, content, focus_handle);
    let route_id = content_id;
    content_element =
        proto_surface::dialog_content_key_surface(content_element, cx, move |this, key, _, _| {
            let key = if key.eq_ignore_ascii_case("escape") {
                "Escape"
            } else {
                key
            };
            this.handle_dialog_key(route_id, key);
        });
    if let Some(header) = snapshot.header.as_ref() {
        let mut header_element = proto_surface::dialog_header_element(header_id, header);
        if let Some(title) = snapshot.title.as_ref() {
            header_element =
                header_element.child(proto_surface::dialog_title_element(title_id, title));
        }
        if let Some(description) = snapshot.description.as_ref() {
            header_element = header_element.child(proto_surface::dialog_description_element(
                description_id,
                description,
            ));
        }
        content_element = content_element.child(header_element);
    }
    if let Some(footer) = snapshot.footer.as_ref() {
        let mut footer_element = proto_surface::dialog_footer_element(footer_id, footer);
        footer_element = footer_element
            .child(dialog_close_view(
                dashboard, cx, snapshot, cancel_id, bios, false,
            ))
            .child(dialog_close_view(
                dashboard, cx, snapshot, confirm_id, bios, true,
            ));
        content_element = content_element.child(footer_element);
    }
    let mut root = div().id(if bios {
        BIOS_DIALOG_ROOT_ID
    } else {
        PROFILE_DIALOG_ROOT_ID
    });
    if let Some(mask) = snapshot.mask.as_ref() {
        let mask_id = if bios {
            BIOS_DIALOG_MASK_ID
        } else {
            PROFILE_DIALOG_MASK_ID
        };
        root = root.child(proto_surface::dialog_mask_element(mask_id, mask));
    }
    root.child(content_element)
}

fn profile_dialog_view(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> Stateful<Div> {
    dashboard.proto.profile_dialog_snapshot().map_or_else(
        || unavailable_button(PROFILE_DIALOG_ROOT_ID, "PROFILE CONFIRMATION UNAVAILABLE"),
        |snapshot| dialog_surface_view(dashboard, cx, snapshot, false),
    )
}

fn bios_dialog_view(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> Stateful<Div> {
    dashboard.proto.bios_dialog_snapshot().map_or_else(
        || unavailable_button(BIOS_DIALOG_ROOT_ID, "BIOS CONFIRMATION UNAVAILABLE"),
        |snapshot| dialog_surface_view(dashboard, cx, snapshot, true),
    )
}

fn bios_panel(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(bios_dialog_trigger_view(dashboard, cx))
        .child(bios_dialog_view(dashboard, cx))
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child("Writes remain guarded by the CLI --yes safety path and typed readback."),
        )
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
    let apply = profile_dialog_trigger_view(dashboard, cx);
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
        .child(profile_dialog_view(dashboard, cx))
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

/// Proto component families exercised by the Sailbreak dashboard dogfood.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DogfoodFamily {
    Button,
    Toggle,
    Switch,
    Checkbox,
    Tabs,
    Select,
    Dropdown,
    Dialog,
    Textarea,
    HoverCard,
    Separator,
}

impl DogfoodFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Toggle => "Toggle",
            Self::Switch => "Switch",
            Self::Checkbox => "Checkbox",
            Self::Tabs => "Tabs",
            Self::Select => "Select",
            Self::Dropdown => "Dropdown",
            Self::Dialog => "Dialog",
            Self::Textarea => "Textarea",
            Self::HoverCard => "Hover Card",
            Self::Separator => "Separator",
        }
    }
}

/// One resolved semantic surface in the executable dogfood inventory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DogfoodSurface {
    pub family: DogfoodFamily,
    pub id: String,
    pub label: String,
    pub disabled: bool,
    pub present: bool,
    /// `None` means the current boolean value is intentionally unknown.
    pub checked: Option<bool>,
    pub active: Option<bool>,
    pub unavailable_reason: Option<String>,
}

/// Resolved inventory and selected/presence facts from the actual Proto hosts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DogfoodInventory {
    families: BTreeSet<DogfoodFamily>,
    pub surfaces: Vec<DogfoodSurface>,
    pub selected_tab: Option<String>,
    pub present_content_ids: BTreeSet<String>,
    pub modal_blocking: bool,
}

impl DogfoodInventory {
    #[must_use]
    pub fn has_family(&self, family: DogfoodFamily) -> bool {
        self.families.contains(&family)
    }

    pub fn families(&self) -> impl Iterator<Item = &DogfoodFamily> {
        self.families.iter()
    }

    #[must_use]
    pub fn surface(&self, family: DogfoodFamily, id: &str) -> Option<&DogfoodSurface> {
        self.surfaces
            .iter()
            .find(|surface| surface.family == family && surface.id == id)
    }
}

/// A small executable controller for contract tests and headless dogfood.
/// It owns the same Dashboard composition used by the desktop executable;
/// no alternate component state or HAL path is introduced.
pub struct DogfoodSession {
    dashboard: Dashboard,
}

impl DogfoodSession {
    #[must_use]
    pub fn with_controller(
        snapshot: DashboardSnapshot,
        controller: Arc<dyn GuiController>,
    ) -> Self {
        Self {
            dashboard: Dashboard::with_controller(snapshot, controller),
        }
    }

    #[must_use]
    pub fn inventory(&self) -> DogfoodInventory {
        self.dashboard.dogfood_inventory()
    }

    #[must_use]
    pub fn activate_button(&mut self, id: &str) -> bool {
        let Some(action) = button_action(id) else {
            return false;
        };
        let before = self
            .dashboard
            .proto
            .button(id)
            .map(|state| state.click_count);
        self.dashboard
            .handle_proto_click(id, action, InputSource::Accessibility);
        let after = self
            .dashboard
            .proto
            .button(id)
            .map(|state| state.click_count);
        matches!((before, after), (Some(before), Some(after)) if after == before + 1)
    }

    #[must_use]
    pub fn activate_toggle(&mut self, id: &str) -> bool {
        if id != PERFORMANCE_PREVIEW_TOGGLE_ID {
            return false;
        }
        self.dashboard.handle_proto_toggle(
            id,
            DashboardAction::RunCommand(PERFORMANCE_DRY_RUN_COMMAND),
            InputSource::Accessibility,
        )
    }

    /// Dispatch a commit through the disabled, unknown-state Switch host.
    /// Returns whether Proto UI emitted a checked-state change.
    #[must_use]
    pub fn activate_switch(&mut self) -> bool {
        self.dashboard
            .proto
            .dispatch_switch(
                CAPABILITY_SWITCH_ID,
                InputKind::PressCommit,
                InputSource::Accessibility,
                None,
            )
            .is_ok_and(|outcome| outcome.checked_change_count == 1)
    }

    /// Dispatch a commit through the disabled, unknown-state Checkbox host.
    /// Returns whether Proto UI emitted a checked-state change.
    #[must_use]
    pub fn activate_checkbox(&mut self) -> bool {
        self.dashboard
            .proto
            .dispatch_checkbox(
                CAPABILITY_CHECKBOX_ID,
                InputKind::PressCommit,
                InputSource::Accessibility,
                None,
            )
            .is_ok_and(|outcome| outcome.checked_change_count == 1)
    }

    #[must_use]
    pub fn activate_dropdown_item(&mut self, root_id: &str, item_id: &str) -> bool {
        let Some(spec) = dropdown_specs(root_id)
            .iter()
            .find(|spec| spec.id == item_id)
        else {
            return false;
        };
        let Some(trigger_id) = dropdown_trigger_id(root_id) else {
            return false;
        };
        let is_open = self
            .dashboard
            .proto
            .dropdown_snapshot(root_id)
            .map(|snapshot| snapshot.root.open)
            .unwrap_or(false);
        if !is_open {
            self.dashboard
                .handle_dropdown_trigger(root_id, trigger_id, InputSource::Accessibility);
        }
        self.dashboard.handle_dropdown_item(
            root_id,
            item_id,
            spec.action,
            InputSource::Accessibility,
        )
    }

    #[must_use]
    pub fn open_profile_confirmation(&mut self) -> bool {
        self.dashboard
            .open_profile_dialog(InputSource::Accessibility)
    }

    #[must_use]
    pub fn confirm_profile(&mut self) -> bool {
        self.dashboard.confirm_profile_dialog()
    }

    #[must_use]
    pub fn open_bios_confirmation(&mut self) -> bool {
        self.dashboard.open_bios_dialog(InputSource::Accessibility)
    }

    #[must_use]
    pub fn confirm_bios(&mut self) -> bool {
        self.dashboard.confirm_bios_dialog()
    }
}

/// Build the executable dogfood inventory from the same Proto host graph used
/// by [`Dashboard`].
#[must_use]
pub fn dogfood_inventory(snapshot: &DashboardSnapshot) -> DogfoodInventory {
    let session = DogfoodSession::with_controller(
        snapshot.clone(),
        Arc::new(StaticController {
            snapshot: snapshot.clone(),
        }),
    );
    session.inventory()
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
    dropdown_focus_handles: BTreeMap<&'static str, FocusHandle>,
    dropdown_focus_subscriptions: Vec<Subscription>,
    dialog_focus_handles: BTreeMap<&'static str, FocusHandle>,
    dialog_focus_subscriptions: Vec<Subscription>,
    hover_card_focus_handles: BTreeMap<String, FocusHandle>,
    hover_card_focus_subscriptions: Vec<Subscription>,
    hover_clock_last: Option<std::time::Instant>,
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
        let mut proto = ProtoUiState::new(&snapshot.capabilities);
        proto.attach_bios_dialog(controller.as_ref());
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
            dropdown_focus_handles: BTreeMap::new(),
            dropdown_focus_subscriptions: Vec::new(),
            dialog_focus_handles: BTreeMap::new(),
            dialog_focus_subscriptions: Vec::new(),
            hover_card_focus_handles: BTreeMap::new(),
            hover_card_focus_subscriptions: Vec::new(),
            hover_clock_last: None,
        }
    }

    fn ensure_tab_focus_handles(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.tab_focus_handles.is_empty() {
            return;
        }
        for id in SIDEBAR_BUTTON_IDS {
            let handle = cx.focus_handle();
            let (focus_subscription, blur_subscription) = proto_surface::focus_subscriptions(
                cx,
                window,
                &handle,
                move |this, source, cx| this.handle_tab_focus(id, source, cx),
                move |this, cx| this.handle_tab_blur(id, cx),
            );
            if let Err(error) = self.proto.set_tab_focus_ready(id, true) {
                self.snapshot.status_message = format!("Proto UI tab focus unavailable: {error}");
            }
            self.tab_focus_handles.insert(id, handle);
            self.tab_focus_subscriptions.push(focus_subscription);
            self.tab_focus_subscriptions.push(blur_subscription);
        }
    }

    fn ensure_dropdown_focus_handles(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        for (root_id, trigger_id) in [
            (
                STATUS_ACTIONS_DROPDOWN_ROOT_ID,
                STATUS_ACTIONS_DROPDOWN_TRIGGER_ID,
            ),
            (
                SYSTEM_ACTIONS_DROPDOWN_ROOT_ID,
                SYSTEM_ACTIONS_DROPDOWN_TRIGGER_ID,
            ),
        ] {
            if self.dropdown_focus_handles.contains_key(trigger_id) {
                continue;
            }
            let handle = cx.focus_handle();
            let (focus_subscription, blur_subscription) = proto_surface::focus_subscriptions(
                cx,
                window,
                &handle,
                move |this, source, _| this.handle_dropdown_focus(root_id, trigger_id, source),
                move |this, _| this.handle_dropdown_blur(root_id, trigger_id),
            );
            if let Err(error) = self
                .proto
                .set_dropdown_focus_ready(root_id, trigger_id, true)
            {
                self.snapshot.status_message =
                    format!("Proto UI Dropdown focus unavailable: {error}");
            }
            self.dropdown_focus_handles.insert(trigger_id, handle);
            self.dropdown_focus_subscriptions.push(focus_subscription);
            self.dropdown_focus_subscriptions.push(blur_subscription);
        }
    }

    fn ensure_dialog_focus_handles(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut ids = vec![
            TUNING_PROFILE_APPLY_ID,
            PROFILE_DIALOG_CANCEL_ID,
            PROFILE_DIALOG_CONFIRM_ID,
        ];
        if self.proto.bios_dialog_snapshot().is_some() {
            ids.extend([
                BIOS_DIALOG_TRIGGER_ID,
                BIOS_DIALOG_CANCEL_ID,
                BIOS_DIALOG_CONFIRM_ID,
            ]);
        }
        for id in ids {
            if self.dialog_focus_handles.contains_key(id) {
                continue;
            }
            let handle = cx.focus_handle();
            let (focus_subscription, blur_subscription) = proto_surface::focus_subscriptions(
                cx,
                window,
                &handle,
                move |this, source, _| this.handle_dialog_focus(id, source),
                move |this, _| this.handle_dialog_blur(id),
            );
            if let Err(error) = self.proto.set_dialog_focus_ready(id, true) {
                self.snapshot.status_message =
                    format!("Proto UI Dialog focus unavailable: {error}");
            }
            self.dialog_focus_handles.insert(id, handle);
            self.dialog_focus_subscriptions.push(focus_subscription);
            self.dialog_focus_subscriptions.push(blur_subscription);
        }
    }

    fn ensure_hover_card_focus_handles(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.proto.hover_card_error.is_some() {
            return;
        }
        let features: Vec<String> = self
            .snapshot
            .capabilities
            .features
            .keys()
            .cloned()
            .collect();
        for feature in features {
            if self.hover_card_focus_handles.contains_key(&feature) {
                continue;
            }
            let handle = cx.focus_handle();
            let focus_feature = feature.clone();
            let blur_feature = feature.clone();
            let (focus_subscription, blur_subscription) = proto_surface::focus_subscriptions(
                cx,
                window,
                &handle,
                move |this, source, _| this.handle_hover_focus(&focus_feature, source),
                move |this, _| this.handle_hover_blur(&blur_feature),
            );
            if let Err(error) = self.proto.set_hover_focus_ready(&feature, true) {
                self.snapshot.status_message =
                    format!("Proto UI Hover Card focus unavailable: {error}");
            }
            self.hover_card_focus_handles.insert(feature, handle);
            self.hover_card_focus_subscriptions.push(focus_subscription);
            self.hover_card_focus_subscriptions.push(blur_subscription);
        }
    }

    /// Pump the Proto Hover Card virtual clock with real frame deltas while
    /// any capability card is interacting. The pump only transports wall time
    /// into the existing bridge scheduler; Proto owns every open/close
    /// decision and the host installs no native semantic timer.
    fn pump_hover_card_clocks(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = std::time::Instant::now();
        if let Some(last) = self.hover_clock_last
            && let Some(delta) = now.checked_duration_since(last)
        {
            let millis = u64::try_from(delta.as_millis()).unwrap_or(u64::MAX);
            if millis > 0
                && let Err(error) = self.proto.advance_all_hover_cards(millis)
            {
                self.snapshot.status_message = format!("Proto UI Hover Card clock failed: {error}");
            }
        }
        if !self.proto.hover_interacting() {
            self.hover_clock_last = None;
            return;
        }
        self.hover_clock_last = Some(now);
        let weak = cx.entity().downgrade();
        window.on_next_frame(move |window, cx| {
            if let Some(entity) = weak.upgrade() {
                entity.update(cx, |this, cx| {
                    this.pump_hover_card_clocks(window, cx);
                    cx.notify();
                });
            }
        });
    }

    fn handle_hover_input(&mut self, feature: &str, kind: InputKind, source: InputSource) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
        if let Err(error) = self.proto.dispatch_hover_card(feature, kind, source) {
            self.snapshot.status_message = format!("Proto UI Hover Card input failed: {error}");
        }
    }

    fn handle_hover_focus(&mut self, feature: &str, source: InputSource) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
        match self.proto.focus_hover_card(feature, source) {
            Ok(FocusOperationResult::Accepted) => {}
            Ok(FocusOperationResult::NotReady) => {
                self.snapshot.status_message =
                    format!("Proto UI Hover Card focus is not ready: {feature}");
            }
            Ok(FocusOperationResult::Rejected) => {
                self.snapshot.status_message =
                    format!("Proto UI Hover Card focus was rejected: {feature}");
            }
            Err(error) => {
                self.snapshot.status_message = format!("Proto UI Hover Card focus failed: {error}");
            }
        }
    }

    fn handle_hover_blur(&mut self, feature: &str) {
        if let Err(error) = self.proto.blur_hover_card(feature) {
            self.snapshot.status_message = format!("Proto UI Hover Card blur failed: {error}");
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
        let blur_input = input.clone();
        let (textarea_focus_subscription, textarea_blur_subscription) =
            proto_surface::focus_subscriptions(
                cx,
                window,
                &focus_handle,
                move |this, source, cx| {
                    this.handle_textarea_focus(TUNING_PROFILE_EDITOR_ID, true, source, cx);
                },
                move |this, cx| {
                    let changed = blur_input.update(cx, |input, _| input.take_dirty());
                    this.handle_textarea_blur(TUNING_PROFILE_EDITOR_ID, changed, cx);
                },
            );
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
        if self.proto.dialog_modal_blocking() {
            return;
        }
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
        if self.proto.dialog_modal_blocking() {
            return;
        }
        if let Err(error) = self.proto.dispatch_textarea_focus(id, focused, source) {
            self.snapshot.status_message = format!("Profile editor focus failed: {error}");
        }
        self.sync_textarea_input(cx);
    }

    fn handle_textarea_blur(&mut self, id: &str, changed: bool, cx: &mut Context<Self>) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
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
        if self.proto.dialog_modal_blocking() {
            return;
        }
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
        if self.proto.dialog_modal_blocking() {
            return;
        }
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
        if self.proto.dialog_modal_blocking() {
            return;
        }
        match self.proto.press_tab(id, source) {
            Ok(true) => self.sync_active_section_from_tabs(),
            Ok(false) => {}
            Err(error) => {
                self.snapshot.status_message = format!("Proto UI tab activation failed: {error}");
            }
        }
    }
    fn forward_dropdown_input(
        &mut self,
        root_id: &str,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
        if let Err(error) = self
            .proto
            .dispatch_dropdown(root_id, id, kind, source, detail)
        {
            self.snapshot.status_message = format!("Proto UI Dropdown input failed: {error}");
        }
    }

    fn forward_select_input(
        &mut self,
        root_id: &str,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
        if let Err(error) = self
            .proto
            .dispatch_select(root_id, id, kind, source, detail)
        {
            self.snapshot.status_message = format!("Proto UI Select input failed: {error}");
        }
    }

    fn handle_select_trigger(&mut self, root_id: &str, id: &str, source: InputSource) {
        self.forward_select_input(root_id, id, InputKind::PressCommit, source, None);
    }

    fn forward_dialog_input(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        detail: Option<serde_json::Value>,
    ) {
        if !ProtoUiState::is_dialog_id(id) {
            return;
        }
        if let Err(error) = self.proto.dispatch_dialog_input(id, kind, source, detail) {
            self.snapshot.status_message = format!("Proto UI Dialog input failed: {error}");
        }
    }

    fn handle_dropdown_focus(&mut self, root_id: &str, trigger_id: &str, source: InputSource) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
        match self.proto.focus_dropdown(root_id, trigger_id, source) {
            Ok(FocusOperationResult::Accepted) => {}
            Ok(FocusOperationResult::NotReady) => {
                self.snapshot.status_message =
                    format!("Proto UI Dropdown focus is not ready: {trigger_id}");
            }
            Ok(FocusOperationResult::Rejected) => {
                self.snapshot.status_message =
                    format!("Proto UI Dropdown focus was rejected: {trigger_id}");
            }
            Err(error) => {
                self.snapshot.status_message = format!("Proto UI Dropdown focus failed: {error}");
            }
        }
    }
    fn forward_tabs_input(
        &mut self,
        id: &str,
        kind: InputKind,
        source: InputSource,
        _detail: Option<serde_json::Value>,
    ) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
        if let Err(error) = self.proto.dispatch_tab_input(id, kind, source) {
            self.snapshot.status_message = format!("Proto UI tab input failed: {error}");
        }
    }

    fn handle_dropdown_blur(&mut self, root_id: &str, trigger_id: &str) {
        if let Err(error) = self.proto.blur_dropdown(root_id, trigger_id) {
            self.snapshot.status_message = format!("Proto UI Dropdown blur failed: {error}");
        }
    }

    fn handle_dropdown_key(&mut self, root_id: &str, key: &str) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
        if let Err(error) = self.proto.dispatch_dropdown_key(root_id, key) {
            self.snapshot.status_message = format!("Proto UI Dropdown navigation failed: {error}");
        }
    }

    fn handle_dropdown_trigger(&mut self, root_id: &str, trigger_id: &str, source: InputSource) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
        if let Err(error) =
            self.proto
                .dispatch_dropdown(root_id, trigger_id, InputKind::PressCommit, source, None)
        {
            self.snapshot.status_message = format!("Proto UI Dropdown trigger failed: {error}");
        }
    }

    fn handle_dropdown_item(
        &mut self,
        root_id: &str,
        item_id: &str,
        action: DashboardAction,
        source: InputSource,
    ) -> bool {
        if self.proto.dialog_modal_blocking() {
            return false;
        }
        match self
            .proto
            .dispatch_dropdown(root_id, item_id, InputKind::PressCommit, source, None)
        {
            Ok(outcome) if outcome.item_select_count == 1 => {
                self.apply_action(action);
                true
            }
            Ok(_) => false,
            Err(error) => {
                self.snapshot.status_message =
                    format!("Proto UI Dropdown item activation failed: {error}");
                false
            }
        }
    }

    fn handle_dialog_focus(&mut self, id: &str, source: InputSource) {
        match self.proto.focus_dialog(id, source) {
            Ok(FocusOperationResult::Accepted) => {}
            Ok(FocusOperationResult::NotReady) => {
                self.snapshot.status_message = format!("Proto UI Dialog focus is not ready: {id}");
            }
            Ok(FocusOperationResult::Rejected) => {
                self.snapshot.status_message = format!("Proto UI Dialog focus was rejected: {id}");
            }
            Err(error) => {
                self.snapshot.status_message = format!("Proto UI Dialog focus failed: {error}");
            }
        }
    }

    fn handle_dialog_key(&mut self, id: &str, key: &str) {
        if let Err(error) = self.proto.dispatch_dialog_key(id, key) {
            self.snapshot.status_message = format!("Proto UI Dialog key failed: {error}");
        }
    }

    fn handle_dialog_blur(&mut self, id: &str) {
        if let Err(error) = self.proto.blur_dialog(id) {
            self.snapshot.status_message = format!("Proto UI Dialog blur failed: {error}");
        }
    }

    fn open_profile_dialog(&mut self, source: InputSource) -> bool {
        if self.proto.dialog_modal_blocking() {
            return false;
        }
        match self.proto.press_profile_trigger(source) {
            Ok(outcome) if outcome.trigger_press_count == 1 => {
                self.snapshot.status_message =
                    "Profile apply requires explicit confirmation".to_owned();
                true
            }
            Ok(_) => false,
            Err(error) => {
                self.snapshot.status_message = format!("Profile confirmation unavailable: {error}");
                false
            }
        }
    }

    fn cancel_profile_dialog(&mut self) {
        match self
            .proto
            .press_profile_close(PROFILE_DIALOG_CANCEL_ID, InputSource::Accessibility)
        {
            Ok(_) => {}
            Err(error) => {
                self.snapshot.status_message =
                    format!("Profile confirmation close failed: {error}");
            }
        }
    }

    fn confirm_profile_dialog(&mut self) -> bool {
        match self
            .proto
            .press_profile_close(PROFILE_DIALOG_CONFIRM_ID, InputSource::Accessibility)
        {
            Ok(outcome) if outcome.close_press_count == 1 => {
                self.apply_profile_confirmed();
                true
            }
            Ok(_) => false,
            Err(error) => {
                self.snapshot.status_message = format!("Profile confirmation failed: {error}");
                false
            }
        }
    }

    fn open_bios_dialog(&mut self, source: InputSource) -> bool {
        if self.proto.dialog_modal_blocking() {
            return false;
        }
        match self.proto.press_bios_trigger(source) {
            Ok(outcome) if outcome.trigger_press_count == 1 => {
                self.snapshot.status_message =
                    "BIOS write requires explicit confirmation".to_owned();
                true
            }
            Ok(_) => false,
            Err(error) => {
                self.snapshot.status_message = format!("BIOS confirmation unavailable: {error}");
                false
            }
        }
    }

    fn cancel_bios_dialog(&mut self) {
        match self
            .proto
            .press_bios_close(BIOS_DIALOG_CANCEL_ID, InputSource::Accessibility)
        {
            Ok(_) => {}
            Err(error) => {
                self.snapshot.status_message = format!("BIOS confirmation close failed: {error}");
            }
        }
    }

    fn confirm_bios_dialog(&mut self) -> bool {
        match self
            .proto
            .press_bios_close(BIOS_DIALOG_CONFIRM_ID, InputSource::Accessibility)
        {
            Ok(outcome) if outcome.close_press_count == 1 => {
                self.apply_bios_write();
                true
            }
            Ok(_) => false,
            Err(error) => {
                self.snapshot.status_message = format!("BIOS confirmation failed: {error}");
                false
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

    fn apply_profile_confirmed(&mut self) {
        let result = self.profile_source().and_then(|source| {
            let name = Self::validated_profile_name(&source)?;
            let saved_name = self.controller.save_profile(&source)?;
            if saved_name != name {
                return Err(LctrlError::InvalidArgument {
                    detail: format!("profile saved as {saved_name}, expected {name}"),
                });
            }
            let args = ["--yes", "tune", "profile", "apply", name.as_str()];
            self.controller.execute(&args)
        });
        self.snapshot.status_message = match result {
            Ok(message) => message.trim().to_owned(),
            Err(error) => {
                format!("Profile apply failed: {error}; recovery: sailbreak tune restore")
            }
        };
    }

    fn apply_bios_write(&mut self) {
        let Some(request) = self.proto.bios_write_request.clone() else {
            self.snapshot.status_message =
                "BIOS write unavailable: typed readback is not attached".to_owned();
            return;
        };
        let args = [
            "--yes",
            "bios",
            "set",
            request.setting.as_str(),
            request.requested_value.as_str(),
            "--save",
        ];
        self.snapshot.status_message = match self.controller.execute(&args) {
            Ok(message) => message.trim().to_owned(),
            Err(error) => {
                format!(
                    "BIOS write failed: {error}; recovery: {}",
                    request.recovery_command()
                )
            }
        };
    }

    fn apply_action(&mut self, action: DashboardAction) {
        if self.proto.dialog_modal_blocking() {
            return;
        }
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
        if self.proto.dialog_modal_blocking() && !ProtoUiState::is_dialog_id(id) {
            return false;
        }
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
        if self.proto.dialog_modal_blocking() {
            return false;
        }
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

    fn handle_proto_toggle(
        &mut self,
        id: &str,
        action: DashboardAction,
        source: InputSource,
    ) -> bool {
        let Ok(current) = self.proto.toggle(id) else {
            self.snapshot.status_message = "Proto UI toggle is unavailable".to_owned();
            return false;
        };
        let next_active = !current.active;
        if !self.dispatch_proto_toggle(id, InputKind::PressCommit, source, None) {
            return false;
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
        next_active && command_succeeded
    }

    fn dogfood_inventory(&self) -> DogfoodInventory {
        let mut families = BTreeSet::new();
        let mut surfaces = Vec::new();
        let mut present_content_ids = BTreeSet::new();
        let mut add =
            |family, id: &str, label: String, disabled, present, checked, active, reason| {
                families.insert(family);
                surfaces.push(DogfoodSurface {
                    family,
                    id: id.to_owned(),
                    label,
                    disabled,
                    present,
                    checked,
                    active,
                    unavailable_reason: reason,
                });
            };

        if self.proto.host.is_some() {
            add(
                DogfoodFamily::Button,
                "refresh-status",
                "REFRESH STATUS".to_owned(),
                false,
                true,
                None,
                None,
                None,
            );
            add(
                DogfoodFamily::Button,
                TUNING_PROFILE_SAVE_ID,
                "SAVE PROFILE".to_owned(),
                false,
                true,
                None,
                None,
                None,
            );
        }
        if let Ok(toggle) = self.proto.toggle(PERFORMANCE_PREVIEW_TOGGLE_ID) {
            add(
                DogfoodFamily::Toggle,
                PERFORMANCE_PREVIEW_TOGGLE_ID,
                toggle.label,
                toggle.disabled,
                true,
                None,
                Some(toggle.active),
                None,
            );
        }
        if let Some(switch) = self.proto.switch_snapshot() {
            add(
                DogfoodFamily::Switch,
                CAPABILITY_SWITCH_ID,
                switch.label.clone(),
                switch.disabled,
                true,
                None,
                None,
                switch
                    .disabled
                    .then(|| "typed DeviceState readback unavailable".to_owned()),
            );
        }
        if let Some(checkbox) = self.proto.checkbox_snapshot() {
            add(
                DogfoodFamily::Checkbox,
                CAPABILITY_CHECKBOX_ID,
                checkbox.label.clone(),
                checkbox.disabled,
                true,
                None,
                None,
                checkbox
                    .disabled
                    .then(|| "typed DeviceState readback unavailable".to_owned()),
            );
        }
        if let Some(tabs) = self.proto.tabs_snapshot() {
            if let Some(root) = tabs.root.as_ref() {
                add(
                    DogfoodFamily::Tabs,
                    &root.id,
                    "Dashboard sections".to_owned(),
                    false,
                    true,
                    None,
                    None,
                    None,
                );
                if !root.value.is_empty() {
                    present_content_ids.insert(root.value.clone());
                }
            }
            for trigger in &tabs.triggers {
                add(
                    DogfoodFamily::Tabs,
                    &trigger.id,
                    trigger.label.clone(),
                    trigger.disabled,
                    trigger.selected,
                    None,
                    None,
                    None,
                );
            }
            for content in &tabs.contents {
                if content.present {
                    present_content_ids.insert(content.id.clone());
                }
            }
        }
        for select in [
            self.proto.performance_select_snapshot.as_ref(),
            self.proto.tuning_select_snapshot.as_ref(),
            self.proto.power_select_snapshot.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            add(
                DogfoodFamily::Select,
                &select.root.id,
                select.root.label.clone(),
                select.root.disabled,
                true,
                None,
                None,
                select
                    .root
                    .disabled
                    .then(|| "typed value readback unavailable".to_owned()),
            );
            if let Some(content) = select.content.as_ref()
                && content.present
            {
                present_content_ids.insert(content.id.clone());
            }
        }
        for dropdown in [
            self.proto.status_dropdown_snapshot.as_ref(),
            self.proto.system_dropdown_snapshot.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            add(
                DogfoodFamily::Dropdown,
                &dropdown.root.id,
                dropdown.root.label.clone(),
                dropdown.root.disabled,
                true,
                None,
                None,
                None,
            );
            if let Some(content) = dropdown.content.as_ref()
                && content.present
            {
                present_content_ids.insert(content.id.clone());
            }
        }
        if let Some(profile) = self.proto.profile_dialog_snapshot.as_ref() {
            add(
                DogfoodFamily::Dialog,
                &profile.root.id,
                profile.root.label.clone(),
                profile.root.disabled,
                profile.root.open,
                None,
                None,
                None,
            );
            if profile
                .content
                .as_ref()
                .is_some_and(|content| content.present)
            {
                present_content_ids.insert(PROFILE_DIALOG_CONTENT_ID.to_owned());
            }
        }
        if let Some(bios) = self.proto.bios_dialog_snapshot.as_ref() {
            add(
                DogfoodFamily::Dialog,
                &bios.root.id,
                bios.root.label.clone(),
                bios.root.disabled,
                bios.root.open,
                None,
                None,
                None,
            );
        } else {
            add(
                DogfoodFamily::Dialog,
                BIOS_DIALOG_ROOT_ID,
                "Confirm BIOS write".to_owned(),
                true,
                false,
                None,
                None,
                Some("typed BIOS readback unavailable".to_owned()),
            );
        }
        if let Some(textarea) = self.proto.textarea_snapshot() {
            add(
                DogfoodFamily::Textarea,
                &textarea.id,
                textarea.label.clone(),
                textarea.disabled,
                true,
                None,
                None,
                None,
            );
        }
        for (feature, snapshot) in &self.proto.hover_card_snapshots {
            add(
                DogfoodFamily::HoverCard,
                &format!("hover-card:{feature}"),
                feature.clone(),
                false,
                snapshot
                    .content
                    .as_ref()
                    .is_some_and(|content| content.present),
                None,
                None,
                None,
            );
        }
        if self.proto.hover_card_snapshots.is_empty() {
            add(
                DogfoodFamily::HoverCard,
                EMPTY_HOVER_CARD_FEATURE_ID,
                "Capability detail unavailable".to_owned(),
                true,
                false,
                None,
                None,
                Some("no capability detail is reported".to_owned()),
            );
        }
        for id in [
            MAIN_HEADER_SEPARATOR_ID,
            ACTION_SEPARATOR_ID,
            CAPABILITY_SEPARATOR_ID,
        ] {
            if self.proto.separator_host.is_some() {
                add(
                    DogfoodFamily::Separator,
                    id,
                    "Layout separator".to_owned(),
                    false,
                    true,
                    None,
                    None,
                    None,
                );
            }
        }
        let selected_tab = self
            .proto
            .tabs_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.root.as_ref())
            .map(|root| root.value.clone());
        let modal_blocking = self
            .proto
            .profile_dialog_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot.root.open)
            || self
                .proto
                .bios_dialog_snapshot
                .as_ref()
                .is_some_and(|snapshot| snapshot.root.open);
        DogfoodInventory {
            families,
            surfaces,
            selected_tab,
            present_content_ids,
            modal_blocking,
        }
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
    let Some(focus_handle) = dashboard.tab_focus_handles.get(id).cloned() else {
        return unavailable_button(id, label);
    };
    proto_surface::tab_trigger_surface(
        id,
        state,
        focus_handle,
        cx,
        move |this, kind, source, detail| {
            this.forward_tabs_input(id, kind, source, detail);
        },
        move |this, source| this.handle_tab_press(id, source),
        move |this, key, window, cx| {
            if let Some(key) = proto_surface::tab_navigation_key(key) {
                this.handle_tab_key(key, window, cx);
            }
        },
    )
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
    } else if index == BIOS_SECTION_INDEX {
        panel = panel.child(bios_panel(dashboard, cx));
    } else {
        panel = panel
            .child(boolean_capability_rows(dashboard, cx))
            .child(capability_matrix(dashboard, cx))
            .child(telemetry_panel(&dashboard.snapshot));
    }
    panel
}

impl Render for Dashboard {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_tab_focus_handles(window, cx);
        self.ensure_dropdown_focus_handles(window, cx);
        self.ensure_dialog_focus_handles(window, cx);
        self.ensure_hover_card_focus_handles(window, cx);
        self.pump_hover_card_clocks(window, cx);
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

fn capability_row_unavailable(
    feature: &str,
    label: &str,
    color: gpui::Hsla,
    detail: &str,
) -> Stateful<Div> {
    div()
        .id(format!("capability-row-{feature}"))
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
                .child(feature.to_string()),
        )
        .child(
            div()
                .w(px(AVAILABILITY_COLUMN_WIDTH))
                .flex_none()
                .text_xs()
                .text_color(color)
                .child(label.to_string()),
        )
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(rgb(MUTED))
                .child(detail.to_string()),
        )
}

/// Capability rows use a Shadcn Hover Card for detail and evidence content.
///
/// The content remains a Rust-owned Slot subtree: the trigger, availability
/// marker, and the detail/evidence text are all projected through the Proto
/// snapshot, while the caller attaches the native Slot children to the content
/// portal. When the Hover Card host is unavailable the row keeps its plain
/// Rust text — an explicit unavailable state, never a local interactive
/// fallback.
fn boolean_capability_rows(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> Div {
    let switch = dashboard.proto.switch_snapshot().map(|state| {
        let state = state.clone();
        proto_surface::switch_surface(
            CAPABILITY_SWITCH_ID,
            &state,
            None,
            cx,
            true,
            move |this, kind, source, detail| {
                if let Err(error) =
                    this.proto
                        .dispatch_switch(CAPABILITY_SWITCH_ID, kind, source, detail)
                {
                    this.snapshot.status_message = format!("Proto UI Switch input failed: {error}");
                }
            },
            |_, _| {},
        )
    });
    let checkbox = dashboard.proto.checkbox_snapshot().map(|state| {
        let state = state.clone();
        proto_surface::checkbox_surface(
            CAPABILITY_CHECKBOX_ID,
            &state,
            None,
            cx,
            true,
            move |this, kind, source, detail| {
                if let Err(error) =
                    this.proto
                        .dispatch_checkbox(CAPABILITY_CHECKBOX_ID, kind, source, detail)
                {
                    this.snapshot.status_message =
                        format!("Proto UI Checkbox input failed: {error}");
                }
            },
            |_, _| {},
        )
    });
    let mut rows = div().flex().flex_row().flex_wrap().gap_2();
    if let Some(switch) = switch {
        rows = rows.child(
            div().flex().flex_col().gap_1().child(switch).child(
                div()
                    .text_xs()
                    .text_color(rgb(UNAVAILABLE))
                    .child("BOOLEAN CAPABILITY / TYPED READBACK UNAVAILABLE"),
            ),
        );
    }
    if let Some(checkbox) = checkbox {
        rows = rows.child(
            div().flex().flex_col().gap_1().child(checkbox).child(
                div()
                    .text_xs()
                    .text_color(rgb(UNAVAILABLE))
                    .child("BOOLEAN CAPABILITY / TYPED READBACK UNAVAILABLE"),
            ),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(rgb(MUTED))
                .child("BOOLEAN CAPABILITIES"),
        )
        .child(rows)
}

fn capability_matrix(dashboard: &Dashboard, cx: &mut Context<Dashboard>) -> impl IntoElement {
    let rows = dashboard
        .snapshot
        .capabilities
        .features
        .iter()
        .map(|(feature, capability)| {
            let (label, color) = availability_presentation(capability.availability);
            let detail = capability
                .detail
                .as_deref()
                .unwrap_or("No additional detail reported");
            let Some(trigger_snapshot) = dashboard
                .proto
                .hover_card_snapshot_for(feature)
                .and_then(|snapshot| snapshot.trigger.as_ref())
            else {
                return capability_row_unavailable(feature, label, color, detail);
            };
            let Some(content_snapshot) = dashboard
                .proto
                .hover_card_snapshot_for(feature)
                .and_then(|snapshot| snapshot.content.as_ref())
            else {
                return capability_row_unavailable(feature, label, color, detail);
            };
            let Some(focus_handle) = dashboard.hover_card_focus_handles.get(feature).cloned()
            else {
                return capability_row_unavailable(feature, label, color, detail);
            };

            let trigger_id = dashboard.proto.hover_card_trigger_id(feature);
            let content_id = dashboard.proto.hover_card_content_id(feature);
            let hover_feature = feature.clone();
            let geometry_feature = feature.clone();
            let trigger = proto_surface::hover_card_trigger_surface(
                trigger_id,
                feature.clone(),
                trigger_snapshot,
                Some(focus_handle),
                cx,
                move |this, kind, source, _| {
                    this.handle_hover_input(&hover_feature, kind, source);
                },
                move |this, anchor, floating_size, viewport| {
                    if let Err(error) = this.proto.set_hover_card_geometry(
                        &geometry_feature,
                        anchor,
                        floating_size,
                        viewport,
                    ) {
                        this.snapshot.status_message =
                            format!("Proto UI Hover Card placement failed: {error}");
                    }
                },
            );

            let slot_subtree = div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_sm()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(TEXT))
                        .child(feature.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(MUTED))
                        .child(detail.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(color)
                        .child(format!("STATE  {label}")),
                );

            let content = proto_surface::hover_card_content_element(content_id, content_snapshot)
                .child(slot_subtree);
            let content_present = content_snapshot.present;

            div()
                .id(format!("capability-row-{feature}"))
                .relative()
                .flex()
                .flex_row()
                .items_center()
                .gap_3()
                .px_3()
                .py_3()
                .border_b_1()
                .border_color(rgb(RULE))
                .child(div().w(px(MARKER_SIZE)).h(px(MARKER_SIZE)).bg(color))
                .child(trigger)
                .child(
                    div()
                        .w(px(AVAILABILITY_COLUMN_WIDTH))
                        .flex_none()
                        .text_xs()
                        .text_color(color)
                        .child(label),
                )
                .when(content_present, |row| row.child(content))
        });

    let feature_count = dashboard.snapshot.capabilities.features.len();
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
                        .child(format!("{feature_count} SIGNALS")),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .children(rows)
                .when(feature_count == 0, |this| {
                    this.child(
                        div()
                            .px_3()
                            .py_4()
                            .text_sm()
                            .text_color(rgb(UNAVAILABLE))
                            .child("No capability claims available; awaiting a platform HAL."),
                    )
                }),
        )
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
    fn dropdown_item_activation_routes_once_and_disabled_channels_stay_disabled() {
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

        let mut snapshot = DashboardSnapshot::unavailable(Platform::Linux, "initial");
        snapshot
            .capabilities
            .record("battery.status", Availability::Available, None)
            .expect("battery capability");
        snapshot
            .capabilities
            .record(
                "perf.temp",
                Availability::Unavailable,
                Some("temperature channel is unavailable".into()),
            )
            .expect("temperature capability");
        let controller = Arc::new(Recorder {
            calls: AtomicUsize::new(0),
        });
        let mut dashboard = Dashboard::with_controller(snapshot, controller.clone());

        let menu = dashboard
            .proto
            .dropdown_snapshot(STATUS_ACTIONS_DROPDOWN_ROOT_ID)
            .expect("status menu");
        assert!(
            !menu
                .items
                .iter()
                .find(|item| item.id == "battery-status")
                .expect("battery item")
                .disabled
        );
        assert!(
            menu.items
                .iter()
                .find(|item| item.id == "thermal-sensors")
                .expect("thermal item")
                .disabled
        );

        dashboard.handle_dropdown_trigger(
            STATUS_ACTIONS_DROPDOWN_ROOT_ID,
            STATUS_ACTIONS_DROPDOWN_TRIGGER_ID,
            InputSource::Accessibility,
        );
        dashboard.handle_dropdown_item(
            STATUS_ACTIONS_DROPDOWN_ROOT_ID,
            "battery-status",
            DashboardAction::RunCommand(BATTERY_STATUS_COMMAND),
            InputSource::Accessibility,
        );
        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
        assert_eq!(dashboard.snapshot.status_message, "executed battery status");

        dashboard.handle_dropdown_trigger(
            STATUS_ACTIONS_DROPDOWN_ROOT_ID,
            STATUS_ACTIONS_DROPDOWN_TRIGGER_ID,
            InputSource::Accessibility,
        );
        dashboard.handle_dropdown_item(
            STATUS_ACTIONS_DROPDOWN_ROOT_ID,
            "thermal-sensors",
            DashboardAction::RunCommand(THERMAL_SENSORS_COMMAND),
            InputSource::Accessibility,
        );
        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
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
                self.calls.fetch_add(1, Ordering::SeqCst);
                Ok(DashboardSnapshot::unavailable(Platform::Linux, "refreshed"))
            }

            fn execute(&self, args: &[&str]) -> Result<String> {
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

        dashboard.handle_proto_click(
            "refresh-status",
            DashboardAction::Refresh,
            InputSource::Accessibility,
        );

        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            dashboard
                .proto
                .button("refresh-status")
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
        dashboard.handle_proto_toggle(
            PERFORMANCE_PREVIEW_TOGGLE_ID,
            DashboardAction::RunCommand(PERFORMANCE_DRY_RUN_COMMAND),
            InputSource::Accessibility,
        );
        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
        assert!(
            dashboard
                .proto
                .toggle(PERFORMANCE_PREVIEW_TOGGLE_ID)
                .unwrap()
                .active
        );

        dashboard.handle_proto_toggle(
            PERFORMANCE_PREVIEW_TOGGLE_ID,
            DashboardAction::RunCommand(PERFORMANCE_DRY_RUN_COMMAND),
            InputSource::Accessibility,
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
    fn profile_confirmation_is_modal_and_commits_through_controller_once() {
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

        dashboard.open_profile_dialog(InputSource::Accessibility);
        assert!(dashboard.proto.dialog_modal_blocking());
        dashboard.handle_dialog_key(PROFILE_DIALOG_CONTENT_ID, "Escape");
        assert!(!dashboard.proto.dialog_modal_blocking());
        assert_eq!(controller.calls.load(Ordering::SeqCst), 0);
        dashboard.open_profile_dialog(InputSource::Keyboard);
        assert!(dashboard.proto.dialog_modal_blocking());
        dashboard.handle_proto_click(
            "battery-status",
            DashboardAction::RunCommand(BATTERY_STATUS_COMMAND),
            InputSource::Accessibility,
        );
        assert_eq!(controller.calls.load(Ordering::SeqCst), 0);

        dashboard.confirm_profile_dialog();
        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
        assert_eq!(controller.saves.load(Ordering::SeqCst), 1);
        assert_eq!(
            dashboard.snapshot.status_message,
            "executed --yes tune profile apply sailbreak-gui"
        );

        dashboard.confirm_profile_dialog();
        assert_eq!(controller.calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn bios_confirmation_stays_unavailable_without_typed_readback() {
        let dashboard = Dashboard::new(DashboardSnapshot::unavailable(Platform::Linux, "initial"));
        assert!(dashboard.proto.bios_dialog_snapshot().is_none());
        assert!(
            dashboard
                .proto
                .bios_dialog_error
                .as_deref()
                .is_some_and(|error| error.contains("readback") || error.contains("unsupported"))
        );
    }

    #[test]
    fn capability_hover_card_drives_delay_presence_and_detail_via_slot_projection() {
        let mut capabilities = CapabilitySet::new(Platform::Linux);
        capabilities
            .record(
                "battery.status",
                Availability::Available,
                Some("battery telemetry is live".into()),
            )
            .expect("battery capability");
        capabilities
            .record(
                "perf.temp",
                Availability::Unavailable,
                Some("temperature channel is unavailable".into()),
            )
            .expect("temperature capability");
        let snapshot = DashboardSnapshot {
            platform: Platform::Linux,
            hardware: HardwareInfo {
                product_name: Some("Test".into()),
                family: Some("Test".into()),
                bios_version: Some("0.1".into()),
            },
            capabilities,
            status_message: "test".into(),
        };
        let mut dashboard =
            Dashboard::with_controller(snapshot.clone(), Arc::new(StaticController { snapshot }));

        assert!(
            dashboard.proto.hover_card_error.is_none(),
            "hover card host should be available: {:?}",
            dashboard.proto.hover_card_error
        );

        // Dispatch pointer enter and advance the virtual clock by the
        // configured open delay (400 ms) — Proto owns the semantics.
        dashboard
            .proto
            .dispatch_hover_card(
                "battery.status",
                InputKind::PointerEnter,
                InputSource::Mouse,
            )
            .expect("pointer enter");
        dashboard
            .proto
            .advance_all_hover_cards(400)
            .expect("time advance");

        let battery = dashboard
            .proto
            .hover_card_snapshots
            .get("battery.status")
            .expect("battery snapshot");
        assert!(
            battery.content.as_ref().expect("battery content").present,
            "battery hover card is present after delay"
        );
        assert!(
            battery
                .content
                .as_ref()
                .expect("battery content")
                .slot
                .accessible_name
                .contains("battery telemetry is live"),
            "slot accessible_name carries the available detail text"
        );

        // The unavailable card is NOT open and its slot keeps honest text.
        let temp = dashboard
            .proto
            .hover_card_snapshots
            .get("perf.temp")
            .expect("perf.temp snapshot");
        assert!(
            !temp.content.as_ref().expect("temp content").present,
            "unavailable feature card is not open"
        );
        assert!(
            temp.content
                .as_ref()
                .expect("temp content")
                .slot
                .accessible_name
                .contains("unavailable"),
            "unavailable detail stays explicit"
        );

        dashboard
            .proto
            .dispatch_hover_card(
                "battery.status",
                InputKind::PointerLeave,
                InputSource::Mouse,
            )
            .expect("pointer leave");
        assert!(dashboard.proto.hover_interacting());
        dashboard
            .proto
            .advance_all_hover_cards(199)
            .expect("close delay before boundary");
        assert!(
            dashboard.proto.hover_card_snapshots["battery.status"]
                .content
                .as_ref()
                .expect("battery content")
                .present
        );
        dashboard
            .proto
            .advance_all_hover_cards(1)
            .expect("close delay boundary");
        assert!(
            !dashboard.proto.hover_card_snapshots["battery.status"]
                .content
                .as_ref()
                .expect("battery content")
                .present
        );
        assert!(!dashboard.proto.hover_interacting());
    }
}
