use std::sync::{Arc, Mutex};

use lctrl_core::{CapabilitySet, Platform, Result};
use sailbreak_gui::{DashboardSnapshot, DogfoodFamily, DogfoodSession, GuiController};

#[derive(Default)]
struct Recorder {
    executes: Mutex<Vec<Vec<String>>>,
    saves: Mutex<Vec<String>>,
}

impl GuiController for Recorder {
    fn refresh(&self) -> Result<DashboardSnapshot> {
        Ok(DashboardSnapshot::unavailable(Platform::Linux, "refreshed"))
    }

    fn execute(&self, args: &[&str]) -> Result<String> {
        self.executes
            .lock()
            .expect("execute recorder")
            .push(args.iter().map(|arg| (*arg).to_owned()).collect());
        Ok(format!("executed {}", args.join(" ")))
    }

    fn save_profile(&self, source: &str) -> Result<String> {
        let name = lctrl_tune::parse_profile_toml(source, lctrl_tune::ProfileOrigin::User)?
            .profile
            .name
            .as_str()
            .to_owned();
        self.saves
            .lock()
            .expect("save recorder")
            .push(source.to_owned());
        Ok(name)
    }
}

fn snapshot_with_capabilities() -> DashboardSnapshot {
    let mut capabilities = CapabilitySet::new(Platform::Linux);
    capabilities
        .record(
            "battery.status",
            lctrl_core::Availability::Available,
            Some("battery telemetry is live".to_owned()),
        )
        .expect("battery capability");
    capabilities
        .record(
            "perf.temp",
            lctrl_core::Availability::Unavailable,
            Some("temperature channel is unavailable".to_owned()),
        )
        .expect("temperature capability");
    DashboardSnapshot {
        platform: Platform::Linux,
        hardware: Default::default(),
        capabilities,
        status_message: "initial".to_owned(),
    }
}

#[test]
fn inventory_is_executable_and_contains_every_dogfood_family() {
    let session = DogfoodSession::with_controller(
        snapshot_with_capabilities(),
        Arc::new(Recorder::default()),
    );
    let inventory = session.inventory();

    for family in [
        DogfoodFamily::Button,
        DogfoodFamily::Toggle,
        DogfoodFamily::Switch,
        DogfoodFamily::Checkbox,
        DogfoodFamily::Tabs,
        DogfoodFamily::Select,
        DogfoodFamily::Dropdown,
        DogfoodFamily::Dialog,
        DogfoodFamily::Textarea,
        DogfoodFamily::HoverCard,
        DogfoodFamily::Separator,
    ] {
        assert!(inventory.has_family(family), "missing {family:?}");
    }
    assert!(
        inventory
            .surfaces
            .iter()
            .any(|surface| surface.family == DogfoodFamily::HoverCard)
    );
    assert_eq!(inventory.families().count(), 11);
    assert_eq!(inventory.selected_tab.as_deref(), Some("overview"));
    assert!(inventory.present_content_ids.contains("sidebar-content-0"));
}

#[test]
fn boolean_controls_are_disabled_and_unknown_without_typed_readback() {
    let recorder = Arc::new(Recorder::default());
    let mut session = DogfoodSession::with_controller(
        DashboardSnapshot::unavailable(Platform::Linux, "initial"),
        recorder.clone(),
    );
    let inventory = session.inventory();

    for family in [DogfoodFamily::Switch, DogfoodFamily::Checkbox] {
        let surface = inventory
            .surfaces
            .iter()
            .find(|surface| surface.family == family)
            .expect("boolean surface");
        assert!(surface.disabled);
        assert!(!surface.present || surface.unavailable_reason.is_some());
        assert!(surface.checked.is_none(), "unknown state must not be false");
        assert!(
            surface
                .unavailable_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("readback"))
        );
    }

    assert!(!session.activate_switch());
    assert!(!session.activate_checkbox());
    assert!(
        recorder
            .executes
            .lock()
            .expect("execute recorder")
            .is_empty()
    );
    for family in [DogfoodFamily::Switch, DogfoodFamily::Checkbox] {
        let surface = session
            .inventory()
            .surface(
                family,
                if family == DogfoodFamily::Switch {
                    "capability-readback-switch"
                } else {
                    "capability-readback-checkbox"
                },
            )
            .cloned()
            .expect("boolean surface after blocked commit");
        assert!(surface.checked.is_none());
        assert!(surface.disabled);
    }
}

