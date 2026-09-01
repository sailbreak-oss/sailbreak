# Proto-UI GPUI Full Dogfood Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the merged Button slice into a complete Proto-UI main-HEAD Shadcn host for Sailbreak, with one independently tested component family at a time and no Node/Bun runtime dependency.

**Architecture:** Proto-UI `main` remains the semantic authority and is rebuilt into a recorded bundle before every slice. QuickJS executes the bundle in-process; Rust owns the data-only bridge, native GPUI projection, host capabilities, Slot content, and hardware-command boundary. The bridge is transport-neutral and never invokes JavaScript synchronously from layout, prepaint, or paint.

**Tech Stack:** Rust 2024, Rust 1.85+, GPUI 0.2.2 or a separately recorded Zed GPUI revision with AccessKit support, `rquickjs` 0.6.2, `serde`/`serde_json`, Bun 1.4 for build-time bundling, Proto-UI `main` source snapshots, and the existing Sailbreak HAL/CLI crates.

**Spec:** `docs/superpowers/specs/2026-08-31-proto-ui-gpui-full-dogfood-design.md`

## Global Constraints

- At the beginning of every task, read Proto-UI `main` and record its current SHA; the plan-authoring baseline is `9c8891ca22fefafb1346e6ebd02d1f80cae2ec24`.
- Every generated bundle records the exact Proto-UI SHA, package graph, source license notices, and bundle SHA-256.
- Proto-UI `main` changes are reviewed for API, semantic, and package-graph drift before the affected slice starts.
- QuickJS is embedded in the released binary; Node and Bun are build-time tools only and must never be launched by Sailbreak.
- Proto-UI Runtime is the only owner of component semantics, state transitions, lifecycle, and outward signals.
- Rust/GPUI owns only native host capabilities, layout, painting, Slot content, native focus/input, accessibility projection, and command callbacks.
- No `EventTarget`, `HTMLElement`, JS callback, GPUI `Entity`, `AnyElement`, or arbitrary host object crosses the Rust protocol.
- Durable IDs are restricted to semantic surfaces, sessions, view epochs, Slots, routes, focus targets, and accessibility objects.
- Every lossless semantic message is ordered and bounded; overflow is a fatal diagnostic, never silent loss.
- Every slice uses RED → GREEN TDD, focused tests, a real desktop smoke where a display exists, full workspace checks, and one reviewed commit before the next slice.
- No component is admitted as supported because it appears in the registry; support requires executable native evidence.
- Hardware mutations remain behind `sailbreak-cli` verified-write services and never run from Proto-UI callbacks directly.
- Main-snapshot Proto-UI code is never published under the stable `0.2.0` package identity.

---

## File Map

- `tools/proto-ui-bridge/`: source synchronization, package graph, Bun bundle generation, and third-party notices.
- `crates/proto-ui-gpui/src/protocol.rs`: wire identities, commands, events, snapshots, and validation.
- `crates/proto-ui-gpui/src/quickjs.rs`: QuickJS context, bundle loading, JSON dispatch, and bundle identity.
- `crates/proto-ui-gpui/src/runtime.rs`: generic Proto session ownership and lifecycle barrier.
- `crates/proto-ui-gpui/src/events.rs`: host-neutral event plans and independent leases.
- `crates/proto-ui-gpui/src/focus.rs`: opaque target readiness and native focus operations.
- `crates/proto-ui-gpui/src/template.rs`: Template/Slot/SVG projection and identity pruning.
- `crates/proto-ui-gpui/src/style.rs`: Shadcn token translation and unsupported-token diagnostics.
- `crates/proto-ui-gpui/src/overlay.rs`: portal, positioning, presence, dismissal, and lease ownership.
- `crates/proto-ui-gpui/src/components/`: one host-side family adapter per Proto-UI component family.
- `crates/sailbreak-gui/src/proto_surface.rs`: GPUI rendering and native event wiring for resolved host snapshots.
- `crates/sailbreak-gui/src/lib.rs`: dashboard composition and CLI action integration only; no semantic component state.
- `crates/proto-ui-gpui/tests/`: protocol, host capability, component, lifecycle, and stale-message contracts.
- `crates/sailbreak-gui/tests/` or `src/lib.rs` tests: dashboard consumer and command gateway contracts.

---

### Task 1: Synchronize Proto-UI main and build reproducible bundles

**Files:**
- Create: `tools/proto-ui-bridge/upstream.json`
- Create: `tools/proto-ui-bridge/scripts/build-bundle.mjs`
- Modify: `tools/proto-ui-bridge/package.json`
- Modify: `tools/proto-ui-bridge/bun.lock`
- Modify: `tools/proto-ui-bridge/NOTICE`
- Modify: `crates/proto-ui-gpui/assets/proto-ui-bridge.js`
- Test: `tools/proto-ui-bridge/scripts/build-bundle.test.mjs`

