//! Task 17 conformance contract.
//!
//! The profile under `docs/proto-ui-gpui-profile.json` is the versioned
//! Sailbreak independent-dogfood record. These tests treat that file as the
//! executable contract: exact manifest linkage, exact matrix/family/prototype
//! sets, authority-tier separation, honest `limited` rows, the absence of
//! invented Dialog Portal/Overlay and native CloseIcon claims, and the
//! acknowledged semantic-fallback limitation list.
//!
//! The negative source guards intentionally mirror the existing
//! `sailbreak-gui/tests/dogfood_contract.rs` source-text convention: they
//! assert *what the bridge is not* (no local `definePrototype`, no deep
//! runtime import, no browser layout/paint APIs, no GPUI/closures/JS objects
//! on the Rust wire) rather than re-testing Runtime behavior.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;

const PROFILE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/proto-ui-gpui-profile.json"
);
const MANIFEST_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/proto-ui-bridge/upstream.json"
);

fn load(path: &str) -> Value {
    let source = std::fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("cannot read {path}: {error}");
    });
    serde_json::from_str(&source).unwrap_or_else(|error| panic!("cannot parse {path}: {error}"))
}

fn profile() -> Value {
    load(PROFILE_PATH)
}

fn manifest() -> Value {
    load(MANIFEST_PATH)
}

fn strings(value: &Value, field: &str) -> BTreeSet<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// Accept either a single string or an array of strings for the descriptive
/// matrix fields so the profile author can switch representation without
/// breaking the schema contract.
fn strings_or_single(value: &Value, field: &str) -> BTreeSet<String> {
    match value.get(field) {
        Some(Value::String(text)) => BTreeSet::from([text.clone()]),
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(str::to_owned))
            .collect(),
        _ => BTreeSet::new(),
    }
}

fn nonempty_text(value: &Value, field: &str, row: &str) {
    let text = value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .unwrap_or("");
    assert!(
        !text.is_empty(),
        "matrix row {row} must have a nonempty {field}"
    );
}

const MATRIX_IDS: &[&str] = &[
    "identity",
    "props",
    "state",
    "expose",
    "event",
    "focus",
    "template",
    "slot",
    "style",
    "svg",
    "a11y",
    "overlay",
    "lifecycle",
    "remount",
    "stale_rejection",
    "dispose",
    "linux",
    "windows",
];

const FAMILIES: &[&str] = &[
    "button",
    "toggle",
    "switch",
    "checkbox",
    "separator",
    "textarea",
    "tabs",
    "select",
    "dropdown",
    "dialog",
    "hover_card",
];

const PROTOTYPE_IDS: &[&str] = &[
    "P-SHADCN-BUTTON",
    "P-SHADCN-TOGGLE",
    "P-SHADCN-SWITCH",
    "P-SHADCN-SWITCH-THUMB",
    "P-SHADCN-CHECKBOX",
    "P-SHADCN-CHECKBOX-INDICATOR",
    "P-SHADCN-SEPARATOR",
    "P-SHADCN-TEXTAREA",
    "P-SHADCN-TABS",
    "P-SHADCN-TABS-LIST",
    "P-SHADCN-TABS-TRIGGER",
    "P-SHADCN-TABS-CONTENT",
    "P-SHADCN-SELECT",
    "P-SHADCN-SELECT-TRIGGER",
    "P-SHADCN-SELECT-VALUE",
    "P-SHADCN-SELECT-CONTENT",
    "P-SHADCN-SELECT-ITEM",
    "P-SHADCN-DROPDOWN-MENU",
    "P-SHADCN-DROPDOWN-MENU-TRIGGER",
    "P-SHADCN-DROPDOWN-MENU-CONTENT",
    "P-SHADCN-DROPDOWN-MENU-ITEM",
    "P-SHADCN-DIALOG",
    "P-SHADCN-DIALOG-TRIGGER",
    "P-SHADCN-DIALOG-MASK",
    "P-SHADCN-DIALOG-CONTENT",
    "P-SHADCN-DIALOG-TITLE",
    "P-SHADCN-DIALOG-DESCRIPTION",
    "P-SHADCN-DIALOG-HEADER",
    "P-SHADCN-DIALOG-FOOTER",
    "P-SHADCN-DIALOG-CLOSE",
    "P-SHADCN-HOVER-CARD",
    "P-SHADCN-HOVER-CARD-TRIGGER",
    "P-SHADCN-HOVER-CARD-CONTENT",
];