#[test]
fn semantic_actions_are_exact_once_and_safety_boundaries_hold() {
    let recorder = Arc::new(Recorder::default());
    let mut session =
        DogfoodSession::with_controller(snapshot_with_capabilities(), recorder.clone());

    assert!(session.activate_dropdown_item("status-actions-dropdown", "battery-status"));
    assert!(session.activate_toggle("performance-preview"));
    assert!(!session.activate_toggle("performance-preview"));
    assert_eq!(
        *recorder.executes.lock().expect("execute recorder"),
        vec![
            vec!["battery".to_owned(), "status".to_owned()],
            vec![
                "--dry-run".to_owned(),
                "perf".to_owned(),
                "mode".to_owned(),
                "performance".to_owned(),
            ],
        ]
    );

    assert!(!session.activate_dropdown_item("status-actions-dropdown", "thermal-sensors"));
    assert_eq!(recorder.executes.lock().expect("execute recorder").len(), 2);

    assert!(session.open_profile_confirmation());
    assert!(session.inventory().modal_blocking);
    assert!(!session.activate_dropdown_item("status-actions-dropdown", "battery-status"));
    assert_eq!(recorder.executes.lock().expect("execute recorder").len(), 2);

    assert!(session.confirm_profile());
    assert!(!session.confirm_profile());
    assert!(!session.inventory().modal_blocking);
    assert_eq!(recorder.executes.lock().expect("execute recorder").len(), 3);
    assert_eq!(
        recorder.saves.lock().expect("save recorder").len(),
        1,
        "confirmation validates and saves exactly once"
    );
    assert_eq!(
        recorder.executes.lock().expect("execute recorder")[2],
        vec![
            "--yes".to_owned(),
            "tune".to_owned(),
            "profile".to_owned(),
            "apply".to_owned(),
            "sailbreak-gui".to_owned(),
        ]
    );
}

#[test]
fn disabled_selects_and_unavailable_bios_dialog_never_activate() {
    let recorder = Arc::new(Recorder::default());
    let session = DogfoodSession::with_controller(
        DashboardSnapshot::unavailable(Platform::Linux, "initial"),
        recorder.clone(),
    );
    let inventory = session.inventory();

    assert!(
        inventory
            .surfaces
            .iter()
            .filter(|surface| surface.family == DogfoodFamily::Select)
            .all(|surface| surface.disabled)
    );
    assert!(
        inventory
            .surfaces
            .iter()
            .any(|surface| surface.family == DogfoodFamily::Dialog
                && surface.id == "bios-write-dialog"
                && surface.unavailable_reason.is_some())
    );
    assert_eq!(recorder.executes.lock().expect("execute recorder").len(), 0);
}

#[test]
fn dashboard_native_event_wiring_stays_in_proto_surface() {
    let dashboard_source = include_str!("../src/lib.rs");
    let projection_source = include_str!("../src/proto_surface.rs");

    for native_hook in [
        ".on_click(",
        ".on_key_down(",
        ".on_key_up(",
        ".on_mouse_down(",
        ".on_mouse_up(",
        ".on_mouse_move(",
        ".on_hover(",
        ".track_focus(",
    ] {
        assert!(
            !dashboard_source.contains(native_hook),
            "dashboard installed native interaction hook {native_hook}"
        );
        assert!(
            projection_source.contains(native_hook),
            "projection layer no longer owns native hook {native_hook}"
        );
    }
}