**Interfaces:**
- Produces `bun run sync-head -- --sha <40-hex-sha>` and `bun run bundle:check`.
- Produces `upstream.json` with `repository`, `ref`, `commit`, `package_manager`, `package_versions`, `bundle_sha256`, and `license_sources`.
- Produces the same `proto-ui-bridge.js` asset consumed by `QuickJsBridge`.

- [ ] **Step 1: Write the failing source-manifest test**

Create `build-bundle.test.mjs` with these assertions:

```js
import { readFile } from 'node:fs/promises';
import { test } from 'node:test';
import assert from 'node:assert/strict';

const manifest = JSON.parse(
  await readFile(new URL('../upstream.json', import.meta.url), 'utf8')
);

await test('source manifest records one exact Proto UI main commit', () => {
  assert.match(manifest.repository, /^https:\/\/github\.com\/Proto-UI\/Proto-UI\.git$/);
  assert.equal(manifest.ref, 'main');
  assert.match(manifest.commit, /^[0-9a-f]{40}$/);
  assert.equal(manifest.package_manager, 'pnpm@10.32.1');
  assert.match(manifest.bundle_sha256, /^sha256:[0-9a-f]{64}$/);
});
```

- [ ] **Step 2: Run the test and verify RED**

Run:

```bash
node --test tools/proto-ui-bridge/scripts/build-bundle.test.mjs
```

Expected: FAIL because the source manifest and bundle-check script do not exist.

- [ ] **Step 3: Implement exact-head synchronization**

Implement `sync-head` so it accepts only a 40-hex SHA, checks out that SHA in a temporary Proto-UI clone, runs the pinned `corepack pnpm@10.32.1 install --frozen-lockfile` and `corepack pnpm@10.32.1 build:packages`, copies the bridge entrypoint into the temporary workspace, and runs Bun from that workspace so package aliases resolve to the checked-out packages. The script must write the package versions and generated bundle digest to `upstream.json`; it must not read an import path from user input.

Use this exact command at the start of every later slice:

```bash
PROTO_UI_SHA=$(env -u http_proxy -u https_proxy -u no_proxy gh api repos/Proto-UI/Proto-UI/commits/main --jq .sha)
(cd tools/proto-ui-bridge && bun run sync-head -- --sha "$PROTO_UI_SHA")
(cd tools/proto-ui-bridge && bun run bundle:check)
```

- [ ] **Step 4: Implement bundle drift checking**

`bundle:check` must rebuild into a temporary output, compare the generated bytes with `crates/proto-ui-gpui/assets/proto-ui-bridge.js`, and fail with the recorded SHA and file paths when they differ. Add the Shadcn package's MIT third-party notice and retain Proto-UI package license URLs.

- [ ] **Step 5: Run the focused test and build check GREEN**

Run:

```bash
node --test tools/proto-ui-bridge/scripts/build-bundle.test.mjs
(cd tools/proto-ui-bridge && bun install --frozen-lockfile --no-progress && bun run bundle:check)
```

Expected: the manifest test passes and the checked-in asset is byte-identical to the bundle generated from the recorded SHA.

- [ ] **Step 6: Commit the source identity boundary**

```bash
git add tools/proto-ui-bridge crates/proto-ui-gpui/assets/proto-ui-bridge.js
git commit -m "build(proto-ui): track main snapshot bundle identity"
```

---

### Task 2: Generalize the bridge protocol and lifecycle host

**Files:**
- Modify: `crates/proto-ui-gpui/src/protocol.rs`
- Modify: `crates/proto-ui-gpui/src/quickjs.rs`
- Create: `crates/proto-ui-gpui/src/runtime.rs`
- Create: `crates/proto-ui-gpui/tests/runtime_contract.rs`
- Modify: `crates/proto-ui-gpui/tests/protocol_contract.rs`
- Modify: `crates/proto-ui-gpui/src/lib.rs`

**Interfaces:**
- Consumes: `upstream.json`, the existing `QuickJsBridge`, `BridgeState`, `BridgeCommand`, and `BridgeEvent`.
- Produces:

```rust
pub struct ProtoSessionHost {
    pub fn new() -> Result<Self>;
    pub fn start(&mut self, request: StartRequest) -> Result<SessionSnapshot>;
    pub fn acknowledge(&mut self, ack: ProjectionAck) -> Result<CommitDisposition>;
    pub fn input(&mut self, input: InputRequest) -> Result<DispatchOutcome>;
    pub fn set_props(&mut self, request: PropsRequest) -> Result<CommitDisposition>;
    pub fn remount(&mut self) -> Result<ViewEpoch>;
    pub fn unmount(&mut self) -> Result<()>;
    pub fn dispose(&mut self) -> Result<()>;
}
```