const FORBIDDEN_DIALOG_IDS: &[&str] = &[
    "P-SHADCN-DIALOG-PORTAL",
    "P-SHADCN-DIALOG-OVERLAY",
    "P-SHADCN-DIALOG-CLOSE-ICON",
];

fn workspace_root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn profile_is_the_independent_dogfood_record_without_official_claims() {
    let profile = profile();
    assert_eq!(profile["schema_version"], 1);
    assert_eq!(profile["profile"], "sailbreak-proto-ui-gpui");
    assert_eq!(profile["status"], "independent_dogfood");
    assert_eq!(profile["support_level"], "limited");
    assert_eq!(profile["official_proto_ui_adapter_claim"], false);
}

#[test]
fn profile_links_exactly_the_checked_in_upstream_manifest() {
    let profile = profile();
    let manifest = manifest();

    let profile_source = &profile["source"];
    assert_eq!(profile_source["repository"], manifest["repository"]);
    assert_eq!(profile_source["ref"], manifest["ref"]);
    assert_eq!(profile_source["commit"], manifest["commit"]);
    assert_eq!(
        profile_source["lockfile_sha256"],
        manifest["lockfile_sha256"]
    );
    assert_eq!(profile_source["bundle_sha256"], manifest["bundle_sha256"]);
    assert_eq!(
        profile_source["package_manager"],
        manifest["package_manager"]
    );

    for package in ["@proto.ui/runtime", "@proto.ui/prototypes-shadcn"] {
        let manifest_version = manifest["package_versions"]
            .as_array()
            .and_then(|versions| {
                versions
                    .iter()
                    .find(|entry| entry["name"] == package)
                    .and_then(|entry| entry["version"].as_str())
            })
            .expect("manifest records the package");
        assert_eq!(
            profile_source["package_versions"][package].as_str(),
            Some(manifest_version),
            "profile package version diverges from upstream.json for {package}"
        );
    }

    assert_eq!(
        profile["host"]["zed_revision"],
        "399258feeaf90ad8a3a208c99221ee87b6452f38"
    );
    assert_eq!(profile["host"]["gpui_version"], "0.2.2");
    assert_eq!(profile["host"]["rust_toolchain"], "1.95.0");
    let platform_scope = strings(&profile["host"], "platform_scope");
    assert!(platform_scope.contains("linux"));
    assert!(platform_scope.contains("windows"));
}

#[test]
fn matrix_has_exactly_the_required_rows_with_complete_fields() {
    let profile = profile();
    let rows = profile["matrix"].as_array().expect("matrix is an array");
    assert_eq!(rows.len(), MATRIX_IDS.len(), "matrix row count");

    let actual: BTreeSet<String> = rows
        .iter()
        .map(|row| row["id"].as_str().expect("row id").to_owned())
        .collect();
    let expected: BTreeSet<String> = MATRIX_IDS.iter().map(|id| (*id).to_string()).collect();
    assert_eq!(actual, expected, "matrix row ids");

    for row in rows {
        let id = row["id"].as_str().expect("row id");
        for field in [
            "status",
            "authority",
            "implementation",
            "test",
            "platform",
            "limitation",
        ] {
            let set = strings_or_single(row, field);
            assert!(
                !set.is_empty(),
                "matrix row {id} must have a nonempty {field}"
            );
        }
        nonempty_text(row, "status", id);
        nonempty_text(row, "authority", id);
        nonempty_text(row, "implementation", id);
        nonempty_text(row, "limitation", id);
    }
}

