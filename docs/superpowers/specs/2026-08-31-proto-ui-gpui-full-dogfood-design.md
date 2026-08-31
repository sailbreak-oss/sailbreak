# Proto-UI GPUI Full Dogfood Design

## Status

Proposed design; the Proto-UI main-snapshot baseline and no-runtime-Node/Bun constraint are confirmed. The first Button slice is already merged into Sailbreak `main`; this document governs the remaining component and host work.

## Goal

Make Sailbreak a serious non-Web dogfood host for the Proto-UI Shadcn prototype library. Every Sailbreak interactive surface must use a real Proto-UI prototype executed by the embedded Runtime, while Rust/GPUI supplies only host capabilities, native composition, and hardware-command integration.

## Source and version identity

The source policy is **Proto-UI `main` HEAD bump per slice**, not a permanent pin. The design checkpoint used `1f7c2bdb6b2f4a6908d422f1e205e01f62c8c9ed`; the current synchronized HEAD for plan authoring is `9c8891ca22fefafb1346e6ebd02d1f80cae2ec24`.

Before each implementation slice, record the then-current Proto-UI `main` SHA in that slice's source manifest and regenerate the bundle from that exact SHA. A slice may not silently consume a newer HEAD, and a newer HEAD must be reviewed for API, semantic, and package-graph drift before the slice begins.

The bundle build must:

- fetch or consume the exact source snapshot recorded by the current slice;
- build all referenced `@proto.ui/*` packages from that snapshot's lockfile and package manifests;
- record the Proto-UI commit, package versions, bundle SHA-256, and source license notices;
- fail when the source commit, lockfile, or generated bundle changes without an explicit source-manifest update;
- never use arbitrary import strings from Rust or user input;
- never label a main-snapshot bundle as the published `0.2.0` release.

Runtime identity is separate from the host identity:

```text
Proto UI source commit + package graph + bundle digest
GPUI source revision + host platform/backend + Rust toolchain
bridge protocol major/minor + feature set
```

The existing crates.io GPUI `0.2.2` dependency remains the first compatibility baseline. Native AccessKit work is a separate host task: it must either move Sailbreak to a pinned GPUI revision that exposes the required AccessKit API or add a reviewed direct host integration. It must not be claimed from the current 0.2.2 API surface.

## Ownership and data flow

```text
Proto-UI Prototype definition
        │
        ▼
Embedded QuickJS Runtime
  - Module semantics
  - props/state/expose
  - lifecycle
  - semantic events
  - style/a11y intent
        │ versioned data-only bridge
        ▼
GPUI host adapter
  - host capabilities
  - projection transaction
  - native layout/painting
  - Slot composition
  - FocusHandle/input translation
  - AccessKit projection
        │
        ▼
Sailbreak dashboard and hardware command controller
```

The bridge is transport-neutral even though the first implementation is in-process. No GPUI layout, prepaint, or paint callback synchronously invokes JavaScript. The Rust host sends an input command or queues a projection acknowledgment outside the frame path; QuickJS returns data-only events and projections.

Durable identity exists only for semantic surfaces, sessions, view epochs, Slots, routes, focus targets, and accessibility objects. Anonymous structural containers and text do not receive invented component identity.

## Common host substrate

Before adding the next component, extend the current Button-only host into these focused units:

- `protocol`: handshake, bounded JSON values, identity/revision validation, projection transactions, ACK status, input envelopes, diagnostics;
- `runtime`: QuickJS context, pinned bundle loading, governed registry, lifecycle command dispatch, terminal disposal;
- `events`: opaque route refs, semantic event plan, independent idempotent leases, exact-once sample handling, local default-action results;
- `focus`: opaque target refs, readiness state machine, native focus requests, blur, modality, focus-visible, replacement replay, disposal cancellation;
- `template`: anonymous containers/text, Slot markers, SVG nodes, style handles, full-tree replacement and stale-node pruning;
- `overlay`: portal ownership, anchor/surface targets, positioning leases, presence, outside press, Escape, focus restore, view-epoch cleanup;
- `a11y`: role/name/state/action/relationship snapshots mapped to the pinned GPUI AccessKit surface;
- `style`: Shadcn semantic tokens to GPUI style refinement, with explicit unsupported-token diagnostics;
- `components`: one host adapter per semantic family, with no copied Proto-UI behavior.

The current private QuickJS EventTarget-shaped objects remain an implementation compatibility layer for the pinned Runtime only. They are not exported through the Rust protocol and are removed or isolated when the upstream Runtime exposes the required host-neutral ports.

## Component admission sequence

Every component family is completed as a vertical slice before the next family starts. A slice is not complete until its Runtime execution, host translation, visual projection, interaction behavior, lifecycle cleanup, focused tests, and Sailbreak consumer are all present.

### Slice A — style, Template, SVG, and host readiness

Implement the generic projection dialect required by every later component:

- container/text/Slot/SVG serialization;
- stable semantic IDs and full-tree pruning;
- style token parser and theme meta;
- host projection ACK barrier;
- bounded scheduler and delay tasks;
- opaque event/focus routes;
- explicit diagnostics for unsupported node kinds, tokens, and capabilities.

Acceptance: a fixture containing nested containers, text, Slot, and SVG round-trips through QuickJS and GPUI data models with no durable identity leaks or stale nodes.

### Slice B — stateful primitives

Implement in order:

1. Shadcn Toggle;
2. Shadcn Switch Root and Thumb;
3. Shadcn Checkbox from the pinned snapshot;
4. Shadcn Separator;
5. Shadcn Textarea.

Each slice must cover controlled/default props, state/expose projection, native activation, disabled behavior, keyboard behavior, focus-visible styling, a11y intent, Slot composition, remount, stale input, and dispose. Textarea additionally requires a GPUI text-control capability with value/input/change synchronization and cursor-safe host ownership.

### Slice C — Tabs and focus navigation

Implement Tabs Root/List/Trigger/Content with:

- logical parent graph established before setup;
- roving focus and selected state;
- arrow/Home/End navigation;
- loop and disabled-item behavior;
- keyboard and AccessKit activation through one commit path;
- stable tab/tabpanel a11y relations;
- focus restoration after content replacement and remount.

### Slice D — shared overlay and positioning

Implement the substrate before any compound overlay:

- Rust-owned portal layer;
- anchor measurement and placement;
- item-aligned and popper-style placement policies;
- collision and available-space data;
- outside press, Escape, focus outside;
- enter/leave presence and delayed work;
- layer ordering;
- stale connection and view-epoch rejection;
- idempotent lease disposal.

### Slice E — Select

Implement Select Root/Trigger/Value/Content/Item:

- controlled and uncontrolled values;
- placeholder and selected indicator;
- keyboard navigation and typeahead;
- disabled items;
- trigger/content portal lifecycle;
- item-aligned and popper positions;
- trigger focus restoration;
- combobox/listbox/option semantic projection.

### Slice F — Dropdown

Implement Dropdown Root/Trigger/Content/Item:

- menu open/close state;
- keyboard item navigation;
- disabled items;
- outside press and Escape;
- menu/item roles and action projection;
- portal cleanup and trigger restoration;
- no duplicate selection or close signals.

### Slice G — Dialog

Implement Dialog Root/Trigger/Mask/Content/Title/Description/Close/Header/Footer:

- modal mask and input gating;
- Escape and outside close policy;
- focus trap and restoration;
- title/description relations;
- presence transitions;
- stale open/close completion rejection;
- terminal disposal of all leases and references.

### Slice H — Hover Card

Implement Hover Card Root/Trigger/Content:

- pointer and focus entry;
- show/hide delay;
- outside dismissal;
- placement and portal ownership;
- transition cancellation;
- remount/dispose cleanup;
- no duplicate open/close state transitions.

## Sailbreak dogfood composition

After each family slice, migrate the dashboard immediately instead of building an unused showcase:

- existing sidebar navigation becomes Tabs;
- performance mode, profile, and power-scheme selectors become Select;
- grouped actions become Dropdown;
- BIOS and other risk-confirmed operations become Dialog;
- Boolean capabilities become Switch or Toggle;
- profile/DSL editing becomes Textarea;
- capability explanations become Hover Card;
- separators and icon content use the corresponding Proto-UI projections.

Hardware mutation remains owned by `sailbreak-cli` and its verified-write services. Proto-UI components only emit semantic actions; they never call HAL implementations directly. Unsupported hardware capabilities remain disabled or unavailable with the existing structured error semantics.

No native HTML-like or hand-written interactive fallback remains in the final dashboard. A host capability omission is displayed as an explicit unavailable state, not silently replaced with a local Rust state machine.

## Testing and evidence gates

For each slice:

1. write the failing protocol/host contract test;
2. confirm the intended RED failure;
3. implement the smallest host translation;
4. run the focused GREEN suite;
5. run a real GPUI desktop smoke for the changed surface;
6. run workspace formatting, check, test, and clippy checks;
7. update the support/omission matrix and pinned-source metadata;
8. review and commit before opening the next slice.

Required evidence layers:

- pure protocol and stale-message tests;
- QuickJS Runtime tests using the real pinned prototype;
- host capability lease and lifecycle tests;
- GPUI style/template projection tests;
- component keyboard/activation/focus tests;
- semantic a11y snapshots and AccessKit integration tests when the pinned GPUI surface supports them;
- Linux and Windows compile/CI checks;
- actual desktop smoke tests on the available host;
- no claim that a fake host proves a native capability.

## Upstream contribution boundary

Sailbreak remains the independent dogfood consumer. Upstream contributions are prepared only after the corresponding slice has real evidence and no Sailbreak hardware-specific code:

1. host-neutral protocol and capability shapes;
2. GPUI host realization and lifecycle rules;
3. generic Template/style/a11y translation;
4. one Adapter-profile component slice per pull request;
5. profile metadata and conformance evidence.

The first upstream contribution must explicitly disclose that Sailbreak dogfooded the implementation and must distinguish copied public API semantics from original Rust host translation. No official Proto-UI GPUI Adapter claim is made before upstream review and the complete native evidence for the declared profile.

## Risks and explicit non-goals

- No arbitrary dynamic module loading.
- No synchronous JavaScript calls from a GPUI frame path.
- No raw JS functions, GPUI entities, or host-local objects on the wire.
- No generic desktop guarantee from one Linux or Windows run.
- No touch guarantee until the exact pinned GPUI revision has executable evidence.
- No full AccessKit guarantee while the host remains on an API surface without the required public integration.
- No visual-token parity claim for a token that the GPUI translator reports unsupported.
- No release publication of an unreleased Proto-UI main snapshot under the stable package identity.
