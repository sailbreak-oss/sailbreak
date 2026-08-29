# Sailbreak

Cross-platform hardware control for Lenovo ThinkBook systems, with truthful capability reporting and verified readback for every supported write path.

Repository: [voyage-control/sailbreak](https://github.com/voyage-control/sailbreak)

## Scope

Sailbreak targets Lenovo 21VG / ThinkBook Panther Lake hardware while keeping the platform boundary explicit:

- `sailbreak` — CLI for hardware status, control, tuning, diagnostics, and snapshots.
- `sailbreakd` — optional local event daemon; it does not become a second source of hardware truth.
- `sailbreak-gui` — read-only GPUI dashboard.
- `crates/lctrl-*` — internal Rust HAL, domain, and tuning crates retained as implementation names.

Unsupported or unverified hardware channels return structured errors or appear as `limited`/`unavailable` in `sailbreak info`. The project does not include vendor binaries, firmware packages, advertising, accounts, telemetry, or an application store.

## Build and test

Requires Rust 1.85 or newer.

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --workspace
```

The CI workflow exercises Linux and Windows. Hardware smoke tests are intentionally not part of CI; run them only on the target machine with the recovery path prepared.

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
You are contributing to sailbreak-cli, a Rust 2024 workspace for cross-platform Lenovo hardware control. The toolchain baseline is Rust 1.85 or newer.

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