#[test]
fn families_and_prototype_ids_are_exact_and_consistent() {
    let profile = profile();

    let families = strings(&profile, "families");
    let expected_families: BTreeSet<String> = FAMILIES
        .iter()
        .map(|family| (*family).to_string())
        .collect();
    assert_eq!(families, expected_families, "families set");

    let prototype_ids = strings(&profile, "prototype_ids");
    let expected_ids: BTreeSet<String> = PROTOTYPE_IDS.iter().map(|id| (*id).to_string()).collect();
    assert_eq!(prototype_ids, expected_ids, "prototype_ids set");
    assert_eq!(prototype_ids.len(), 33, "prototype_ids count");

    let family_map = profile["family_prototype_ids"]
        .as_object()
        .expect("family_prototype_ids is an object");
    assert_eq!(family_map.len(), 11, "family_prototype_ids count");

    let mut mapped: BTreeSet<String> = BTreeSet::new();
    for (family, ids) in family_map {
        assert!(
            families.contains(family),
            "family {family} is mapped but not listed in families"
        );
        let id_set: BTreeSet<String> = ids
            .as_array()
            .expect("family prototype ids array")
            .iter()
            .filter_map(|id| id.as_str().map(str::to_owned))
            .collect();
        assert!(!id_set.is_empty(), "family {family} has no prototype ids");
        assert!(
            id_set.is_subset(&prototype_ids),
            "family {family} references an uncatalogued prototype id"
        );
        mapped.extend(id_set);
    }
    assert_eq!(
        mapped, prototype_ids,
        "family map must cover every prototype id"
    );
}

#[test]
fn dialog_admission_uses_mask_and_no_bogus_portal_overlay_or_native_close_icon() {
    let profile = profile();
    let prototype_ids = strings(&profile, "prototype_ids");

    for forbidden in FORBIDDEN_DIALOG_IDS {
        assert!(
            !prototype_ids.contains(*forbidden),
            "canonical prototype ids must not contain {forbidden}"
        );
    }

    assert!(prototype_ids.contains("P-SHADCN-DIALOG-MASK"));
    assert!(prototype_ids.contains("P-SHADCN-DIALOG-CLOSE"));

    let omission_ids: BTreeSet<String> = profile["omissions"]
        .as_array()
        .expect("omissions array")
        .iter()
        .map(|entry| entry["id"].as_str().expect("omission id").to_owned())
        .collect();
    assert!(omission_ids.contains("dialog_portal"));
    assert!(omission_ids.contains("dialog_overlay"));

    let close_icon = profile["uncataloged_capabilities"]
        .as_array()
        .expect("uncataloged_capabilities array")
        .iter()
        .find(|entry| entry["prototype_id"] == "P-SHADCN-DIALOG-CLOSE-ICON")
        .expect("CloseIcon is declared uncataloged");
    assert_eq!(close_icon["status"], "registry_only_native_unproven");
}

#[test]
fn evidence_tiers_separate_fake_host_from_native_and_desktop_proof() {
    let profile = profile();
    let evidence = &profile["evidence"];
    let tiers = evidence["authority_tiers"]
        .as_object()
        .expect("authority_tiers object");

    for tier in [
        "upstream_manifest",
        "upstream_runtime",
        "upstream_catalog",
        "upstream_types",
        "upstream_web_comparison",
        "upstream_portable_and_web_only",
        "local_bundle_tool",
        "runtime_fake_host",
        "host_capability",
        "native_gpui",
        "desktop_smoke",
        "windows_ci",
        "local_workspace",
    ] {
        let description = tiers[tier].as_str().unwrap_or_default();
        assert!(
            !description.is_empty(),
            "authority tier {tier} must be described"
        );
    }

    for tier in ["desktop_smoke", "windows_ci"] {
        let description = tiers[tier]
            .as_str()
            .unwrap_or_default()
            .to_ascii_lowercase();
        assert!(
            description.contains("not") || description.contains("never"),
            "tier {tier} must not overclaim"
        );
    }

    let observations = &evidence["observations"];
    assert_eq!(observations["desktop_smoke"]["status"], "not_observed");
    let windows_ci = &observations["windows_ci"];
    assert_eq!(windows_ci["status"], "passed_build_evidence");
    assert_eq!(
        windows_ci["run_url"],
        "https://github.com/sailbreak-oss/sailbreak/actions/runs/33722965284"
    );
    assert_eq!(
        windows_ci["head_sha"],
        "a4b8c3fc4c0232c9df9ecd4ecb41be9d8c58603f"
    );
    let jobs = windows_ci["jobs"].as_array().expect("CI jobs array");
    assert_eq!(jobs.len(), 2);
    assert!(jobs.iter().all(|job| job["conclusion"] == "success"));
    assert_eq!(
        windows_ci["rust_toolchain"],
        profile["host"]["rust_toolchain"]
    );
    assert_eq!(windows_ci["gpui_revision"], profile["host"]["zed_revision"]);
    assert_eq!(windows_ci["proto_ui_commit"], profile["source"]["commit"]);
    assert_eq!(
        windows_ci["bundle_sha256"],
        profile["source"]["bundle_sha256"]
    );
    assert!(
        observations["native_gpui"]["status"]
            .as_str()
            .unwrap_or_default()
            .contains("no_display_run")
    );

    let native_paths = strings(&evidence["paths"], "native_gpui");
    for path in [
        "crates/sailbreak-gui/src/proto_surface.rs",
        "crates/sailbreak-gui/tests/accessibility_contract.rs",
        "crates/sailbreak-gui/tests/dogfood_contract.rs",
    ] {
        assert!(native_paths.contains(path), "missing native path {path}");
    }
    assert!(
        evidence["paths"]["desktop_smoke"]
            .as_array()
            .expect("desktop_smoke paths array")
            .is_empty(),
        "no desktop smoke path may be recorded yet"
    );
}