- `StartRequest` contains `SessionId`, `InstanceId`, `PrototypeKey`, JSON props, Slot ID, and accessible content alternative.
- `SessionSnapshot` contains the latest `ProjectionTransaction`, `StyleProjection`, optional `A11ySnapshot`, JSON state map, and diagnostics.
- `CommitDisposition` is `Applied` or `Superseded`; unsupported and failed projection status remains an error.

- [ ] **Step 1: Write failing generic session tests**

Cover these observable transitions:

```rust
fn session_lifecycle_is_epoch_safe() -> Result<(), BridgeError> {
    let request = StartRequest::button(
        SessionId::new("session-1")?,
        InstanceId::new("instance-1")?,
        "Apply",
    );
    let mut host = ProtoSessionHost::new()?;
    let snapshot = host.start(request)?;
    assert_eq!(snapshot.prototype.as_str(), "shadcn-button");
    assert_eq!(snapshot.projection.view_epoch.get(), 1);
    assert!(snapshot.pending_commit);

    let first_epoch = snapshot.projection.view_epoch;
    host.acknowledge(ProjectionAck::applied(
        SessionId::new("session-1")?,
        InstanceId::new("instance-1")?,
        first_epoch,
        snapshot.projection.commit_id,
    ))?;
    host.remount()?;

    let stale = ProjectionAck::applied(
        SessionId::new("session-1")?,
        InstanceId::new("instance-1")?,
        first_epoch,
        snapshot.projection.commit_id + 1,
    );
    assert!(matches!(
        host.acknowledge(stale),
        Err(BridgeError::StaleEpoch { .. })
    ));
    Ok(())
}
```

Also assert that dispose rejects input, a second `dispose()` is idempotent, duplicate session IDs are rejected, and the registry rejects an arbitrary import string before QuickJS execution.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test runtime_contract
```

Expected: compile failure for the missing generic host types.

- [ ] **Step 3: Implement generic QuickJS session ownership**

Move Button-specific session bookkeeping out of `QuickJsBridge` into `ProtoSessionHost`. Keep `QuickJsBridge` responsible only for evaluating the embedded bundle and decoding bounded JSON events. `ProtoSessionHost` must:

- install one view epoch before accepting an ACK;
- apply only the matching commit ACK;
- reject stale session, instance, view, route, commit, and sequence values;
- keep one projection in flight per session;
- turn fatal diagnostics into `BridgeError::Runtime`;
- terminally revoke all sessions on dispose;
- never invoke QuickJS from a GPUI render/prepaint/paint callback.

- [ ] **Step 4: Add bounded JSON validation**

Reject bridge messages larger than 256 KiB or nested deeper than 16 levels with `BridgeError::Decode`. Validate every object/array before passing it to QuickJS; keep omission and JSON `null` distinct.

- [ ] **Step 5: Run GREEN and the existing bridge tests**

```bash
cargo test --offline -p proto-ui-gpui --test protocol_contract
cargo test --offline -p proto-ui-gpui --test quickjs_contract
cargo test --offline -p proto-ui-gpui --test runtime_contract
```

- [ ] **Step 6: Commit**

```bash
git add crates/proto-ui-gpui/src crates/proto-ui-gpui/tests
 git commit -m "feat(proto-ui): generalize embedded runtime session host"
```

---

### Task 3: Complete Template, Slot, SVG, style, and host readiness projection

**Files:**
- Create: `crates/proto-ui-gpui/src/template.rs`
- Create: `crates/proto-ui-gpui/src/style.rs`
- Modify: `crates/proto-ui-gpui/src/protocol.rs`
- Modify: `crates/proto-ui-gpui/src/lib.rs`
- Modify: `crates/sailbreak-gui/src/proto_surface.rs`
- Create: `crates/proto-ui-gpui/tests/template_contract.rs`
- Create: `crates/proto-ui-gpui/tests/style_contract.rs`

**Interfaces:**
- Produces:

```rust
pub enum TemplateNode {
    Container { tag: String, style: Vec<String>, children: Vec<Self> },
    Text { text: String },
    Slot { slot_id: String },
    Svg { tag: String, attributes: BTreeMap<String, String>, children: Vec<Self> },
}

pub struct SemanticId(String);

pub struct TemplateSnapshot {
    pub nodes: Vec<TemplateNode>,
    pub semantic_ids: BTreeSet<SemanticId>,
}

pub struct NativeStyle {
    pub tokens: Vec<String>,
    pub unsupported: Vec<String>,
}

pub fn translate_style(tokens: Vec<String>, theme: ShadcnTheme) -> NativeStyle;
pub fn prune_replaced_tree(previous: &TemplateSnapshot, next: &TemplateSnapshot) -> Vec<SemanticId>;
```

- `proto_surface.rs` consumes only typed `TemplateSnapshot`, `NativeStyle`, Slot content, and semantic event callbacks.

- [ ] **Step 1: Write failing nested Template and token tests**

Assert that container/text/Slot/SVG data survives JSON round-trip, structural nodes remain anonymous, duplicate semantic IDs fail, removed nodes are pruned, and all current Shadcn Button/Toggle/Switch/Tabs/Select/Dialog token families either translate or return a named unsupported diagnostic.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test template_contract
cargo test --offline -p proto-ui-gpui --test style_contract
```

