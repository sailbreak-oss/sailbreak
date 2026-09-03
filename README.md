# Sailbreak

Cross-platform hardware control for Lenovo ThinkBook systems, with truthful capability reporting and verified readback for every supported write path.

Repository: [sailbreak-oss/sailbreak](https://github.com/sailbreak-oss/sailbreak)

## Scope

Sailbreak targets Lenovo 21VG / ThinkBook Panther Lake hardware while keeping the platform boundary explicit:

- `sailbreak` — CLI for hardware status, control, tuning, diagnostics, and snapshots.
- `sailbreak-cli` — identical CLI entry point for scripts or PATH installations.
- `sailbreakd` — optional local event daemon; it does not become a second source of hardware truth.
- `sailbreak-gui` — read-only GPUI dashboard.
- `crates/lctrl-*` — internal Rust HAL, domain, and tuning crates retained as implementation names.

Unsupported or unverified hardware channels return structured errors or appear as `limited`/`unavailable` in `sailbreak info`. The project does not include vendor binaries, firmware packages, advertising, accounts, telemetry, or an application store.

## Build and test

Requires Rust 1.95 or newer. The repository pins Rust 1.95.0 because the reviewed GPUI revision uses `std::hint::cold_path`.

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --workspace
```

Regenerate the embedded Proto-UI bundle only when upgrading its recorded `main` commit:

```bash
PROTO_UI_SHA=$(gh api repos/Proto-UI/Proto-UI/commits/main --jq .sha)
(cd tools/proto-ui-bridge && bun run sync-head -- --sha "$PROTO_UI_SHA" && bun run bundle:check)
```

The CI workflow exercises Linux and Windows. Hardware smoke tests are intentionally not part of CI; run them only on the target machine with the recovery path prepared.

## GUI status

The GUI pins Zed GPUI commit [`399258feeaf90ad8a3a208c99221ee87b6452f38`](https://github.com/zed-industries/zed/tree/399258feeaf90ad8a3a208c99221ee87b6452f38/crates/gpui) and embeds QuickJS. It executes the exact Proto-UI `main` snapshot recorded in `tools/proto-ui-bridge/upstream.json`; Bun is a bundle-generation tool only and is never spawned by the released GUI. The dashboard now dogfoods the complete eleven-family surface: Tabs, Select, Dropdown, Dialog, Toggle, Switch, Checkbox, Textarea, Hover Card, Separator, and Button. Rust owns native GPUI layout, paint, focus handles, and Slot content; Proto UI owns each family’s state, lifecycle, event semantics, accessibility intent, and style tokens.

The pinned revision changes the published 0.2.2 host API: accessibility builders live in `crates/gpui/src/elements/div.rs`, platform construction is `gpui_platform::application()` from `crates/gpui_platform/src/gpui_platform.rs`, and `ClickEvent` includes a `Touch` variant. Sailbreak records these deltas explicitly. This is a dogfood integration, not an official Proto-UI A-GPUI conformance claim.

The dashboard mapping is structural: sidebar sections use Tabs, selectors use Select, action groups use Dropdown, confirmations use Dialog, profile editing uses Textarea, capability details use Hover Card, and layout rules use Separator plus Proto SVG. Headings, capability rows, telemetry cards, and safety copy remain host-owned structural layout.

The Dashboard exposes an executable `DogfoodSession`/`DogfoodInventory` contract so family presence, selected/present content, disabled channels, and semantic action routing are checked through resolved host snapshots rather than source-text fixtures. The `GuiController` remains the only CLI/HAL/config bridge.

The embedded dogfood contract currently proves:

- each admitted Proto family is mounted through its Task 5–15 family host and projected into the native GPUI surface; no local component state machine or HTML-like fallback is used;
- native pointer, keyboard, focus, accessibility, Slot, and semantic activation facts are routed through `proto_surface.rs`, with one semantic signal producing at most one controller action;
- Select controls stay disabled because the snapshot has no typed current value, and representative Switch/Checkbox controls stay disabled with explicit “typed readback unavailable” copy. Their unknown state is not rendered or exposed as unchecked/off;
- profile confirmation is modal and only its explicit `--yes` confirmation reaches `GuiController::execute`; BIOS confirmation remains unavailable until a typed BIOS read/modify/write request exists;
- the performance-preview Toggle invokes only `--dry-run`, and clearing it never turns into a commit. `GuiController` is the sole CLI/HAL/config gateway.

### Conformance profile

The machine-readable record [`docs/proto-ui-gpui-profile.json`](docs/proto-ui-gpui-profile.json) is the versioned Sailbreak independent-dogfood profile (schema v1, `independent_dogfood`/`limited`, `official_proto_ui_adapter_claim: false`). It pins the exact source identity — Proto-UI `main` at `8c6dadc00554ad89040a5f36eb2df56eb9ad3c17` (lockfile sha256:4224dc5426a51c227714008d7a3a360cb6dd813a1b03b0c5fc94342699d6f6cf, bundle sha256:e7cbcc7e45037c7e71117a1adc93bc075fc7cb7b404621d67c70093d353d26d0) — plus the GPUI 0.2.2/Zed `399258feeaf90ad8a3a208c99221ee87b6452f38` host, the exact eleven-family/thirty-three-prototype admission set, an 18-row capability matrix, authority-tiered evidence, and the recorded canonical upstream and local runs.

`crates/proto-ui-gpui/tests/conformance_contract.rs` treats the profile as an executable contract and enforces the boundaries below:

- exact manifest linkage (commit, lockfile, bundle digest, package versions) and exact matrix/family/prototype sets;
- evidence tiers separate Runtime/fake-host/host-capability proof from native GPUI (AccessKit/dogfood) evidence; no tier claims a desktop smoke that has not run;
- Linux stays `limited` until an actual `DISPLAY`/`WAYLAND_DISPLAY` session records visual evidence; Windows CI compilation/tests are recorded as build evidence only, never GUI, AccessKit, or hardware proof;
- the bridge imports `@proto.ui/runtime` from its public package root only, contains no local `definePrototype`, no browser layout/paint APIs, and no GPUI/closures/JS objects on the Rust wire;
- Dialog admission uses the MASK part — no invented Portal/Overlay prototypes, and CloseIcon stays registry-only/native-unproven;
- the current props/default semantic fallback paths in the family facades are explicitly enumerated as acknowledged limitations rather than hidden.

Upstream contribution candidates are prepared HAL-free in five slices (protocol/leases, GPUI host capability and lifecycle, Template/Slot/Style/SVG translation, one Shadcn Adapter family per pull request, profile metadata and evidence). Admission is maintainer-gated: no official Proto-UI GPUI Adapter claim is made before upstream review and complete native evidence for the declared profile.

## Install

Download the latest tested binaries from the [Releases page](https://github.com/sailbreak-oss/sailbreak/releases/latest):

- Windows recommended: `sailbreak-v0.1.1-windows-x86_64-setup.exe` (NSIS installer)
- Linux x86_64: `sailbreak-linux-x86_64.tar.gz`
- Windows x86_64 portable: `sailbreak-windows-x86_64.zip` (contains `sailbreak.exe`, `sailbreak-cli.exe`, `sailbreakd.exe`, and `sailbreak-gui.exe`)

The NSIS installer installs `sailbreak.exe`, `sailbreak-cli.exe`, `sailbreakd.exe`, and `sailbreak-gui.exe` under `%LOCALAPPDATA%\\Sailbreak`, adds that directory to the current user's `PATH`, and creates Start Menu/Desktop shortcuts for `sailbreak-gui.exe` as the primary entry point. CLI binaries are terminal-only and have no flash-exit shortcuts; the daemon is not started automatically.

For Linux, extract the archive and install the command and daemon into a directory on `PATH`:

```bash
sha256sum -c sailbreak-linux-x86_64.tar.gz.sha256
tar -xzf sailbreak-linux-x86_64.tar.gz
sudo install -m 0755 sailbreak /usr/local/bin/sailbreak
sudo install -m 0755 sailbreak-cli /usr/local/bin/sailbreak-cli
sudo install -m 0755 sailbreakd /usr/local/bin/sailbreakd
sudo install -m 0755 sailbreak-gui /usr/local/bin/sailbreak-gui
```

For portable Windows use, compare the archive hash with `Get-FileHash .\\sailbreak-windows-x86_64.zip -Algorithm SHA256`, extract it, and add the extracted directory to `PATH` manually. Hardware writes still require the platform permissions and safety confirmations described below.

If no release binary matches the host, build from source with the commands in [Build and test](#build-and-test). To install the CLI through Cargo:

```bash
cargo install --path . --locked --bin sailbreak
ln -sf "$HOME/.cargo/bin/sailbreak" "$HOME/.cargo/bin/sailbreak-cli"
```

The release alias `sailbreak-cli` and the source-built `sailbreak` invoke the same command surface.

## Quick start

Read-only discovery is safe to run without a daemon:

```bash
sailbreak info --json
sailbreak doctor --json
sailbreak battery status --json
sailbreak perf temp --json
sailbreak magicbay detect --json
```

Mutating commands support `--dry-run`. Risky BIOS and persistent privacy operations require `--yes`; successful writes report the requested value and verified readback. Check the capability matrix before scripting a platform-specific command.

```bash
sailbreak --dry-run perf mode performance
sailbreak --dry-run tune profile apply balanced
sailbreak bios list --json
```

The optional daemon uses a per-user Unix socket at `$XDG_RUNTIME_DIR/sailbreak.sock` (fallback `/run/sailbreak.sock`) or the Windows named pipe `\\.\pipe\sailbreak.sock`. Override those endpoints with `SAILBREAK_SOCKET` or `SAILBREAK_PIPE`.

## Daemon management

The daemon is optional. Use the CLI to inspect and manage its lifecycle; `status` reports a structured channel error when it is not running:

```bash
sailbreak daemon status
sailbreak daemon install --dry-run
sailbreak daemon install
sailbreak daemon start
sailbreak daemon stop
```

Run `sailbreak daemon install --dry-run` first. Linux installs a systemd user unit; Windows installs a scheduled task with the required elevation policy. The installer only places binaries and PATH entries; it never enables a background service implicitly.

## Verified Windows boundary

On the documented 21VG Windows baseline, the following channels remain deliberately unavailable or limited until a safe, independently verified contract exists:

- arbitrary battery-threshold writes and charge-mode readback;
- Windows MSR/RAPL power-limit writes, including PL1, PL2, and tau;
- GameZone performance, fan, and temperature method families;
- panel refresh-rate mutation and persistent privacy writes;
- MagicBay inventory association is limited to the separate SetupAPI service path.

These states are reported by `sailbreak info`; they are not silently replaced with guessed IOCTLs, registry writes, or synthetic success.

## Configuration

Profile and state locations use the `sailbreak` identity:

- System profiles: `/etc/sailbreak/profiles.d` or `%ProgramData%\\sailbreak\\profiles.d`.
- User profiles: `${XDG_CONFIG_HOME:-~/.config}/sailbreak/profiles.d` or `%APPDATA%\\sailbreak\\profiles.d`.
- Tune state: `/run/sailbreak/state.json` or `%ProgramData%\\sailbreak\\state.json`.
- Managed snapshots: `${XDG_STATE_HOME:-~/.local/state}/sailbreak/snapshot.json` or `%ProgramData%\\sailbreak\\snapshot.json`.

Environment overrides use `SAILBREAK_SYSTEM_PROFILE_DIR`, `SAILBREAK_USER_PROFILE_DIR`, `SAILBREAK_STATE_PATH`, and `SAILBREAK_SNAPSHOT_PATH`.

## Clean-room boundary

Implementation is based only on the repository specifications, target-machine read-only observations recorded in the evidence document, and public APIs and standards. Do not obtain, reverse engineer, or distribute Lenovo or Intel vendor binaries. If an interface is unclear, record the call sequence, return value, expected behavior, and observed behavior for specification review instead of guessing.

Before any write path is considered complete, verify:

1. dry-run performs no hardware mutation;
2. the previous value is captured;
3. the write is followed by bounded readback;
4. a mismatch is reported as a verification failure;
5. a failed write restores the previous value where the channel permits rollback;
6. BIOS, EC, fan-table, power-limit, and privilege-specific safety rules remain intact.

## Contributing

Small focused pull requests are preferred. Keep platform-specific code in its HAL crate, shared contracts in `lctrl-core`/`lctrl-hal`, and user-facing behavior covered by an observable contract test. Do not broaden an unavailable channel into a best-effort write.

### Agent contribution workflow

Paste the following prompt into a fresh coding-agent session when delegating a change:

```text
You are contributing to sailbreak-cli, a Rust 2024 workspace for cross-platform Lenovo hardware control. The pinned toolchain baseline is Rust 1.95.0.

Goal: <one observable behavior>. Name the command or channel, relevant specification section, and acceptance evidence.
Repository rules:
- Read HANDOFF.md and the relevant feature-specific document under docs/ before editing.
- Stay within the clean-room boundary: use repository specifications, recorded read-only evidence, and public APIs only. Never inspect vendor binaries.
- Reuse existing traits, error variants, capability IDs, and readback helpers. Do not add a parallel abstraction.
- Every mutating path must support --dry-run, capture the previous value, perform bounded readback, and restore the previous value on a failed transition when the channel permits it.
- Report unverified or unavailable channels honestly; do not turn a probe into a fake successful write.
- Keep Windows and Linux implementations isolated in their existing HAL crates.
- Do not promote a capability from `limited` or `unavailable` to `available` without target-machine smoke evidence and verified readback; compilation is not hardware support evidence.

Before implementation:
1. Trace all callers and existing tests for the affected symbol or command.
2. Read `docs/00-cleanroom-charter.md`, `docs/08-architecture.md`, and `docs/B-evidence.md`, plus the subsystem specification named by the Goal.
3. State the contract, edge cases, and expected error semantics.
4. Write a focused failing test for each new observable behavior and run it to confirm RED.

Implementation:
1. Make the smallest production change that satisfies the failing test.
2. Add rollback/readback and capability details where the contract requires them.
3. Update relevant specifications or configuration examples if the user-facing contract changed.

Verification:
1. Run the focused test and confirm GREEN.
2. Run `cargo fmt --all -- --check`.
3. Run `cargo check --workspace --all-targets` and, when cross-compiling, the relevant target check (for example `--target x86_64-pc-windows-msvc`). Otherwise run the platform-specific check on its native host.
4. Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` unless the task explicitly narrows validation.
5. Report exact commands and outcomes, plus any platform-only check that could not run.

Return:
- files changed;
- observable behavior implemented;
- verification evidence;
- remaining limitations or risks.
```

Agents must not claim hardware support from compilation alone. A target-machine smoke test and readback evidence are required before a capability changes from `limited` or `unavailable` to `available`.

## Specifications and evidence

The clean-room specifications and evidence map live under `docs/`. Review `docs/00-cleanroom-charter.md`, `docs/08-architecture.md`, and `docs/B-evidence.md` before changing an interface or safety boundary.