#[test]
fn canonical_runs_record_observed_upstream_and_local_evidence() {
    let profile = profile();
    let runs = profile["canonical_runs"]
        .as_array()
        .expect("canonical_runs array");
    assert!(!runs.is_empty(), "canonical_runs must not be empty");

    let mut authorities: BTreeSet<&str> = BTreeSet::new();
    for run in runs {
        assert_eq!(run["status"], "passed");
        assert_eq!(run["exit_code"], 0);
        assert!(!run["command"].as_str().unwrap_or_default().is_empty());
        assert!(!run["summary"].as_str().unwrap_or_default().is_empty());
        let authority = run["authority"].as_str().expect("run authority");
        assert!(
            !authority.is_empty(),
            "canonical run must name an authority tier"
        );
        authorities.insert(authority);
    }

    let expected = [
        "upstream_portable_and_web_only",
        "upstream_runtime",
        "upstream_catalog",
        "upstream_types",
        "upstream_web_comparison",
        "local_bundle_tool",
        "local_workspace",
    ];
    for authority in expected {
        assert!(
            authorities.contains(authority),
            "missing canonical run authority {authority}"
        );
    }

    // The full portable suite is explicitly Web/portable-shaped evidence.
    let full = runs
        .iter()
        .find(|run| run["command"].as_str() == Some("corepack pnpm@10.32.1 test"))
        .expect("full upstream portable test run is recorded");
    assert_eq!(full["authority"], "upstream_portable_and_web_only");
}

#[test]
fn limited_rows_and_desktop_gaps_are_honest() {
    let profile = profile();
    let rows = profile["matrix"].as_array().expect("matrix array");
    for id in ["props", "state", "expose", "event"] {
        let row = rows
            .iter()
            .find(|row| row["id"] == id)
            .unwrap_or_else(|| panic!("missing matrix row {id}"));
        assert_eq!(row["status"], "limited", "row {id} must stay limited");
    }

    let linux = rows
        .iter()
        .find(|row| row["id"] == "linux")
        .expect("linux row");
    assert_eq!(linux["status"], "limited");
    let linux_limitation = linux["limitation"]
        .as_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(
        linux_limitation.contains("headless") || linux_limitation.contains("display"),
        "linux row must name the headless limitation"
    );

    let windows = rows
        .iter()
        .find(|row| row["id"] == "windows")
        .expect("windows row");
    assert_eq!(windows["status"], "limited");
    let windows_limitation = windows["limitation"]
        .as_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(
        windows_limitation.contains("ci") && windows_limitation.contains("not"),
        "windows row must say CI is not GUI/hardware proof"
    );
    assert_eq!(windows["authority"], "windows_ci");
    assert_eq!(linux["authority"], "desktop_smoke");
}