- [ ] **Step 3: Implement the generic projection dialect**

Decode Proto-UI `TemplateChildren` without serializing JS objects. Map reserved Slot nodes to the host-provided Slot ID, preserve explicit content alternatives, and reject unsupported node kinds. Implement semantic color, size, spacing, radius, border, opacity, transform, shadow, focus-ring, and SVG attribute translation with per-token diagnostics.

- [ ] **Step 4: Add GPUI projection without local component semantics**

Keep the existing `button_element` behavior as the first specialization, but move common layout/style application into `ProtoSurface`. The GPUI element tree may add layout-only wrappers; it may not add a second hover, pressed, disabled, selected, or open state machine.
- [x] **Deferred (Task 3 Step 4):** `proto_surface.rs` does not yet consume the typed `TemplateSnapshot`/`NativeStyle`; the first production consumer is the `ProtoSurface` introduced in Task 4 (AccessKit) and Task 5 (stateful primitives). Data-model round-trip coverage is in `template_contract.rs`/`style_contract.rs`.

- [ ] **Step 5: Run GREEN and commit**

```bash
cargo test --offline -p proto-ui-gpui --test template_contract
cargo test --offline -p proto-ui-gpui --test style_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui crates/sailbreak-gui/src/proto_surface.rs
git commit -m "feat(proto-ui): add generic template and style projection"
```

---

### Task 4: Add native AccessKit projection and pin the GPUI host revision

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `crates/sailbreak-gui/Cargo.toml`
- Modify: `crates/sailbreak-gui/src/proto_surface.rs`
- Create: `crates/sailbreak-gui/tests/accessibility_contract.rs`
- Modify: `README.md`

**Interfaces:**
- The GPUI host revision is resolved at implementation time against the checked-in `crates/sailbreak-gui/Cargo.toml` dependency, and the chosen Zed commit and any platform API deltas are recorded in the Task commit. Task authoring verified that Zed `399258feeaf90ad8a3a208c99221ee87b6452f38` exposes `accesskit` re-exports, `StatefulInteractiveElement::role`, `aria_label`, and `on_a11y_action`.
- `ProtoSurface` maps `A11ySnapshot` to `gpui::accesskit::Role`, accessible label, disabled/toggled/selected state through the accessible-node state API, and one `AccessibleAction::Click` route.

- [ ] **Step 1: Write the failing native a11y test**

Create a host test seam that materializes a resolved `A11ySnapshot` and asserts the resulting GPUI element configuration contains role `Button`, the stable label `Apply`, and the `AccessibleAction::Click` handler. Do not call nonexistent `surface.role()` or `surface.label()` accessors on `Div`; inspect the AccessKit node through the pinned GPUI test/inspection surface or use a named projection helper whose contract exposes those values.

The fixture must also prove the AccessKit Click callback invokes the same Proto `PressCommit` route as GPUI keyboard/mouse click and does not increment the signal twice.

- [ ] **Step 2: Run RED against the current published GPUI**

```bash
cargo test --offline -p sailbreak-gui --test accessibility_contract
```

Expected: fail because crates.io GPUI 0.2.2 does not expose the required public AccessKit projection API; this establishes the exact host-revision admission blocker before changing the dependency.

- [ ] **Step 3: Pin the reviewed GPUI source and implement the mapping**

Pin the Zed git dependency at the reviewed revision and add the exact public calls `.role(gpui::accesskit::Role::Button)`, `.aria_label("Apply")`, and `.on_a11y_action(gpui::AccessibleAction::Click, click_handler)` to the native surface. Project disabled/toggled/selected state through the AccessKit node state API at that revision; do not add an `aria_disabled` builder method. If the chosen GPUI revision changes any API, record its exact source path and update the contract test to that API before proceeding.
- [ ] **Step 4: Run the focused native checks**

```bash
cargo test --offline -p sailbreak-gui --test accessibility_contract
cargo check --offline --workspace --all-targets
cargo clippy --offline --workspace --all-targets -- -D warnings
```

- [ ] **Step 5: Commit the host revision boundary**

```bash
git add Cargo.toml Cargo.lock crates/sailbreak-gui README.md
 git commit -m "feat(gui): project Proto UI semantics through AccessKit"
```

---

### Task 5: Implement Shadcn Toggle

**Files:**
- Create: `crates/proto-ui-gpui/src/components/toggle.rs`
- Modify: `crates/proto-ui-gpui/src/components/mod.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/toggle_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces `ProtoToggleHost` with `register`, `dispatch`, `set_props`, and `snapshot` methods.
- Uses the real main-HEAD `shadcn-toggle` prototype and its `variant`, `size`, `active`, `defaultActive`, and `disabled` props.

- [ ] **Step 1: Bump Proto-UI HEAD and write RED tests**

The tests must assert defaultActive initialization, controlled active updates, one activation signal, disabled suppression, focus-visible style, variant/size token translation, a11y role/name, replacement, and disposal.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test toggle_contract
```

