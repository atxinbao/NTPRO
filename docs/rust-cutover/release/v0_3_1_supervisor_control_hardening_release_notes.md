# NTPRO Rust-only v0.3.1 Release Notes

Date: 2026-06-13
Executor: Codex

## Release Identity

```text
Current source tag: ntpro-rust-only-v0.3.1
Capability: Local Supervisor Control Console Hardening
```

The v0.3.1 release is a patch/hardening/closure release for the existing local
sandbox-only Supervisor control-console line. It does not introduce a new
trading milestone. It keeps the v0.3.0 shipped capability claim and closes the
remaining release-surface, release-gate, and hosted-evidence gaps before the
next milestone.

## Summary

NTPRO v0.3.1 closes the Local Supervisor Control Console hardening queue.

本次正式版的重点不是新增交易产品能力，而是把 `v0.3.0` 之后已经 merged 到
`main` 的 release hardening、release gate、readiness、README、hosted gate
evidence 和 control semantics 一次收口进正式发布口径。

大白话说：`v0.3.1` 还是本地 sandbox Supervisor 控制台，不是实盘产品升级。
这次主要把“源码树里已经有的内容”和“正式 release 真正宣称的能力”重新对齐，
避免 tag、README、readiness、GitHub Release body 互相打架。

## Included in v0.3.1

- Existing local sandbox-only Supervisor control console from `v0.3.0`
- Rust CLI supervisor controls for:
  - `start`
  - `stop`
  - `pause`
  - `resume`
  - `reconnect-data` as explicit local sandbox `not_supported`
  - `reconnect-execution` as explicit local sandbox `not_supported`
- Local Dashboard control API and UI for the same scoped actions
- Release-gate hardening and hosted-runner stabilization work, including:
  - hosted linker-crash mitigation
  - serialized cargo build steps where needed
  - staged release-gate job split
  - release-binary supervisor/dashboard smoke coverage
  - dashboard smoke CI fallback hardening
  - release test / golden trace / workspace-test split
- Release-surface and semantics hardening, including:
  - README alignment with the v0.3 release line
  - stale supervisor registry lock recovery
  - stronger supervisor process identity checks
  - explicit pause/resume semantics contract
  - explicit reconnect `not_supported` contract
  - negative-path supervisor/dashboard control tests
  - final readiness and closeout accounting

## Included in the Source Tree but Not a Capability Expansion

The `ntpro-rust-only-v0.3.1` tag also contains merged source-tree deltas that
must be acknowledged, but they do not expand the shipped v0.3.1 capability
claim:

- Dashboard UI copy localization to Chinese
- Supervisor / trader product-shape documents under `docs/architecture/`
- High-precision catalog fixture scoping and ignored-test register updates
- v0.4 planning/task documents under `docs/rust-cutover/tasks/V04-*`

These changes are present in the tagged source tree, but they do **not** mean
that v0.3.1 now claims:

- trader-terminal implementation
- manual order entry / modify / cancel
- v0.4 product scope delivery
- production trading or real-exchange reconnect

## Explicitly Not Included

- Production real-exchange live trading
- Real account connectivity
- Real order submission
- Manual order entry
- Order modification / cancellation workflows
- Production reconnect controls
- Remote or distributed dashboard operation
- Multi-user permission model
- Prebuilt binary or Docker release artifact delivery
- v0.4 exchange, strategy, or trader-terminal implementation

## Validation Source

Primary release evidence:

- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_readiness_report.md`
- `docs/rust-cutover/release/v0_3_1_supervisor_control_hardening_closeout.md`

Release-gate supporting evidence:

- `docs/rust-cutover/evidence/V031-001.md` through `docs/rust-cutover/evidence/V031-010.md`
- `docs/rust-cutover/evidence/GH-VERIFY-RELEASE-LINKER-CRASH.md`
- `docs/rust-cutover/evidence/GH-RELEASE-DASHBOARD-SMOKE-CI-FALLBACK.md`
- `docs/rust-cutover/evidence/GH-RELEASE-GATE-RUST-TEST-GOLDEN-SPLIT.md`
- `docs/rust-cutover/evidence/GH-RELEASE-GATE-WORKSPACE-TEST-SPLIT.md`

## Migration Note Status

No Python, PyO3, or Cython product surface is restored by `v0.3.1`. Users
should still follow the Rust CLI, Rust crates, Rust examples, Rust documents,
and Rust release-verification paths.