#[test]
fn semantic_fallback_paths_are_acknowledged_and_present() {
    let limitations = &profile()["limitations"];
    let paths = limitations["semantic_fallback_paths"]
        .as_array()
        .expect("semantic_fallback_paths array");
    assert!(!paths.is_empty(), "fallback paths must be enumerated");

    for entry in paths {
        let path = entry.as_str().expect("fallback path");
        let full = workspace_root().join(path);
        assert!(
            full.is_file(),
            "enumerated fallback path must exist: {path}"
        );
    }

    let policy = limitations["semantic_fallback_policy"]
        .as_str()
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(
        policy.contains("fallback") && policy.contains("acknowledged"),
        "fallback policy must be explicit and self-critical"
    );
}

#[test]
fn upstream_slices_are_hal_free_and_maintainer_gated() {
    let profile = profile();
    let slices = profile["upstream_slices"]
        .as_array()
        .expect("upstream_slices array");
    assert_eq!(slices.len(), 5, "five external upstream candidate slices");

    let mut ids = BTreeSet::new();
    for slice in slices {
        let id = slice["id"].as_str().expect("slice id");
        assert!(ids.insert(id.to_owned()), "duplicate slice id {id}");
        assert_eq!(slice["hal_free"], true, "slice {id} must be HAL-free");
        assert_eq!(
            slice["sailbreak_hardware_code"], false,
            "slice {id} must not carry Sailbreak hardware code"
        );
        nonempty_text(slice, "scope", id);
        nonempty_text(slice, "admission_gate", id);
    }
}

#[test]
fn bridge_source_uses_only_the_public_runtime_root_without_browser_apis() {
    let bridge = include_str!("../../../tools/proto-ui-bridge/src/index.ts");

    // The Runtime is imported from its public package root only.
    assert!(
        bridge.contains("from '@proto.ui/runtime'"),
        "bridge must import the Runtime from the public package root"
    );
    assert!(
        !bridge.contains("@proto.ui/runtime/"),
        "bridge must not deep-import into the Runtime package internals"
    );

    // No local prototype-definition machinery and no browser layout/paint
    // entry points may exist in the bridge source.
    for token in [
        "definePrototype",
        "getBoundingClientRect",
        "requestAnimationFrame",
        "getComputedStyle",
        "ResizeObserver",
        "IntersectionObserver",
        "document.",
        "window.",
    ] {
        assert!(
            !bridge.contains(token),
            "bridge source must not contain browser/layout/paint API {token}"
        );
    }
}

#[test]
fn notice_names_the_exact_pin_without_conformance_claims() {
    let notice = include_str!("../../../tools/proto-ui-bridge/NOTICE");
    assert!(
        notice.contains("8c6dadc00554ad89040a5f36eb2df56eb9ad3c17"),
        "NOTICE must name the exact pinned Proto-UI commit"
    );
    assert!(
        notice.contains("https://github.com/Proto-UI/Proto-UI"),
        "NOTICE must name the upstream source"
    );
    assert!(
        notice.to_ascii_lowercase().contains("mit"),
        "NOTICE must name the MIT license"
    );
    for claim in ["conformance", "a-gpui", "official adapter"] {
        assert!(
            !notice.to_ascii_lowercase().contains(claim),
            "NOTICE must not claim {claim}"
        );
    }
}

#[test]
fn rust_wire_is_data_only_without_gpui_closures_or_js_objects() {
    let protocol = include_str!("../src/protocol.rs");
    for token in [
        "use gpui",
        "gpui::",
        "rquickjs",
        "Function",
        "Closure",
        "JsValue",
        "js_sys",
        "wasm_bindgen",
        "dyn Fn",
        "Rc<",
        "Arc<",
    ] {
        assert!(
            !protocol.contains(token),
            "protocol.rs must not contain {token}"
        );
    }
    let quickjs = include_str!("../src/quickjs.rs");
    assert!(
        quickjs.contains("serde_json::from_str"),
        "the Rust side decodes JSON output"
    );
}