- [ ] **Step 3: Implement the host family adapter**

Use `ProtoSessionHost` and the generic event/focus/style/template paths. Do not mirror `active` in Rust except as the latest exposed state snapshot used for rendering. Route native Click, keyboard activation, and AccessKit Click through one `press.commit` input.

- [ ] **Step 4: Migrate one Sailbreak boolean control**

Replace one existing boolean action in the dashboard with the real `ProtoToggleHost`; keep the hardware command in the existing CLI controller.

- [ ] **Step 5: Run GREEN, GUI smoke, and commit**

```bash
cargo test --offline -p proto-ui-gpui --test toggle_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Toggle host slice"
```

---

### Task 6: Implement Shadcn Switch Root and Thumb

**Files:**
- Create: `crates/proto-ui-gpui/src/components/switch.rs`
- Modify: `crates/proto-ui-gpui/src/components/mod.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/switch_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces `ProtoSwitchHost` for `shadcn-switch-root` and `shadcn-switch-thumb`, with checked, disabled, hovered, pressed, focus-visible, and Slot projections.

- [ ] **Step 1: Bump HEAD and write RED tests**

Cover controlled/uncontrolled checked state, root/thumb composition, native click exact-once behavior, disabled behavior, focus-visible ring, dark color-scheme rules, remount, and stale thumb replacement.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test switch_contract
```

- [ ] **Step 3: Implement the two semantic surfaces**

Keep Root and Thumb as separate Proto sessions connected by opaque logical parent/route references. The Rust host owns only their GPUI composition and never sets checked state independently.

- [ ] **Step 4: Dogfood a real Boolean hardware capability**

Use Switch Root/Thumb for a capability whose current HAL reports `Available` or `Limited`; unavailable capabilities remain visibly unavailable and cannot emit a write command.

- [ ] **Step 5: Run GREEN and commit**

```bash
cargo test --offline -p proto-ui-gpui --test switch_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Switch host slice"
```

---

### Task 7: Implement Shadcn Checkbox

**Files:**
- Create: `crates/proto-ui-gpui/src/components/checkbox.rs`
- Modify: `crates/proto-ui-gpui/src/components/mod.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/checkbox_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces `ProtoCheckboxHost` for the current main-HEAD Checkbox Root and Indicator entries.

- [ ] **Step 1: Bump HEAD and write RED tests**

Cover checked/unchecked/indeterminate values, disabled suppression, keyboard Space activation, focus-visible styling, indicator Slot/SVG output, accessible checkbox role/name/state, and remount/dispose cleanup.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test checkbox_contract
```

- [ ] **Step 3: Implement and migrate a capability matrix action**

The Rust host renders the Indicator projection from the received Template/SVG snapshot; it does not draw its own check mark or toggle the state locally.

- [ ] **Step 4: Run GREEN and commit**

```bash
cargo test --offline -p proto-ui-gpui --test checkbox_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Checkbox host slice"
```

---

### Task 8: Implement Shadcn Separator

**Files:**
- Create: `crates/proto-ui-gpui/src/components/separator.rs`
- Modify: `crates/proto-ui-gpui/src/components/mod.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/separator_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces a static `ProtoSeparatorHost` for the main-HEAD Shadcn Separator Root, including orientation and decorative semantics.

- [ ] **Step 1: Bump HEAD and write RED tests**

Assert horizontal/vertical orientation, decorative vs semantic accessibility behavior, style token closure, stable structural identity, and replacement without stale nodes.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test separator_contract
```

- [ ] **Step 3: Implement the static projection**

Reuse `ProtoSurface` and `TemplateNode`; no event or focus lease is installed for decorative separators.

- [ ] **Step 4: Replace hand-written dashboard rules**

Use Proto Separator between the sidebar/header/action/capability regions. Keep the surrounding layout containers host-owned.

- [ ] **Step 5: Run GREEN and commit**

```bash
cargo test --offline -p proto-ui-gpui --test separator_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Separator host slice"
```

---

### Task 9: Implement Shadcn Textarea and native text control

**Files:**
- Create: `crates/proto-ui-gpui/src/components/textarea.rs`
- Modify: `crates/proto-ui-gpui/src/components/mod.rs`
- Modify: `crates/proto-ui-gpui/src/focus.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/textarea_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces `ProtoTextareaHost` with JSON value, placeholder, disabled, focus, input/change, selection-safe updates, and Slot/content projections.
- Native text capability owns the GPUI text input handle; Proto state owns the semantic value.

- [ ] **Step 1: Bump HEAD and write RED tests**

Cover initial value, controlled value replacement, user input, change dispatch, placeholder, disabled focus rejection, IME-safe host updates, cursor preservation, focus-visible state, remount, and disposal.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test textarea_contract
```

