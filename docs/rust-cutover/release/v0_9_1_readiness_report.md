# NTPRO v0.9.1 Strategy Runtime Hardening Readiness Report

Date: 2026-06-19
Executor: Codex
Milestone: `ntpro-rust-only-v0.9.1`
Status: READY FOR OWNER RELEASE DECISION - NOT RELEASED

## Summary

`v0.9.1` is the Strategy Runtime Semantics & Audit Hardening patch for the
published v0.9 local deterministic Strategy Runtime batch foundation line.

Plain Chinese summary: v0.9.1 的 V091 队列已经完成，可以进入 owner release
决策。它不是 Binance 下单版本，也不是实盘版本。它只是在 v0.9.0 本地批处理策略证明
之后，把 node/session/market/risk/heartbeat/artifact 语义补严，并补上
Supervisor/Dashboard 的工件健康审计和重启集成 smoke。v0.10.0 仍然是最早的
Binance testnet order proof 版本。

## Product Claim

```text
capability = Strategy Runtime Semantics & Audit Hardening
current published release = ntpro-rust-only-v0.9.0
release decision candidate = ntpro-rust-only-v0.9.1
next capability track = v0.10.0 Binance Testnet Order Proof
runtime code changes = scoped to strategy runtime semantics and local supervisor restart hardening
order submission = not included
production trading = not included
release tag = not created by V091-010
GitHub Release = not published by V091-010
```

## Patch Scope

Included:

```text
release-surface correction
StrategyNode config contract unification
persistent StrategySession lifecycle semantics
node/session/market status coherence
runtime counter snapshot semantics
kill-switch enabled/active split
Strategy Session manifest and child artifact audit
Supervisor/Dashboard degraded status visibility
Supervisor restart identity hardening
integration, heartbeat, shutdown, and restart smoke coverage
final v0.9.1 readiness and release-note closure
```

Not included:

```text
adapter behavior
trading semantics
release tag
GitHub Release publication
Binance testnet order proof
Binance testnet order submission
real funds
production trading
Dashboard order controls
```

## Task Gate Matrix

| Task | Status | Evidence | Notes |
| --- | --- | --- | --- |
| V091-001 | PASS | `docs/rust-cutover/evidence/V091-001.md` | Public release surface points at v0.9.0 released state and frames v0.9.1 as a correction patch. |
| V091-002 | PASS | `docs/rust-cutover/evidence/V091-002.md` | StrategyNode config loading and validation are unified for the v0.9.1 node path. |
| V091-003 | PASS | `docs/rust-cutover/evidence/V091-003.md` | StrategySession lifecycle remains persistent until stop/shutdown. |
| V091-004 | PASS | `docs/rust-cutover/evidence/V091-004.md` | Node/session/market lifecycle mapping is coherent. |
| V091-005 | PASS | `docs/rust-cutover/evidence/V091-005.md` | Runtime counters are shared across heartbeat, status, metrics, summary, Supervisor, and Dashboard. |
| V091-006 | PASS | `docs/rust-cutover/evidence/V091-006.md` | Kill switch enabled and active semantics are split while no-order behavior remains enforced. |
| V091-007 | PASS | `docs/rust-cutover/evidence/V091-007.md` | Strategy Session manifest records child artifacts, counts, bytes, and checksums. |
| V091-008 | PASS | `docs/rust-cutover/evidence/V091-008.md` | Supervisor and Dashboard surface degraded health for artifact errors or state conflicts. |
| V091-009 | PASS | `docs/rust-cutover/evidence/V091-009.md` | Offline integration smoke covers Supervisor, Dashboard, heartbeat, shutdown, and restart. |
| V091-010 | PASS | `docs/rust-cutover/evidence/V091-010.md` | Final readiness report and release notes are closed for owner release decision. |

## Validation Evidence

Local validation collected across the V091 queue includes:

```text
scripts/ai/verify_fast.sh
cargo fmt --check
cargo check -p nautilus-cli
cargo clippy -p nautilus-cli --all-targets -- -D warnings
cargo test -p nautilus-cli supervisor --lib
cargo test -p nautilus-cli dashboard --lib
cargo test -p nautilus-cli strategy_session --lib
scripts/ai/verify_v09_strategy_runtime_smoke.sh
scripts/ai/verify_v09_shadow_mode_no_order_gate.sh
scripts/ai/verify_v091_strategy_supervisor_dashboard_integration.sh
scripts/ai/verify_release.sh v091-strategy-supervisor-dashboard-integration
git diff --check
```

Hosted validation for the final integration slice:

```text
PR #378 = merged
merge commit = 185ff3cfc8b748870bb7263d512b82f1b4ec191e
Rust Cutover Smoke / smoke = PASS, 24m51s
security-audit / changes = PASS
```

The v0.9.1 queue is ready for owner release decision after V091-010 merges.
This report itself does not create the release tag and does not publish the
GitHub Release.

## Release Closure Status

This report does not create a tag and does not publish a GitHub Release. It is
the release-readiness closure document for a possible owner-approved
`ntpro-rust-only-v0.9.1` release after the V091 queue completes.

If the owner approves publication later, the release must preserve this
boundary:

```text
Strategy Runtime semantics and audit hardening only
current published capability remains v0.9 local deterministic batch foundation
v0.10.0 is earliest Binance testnet order proof track
no Binance testnet order submission
no real funds
no production trading
no Dashboard order controls
```

## Final Verdict

`v0.9.1` is ready for owner release decision after V091-010 merges. It is not
yet tagged and not yet published as a GitHub Release.

Do not describe this readiness PASS as Binance testnet order readiness,
real-funds readiness, production trading readiness, or Dashboard order-control
readiness.
