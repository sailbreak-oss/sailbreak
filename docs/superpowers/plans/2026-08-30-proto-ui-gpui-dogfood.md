# Proto-UI GPUI Dogfood Implementation Plan

## Goal

Make Sailbreak the first real non-Web Proto-UI dogfood host: embed QuickJS in the Rust GUI, execute the pinned Proto-UI 0.2.0 Runtime and Shadcn prototypes without reimplementing their semantics, and migrate the current GPUI dashboard's interactive controls to a real Shadcn Button projection.

## Architecture

- `@proto.ui/*` 0.2.0 remains the semantic source of truth for the first bridge revision.
- Bun is a build-time bundler only. It produces one pinned ES2020/IIFE asset containing the Runtime and a governed Shadcn prototype registry.
- `rquickjs` embeds QuickJS in the Sailbreak process. No Node/Bun child process or runtime dependency.
- Rust and JavaScript exchange only bounded JSON data through an in-process bridge API. The API keeps the same ordering, identity, projection, and ACK rules required for a later out-of-process transport.
- Rust/GPUI owns windows, layout, painting, hit testing, FocusHandle/AccessKit projection, and Rust-owned Slot content. QuickJS owns Proto modules, state, lifecycle, event semantics, and Shadcn style intent.
- The first admitted host slice is `shadcn-button`: one canonical GPUI click path emits one `press.commit`; pointer/keyboard/focus/a11y state is projected through the Runtime.
- The registry contains every direct Shadcn 0.2.0 prototype export. A prototype that requests a capability outside the implemented host profile fails with an explicit structured diagnostic; no local Rust semantic clone or silent fallback is allowed.

## Tasks

### 1. Bridge contract and QuickJS bundle

- Add a `proto-ui-gpui` crate with versioned protocol values for handshake, session/instance/view/commit identity, projection transactions, ACKs, input envelopes, style tokens, a11y snapshots, Slot markers, and diagnostics.
- Add contract tests for monotonic IDs, stale ACK/input rejection, bounded JSON values, Slot preservation, and terminal disposal.
- Add a Bun package under `tools/proto-ui-bridge` with exact `@proto.ui/core`, `@proto.ui/runtime`, `@proto.ui/prototypes-base`, and `@proto.ui/prototypes-shadcn` 0.2.0 dependencies.
- Bundle a real registry of all Shadcn direct entries and the bridge dispatcher; retain the generated bundle and source-package revision metadata as build inputs.
- Embed the generated bundle in Rust and expose a typed `QuickJsRuntime` wrapper with no raw JavaScript objects crossing the Rust boundary.

### 2. Runtime host and Button conformance

- Build the internal host capability layer required by the pinned 0.2.0 Runtime, keeping EventTarget/HTMLElement-shaped objects private to the QuickJS compatibility boundary and never exposing them in the Rust protocol.
- Implement host-owned Slot composition and explicit content-alternative input for Button names.
- Translate GPUI input into data-only pointer/key/focus envelopes with exact-once activation and default-action diagnostics.
- Convert Runtime projections into typed Rust state: template/Slot plan, merged Shadcn style tokens, a11y role/name/state/action, exposed primitive states, and outward `click` signal.
- Test mount, commit ACK, hover/press/focus/focus-visible, disabled gating, pointer and keyboard activation, style changes, a11y projection, remount epoch, stale messages, and disposal.

### 3. Generic GPUI ProtoSurface projection

- Add a GPUI-facing `ProtoSurface`/`ProtoButton` composition API that consumes projection data and renders Rust-owned child content without duplicating Button behavior.
- Implement the bounded Shadcn token-to-GPUI theme translator for the emitted Button surface, variants, sizes, interaction states, and dark theme; unsupported tokens produce diagnostics rather than disappearing silently.
- Give each semantic surface stable IDs, native focus behavior, AccessKit role/name/action/state, and one GPUI click handler that feeds the bridge.
- Keep structural containers and text anonymous; only semantic Proto surfaces and Slot boundaries receive durable identity.
- Add focused renderer and translator tests independent of the live desktop display.

### 4. Sailbreak GUI dogfood migration

- Replace every current action-bar and sidebar interactive `div` control with `ProtoButton` using actual `shadcn-button` props (`variant`, `size`, `disabled`) and visible Slot content.
- Preserve existing CLI-backed refresh, daemon, battery, thermal, power, diagnostics, MagicBay, and dry-run behavior and error wording.
- Keep the dashboard's information hierarchy and safety boundary, but derive control presentation from the Shadcn projection/theme rather than a second hand-written button style system.
- Add GUI controller tests proving one click maps to one command and unsupported command results remain visible as failures.

### 5. Verification and contribution boundary

- Run focused protocol, QuickJS, bridge, translator, GUI, and CLI integration tests.
- Run `cargo fmt --all -- --check`, workspace check/test/clippy, and release build checks proportional to the changed crates.
- Launch the actual GPUI GUI in a desktop-capable session and exercise refresh, navigation, and representative action buttons; record headless limitation if no display is available.
- Verify the binary does not spawn Node/Bun and that the embedded registry reports the pinned Proto-UI package identity.
- Document the exact supported/omitted host profile and keep any future upstream contribution as a separate clean patch; do not claim official Proto-UI `A-GPUI-*` conformance from this dogfood until native evidence is complete.