- [ ] **Step 3: Implement the text-control capability**

Use opaque `TextControlRef` values and explicit `input`/`change` messages. Never send GPUI text entities or closures to QuickJS. A stale view epoch must not overwrite a newer native text buffer.

- [ ] **Step 4: Dogfood profile/DSL editing**

Add a profile editor surface to the Tuning section. Its Save/Apply actions remain routed through the existing config/CLI safety layer.

- [ ] **Step 5: Run GREEN and commit**

```bash
cargo test --offline -p proto-ui-gpui --test textarea_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Textarea host slice"
```

---

### Task 10: Implement Tabs and roving focus

**Files:**
- Create: `crates/proto-ui-gpui/src/components/tabs.rs`
- Modify: `crates/proto-ui-gpui/src/focus.rs`
- Modify: `crates/proto-ui-gpui/src/events.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/tabs_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces `ProtoTabsHost` for Root/List/Trigger/Content and exposes selected tab, active tab, roving focus, and content presence snapshots.

- [ ] **Step 1: Bump HEAD and write RED tests**

Cover parent graph before setup, arrow/Home/End navigation, loop policy, disabled triggers, selected/active state, keyboard activation exactly once, AccessKit tab action, tab/tabpanel relations, remount, and stale focus target rejection.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test tabs_contract
```

- [ ] **Step 3: Implement logical parent and roving focus**

Build the complete logical graph before creating Runtime sessions. Focus operations return `accepted`, `not_ready`, or `rejected`; a native FocusHandle allocation is not treated as readiness.

- [ ] **Step 4: Migrate the dashboard sections**

Replace the current sidebar section selector with a Tabs Root/List/Trigger/Content composition. The selected section controls which dashboard content is present; the hardware snapshot remains the only fact source.

- [ ] **Step 5: Run GREEN, desktop smoke, and commit**

```bash
cargo test --offline -p proto-ui-gpui --test tabs_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Tabs and roving focus"
```

---

### Task 11: Implement shared overlay, portal, positioning, and dismissal substrate

**Files:**
- Create: `crates/proto-ui-gpui/src/overlay.rs`
- Create: `crates/proto-ui-gpui/tests/overlay_contract.rs`
- Modify: `crates/proto-ui-gpui/src/protocol.rs`
- Modify: `crates/sailbreak-gui/src/proto_surface.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces:

```rust
pub struct OverlayHost {
    pub fn attach(&mut self, request: OverlayRequest) -> Result<OverlayLease>;
}

pub struct OverlayLease {
    pub fn update(&mut self, placement: PlacementSnapshot) -> Result<()>;
    pub fn close(&mut self, reason: CloseReason) -> Result<()>;
    pub fn dispose(&mut self);
}
```

- `OverlayRequest` contains opaque anchor/surface refs, view epoch, layer role, placement policy, and dismissal policy.
- `OverlayLease` is idempotent and rejects stale connection/view revisions.

- [ ] **Step 1: Write RED lease tests**

Assert portal attachment, replacement disposal, outside press, Escape, focus restore, stale placement rejection, layer ordering, transition cancellation, and terminal cleanup. Assert that a lossless queue overflow fails the session.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test overlay_contract
```

- [ ] **Step 3: Implement Rust-owned portal and positioning**

GPUI owns the overlay layer and anchor measurements. QuickJS receives only placement facts and semantic dismissal events. No floating component calls JavaScript from layout/prepaint/paint.

- [ ] **Step 4: Run GREEN and commit**

```bash
cargo test --offline -p proto-ui-gpui --test overlay_contract
cargo test --offline -p sailbreak-gui --lib
cargo clippy --offline --workspace --all-targets -- -D warnings

git add crates/proto-ui-gpui crates/sailbreak-gui
 git commit -m "feat(proto-ui): add GPUI overlay and positioning substrate"
```

---

### Task 12: Implement Shadcn Select

**Files:**
- Create: `crates/proto-ui-gpui/src/components/select.rs`
- Modify: `crates/proto-ui-gpui/src/components/mod.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/select_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces `ProtoSelectHost` for Root/Trigger/Value/Content/Item, backed by `OverlayHost` and `ProtoSessionHost`.

- [ ] **Step 1: Bump HEAD and write RED tests**

Cover controlled/uncontrolled values, placeholder, typeahead, keyboard navigation, disabled items, selected indicator, item-aligned/popper placement, open/close transitions, trigger focus restoration, combobox/listbox/option snapshots, stale portal completion, and no duplicate value-change signal.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test select_contract
```

- [ ] **Step 3: Implement Select composition**

Keep Root, Trigger, Value, Content, and Item as independent Proto instances linked through opaque family/route refs. Rust owns the portal and native geometry; Proto Runtime owns open/value/selection semantics.

