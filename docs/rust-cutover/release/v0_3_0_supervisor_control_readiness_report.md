# NTPRO v0.3.0 Supervisor Control Readiness Report

Date: 2026-06-12
Executor: Codex
Milestone: v0.3.0 Local Supervisor Control Console
Decision: PASS for local scope; hosted release gate requires v0.3.1 rerun

## Plain Chinese Summary

v0.3.0 这次收口的重点不是再加新 runtime，而是把本地 Supervisor 控制台的
发布口径和实际能力对齐。

大白话说：现在本地 Rust CLI 和本地 Dashboard 都已经能覆盖
`start`、`stop`、`pause`、`resume`、`reconnect data`、
`reconnect execution` 这六个控制动作；其中两个 reconnect 不会假装接通真实
交易所，而是明确返回 `not_supported`，并把结果写进本地 status、metrics 和
events artifacts，便于审计和排障。

这份报告只说明：

- v0.3.0 的本地 sandbox Supervisor 控制台范围通过；
- 发布门禁已经把 v0.3.0 控制 smoke 纳入；
- 依然不代表生产交易、真实交易所连接、真实下单、手工下单或远程多用户控制
  已经就绪。

## Scope Decision

Scope source:

- `docs/rust-cutover/scope/v0_3_supervisor_control_console.md`
- `docs/rust-cutover/evidence/V03-011.md`

Release claim:

```text
Local Supervisor Control Console
```

Product boundary:

```text
SupervisorRegistry
  + node status / metrics / logs / events artifacts
  -> Rust CLI supervisor controls
  -> local Dashboard HTTP API
  -> local Dashboard UI
```

## V03 Task Readiness

| Task | Scope | Evidence | PR | Status |
| --- | --- | --- | --- | --- |
| V03-011 | v0.3.0 control-console release contract | `docs/rust-cutover/evidence/V03-011.md` | #252 | PASS |
| V03-012 | pause / resume node control | `docs/rust-cutover/evidence/V03-012.md` | #253 | PASS |
| V03-013 | reconnect data source control | `docs/rust-cutover/evidence/V03-013.md` | #254 | PASS |
| V03-014 | reconnect execution gateway control | `docs/rust-cutover/evidence/V03-014.md` | #255 | PASS |
| V03-015 | Dashboard controls and API smoke | `docs/rust-cutover/evidence/V03-015.md` | #256 | PASS |
| V03-016 | release gate and readiness report | `docs/rust-cutover/evidence/V03-016.md` | current task | PASS |

## Supported Controls

v0.3.0 local sandbox-only supported controls:

```text
start node
stop node
pause node
resume node
reconnect data source -> explicit not_supported result
reconnect execution gateway -> explicit not_supported result
```

V031 clarification:

- reconnect controls record unsupported local sandbox results only
- reconnect controls do not perform production venue reconnect
- reconnect controls do not reconnect real data or execution adapters
- reconnect controls do not recover real accounts or real orders

Observable guarantees:

- `external_venue_connection=false`
- `real_orders_submitted=false`
- lifecycle / connection results are written to local artifacts
- Dashboard and CLI present the same local-only boundary

## Unsupported / Out Of Scope

The following are still out of scope for `ntpro-rust-only-v0.3.0`:

- production real-exchange live trading
- real account connectivity
- real order submission
- manual order entry
- order modification / cancellation UI beyond local supervisor scope
- production reconnect controls
- remote or distributed Dashboard operation
- multi-user permissions
- tag creation as part of this readiness task
- GitHub Release publication as part of this readiness task

## Evidence Summary

Referenced evidence:

- `docs/rust-cutover/evidence/V03-011.md`
- `docs/rust-cutover/evidence/V03-012.md`
- `docs/rust-cutover/evidence/V03-013.md`
- `docs/rust-cutover/evidence/V03-014.md`
- `docs/rust-cutover/evidence/V03-015.md`

Direct smoke highlights from this run:

- `scripts/ai/v03_supervisor_control_smoke.sh`
  - PASS
  - root:
    `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-control.5zHLGM`
- `scripts/ai/v03_dashboard_smoke.sh`
  - PASS
  - root:
    `/var/folders/4t/qzfq_xwj43vbx6ty4tkg_w100000gn/T//ntpro-v03-010.5LPldb`
  - final states:
    `sandbox-a=running`, `sandbox-b=stopped`

## Required Validation