- [ ] **Step 4: Migrate dashboard selectors**

Use Select for performance mode, tuning profile, and power scheme. Controlled updates must merge value and disabled state in one projection request so a remount cannot unlock the selector early.

- [ ] **Step 5: Run GREEN, desktop smoke, and commit**

```bash
cargo test --offline -p proto-ui-gpui --test select_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Select host slice"
```

---

### Task 13: Implement Shadcn Dropdown

**Files:**
- Create: `crates/proto-ui-gpui/src/components/dropdown.rs`
- Modify: `crates/proto-ui-gpui/src/components/mod.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/dropdown_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces `ProtoDropdownHost` for Root/Trigger/Content/Item, backed by `OverlayHost`.

- [ ] **Step 1: Bump HEAD and write RED tests**

Cover menu open/close, keyboard navigation, disabled items, Escape, outside press, item selection, menu/item roles, trigger restoration, stale dismissal, and exact-once item activation.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test dropdown_contract
```

- [ ] **Step 3: Implement and dogfood action groups**

Move grouped dashboard actions into Dropdown menus. Each selected item emits one semantic command to the existing `DashboardAction` gateway; no Dropdown-specific command execution is added.

- [ ] **Step 4: Run GREEN and commit**

```bash
cargo test --offline -p proto-ui-gpui --test dropdown_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Dropdown host slice"
```

---

### Task 14: Implement Shadcn Dialog

**Files:**
- Create: `crates/proto-ui-gpui/src/components/dialog.rs`
- Modify: `crates/proto-ui-gpui/src/components/mod.rs`
- Modify: `crates/proto-ui-gpui/src/overlay.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/dialog_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces `ProtoDialogHost` for Root/Trigger/Mask/Content/Title/Description/Close/Header/Footer.

- [ ] **Step 1: Bump HEAD and write RED tests**

Cover modal gating, Escape, outside close policy, title/description relations, focus entering the dialog, focus trap, focus restoration, presence transitions, stale open/close completion, terminal disposal, and AccessKit dialog actions.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test dialog_contract
```

- [ ] **Step 3: Implement Dialog over the common substrate**

Dialog uses the same overlay lease and focus center as Select/Dropdown. It may not install a second global event bus or a second focus restoration algorithm.

- [ ] **Step 4: Dogfood risk-confirmed actions**

BIOS writes, profile apply, and other L3 operations open a Dialog containing the real confirmation text, recovery command, and `--yes` equivalent action. The Dialog does not bypass CLI safety checks.

- [ ] **Step 5: Run GREEN, desktop smoke, and commit**

```bash
cargo test --offline -p proto-ui-gpui --test dialog_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Dialog host slice"
```

---

### Task 15: Implement Shadcn Hover Card

**Files:**
- Create: `crates/proto-ui-gpui/src/components/hover_card.rs`
- Modify: `crates/proto-ui-gpui/src/components/mod.rs`
- Modify: `tools/proto-ui-bridge/src/index.ts`
- Create: `crates/proto-ui-gpui/tests/hover_card_contract.rs`
- Modify: `crates/sailbreak-gui/src/lib.rs`

**Interfaces:**
- Produces `ProtoHoverCardHost` for Root/Trigger/Content, backed by `OverlayHost` and delayed scheduler.

- [ ] **Step 1: Bump HEAD and write RED tests**

Cover pointer/focus entry, show/hide delay, outside dismissal, placement, transition cancellation, remount, dispose, and no duplicate open/close state transitions.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p proto-ui-gpui --test hover_card_contract
```

- [ ] **Step 3: Dogfood capability explanations**

Capability rows use Hover Card content for details and evidence. The content remains a Rust-owned Slot subtree projected through Proto-UI.

- [ ] **Step 4: Run GREEN and commit**

```bash
cargo test --offline -p proto-ui-gpui --test hover_card_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check

git add crates/proto-ui-gpui tools/proto-ui-bridge crates/sailbreak-gui/src/lib.rs
 git commit -m "feat(proto-ui): add Shadcn Hover Card host slice"