| Command | Result | Summary |
| --- | --- | --- |
| `cargo fmt --check` | PASS | Formatting check passed. |
| `cargo check -p nautilus-cli` | PASS | CLI crate compiled. |
| `cargo test -p nautilus-cli dashboard --lib` | PASS | 24 passed, 0 failed. |
| `cargo test -p nautilus-cli supervisor --lib` | PASS | 24 passed, 0 failed. |
| `scripts/ai/v03_supervisor_control_smoke.sh` | PASS | Local CLI control smoke passed and preserved `external_venue_connection=false` and `real_orders_submitted=false`. |
| `scripts/ai/v03_dashboard_smoke.sh` | PASS | Local Dashboard browser smoke passed with six controls and no mobile overflow. |
| `scripts/ai/verify_release.sh` | LOCAL PASS / HOSTED NOT CLEAN | Local release verification covered the v0.3 supervisor and Dashboard smoke. Hosted GitHub `Rust Cutover Release Gate` did not produce a clean PASS; see V031-003. |
| `scripts/ai/check_rust_only_runtime.sh` | PASS | Rust-only product surface gate passed. |
| `git diff --check` | PASS | No whitespace diff errors. |

## Hosted GitHub Actions Status

V031-003 records the hosted release-gate closeout:

- `Rust Cutover Release Gate` run `27384342541`
  - trigger: `push`
  - ref: `ntpro-rust-only-v0.3.0`
  - commit: `2822ef8c29771de8ef1b90b96507ac6f1bcefcb3`
  - conclusion: `failure`
  - summary: hosted runner hit `No space left on device` and linker `Bus error`
    while compiling `nautilus-architect-ax` targets.
- `Rust Cutover Release Gate` run `27421121134`
  - trigger: `workflow_dispatch`
  - ref: `main`
  - commit: `afc805396ad731e93f99252fbf3ca9e81010753a`
  - conclusion: `failure`
  - summary: hosted runner hit linker `Bus error` while compiling
    `nautilus-event-store` test targets.
- `Rust Cutover Release Gate` run `27423501016`
  - trigger: `workflow_dispatch`
  - ref: `main`
  - commit: `5bc497e6e7aa93d615e2d3580c61757de9eb7fbe`
  - conclusion: `cancelled`
  - summary: the verify-release job remained in progress for more than 90
    minutes and was cancelled.

This report remains valid as local scope evidence for the v0.3.0 Supervisor
Control Console, but it is not a clean hosted-release approval. Finish the
V031 hardening queue and rerun the hosted release gate before making the next
tag or GitHub Release claim.

## Release Gate Coverage

`scripts/ai/verify_release.sh` now verifies:

- release build
- Rust CLI product surface help output
- `supervisor` help includes:
  - `start`
  - `stop`
  - `pause`
  - `resume`
  - `reconnect-data`
  - `reconnect-execution`
- Rust-only runtime gate
- Cython removed gate
- v0.2 local two-node supervisor smoke
- v0.3 local supervisor control smoke
- v0.3 local Dashboard control smoke

## Behavior Impact

The release gate now enforces the actual v0.3.0 control-console boundary instead
of only checking the older Dashboard MVP or v0.2 supervisor path.

This does not add production connectivity. It hardens the release claim so the
published version cannot silently regress local control behavior.

## Public API Impact

No Rust library API was added by this task.

Public release-surface impact:

- `scripts/ai/verify_release.sh` is stricter for v0.3.0
- the documented release claim is now backed by CLI + Dashboard control smoke

## Migration Note

No migration note is required. This is a release-gate and readiness-report
task, not a breaking user API change.

## Remaining Risks

- This report proves only the local sandbox control-console boundary.
- Hosted GitHub `Rust Cutover Release Gate` did not produce a clean PASS for
  the v0.3.0 release line; V031-003 tracks the exact runs and requires a
  v0.3.1 rerun after hardening.
- Real adapter reconnect remains explicitly unsupported in v0.3.0.
- Temporary smoke artifact paths are local to this verification run.
- Tag creation and GitHub Release publication are owner actions outside this
  readiness task even when the technical gate is PASS.

## Final Decision

PASS for local scope.

NTPRO `v0.3.0` is technically ready under the
`Local Supervisor Control Console` scope. The ready state means the local Rust
CLI and local Dashboard control paths are verified and the release gate now
checks them. It does not by itself create a tag or publish a GitHub Release,
and it does not replace the need for a clean hosted GitHub release-gate PASS
before the next release claim.