```

---

### Task 16: Complete the Sailbreak full dogfood composition

**Files:**
- Modify: `crates/sailbreak-gui/src/lib.rs`
- Modify: `crates/sailbreak-gui/src/proto_surface.rs`
- Create: `crates/sailbreak-gui/tests/dogfood_contract.rs`
- Modify: `README.md`
- Modify: `docs/08-architecture.md`

**Interfaces:**
- Every interactive dashboard surface is created through a component-family host from Tasks 5–15.
- `GuiController` remains the only bridge to CLI/HAL actions.

- [ ] **Step 1: Write RED consumer coverage**

Assert the final dashboard composition contains Proto surfaces for Tabs, Select, Dropdown, Dialog, Toggle/Switch/Checkbox, Textarea, Hover Card, Separator, and Button. Assert no dashboard source path installs an independent `.on_click`, `.on_key_down`, or focus state machine outside `proto_surface.rs` and the host adapters.

- [ ] **Step 2: Run RED**

```bash
cargo test --offline -p sailbreak-gui --test dogfood_contract
```

- [ ] **Step 3: Replace remaining hand-written interaction paths**

Use this mapping:

```text
sidebar sections       -> Shadcn Tabs
mode/profile selectors -> Shadcn Select
action groups          -> Shadcn Dropdown
BIOS confirmations     -> Shadcn Dialog
boolean capabilities   -> Shadcn Switch / Toggle / Checkbox
profile editor         -> Shadcn Textarea
capability details     -> Shadcn Hover Card
layout rules           -> Shadcn Separator and Proto SVG
```

Keep headings, capability rows, telemetry cards, and safety copy as host-owned structural layout where no matching Proto-UI prototype exists.

- [ ] **Step 4: Verify command and safety behavior**

For every command button/menu/dialog action, assert one semantic signal results in one `GuiController::execute` call. Assert unavailable channels stay unavailable and dry-run actions never become commits.

- [ ] **Step 5: Run GREEN and desktop smoke**

```bash
cargo test --offline -p sailbreak-gui --test dogfood_contract
cargo test --offline -p sailbreak-gui --lib
cargo fmt --all -- --check
```

When a desktop session exists, launch `cargo run --locked -p sailbreak-gui`, exercise Tabs, Select, Dropdown, Dialog, Toggle/Switch, Textarea, and Hover Card, and retain the observed behavior. In a headless environment, the executable must return the existing display-channel error and the report must state that visual evidence was unavailable.

- [ ] **Step 6: Commit the complete consumer cutover**

```bash
git add crates/sailbreak-gui README.md docs/08-architecture.md
 git commit -m "feat(gui): dogfood all Proto UI Shadcn controls"
```

---

### Task 17: Run full conformance, update evidence, and prepare upstream slices

**Files:**
- Create: `crates/proto-ui-gpui/tests/conformance_contract.rs`
- Create: `docs/proto-ui-gpui-profile.json`
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-08-31-proto-ui-gpui-full-dogfood-design.md`
- Modify: `tools/proto-ui-bridge/upstream.json`
- Modify: `tools/proto-ui-bridge/NOTICE`

**Interfaces:**
- Produces a machine-readable profile containing exact Proto-UI/GPUI revisions, supported families, omitted capabilities, platform scope, evidence paths, and bundle digest.
- Produces conformance tests that distinguish Runtime fake-host proof from native GPUI proof.

- [ ] **Step 1: Write the final conformance matrix first**

The matrix must contain rows for identity, props, state, expose, event, focus, Template, Slot, style, SVG, a11y, overlay, lifecycle, remount, stale rejection, dispose, Linux, and Windows. Each row includes `status`, `authority`, `implementation`, `test`, `platform`, and `limitation`.

- [ ] **Step 2: Run focused conformance RED for every missing row**

```bash
cargo test --offline -p proto-ui-gpui --test conformance_contract
```

- [ ] **Step 3: Implement only evidence-backed matrix corrections**

Do not widen support by editing prose. A missing native proof remains `omitted` or `uncataloged`. A token translator gap remains a diagnostic until the actual token mapping and test exist.

- [ ] **Step 4: Run the complete local verification**

```bash
(cd tools/proto-ui-bridge && bun install --frozen-lockfile --no-progress && bun run bundle:check)
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo build --locked --release --workspace
```

- [ ] **Step 5: Run native platform verification**

Run the Linux desktop smoke in a real display session. Run the Windows CI workflow and record the exact successful run URL, job conclusions, compiler, GPUI revision, and Proto-UI bundle revision. Do not describe CI compilation as hardware support.

- [ ] **Step 6: Split upstream contribution candidates**

Prepare separate clean branches/patches with no Sailbreak HAL code:

1. host-neutral protocol and lease shapes;
2. GPUI host capability and lifecycle realization;
3. Template/style/SVG translation;
4. one Shadcn component family per Adapter slice;
5. profile metadata and evidence.

Each patch states that Sailbreak independently dogfooded it, names the exact Proto-UI HEAD used, and avoids claiming official `A-GPUI-*` support until Proto-UI maintainers review it.

- [ ] **Step 7: Commit evidence and profile**

```bash
git add crates/proto-ui-gpui/tests docs/proto-ui-gpui-profile.json README.md docs/superpowers/specs tools/proto-ui-bridge
 git commit -m "docs(proto-ui): record complete GPUI dogfood profile"
```

---

## Execution Rule

Run Tasks 1–17 strictly in order. At the start of each task, bump Proto-UI to current `main` HEAD and review the diff against the previous task's recorded SHA. Do not start the next task until the current task has its focused GREEN suite, proportional full checks, desktop evidence or an explicit headless limitation, updated profile evidence, and a committed clean diff.
